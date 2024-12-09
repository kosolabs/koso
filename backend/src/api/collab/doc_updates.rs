use crate::api::collab::{msg_sync::sync_update, storage};
use crate::api::collab::{projects_state::ProjectState, txn_origin::from_origin};
use crate::api::yproxy::YTaskProxy;
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::{fmt, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_util::task::TaskTracker;
use tracing::Instrument;
use yrs::types::map::MapEvent;
use yrs::Map as _;

use super::projects_state::DocBox;
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
        self.tracker.spawn(
            async move {
                if let Err(e) = doc_update_tx.send(update).await {
                    tracing::error!("Failed to send to doc_update channel: {e}");
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

    /// Callback invoked on update_v2 doc events, triggered by calls to "apply_update" in process_message_internal.
    /// observe_update_v2 only accepts synchronous callbacks thus requiring this function be synchronous
    /// and any async operations, including sending to a channel, to occur in a spawned task.
    pub(super) fn handle_graph_update_event(
        &self,
        project: Arc<ProjectState>,
        txn: &yrs::TransactionMut,
        event: &MapEvent,
    ) {
        let origin = match from_origin(txn.origin()) {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to parse origin: {e}");
                return;
            }
        };

        for (mod_id, change) in event.keys(txn).iter() {
            if let yrs::types::EntryChange::Inserted(yrs::Out::YMap(mod_task)) = change {
                let mod_task: YTaskProxy = YTaskProxy::new(mod_task.clone());
                let mod_num = mod_task.get_num(txn).unwrap().parse::<u64>().unwrap();
                for (id, out) in event.target().iter(txn) {
                    if mod_id.as_ref() != id {
                        if let yrs::Out::YMap(task) = out {
                            let task: YTaskProxy = YTaskProxy::new(task);
                            let num = task.get_num(txn).unwrap().parse::<u64>().unwrap();
                            if num == mod_num {
                                let p = project.clone();
                                let origin = YOrigin {
                                    who: format!("rw-{mod_id}-{}", origin.who),
                                    id: format!("rw-{mod_id}-{}", origin.id),
                                };
                                let id = mod_id.clone();
                                self.tracker.spawn(
                                    async move {
                                        let doc_box = p.doc_box.lock().await;
                                        let doc =
                                            &DocBox::doc_or_error(doc_box.as_ref()).unwrap().ydoc;
                                        let mut txn = doc.transact_mut_with(origin.as_origin());
                                        let num = doc.next_num(&txn).unwrap();
                                        let task = doc.get(&txn, &id).unwrap();
                                        tracing::info!("Rewriting task num {num}");
                                        task.set_num(&mut txn, &num.to_string());
                                    }
                                    .in_current_span(),
                                );
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
