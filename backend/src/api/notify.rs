use super::collab::client_message_handler::YrsMessage;
use super::collab::doc_observer::YrsUpdate;
use super::collab::doc_update_processor::DocUpdateProcessor;
use super::collab::projects_state::{ProjectState, ProjectsState};
use super::collab::storage;
use super::collab::yrs_message_processor::YrsMessageProcessor;
use crate::api::collab::client_message_handler::ClientMessageHandler;
use crate::api::collab::projects_state::DocBox;
use crate::api::{
    self,
    collab::{
        client::{from_socket, ClientClosure, CLOSE_ERROR, CLOSE_UNAUTHORIZED},
        doc_observer::DocObserver,
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
use std::{
    net::SocketAddr,
    sync::{atomic::Ordering::Relaxed, Arc},
    time::Duration,
};
use tokio::sync::mpsc::{self, Sender};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::types::ToJson;
use yrs::{ReadTxn, StateVector, Transact};

pub fn start(pool: &'static PgPool) -> Notifier {
    let (process_tx, process_rx) = mpsc::channel::<YrsMessage>(1);
    let (doc_update_tx, doc_update_rx) = mpsc::channel::<YrsUpdate>(50);
    let tracker = tokio_util::task::TaskTracker::new();
    let notifier = Notifier {
        state: Arc::new(ProjectsState {
            projects: DashMap::new(),
            pool,
            tracker: tracker.clone(),
        }),
        pool,
        process_tx,
        doc_update_tx,
        cancel: CancellationToken::new(),
        tracker,
    };

    let doc_update_processor = DocUpdateProcessor {
        state: Arc::clone(&notifier.state),
        pool,
        doc_update_rx,
        cancel: notifier.cancel.clone(),
    };
    notifier
        .tracker
        .spawn(doc_update_processor.process_doc_updates());

    let yrs_message_processor = YrsMessageProcessor {
        process_rx,
        state: Arc::clone(&notifier.state),
        cancel: notifier.cancel.clone(),
    };
    notifier
        .tracker
        .spawn(yrs_message_processor.process_messages());

    notifier
}

#[derive(Clone)]
pub struct Notifier {
    state: Arc<ProjectsState>,
    pool: &'static PgPool,
    process_tx: Sender<YrsMessage>,
    doc_update_tx: Sender<YrsUpdate>,
    cancel: CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first client being registered, load everything from the database, and construct the initial doc.
// For every client that joins, send the current graph as the initial state vector.
// When a client sends an update, apply the update to the doc, store it in the database
// and broadcast it to other clients.
// When the last client disconnects, consider destroying the graph.
impl Notifier {
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
        let sv = match self.init_doc_box(&project).await {
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
            cancel: self.cancel.clone(),
            receiver,
        };
        self.tracker.spawn(handler.receive_messages_from_client());

        Ok(())
    }

    async fn init_doc_box(&self, project: &Arc<ProjectState>) -> Result<StateVector> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.doc.transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        tracing::debug!("Initializing new YDoc");
        let (doc, update_count) = storage::load_doc(&project.project_id, self.pool).await?;
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        project.updates.store(update_count, Relaxed);

        // Persist and broadcast update events by subscribing to the callback.
        let observer = DocObserver {
            project: Arc::clone(project),
            tracker: self.tracker.clone(),
            doc_update_tx: self.doc_update_tx.clone(),
        };
        let res = doc.observe_update_v2(move |txn, update| {
            observer.handle_doc_update_v2_event(txn, update);
        });
        let sub = match res {
            Ok(sub) => Box::new(sub),
            Err(e) => return Err(anyhow!("Failed to create observer: {e}")),
        };

        let db = DocBox { doc, sub };
        let sv = db.doc.transact().state_vector();
        *doc_box = Some(db);
        Ok(sv)
    }

    #[tracing::instrument(skip(self))]
    pub async fn stop(&self) {
        // Cancel background tasks and wait for them to complete.
        tracing::info!(
            "Waiting for {} outstanding task(s) to finish..",
            self.tracker.len()
        );
        self.cancel.cancel();
        self.tracker.close();
        if let Err(e) = tokio::time::timeout(Duration::from_secs(15), self.tracker.wait()).await {
            tracing::warn!(
                "Timed out waiting for tasks. {} remain: {e}",
                self.tracker.len()
            );
        }

        // Close all the project state, including client connections.
        self.state.close().await;
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
