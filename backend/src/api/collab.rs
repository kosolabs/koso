pub(crate) mod client;
pub(crate) mod client_message_handler;
pub(crate) mod doc_observer;
pub(crate) mod doc_update_processor;
pub(crate) mod msg_sync;
pub(crate) mod projects_state;
pub(crate) mod storage;
pub(crate) mod txn_origin;
pub(crate) mod yrs_message_processor;

use crate::api::{
    self,
    collab::{
        client::{from_socket, CLOSE_UNAUTHORIZED},
        client_message_handler::YrsMessage,
        doc_observer::YrsUpdate,
        doc_update_processor::DocUpdateProcessor,
        projects_state::ProjectsState,
        yrs_message_processor::YrsMessageProcessor,
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
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self};
use tokio::time::sleep;
use tokio_util::task::TaskTracker;
use uuid::Uuid;
use yrs::types::ToJson;
use yrs::{ReadTxn, Transact};

pub(crate) fn start(pool: &'static PgPool) -> Collab {
    let (process_msg_tx, process_msg_rx) = mpsc::channel::<YrsMessage>(1);
    let (doc_update_tx, doc_update_rx) = mpsc::channel::<YrsUpdate>(50);
    let tracker = tokio_util::task::TaskTracker::new();
    let collab = Collab {
        inner: Arc::new(Inner {
            state: ProjectsState {
                projects: DashMap::new(),
                process_msg_tx,
                doc_update_tx,
                pool,
                tracker: tracker.clone(),
            },
            pool,
            tracker,
        }),
    };

    let doc_update_processor = DocUpdateProcessor {
        pool,
        doc_update_rx,
    };
    collab
        .inner
        .tracker
        .spawn(doc_update_processor.process_doc_updates());

    let yrs_message_processor = YrsMessageProcessor { process_msg_rx };
    collab
        .inner
        .tracker
        .spawn(yrs_message_processor.process_messages());

    collab
}

#[derive(Clone)]
pub(crate) struct Collab {
    inner: Arc<Inner>,
}

struct Inner {
    state: ProjectsState,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl Collab {
    #[tracing::instrument(skip(self, socket, who, user), fields(who))]
    pub(super) async fn register_client(
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
        if let Err(e) = api::verify_access(self.inner.pool, user, &project_id).await {
            sender.close(CLOSE_UNAUTHORIZED, "Unauthorized.").await;
            return Err(e.as_err());
        }

        self.inner
            .state
            .add_and_init_client(&project_id, sender, receiver)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn stop(self) {
        tracing::debug!("Closing all clients...");
        self.inner.state.close_all_project_clients().await;

        let tracker = self.inner.tracker.clone();
        // Drop the Collab instance to release inner.state which
        // holds the sender sides of our processing channels.
        // Receivers will abort when all senders are gone.
        drop(self);
        return Collab::wait_for_tasks(tracker).await;
    }

    async fn wait_for_tasks(tracker: TaskTracker) {
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
            tracing::warn!("Timed out waiting for tasks. {} remain: {e}", tracker.len());
        }
    }

    pub(super) async fn get_doc(&self, project_id: &ProjectId) -> Result<yrs::Any, Error> {
        let (doc, _) = storage::load_doc(project_id, self.inner.pool).await?;
        let txn = doc.transact();
        let Some(graph) = txn.get_map("graph") else {
            return Err(anyhow!("No graph present in doc"));
        };
        Ok(graph.to_json(&txn))
    }
}
