use super::{
    projects_state::ProjectState,
    txn_origin::{YOrigin, from_origin},
};
use crate::{
    api::{collab::txn_origin::Actor, model::Task, yproxy::YTaskProxy},
    notifiers::Notifier,
};
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Receiver;
use yrs::{
    TransactionMut,
    types::{EntryChange, Events},
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
                    tracing::error!("Failed to deserialize origin: {e:?}");
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
                    tracing::error!("Failed to convert MapEvent to Koso Task: {e:?}");
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
    event_rx: Receiver<KosoEvent>,
    notifier: Notifier,
}

impl EventProcessor {
    pub(super) fn new(pool: &'static PgPool, event_rx: Receiver<KosoEvent>) -> Result<Self> {
        Ok(EventProcessor {
            event_rx,
            notifier: Notifier::new(pool)?,
        })
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn process_events(mut self) {
        loop {
            let Some(event) = self.event_rx.recv().await else {
                break;
            };
            if let Err(e) = self.process_event(event).await {
                tracing::warn!("Failed to process event: {e:?}");
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
                ) => {
                    if *user.email != *recipient {
                        self.notifier.notify(
                            &recipient,
                            &format!(
                                "🎁 <i>{} &lt;{}&gt;</i> assigned to you:\n<a href=\"https://koso.app/projects/{}?taskId={}\"><b>{}</b></a>",
                                user.name, user.email, event.project.project_id, event.task.id, event.task.name
                            ),
                        ).await?;
                    }
                }
                ("status", EntryChange::Updated(_, yrs::Out::Any(yrs::Any::String(status)))) => {
                    if status.as_ref() == "Done" {
                        let juggled = {
                            let doc = event.project.doc_box.lock().await;
                            let doc = &doc.as_ref().context("No doc initialized.")?.ydoc;
                            let txn = doc.transact();

                            let mut juggled = vec![];
                            for task in doc.tasks(&txn)? {
                                if task.get_kind(&txn)?.unwrap_or_default() == "Juggled"
                                    && task.get_children(&txn)?.contains(&event.task.id)
                                    && task.get_assignee(&txn)?.unwrap_or_default() != user.email
                                {
                                    let mut all_done = true;
                                    for child in &task.get_children(&txn)? {
                                        let child = doc.get(&txn, child)?;
                                        // TODO: Handle rollups
                                        if child.get_status(&txn)?.unwrap_or_default() != "Done" {
                                            all_done = false;
                                            break;
                                        }
                                    }
                                    if all_done {
                                        juggled.push((
                                            task.get_id(&txn)?,
                                            task.get_assignee(&txn)?,
                                            task.get_name(&txn)?,
                                        ));
                                    }
                                }
                            }
                            juggled
                        };

                        for (id, assignee, name) in juggled {
                            if let Some(assignee) = assignee {
                                self.notifier.notify(
                                    &assignee,
                                    &format!(
                                        "🎁 <i>Koso Juggler</i> assigned to you:\n<a href=\"https://koso.app/projects/{}?taskId={}\"><b>{}</b></a>",
                                        event.project.project_id, id, name
                                    ),
                                ).await?;
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        Ok(())
    }
}
