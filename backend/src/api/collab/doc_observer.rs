use crate::api::{
    collab::txn_origin::from_origin,
    notify::{ProjectState, YrsUpdate},
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::task::TaskTracker;

pub struct DocObserver {
    pub project: Arc<ProjectState>,
    pub tracker: TaskTracker,
    pub doc_update_tx: Sender<YrsUpdate>,
}

impl DocObserver {
    /// Callback invoked on update_v2 doc events, triggered by calls to "apply_update" in process_message_internal.
    /// observe_update_v2 only accepts synchronous callbacks thus requiring this function be synchronous
    /// and any async operations, including sendin to a channel, to occur in a spawned task.
    pub fn handle_doc_update_v2_event(&self, txn: &yrs::TransactionMut, event: &yrs::UpdateEvent) {
        let origin = match from_origin(txn.origin()) {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to parse origin: {e}");
                return;
            }
        };
        let update = YrsUpdate {
            who: origin.who,
            project_id: self.project.project_id.clone(),
            id: origin.id,
            data: event.update.clone(),
        };

        let doc_update_tx = self.doc_update_tx.clone();
        self.tracker.spawn(async move {
            if let Err(e) = doc_update_tx.send(update).await {
                tracing::error!("failed to send to broadcast channel: {e}");
            }
        });
    }
}