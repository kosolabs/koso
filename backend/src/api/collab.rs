//! How each message is handling depends on the protocol and message type.
//! Today, there's only one protocol: SYNC. The protocol includes three types of
//! messages:
//!   - SYNC_REQUEST - sent by clients during the initial
//!   - SYNC_RESPONSE
//!   - SYNC_UPDATE -
//!

pub(crate) mod client;
pub(crate) mod client_messages;
pub(crate) mod doc_updates;
pub(crate) mod msg_sync;
pub(crate) mod projects_state;
pub(crate) mod storage;
pub(crate) mod txn_origin;

use crate::api::{
    self,
    collab::{
        client::{from_socket, CLOSE_UNAUTHORIZED},
        client_messages::{ClientMessage, ClientMessageProcessor},
        doc_updates::{DocUpdate, DocUpdateProcessor},
        projects_state::ProjectsState,
    },
    google::User,
    model::{Graph, ProjectId},
    yproxy::YDocProxy,
};
use anyhow::Error;
use anyhow::Result;
use axum::extract::ws::WebSocket;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self};
use tokio::time::sleep;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

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
    pub(crate) fn new(pool: &'static PgPool) -> Collab {
        let (process_msg_tx, process_msg_rx) = mpsc::channel::<ClientMessage>(1);
        let (doc_update_tx, doc_update_rx) = mpsc::channel::<DocUpdate>(50);
        let tracker = tokio_util::task::TaskTracker::new();
        let collab = Collab {
            inner: Arc::new(Inner {
                state: ProjectsState::new(process_msg_tx, doc_update_tx, pool, tracker.clone()),
                pool,
                tracker,
            }),
        };

        collab
            .inner
            .tracker
            .spawn(DocUpdateProcessor::new(pool, doc_update_rx).process_doc_updates());

        collab
            .inner
            .tracker
            .spawn(ClientMessageProcessor::new(process_msg_rx).process_messages());

        collab
    }

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
        if let Err(e) = api::verify_project_access(self.inner.pool, user, &project_id).await {
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

        if let Err(e) = tokio::time::timeout(Duration::from_secs(30), async {
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

    pub(super) async fn get_graph(&self, project_id: &ProjectId) -> Result<Graph, Error> {
        let (ydoc, _) = storage::load_doc(project_id, self.inner.pool).await?;
        let txn = ydoc.transact();
        ydoc.to_graph(&txn)
    }
}
