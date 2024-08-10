use tokio::sync::mpsc::Sender;
use tokio_util::task::TaskTracker;

use crate::api::collab::txn_origin::from_origin;
use std::{fmt, sync::Arc};

use super::projects_state::ProjectState;

pub(super) struct DocObserver {
    pub(super) doc_update_tx: Sender<YrsUpdate>,
    pub(super) tracker: TaskTracker,
}

impl DocObserver {
    /// Callback invoked on update_v2 doc events, triggered by calls to "apply_update" in process_message_internal.
    /// observe_update_v2 only accepts synchronous callbacks thus requiring this function be synchronous
    /// and any async operations, including sendin to a channel, to occur in a spawned task.
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
        let update = YrsUpdate {
            who: origin.who,
            project,
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

pub(super) struct YrsUpdate {
    pub(super) who: String,
    pub(super) project: Arc<ProjectState>,
    pub(super) id: String,
    pub(super) data: Vec<u8>,
}

impl fmt::Debug for YrsUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsUpdate")
            .field("project_id", &self.project.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
