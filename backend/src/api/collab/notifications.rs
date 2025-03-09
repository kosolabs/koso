use super::{
    projects_state::ProjectState,
    txn_origin::{YOrigin, from_origin},
};
use crate::{
    api::{
        collab::txn_origin::Actor,
        model::{Task, User},
        yproxy::{YDocProxy, YTaskProxy},
    },
    notifiers::Notifier,
};
use anyhow::{Context, Result, anyhow};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::sync::mpsc::Receiver;
use yrs::{
    TransactionMut,
    types::{EntryChange, Event, Events, PathSegment},
};

#[derive(Debug)]
pub(super) struct KosoEvent {
    pub(super) project: Arc<ProjectState>,
    pub(super) changes: KosoEventChanges,
    pub(super) task: Task,
    pub(super) origin: YOrigin,
}

#[derive(Debug)]
pub(super) enum KosoEventChanges {
    Task(HashMap<String, EntryChange>),
    Children(),
}

#[tracing::instrument(skip(txn, events, project))]
pub(super) fn handle_deep_graph_update_events(
    txn: &TransactionMut,
    events: &Events,
    project: Arc<ProjectState>,
) {
    for event in events.iter() {
        handle_deep_graph_update_event(txn, event, &project)
    }
}

#[tracing::instrument(skip(txn, event, project), fields(?path=event.path(), ?target=event.target()))]
fn handle_deep_graph_update_event(
    txn: &TransactionMut,
    event: &Event,
    project: &Arc<ProjectState>,
) {
    if let Err(e) = handle_deep_graph_update_event_internal(txn, event, project) {
        tracing::error!("Failed to handle deep_graph_update event: {e:?}");
    }
}

fn handle_deep_graph_update_event_internal(
    txn: &TransactionMut,
    event: &Event,
    project: &Arc<ProjectState>,
) -> Result<()> {
    tracing::trace!("Handling deep_graph_update event");

    match event {
        yrs::types::Event::Map(map_event) => {
            if map_event.path().len() != 1 {
                return Ok(());
            }
            let origin = from_origin(txn.origin())?;
            let changes: HashMap<String, EntryChange> = map_event
                .keys(txn)
                .iter()
                .map(|(mod_id, change)| (mod_id.to_string(), (*change).clone()))
                .collect();
            let task = YTaskProxy::new(map_event.target().clone())
                .to_task(txn)
                .context("Failed to convert MapEvent to Koso Task")?;
            let event = KosoEvent {
                project: project.clone(),
                changes: KosoEventChanges::Task(changes),
                task,
                origin,
            };
            return project
                .event_tx
                .try_send(event)
                .context("Failed to send event to deep graph observer");
        }
        yrs::types::Event::Array(array_event) => {
            if array_event.path().len() != 2 {
                return Ok(());
            }
            if array_event.removes(txn).is_empty() {
                return Ok(());
            }

            let path = array_event.path();
            let PathSegment::Key(task_id) = path.front().context("missing task path segment")?
            else {
                return Err(anyhow!("Expected key path, got: {path:?}"));
            };
            let PathSegment::Key(field) = path.get(1).context("missing task field segment")? else {
                return Err(anyhow!("Expected field path, got: {path:?}"));
            };
            if field.as_ref() != "children" {
                return Ok(());
            }

            let origin = from_origin(txn.origin())?;

            let doc = YDocProxy::new_from_existing_doc(txn.doc().clone(), txn).unwrap();
            let task = doc
                .get(txn, task_id)
                .unwrap()
                .to_task(txn)
                .context("Failed to convert ArrayEvent to Koso Task")?;
            let event = KosoEvent {
                project: project.clone(),
                changes: KosoEventChanges::Children(),
                task,
                origin,
            };
            return project
                .event_tx
                .try_send(event)
                .context("Failed to send array event to deep graph observer");
        }
        _ => (),
    }
    Ok(())
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
            self.process_event(event).await;
        }
        tracing::info!("Stopped processing events");
    }

    #[tracing::instrument(skip(self))]
    async fn process_event(&self, event: KosoEvent) {
        tracing::trace!("Processing event");
        if let Err(e) = self.process_event_internal(event).await {
            tracing::warn!("Failed to process event: {e:?}");
        }
    }

    async fn process_event_internal(&self, event: KosoEvent) -> Result<()> {
        match &event.changes {
            KosoEventChanges::Task(changes) => {
                for (field, change) in changes {
                    match (field.as_str(), change) {
                        (
                            "assignee",
                            EntryChange::Updated(_, yrs::Out::Any(yrs::Any::String(recipient))),
                        )
                        | (
                            "assignee",
                            EntryChange::Inserted(yrs::Out::Any(yrs::Any::String(recipient))),
                        ) => {
                            if let Actor::User(user) = &event.origin.actor {
                                if user.email == recipient.as_ref() {
                                    continue;
                                }
                            };
                            let sender = match &event.origin.actor {
                                Actor::User(user) => Sender::User(user),
                                _ => Sender::KosoJuggler,
                            };
                            let msg = format!(
                                "🎁 <i>{}</i> assigned to you:\n<a href=\"https://koso.app/projects/{}?taskId={}\"><b>{}</b></a>",
                                sender.format(),
                                event.project.project_id,
                                event.task.id,
                                event.task.name
                            );
                            self.notifier.notify(recipient, &msg).await?;
                        }
                        (
                            "status",
                            EntryChange::Updated(_, yrs::Out::Any(yrs::Any::String(status))),
                        ) => {
                            if status.as_ref() == "Done" {
                                self.unblock_and_notify_actionable_tasks(&event).await?;
                            }
                        }
                        _ => continue,
                    }
                }
            }
            KosoEventChanges::Children() => {
                self.unblock_and_notify_actionable_tasks(&event).await?;
            }
        }
        Ok(())
    }

    async fn unblock_and_notify_actionable_tasks(&self, event: &KosoEvent) -> Result<()> {
        let actionable =
            Self::find_actionable_juggled_tasks(&event.task.id, &event.project).await?;
        if actionable.is_empty() {
            return Ok(());
        }

        {
            let doc = event.project.doc_box.lock().await;
            let doc = &doc.as_ref().context("No doc initialized.")?.ydoc;
            let mut txn = doc.transact_mut_with(event.origin.delegated("juggle").as_origin()?);
            // TODO: Handle partial failures.
            for (task_id, _, _) in actionable.iter() {
                tracing::debug!("Unblocking task {task_id}");
                match doc.get(&txn, task_id) {
                    Ok(task) => {
                        task.set_status(&mut txn, Some("Not Started"));
                        task.set_status_time(&mut txn, Some(now()?));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get task {task_id}: {e:?}");
                        continue;
                    }
                }
            }
        }

        // TODO: We could parallelize this.
        for (task_id, assignee, name) in actionable {
            let should_send = match &event.origin.actor {
                Actor::User(user) => user.email != assignee,
                _ => true,
            };
            if should_send {
                let msg = format!(
                    "🎁 <i>Koso Juggler</i> assigned to you:\n<a href=\"https://koso.app/projects/{}?taskId={}\"><b>{}</b></a>",
                    event.project.project_id, task_id, name
                );
                self.notifier.notify(&assignee, &msg).await?;
            }
        }
        Ok(())
    }

    async fn find_actionable_juggled_tasks(
        done_task_id: &String,
        project: &ProjectState,
    ) -> Result<Vec<(String, String, String)>> {
        let doc = project.doc_box.lock().await;
        let doc = &doc.as_ref().context("No doc initialized.")?.ydoc;
        let txn = doc.transact();

        // Perform a DFS starting from all Blocked, juggled tasks not assigned to the
        // events triggering user.
        let mut actionable: Vec<(String, String, String)> = vec![];
        for task in doc.tasks(&txn)? {
            if task.get_kind(&txn)?.unwrap_or_default() == "Juggled"
                && task.get_status(&txn)?.unwrap_or_default() == "Blocked"
            {
                let mut found = *done_task_id == task.get_id(&txn)?;
                let mut complete = true;
                let mut stack = vec![];
                stack.extend(task.get_children(&txn)?);
                loop {
                    let Some(descendent_id) = stack.pop() else {
                        break;
                    };

                    // First, mark if the event's task was found.
                    if descendent_id == *done_task_id {
                        found = true;
                    }

                    // Next, check if this task or all of its descendants are complete.
                    let descendent = doc.get(&txn, &descendent_id)?;
                    let kind = descendent.get_kind(&txn)?;
                    let children = descendent.get_children(&txn)?;
                    if kind.is_some() || children.is_empty() {
                        if descendent.get_status(&txn)?.unwrap_or_default() != "Done" {
                            complete = false;
                            break;
                        }
                    } else {
                        stack.extend(children);
                    }
                }
                if found && complete {
                    if let Some(assignee) = task.get_assignee(&txn)? {
                        actionable.push((task.get_id(&txn)?, assignee, task.get_name(&txn)?));
                    }
                }
            }
        }
        Ok(actionable)
    }
}

fn now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

enum Sender<'a> {
    User(&'a User),
    KosoJuggler,
}

impl Sender<'_> {
    fn format(&self) -> String {
        match self {
            Sender::User(user) => format!("{} &lt;{}&gt;", user.name, user.email),
            Sender::KosoJuggler => "Koso Juggler".to_string(),
        }
    }
}
