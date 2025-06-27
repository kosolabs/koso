//! How each message is handling depends on the protocol and message type.
//! Today, there's only one protocol: SYNC. The protocol includes three types of
//! messages:
//!   - SYNC_REQUEST - sent by clients during the initial
//!   - SYNC_RESPONSE
//!   - SYNC_UPDATE -

use crate::api::{
    self,
    collab::{
        client::{CLOSE_UNAUTHORIZED, from_socket},
        client_messages::{ClientMessage, ClientMessageProcessor},
        doc_updates::{DocUpdate, DocUpdateProcessor},
        projects_state::ProjectsState,
    },
    google::User,
    model::{Graph, ProjectId},
    yproxy::YDocProxy,
};
use anyhow::Result;
use anyhow::{Error, Ok};
use axum::extract::ws::WebSocket;
use notifications::{EventProcessor, KosoEvent};
use projects_state::ProjectState;
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self};
use tokio::time::sleep;
use tokio_util::task::TaskTracker;

pub(crate) mod awareness;
pub(crate) mod client;
pub(crate) mod client_messages;
pub(crate) mod doc_updates;
pub(crate) mod msg_sync;
pub(crate) mod notifications;
pub(crate) mod projects_state;
pub(crate) mod storage;
pub(crate) mod txn_origin;

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
    pub(crate) fn new(pool: &'static PgPool) -> Result<Collab> {
        let (process_msg_tx, process_msg_rx) = mpsc::channel::<ClientMessage>(1);
        let (doc_update_tx, doc_update_rx) = mpsc::channel::<DocUpdate>(50);
        let (event_tx, event_rx) = mpsc::channel::<KosoEvent>(50);
        let tracker = tokio_util::task::TaskTracker::new();
        let collab = Collab {
            inner: Arc::new(Inner {
                state: ProjectsState::new(
                    process_msg_tx,
                    doc_update_tx,
                    event_tx,
                    pool,
                    tracker.clone(),
                ),
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
            .inner
            .tracker
            .spawn(EventProcessor::new(pool, event_rx)?.process_events());

        Ok(collab)
    }

    #[tracing::instrument(skip(self, socket, who, project_id, user))]
    pub(super) async fn register_client(
        self,
        socket: WebSocket,
        who: String,
        project_id: ProjectId,
        user: User,
    ) -> Result<()> {
        tracing::debug!("Registering client");

        let (mut sender, receiver) = from_socket(socket, &who, &user, &project_id);

        // Before doing anything else, make sure the user has access to the project.
        if let Err(e) = api::verify_project_access(self.inner.pool, &user, &project_id).await {
            sender.close(CLOSE_UNAUTHORIZED, "Unauthorized.").await;
            return Err(e.as_err());
        }

        self.inner
            .state
            .add_and_init_client(&project_id, sender, receiver)
            .await?;

        Ok(())
    }

    pub(crate) async fn register_local_client(
        &self,
        project_id: &ProjectId,
    ) -> Result<LocalClient> {
        let project = self
            .inner
            .state
            .add_and_init_local_client(project_id)
            .await?;

        Ok(LocalClient { project })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn stop(self) {
        tracing::debug!("Closing all clients...");
        self.inner.state.stop().await;

        let tracker = self.inner.tracker.clone();
        // Drop the Collab instance to release inner.state which
        // holds the sender sides of our processing channels.
        // Receivers will abort when all senders are gone.
        drop(self);
        return Collab::wait_for_tasks(&tracker).await;
    }

    pub(super) async fn wait_for_tasks(tracker: &TaskTracker) {
        // Wait for background processing tasks to complete.
        tracing::info!(
            "Waiting for {} outstanding task(s) to finish..",
            tracker.len()
        );

        if let Err(e) = tokio::time::timeout(Duration::from_secs(25), async {
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

    pub(super) async fn get_doc(&self, project_id: &ProjectId) -> Result<YDocProxy> {
        let (ydoc, _) = storage::load_doc(project_id, self.inner.pool).await?;
        Ok(ydoc)
    }

    pub(super) async fn get_graph(&self, project_id: &ProjectId) -> Result<Graph, Error> {
        let (ydoc, _) = storage::load_doc(project_id, self.inner.pool).await?;
        let txn = ydoc.transact();
        ydoc.to_graph(&txn)
    }
}

pub struct LocalClient {
    pub project: Arc<ProjectState>,
}
