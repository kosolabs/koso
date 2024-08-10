pub mod client;
pub mod client_message_handler;
pub mod doc_observer;
pub mod doc_update_processor;
pub mod msg_sync;
pub mod projects_state;
pub mod storage;
pub mod txn_origin;
pub mod yrs_message_processor;

use super::collab::client_message_handler::YrsMessage;
use super::collab::doc_observer::YrsUpdate;
use super::collab::doc_update_processor::DocUpdateProcessor;
use super::collab::projects_state::ProjectsState;
use super::collab::yrs_message_processor::YrsMessageProcessor;
use crate::api::collab::client_message_handler::ClientMessageHandler;
use crate::api::collab::projects_state::ProjectState;
use crate::api::{
    self,
    collab::{
        client::{from_socket, ClientClosure, CLOSE_ERROR, CLOSE_UNAUTHORIZED},
        msg_sync::sync_request,
    },
    google::User,
    model::ProjectId,
};
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use axum::extract::ws::WebSocket;
use dashmap::DashMap;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, Sender};
use tokio::time::sleep;
use uuid::Uuid;
use yrs::types::ToJson;
use yrs::{ReadTxn, Transact};

pub fn start(pool: &'static PgPool) -> Collab {
    let (process_tx, process_rx) = mpsc::channel::<YrsMessage>(1);
    let (doc_update_tx, doc_update_rx) = mpsc::channel::<YrsUpdate>(50);
    let tracker = tokio_util::task::TaskTracker::new();
    let collab = Collab {
        state: Arc::new(ProjectsState {
            projects: DashMap::new(),
            doc_update_tx,
            pool,
            tracker: tracker.clone(),
        }),
        pool,
        process_tx,
        tracker,
    };

    let doc_update_processor = DocUpdateProcessor {
        pool,
        doc_update_rx,
    };
    collab
        .tracker
        .spawn(doc_update_processor.process_doc_updates());

    let yrs_message_processor = YrsMessageProcessor { process_rx };
    collab
        .tracker
        .spawn(yrs_message_processor.process_messages());

    collab
}

#[derive(Clone)]
pub struct Collab {
    state: Arc<ProjectsState>,
    pool: &'static PgPool,
    process_tx: Sender<YrsMessage>,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first client being registered, load everything from the database, and construct the initial doc.
// For every client that joins, send the current graph as the initial state vector.
// When a client sends an update, apply the update to the doc, store it in the database
// and broadcast it to other clients.
// When the last client disconnects, consider destroying the graph.
impl Collab {
    #[tracing::instrument(skip(self, socket, who, user), fields(who))]
    pub async fn register_client(
        self,
        socket: WebSocket,
        who: SocketAddr,
        project_id: ProjectId,
        user: User,
    ) -> Result<()> {
        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        tracing::Span::current().record("who", &who);
        tracing::debug!("Registering client");

        let (mut sender, receiver) = from_socket(socket, &who, &project_id);

        // Before doing anything else, make sure the user has access to the project.
        if let Err(e) = api::verify_access(self.pool, user, &project_id).await {
            sender.close(CLOSE_UNAUTHORIZED, "Unauthorized.").await;
            return Err(e.as_err());
        }

        // Get of insert the project state.
        let project = self.state.get_or_insert(&project_id);

        // Store the sender side of the socket in the list of clients.
        // It's important to add the client before calling init_doc_box to avoid
        // races with remove_and_close_client. e.g. a client present without an initialized doc box.
        if let Some(mut existing) = project.add_client(sender).await {
            existing
                .close(CLOSE_ERROR, "Unexpected duplicate connection.")
                .await;
            tracing::error!("Unexpectedly, client already exists: {existing:?}");
            // Intentionally fall through below and start listening for messages on the new sender.
        };

        // Init the doc_box, if necessary and grab the state vector.
        let sv = match ProjectState::init_doc_box(&project).await {
            Ok(sv) => sv,
            Err(e) => {
                project
                    .remove_and_close_client(
                        &who,
                        ClientClosure {
                            code: CLOSE_ERROR,
                            reason: "Failed to init doc.",
                            details: format!("Failed to init doc: {e}"),
                        },
                    )
                    .await;
                return Err(anyhow!("Failed to init doc: {e}"));
            }
        };

        // Send the entire state vector to the client.
        tracing::debug!("Sending sync_request message to client");
        if let Err(e) = project.send_msg(&who, sync_request(sv)).await {
            project
                .remove_and_close_client(
                    &who,
                    ClientClosure {
                        code: CLOSE_ERROR,
                        reason: "Failed to send sync request message.",
                        details: format!("Failed to send sync request message: {e}"),
                    },
                )
                .await;
            return Err(anyhow!(
                "Failed to send sync request message to client: {e}"
            ));
        }

        // Listen for messages on the read side of the socket.
        let handler = ClientMessageHandler {
            project: Arc::clone(&project),
            process_tx: self.process_tx.clone(),
            receiver,
        };
        self.tracker.spawn(handler.receive_messages_from_client());

        Ok(())
    }

    #[allow(clippy::async_yields_async)]
    #[tracing::instrument(skip(self))]
    pub async fn stop(self) -> Pin<Box<dyn Future<Output = ()>>> {
        tracing::debug!("Closing all clients...");
        // Close all client connections.
        self.state.close().await;

        let tracker = self.tracker.clone();
        return Box::pin(async move {
            // Wait for background processing tasks to complete.
            tracing::info!(
                "Waiting for {} outstanding task(s) to finish..",
                tracker.len()
            );

            if let Err(e) = tokio::time::timeout(Duration::from_secs(60), async {
                loop {
                    if tracker.is_empty() {
                        break;
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            })
            .await
            {
                tracing::warn!(
                    "Timed out waiting for tasks. {} remain: {e}",
                    self.tracker.len()
                );
            }
        });
    }

    pub async fn get_doc(&self, project_id: &ProjectId) -> Result<yrs::Any, Error> {
        let (doc, _) = storage::load_doc(project_id, self.pool).await?;
        let txn = doc.transact();
        let Some(graph) = txn.get_map("graph") else {
            return Err(anyhow!("No graph present in doc"));
        };
        Ok(graph.to_json(&txn))
    }
}
