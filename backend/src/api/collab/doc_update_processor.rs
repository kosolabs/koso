use crate::api::collab::{doc_observer::DocUpdate, msg_sync::sync_update, storage};
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;

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
            .broadcast_msg(&update.who, sync_update(&update.data))
            .await;

        Ok(())
    }
}
