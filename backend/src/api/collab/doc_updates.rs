use crate::api::collab::{msg_sync::sync_update, storage};
use crate::api::collab::{projects_state::ProjectState, txn_origin::from_origin};
use crate::api::yproxy::YTaskProxy;
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_util::task::TaskTracker;
use tracing::Instrument;
use yrs::Map as _;
use yrs::types::map::MapEvent;

use super::projects_state::{DocBox, DocBoxProvider};
use super::txn_origin::YOrigin;

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
                tracing::error!("Failed to parse origin: {e:?}");
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
        self.tracker.spawn(
            async move {
                if let Err(e) = doc_update_tx.send(update).await {
                    tracing::error!("Failed to send to doc_update channel: {e:?}");
                }
            }
            .in_current_span(),
        );
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
            self.process_doc_update(update).await;
        }
        tracing::info!("Stopped processing doc updates");
    }

    #[tracing::instrument(skip(self))]
    async fn process_doc_update(&self, update: DocUpdate) {
        if let Err(e) = self.process_doc_update_internal(update).await {
            tracing::warn!("Failed to process doc update: {e:?}");
        }
    }

    async fn process_doc_update_internal(&self, update: DocUpdate) -> Result<()> {
        storage::persist_update(&update.project.project_id, &update.data, self.pool)
            .await
            .context("Failed to persist update")?;
        update
            .project
            .broadcast_msg(sync_update(&update.data), Some(&update.who))
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

pub(super) struct GraphObserver {
    tracker: TaskTracker,
}

impl GraphObserver {
    pub(super) fn new(tracker: TaskTracker) -> Self {
        GraphObserver { tracker }
    }

    /// Callback invoked on graph events triggered by calls to "apply_update" in process_message_internal.
    pub(super) fn handle_graph_update_event<T: DocBoxProvider + 'static>(
        &self,
        project: Arc<T>,
        txn: &yrs::TransactionMut,
        event: &MapEvent,
    ) {
        if let Err(e) = self.handle_graph_update_event_internal(project, txn, event) {
            tracing::warn!("Failed to handle graph update event: {e:?}");
        }
    }

    pub(super) fn handle_graph_update_event_internal<T: DocBoxProvider + 'static>(
        &self,
        project: Arc<T>,
        txn: &yrs::TransactionMut,
        event: &MapEvent,
    ) -> Result<()> {
        for (mod_id, change) in event.keys(txn).iter() {
            tracing::trace!("Handling graph update change {mod_id} - {change:?}");
            let yrs::types::EntryChange::Inserted(yrs::Out::YMap(mod_task)) = change else {
                continue;
            };

            let mod_task: YTaskProxy = YTaskProxy::new(mod_task.clone());
            let mod_num = mod_task.get_num(txn)?;
            for (_, out) in event
                .target()
                .iter(txn)
                .filter(|(id, _)| mod_id.as_ref() != *id)
            {
                let yrs::Out::YMap(task) = out else {
                    continue;
                };
                if YTaskProxy::new(task).get_num(txn)? != mod_num {
                    continue;
                }

                let project = project.clone();
                let origin = from_origin(txn.origin())?.delegated("rw");
                let mod_id = mod_id.clone();
                self.tracker.spawn(
                    async move {
                        if let Err(e) = Self::rewrite_task_num(&mod_id, project, origin).await {
                            tracing::warn!("Failed to rewrite task num for task '{mod_id}': {e:?}");
                        }
                    }
                    .in_current_span(),
                );
                break;
            }
        }
        Ok(())
    }

    async fn rewrite_task_num<T: DocBoxProvider>(
        task_id: &str,
        project: Arc<T>,
        origin: YOrigin,
    ) -> Result<()> {
        let doc_box = project.get_doc_box().await;
        let doc = &DocBox::doc_or_error(doc_box.as_ref())?.ydoc;
        let mut txn = doc.transact_mut_with(origin.as_origin()?);

        let num = doc.next_num(&txn)?;
        tracing::debug!("Rewriting task num for task '{task_id}' to {num}");
        let task = doc.get(&txn, task_id)?;
        task.set_num(&mut txn, &num.to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use self::collab::Collab;

    use super::{DocBox, DocBoxProvider, YOrigin};
    use crate::api::{
        collab::{self, YDocProxy, doc_updates::GraphObserver, txn_origin::Actor},
        model::Task,
    };
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::{Mutex, MutexGuard};
    use tokio_util::task::TaskTracker;

    #[test_log::test(tokio::test)]
    async fn graph_observer_test() {
        let tracker = TaskTracker::new();
        let task1 = Task {
            id: "id1".into(),
            num: "1".into(),
            name: "name1".into(),
            desc: None,
            children: vec![],
            assignee: None,
            reporter: None,
            status: None,
            status_time: None,
            url: None,
            kind: None,
        };
        let mut task2 = Task {
            id: "id2".into(),
            num: "1".into(),
            name: "name2".into(),
            desc: None,
            children: vec![],
            assignee: None,
            reporter: None,
            status: None,
            status_time: None,
            url: None,
            kind: None,
        };
        let origin = YOrigin {
            who: "graph_observer_test".into(),
            id: "test1".into(),
            actor: Actor::None,
        }
        .as_origin()
        .unwrap();

        // Setup the doc and observer.
        let doc_box_provider = Arc::new(TestDocBoxProvider {
            db: Mutex::new(Some(DocBox {
                ydoc: YDocProxy::new(),
                subs: vec![],
            })),
        });
        {
            let mut db: MutexGuard<'_, Option<DocBox>> = doc_box_provider.db.lock().await;
            let db: &mut DocBox = db.as_mut().unwrap();

            let observer = GraphObserver::new(tracker.clone());
            let weak_doc_box = Arc::downgrade(&doc_box_provider);
            let sub = db.ydoc.observe_graph(move |txn, event| {
                observer.handle_graph_update_event(weak_doc_box.upgrade().unwrap(), txn, event)
            });
            db.subs.push(sub);
        }

        // Insert task 1.
        {
            let db = doc_box_provider.db.lock().await;
            let doc = &db.as_ref().unwrap().ydoc;
            doc.set(&mut doc.transact_mut_with(origin.clone()), &task1);
        }
        Collab::wait_for_tasks(&tracker).await;
        assert_eq!(tracker.len(), 0);

        // Insert task 2 which will have a number that conflicts with task 1.
        {
            let db = doc_box_provider.db.lock().await;
            let doc = &db.as_ref().unwrap().ydoc;
            doc.set(&mut doc.transact_mut_with(origin.clone()), &task2);
        }
        Collab::wait_for_tasks(&tracker).await;
        assert_eq!(tracker.len(), 0);

        // Verify task2's num field was rewritten.
        let graph = {
            let db = doc_box_provider.db.lock().await;
            let doc = &db.as_ref().unwrap().ydoc;
            let g = doc.to_graph(&doc.transact()).unwrap();
            g
        };
        assert_eq!(*graph.get("id1").unwrap(), task1);
        task2.num = "2".into();
        assert_eq!(*graph.get("id2").unwrap(), task2);
    }

    struct TestDocBoxProvider {
        db: Mutex<Option<DocBox>>,
    }

    #[async_trait]
    impl DocBoxProvider for TestDocBoxProvider {
        async fn get_doc_box(&self) -> MutexGuard<'_, Option<DocBox>> {
            self.db.lock().await
        }
    }
}
