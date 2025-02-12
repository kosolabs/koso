use anyhow::Result;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;
use yrs::{
    types::{EntryChange, Events},
    TransactionMut,
};

use crate::{
    api::{collab::txn_origin::Actor, model::Task, yproxy::YTaskProxy},
    notifiers::notify,
};

use super::{
    projects_state::ProjectState,
    txn_origin::{from_origin, YOrigin},
};

#[derive(Debug)]
pub(super) struct KosoEvent {
    pub(super) project: Arc<ProjectState>,
    pub(super) changes: HashMap<String, EntryChange>,
    pub(super) task: Task,
    pub(super) origin: YOrigin,
}

#[tracing::instrument(skip(txn, events, project))]
pub(super) fn handle_deep_graph_update_event(
    txn: &TransactionMut,
    events: &Events,
    project: Arc<ProjectState>,
) {
    for event in events.iter() {
        if let yrs::types::Event::Map(map_event) = event {
            // Events on tasks will have a path length of 1.
            if map_event.path().len() != 1 {
                continue;
            }

            let origin = match from_origin(txn.origin()) {
                Ok(origin) => origin,
                Err(e) => {
                    tracing::error!("Failed to deserialize origin: {e}");
                    continue;
                }
            };

            let changes: HashMap<String, EntryChange> = map_event
                .keys(txn)
                .iter()
                .map(|(mod_id, change)| (mod_id.to_string(), (*change).clone()))
                .collect();

            let task = match YTaskProxy::new(map_event.target().clone()).to_task(txn) {
                Ok(task) => task,
                Err(e) => {
                    tracing::error!("Failed to convert MapEvent to Koso Task: {e}");
                    continue;
                }
            };

            let event = KosoEvent {
                project: project.clone(),
                changes,
                task,
                origin,
            };

            if let Err(e) = project.event_tx.try_send(event) {
                tracing::error!("Failed to send event to deep graph observer: {e:?}")
            }
        }
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
        tracing::trace!("Processing event: {event:?}");

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
                                    "🎁 <i>{} &lt;{}&gt;</i> assigned to you:\n<a href=\"https://koso.app/projects/{}\"><b>{}</b></a>",
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
