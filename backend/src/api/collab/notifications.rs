use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::Receiver;
use yrs::types::{EntryChange, Events};

use super::projects_state::ProjectState;

pub(super) struct KosoEvent {
    pub(super) project: Arc<ProjectState>,
    pub(super) changes: Vec<(String, EntryChange)>,
}

impl fmt::Debug for KosoEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KosoEvent")
            .field("project_id", &self.project.project_id)
            .field("changes", &self.changes)
            .finish()
    }
}

pub(super) struct EventProcessor {
    pool: &'static PgPool,
    event_rx: Receiver<KosoEvent>,
}

impl EventProcessor {
    pub(super) fn new(pool: &'static PgPool, event_rx: Receiver<KosoEvent>) -> Self {
        EventProcessor { pool, event_rx }
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn process_events(mut self) {
        loop {
            let Some(update) = self.event_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_event(update).await {
                tracing::warn!("Failed to process event: {e}");
            }
        }
        tracing::info!("Stopped processing events");
    }

    #[tracing::instrument(skip(self))]
    async fn process_event(&self, update: KosoEvent) -> Result<()> {
        // let u = Update::decode_v2(&update.data)?;
        // notify(
        //     self.pool,
        //     "shadanan@gmail.com",
        //     &format!("Got an update from {u}"),
        // )
        // .await?;
        Ok(())
    }
}
