use crate::api::collab::{msg_sync::sync_update, storage};
use crate::api::collab::{projects_state::ProjectState, txn_origin::from_origin};
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_util::task::TaskTracker;

// Handles updates applied to a project doc and forward them to the doc_update_tx
// for handling by the `DocUpdateProcessor`.
pub(super) struct DocObserver {
    doc_update_tx: Sender<DocUpdate>,
    tracker: TaskTracker,
}

impl DocObserver {
    pub(super) fn new(doc_update_tx: Sender<DocUpdate>, tracker: TaskTracker) -> Self {
        DocObserver {
            doc_update_tx,
            tracker,
        }
    }

    /// Callback invoked on update_v2 doc events, triggered by calls to "apply_update" in process_message_internal.
    /// observe_update_v2 only accepts synchronous callbacks thus requiring this function be synchronous
    /// and any async operations, including sending to a channel, to occur in a spawned task.
    pub(super) fn handle_doc_update_v2_event(
        &self,
        project: Arc<ProjectState>,
        txn: &yrs::TransactionMut,
        event: &yrs::UpdateEvent,
    ) {
        let origin = match from_origin(txn.origin()) {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to parse origin: {e}");
                return;
            }
        };
        let update = DocUpdate {
            who: origin.who,
            project,
            id: origin.id,
            data: event.update.clone(),
        };

        let doc_update_tx = self.doc_update_tx.clone();
        self.tracker.spawn(async move {
            if let Err(e) = doc_update_tx.send(update).await {
                tracing::error!("Failed to send to doc_update channel: {e}");
            }
        });
    }
}

/// DocUpdateProcessor receives doc updates from a channel
/// and 1) persists them to the DB, and 2) broadcasts them
/// to other clients connected for the given project.
pub(super) struct DocUpdateProcessor {
    pool: &'static PgPool,
    doc_update_rx: Receiver<DocUpdate>,
}

impl DocUpdateProcessor {
    pub(super) fn new(pool: &'static PgPool, doc_update_rx: Receiver<DocUpdate>) -> Self {
        DocUpdateProcessor {
            pool,
            doc_update_rx,
        }
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn process_doc_updates(mut self) {
        loop {
            let Some(update) = self.doc_update_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_doc_update(update).await {
                tracing::warn!("Failed to process doc update: {e}");
            }
        }
        tracing::info!("Stopped processing doc updates");
    }

    #[tracing::instrument(skip(self))]
    async fn process_doc_update(&self, update: DocUpdate) -> Result<()> {
        if let Err(e) =
            storage::persist_update(&update.project.project_id, &update.data, self.pool).await
        {
            return Err(anyhow!("Failed to persist update: {e}"));
        }
        update
            .project
            .broadcast_msg(
                &update.who,
                sync_update(&update.data, update.project.version),
            )
            .await;

        Ok(())
    }
}

/// An update that has been successfully applied to the doc.
pub(super) struct DocUpdate {
    pub(super) who: String,
    pub(super) project: Arc<ProjectState>,
    /// Unique ID associated with this update.
    /// Piped through from the triggering YrsMessage::id.
    pub(super) id: String,
    /// A yrs Update in the v2 encoding.
    /// Can be decoded via Update::decode_v2.
    pub(super) data: Vec<u8>,
}

impl fmt::Debug for DocUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocUpdate")
            .field("project_id", &self.project.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
