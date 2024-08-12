use crate::{
    api::{
        collab::{
            client::{ClientClosure, ClientReceiver, ClientSender, CLOSE_ERROR, CLOSE_RESTART},
            client_message_handler::{ClientMessageHandler, YrsMessage},
            doc_observer::{DocObserver, YrsUpdate},
            msg_sync::sync_request,
            storage,
            txn_origin::YOrigin,
        },
        model::ProjectId,
    },
    postgres::compact,
};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{
        atomic::{self, Ordering::Relaxed},
        Arc, Weak,
    },
};
use tokio::sync::{mpsc::Sender, Mutex};
use yrs::{Doc, ReadTxn as _, StateVector, Transact as _, Update};

pub(super) struct ProjectsState {
    pub(super) projects: DashMap<ProjectId, Weak<ProjectState>>,
    pub(super) process_msg_tx: Sender<YrsMessage>,
    pub(super) doc_update_tx: Sender<YrsUpdate>,
    pub(super) pool: &'static PgPool,
    pub(super) tracker: tokio_util::task::TaskTracker,
}

impl ProjectsState {
    pub(super) async fn add_client(
        &self,
        project_id: &ProjectId,
        mut sender: ClientSender,
        receiver: ClientReceiver,
    ) -> Result<()> {
        // Get of insert the project state.
        let (project, sv) = match self.get_or_init(project_id).await {
            Ok(r) => r,
            Err(e) => {
                sender.close(CLOSE_ERROR, "Failed to init project.").await;
                return Err(e);
            }
        };

        // Store the sender side of the socket in the list of clients.
        if let Some(mut existing) = project.add_client(sender).await {
            existing
                .close(CLOSE_ERROR, "Unexpected duplicate connection.")
                .await;
            tracing::error!("Unexpectedly, client already exists: {existing:?}");
            // Intentionally fall through below and start listening for messages on the new sender.
        };

        // Send the entire state vector to the client.
        tracing::debug!("Sending sync_request message to client");
        if let Err(e) = project.send_msg(&receiver.who, sync_request(&sv)).await {
            project
                .remove_and_close_client(
                    &receiver.who,
                    ClientClosure {
                        code: CLOSE_ERROR,
                        reason: "Failed to send sync request message.",
                        details: format!("Failed to send sync request message: {e}"),
                    },
                )
                .await;
            return Err(anyhow!(
                "Failed to send sync request message to client: {e}"
            ));
        }

        // Listen for messages on the read side of the socket.
        let handler = ClientMessageHandler {
            project: Arc::clone(&project),
            process_msg_tx: self.process_msg_tx.clone(),
            receiver,
        };
        self.tracker.spawn(handler.receive_messages_from_client());

        Ok(())
    }

    async fn get_or_init(
        &self,
        project_id: &ProjectId,
    ) -> Result<(Arc<ProjectState>, StateVector)> {
        let project = match self.projects.entry(project_id.to_string()) {
            dashmap::Entry::Occupied(mut o) => match o.get().upgrade() {
                Some(p) => p,
                None => {
                    let project = Arc::new(ProjectState {
                        project_id: project_id.to_string(),
                        clients: Mutex::new(HashMap::new()),
                        doc_box: Mutex::new(None),
                        doc_update_tx: self.doc_update_tx.clone(),
                        updates: atomic::AtomicUsize::new(0),
                        pool: self.pool,
                        tracker: self.tracker.clone(),
                    });
                    o.insert(Arc::downgrade(&project));
                    project
                }
            },
            dashmap::Entry::Vacant(v) => {
                let project = Arc::new(ProjectState {
                    project_id: project_id.to_string(),
                    clients: Mutex::new(HashMap::new()),
                    doc_box: Mutex::new(None),
                    doc_update_tx: self.doc_update_tx.clone(),
                    updates: atomic::AtomicUsize::new(0),
                    pool: self.pool,
                    tracker: self.tracker.clone(),
                });
                v.insert(Arc::downgrade(&project));
                project
            }
        };

        // Init the doc_box, if necessary and grab the state vector.
        let sv = ProjectState::init_doc_box(&project).await?;
        Ok((project, sv))
    }

    pub(super) async fn close(&self) {
        let iter = self.projects.iter();
        let mut res = Vec::new();
        for project in iter {
            if let Some(project) = project.upgrade() {
                res.push(ProjectState::close_all(project));
            }
        }
        futures::future::join_all(res).await;
    }
}

pub(super) struct ProjectState {
    pub(super) project_id: ProjectId,
    clients: Mutex<HashMap<String, ClientSender>>,
    doc_box: Mutex<Option<DocBox>>,
    updates: atomic::AtomicUsize,
    doc_update_tx: Sender<YrsUpdate>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl ProjectState {
    async fn add_client(&self, sender: ClientSender) -> Option<ClientSender> {
        self.clients.lock().await.insert(sender.who.clone(), sender)
    }

    async fn init_doc_box(project: &Arc<ProjectState>) -> Result<StateVector> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.doc.transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        tracing::debug!("Initializing new YDoc");
        let (doc, update_count) = storage::load_doc(&project.project_id, project.pool).await?;
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        project.updates.store(update_count, Relaxed);

        // Persist and broadcast update events by subscribing to the callback.
        let observer = DocObserver {
            doc_update_tx: project.doc_update_tx.clone(),
            tracker: project.tracker.clone(),
        };
        let observer_project = Arc::downgrade(project);
        let res = doc.observe_update_v2(move |txn, update| {
            let Some(project) = observer_project.upgrade() else {
                // This will never happen because the observer is invoked syncronously in
                // ProjectState.apply_update while holding a strong reference to the project.
                tracing::error!(
                    "handle_doc_update_v2_event but weak project reference was destroyed"
                );
                return;
            };
            project.updates.fetch_add(1, Relaxed);
            observer.handle_doc_update_v2_event(project, txn, update);
        });
        let sub = match res {
            Ok(sub) => Box::new(sub),
            Err(e) => return Err(anyhow!("Failed to create observer: {e}")),
        };

        let db = DocBox { doc, sub };
        let sv = db.doc.transact().state_vector();
        *doc_box = Some(db);
        Ok(sv)
    }

    pub(super) async fn encode_state_as_update(&self, sv: &StateVector) -> Result<Vec<u8>> {
        let update = DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .doc
            .transact()
            .encode_state_as_update_v2(sv);
        Ok(update)
    }
    pub(super) async fn apply_doc_update(&self, origin: YOrigin, update: Update) -> Result<()> {
        DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .doc
            .transact_mut_with(origin.as_origin())
            .apply_update(update);
        Ok(())
    }
    pub(super) async fn broadcast_msg(&self, from_who: &String, data: Vec<u8>) {
        let mut clients = self.clients.lock().await;

        tracing::debug!("Broadcasting to {} clients", clients.len());
        let mut results = Vec::new();
        for client in clients.values_mut() {
            if client.who != *from_who {
                results.push(client.send(data.to_owned()));
            }
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished broadcasting: {res:?}");
    }

    pub(super) async fn send_msg(&self, to_who: &String, data: Vec<u8>) -> Result<()> {
        let mut clients = self.clients.lock().await;
        let Some(client) = clients.get_mut(to_who) else {
            return Err(anyhow!("Unexpectedly found no client to send to"));
        };
        if let Err(e) = client.send(data).await {
            return Err(anyhow!("Failed to send to client: {e}"));
        };
        Ok(())
    }

    pub(super) async fn remove_and_close_client(&self, who: &String, closure: ClientClosure) {
        let (client, remaining_clients) = {
            let clients = &mut self.clients.lock().await;
            (clients.remove(who), clients.len())
        };
        tracing::debug!(
            "Removed client. {} clients remain. Reason: {}",
            remaining_clients,
            closure.details,
        );

        match client {
            Some(mut client) => {
                client.close(closure.code, closure.reason).await;
            }
            None => tracing::warn!(
                "Tried to remove client ({who}) in project {}, but it was already gone.",
                self.project_id
            ),
        }
    }

    async fn close_all(project: Arc<ProjectState>) {
        let mut clients = project.clients.lock().await;
        tracing::debug!(
            "Closing {} clients in project {}",
            clients.len(),
            project.project_id
        );
        let mut res = Vec::new();
        for client in clients.values_mut() {
            res.push(client.close(CLOSE_RESTART, "The server is shutting down."));
        }
        futures::future::join_all(res).await;
    }
}

impl Drop for ProjectState {
    fn drop(&mut self) {
        tracing::debug!(
            "Last client disconnected, destroying project state: {}",
            self.project_id
        );

        let updates = self.updates.load(Relaxed);
        if updates > 10 {
            self.tracker
                .spawn(compact(self.pool, self.project_id.clone()));
        } else {
            tracing::debug!("Skipping compacting, only {updates} updates exist")
        }
    }
}

struct DocBox {
    doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send>,
}

impl DocBox {
    fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}
