use std::sync::Arc;

use crate::api::notify::{ProjectsState, YrsUpdate};
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::{msg_sync::sync_update, storage};

pub struct DocUpdateProcessor {
    pub state: Arc<ProjectsState>,
    pub pool: &'static PgPool,
    pub doc_update_rx: Receiver<YrsUpdate>,
    pub cancel: CancellationToken,
}

impl DocUpdateProcessor {
    #[tracing::instrument(skip(self))]
    pub async fn process_doc_updates(mut self) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = self.doc_update_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    self.process_doc_update(msg).await;
                }
            }
        }
        tracing::info!("Stopped processing doc updates");
    }

    #[tracing::instrument(skip(self))]
    async fn process_doc_update(&self, update: YrsUpdate) {
        if let Err(e) = self.process_doc_update_internal(update).await {
            tracing::warn!("Failed to process doc update: {e}");
        }
    }

    async fn process_doc_update_internal(&self, update: YrsUpdate) -> Result<()> {
        let Some(project) = self.state.get(&update.project_id) else {
            return Err(anyhow!(
                "Unexpectedly, received close for client but the project is missing."
            ));
        };

        if let Err(e) = storage::persist_update(&project.project_id, &update.data, self.pool).await
        {
            return Err(anyhow!("Failed to persist update: {e}"));
        }
        project
            .broadcast_msg(&update.who, sync_update(update.data))
            .await;

        Ok(())
    }
}
