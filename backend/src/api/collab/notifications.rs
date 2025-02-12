use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use yrs::types::EntryChange;

use crate::{
    api::{collab::txn_origin::Actor, model::Task},
    notifiers::notify,
};

use super::{projects_state::ProjectState, txn_origin::YOrigin};

#[derive(Debug)]
pub(super) struct KosoEvent {
    pub(super) project: Arc<ProjectState>,
    pub(super) changes: Vec<(String, EntryChange)>,
    pub(super) task: Task,
    pub(super) origin: YOrigin,
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
            let Some(event) = self.event_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_event(event).await {
                tracing::warn!("Failed to process event: {e}");
            }
        }
        tracing::info!("Stopped processing events");
    }

    #[tracing::instrument(skip(self))]
    async fn process_event(&self, event: KosoEvent) -> Result<()> {
        tracing::info!("Processing event: {event:?}");

        let Actor::User(user) = event.origin.actor else {
            return Ok(());
        };

        for (field, change) in event.changes {
            match (field.as_str(), change) {
                (
                    "assignee",
                    EntryChange::Updated(_, yrs::Out::Any(yrs::Any::String(recipient))),
                ) if *user.email != *recipient => {
                    notify(
                                self.pool,
                                &recipient,
                                &format!(
                                    "🎁 <i>{} &lt;{}&gt;</i> assigned you: <a href=\"https://koso.app/projects/{}\"><b>{}</b></a>",
                                    user.name, user.email, event.project.project_id, event.task.name
                                ),
                            )
                            .await?;
                }
                _ => continue,
            }
        }
        Ok(())
    }
}
