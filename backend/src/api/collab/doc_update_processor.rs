use super::{doc_observer::YrsUpdate, msg_sync::sync_update, storage};
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;

pub(super) struct DocUpdateProcessor {
    pub(super) pool: &'static PgPool,
    pub(super) doc_update_rx: Receiver<YrsUpdate>,
}

impl DocUpdateProcessor {
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
    async fn process_doc_update(&self, update: YrsUpdate) -> Result<()> {
        if let Err(e) =
            storage::persist_update(&update.project.project_id, &update.data, self.pool).await
        {
            return Err(anyhow!("Failed to persist update: {e}"));
        }
        update
            .project
            .broadcast_msg(&update.who, sync_update(update.data))
            .await;

        Ok(())
    }
}
