use crate::{
    api::{
        collab::{
            client::{
                ClientClosure, ClientReceiver, ClientSender, CLOSE_ERROR, CLOSE_RESTART, OVERLOADED,
            },
            client_messages::{ClientMessage, ClientMessageReceiver},
            doc_updates::{DocObserver, DocUpdate},
            msg_sync::sync_request,
            storage,
            txn_origin::YOrigin,
        },
        model::ProjectId,
    },
    postgres::compact,
};
use anyhow::{anyhow, Error, Result};
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
use yrs::{ReadTxn as _, StateVector, Update};

use super::{awareness::Awareness, msg_sync::koso_awareness_state, YDocProxy};

pub(super) struct ProjectsState {
    projects: DashMap<ProjectId, Weak<ProjectState>>,
    process_msg_tx: Sender<ClientMessage>,
    doc_update_tx: Sender<DocUpdate>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl ProjectsState {
    pub(super) fn new(
        process_msg_tx: Sender<ClientMessage>,
        doc_update_tx: Sender<DocUpdate>,
        pool: &'static PgPool,
        tracker: tokio_util::task::TaskTracker,
    ) -> Self {
        ProjectsState {
            projects: DashMap::new(),
            process_msg_tx,
            doc_update_tx,
            pool,
            tracker,
        }
    }

    pub(super) async fn add_and_init_local_client(
        &self,
        project_id: &ProjectId,
    ) -> Result<Arc<ProjectState>> {
        let (project, _) = self.get_or_init(project_id).await?;
        Ok(project)
    }

    pub(super) async fn add_and_init_client(
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
        match project.insert_client(sender).await {
            Ok(None) => {}
            Ok(Some(mut existing)) => {
                existing
                    .close(CLOSE_ERROR, "Unexpected duplicate connection.")
                    .await;
                tracing::error!("Unexpectedly, client already exists: {existing:?}");
                // Intentionally fall through below and start listening for messages on the new sender.
            }
            Err((mut sender, e)) => {
                sender.close(OVERLOADED, "Too many active clients.").await;
                return Err(e);
            }
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
        let handler =
            ClientMessageReceiver::new(Arc::clone(&project), self.process_msg_tx.clone(), receiver);
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
                    let project = self.new_project(project_id);
                    o.insert(Arc::downgrade(&project));
                    project
                }
            },
            dashmap::Entry::Vacant(v) => {
                let project = self.new_project(project_id);
                v.insert(Arc::downgrade(&project));
                project
            }
        };

        // Init the doc_box, if necessary and grab the state vector.
        let sv = ProjectState::init_doc_box(&project).await?;
        Ok((project, sv))
    }

    fn new_project(&self, project_id: &String) -> Arc<ProjectState> {
        Arc::new(ProjectState {
            project_id: project_id.to_string(),
            clients: Mutex::new(HashMap::new()),
            awarenesses: Mutex::new(HashMap::new()),
            doc_box: Mutex::new(None),
            doc_update_tx: self.doc_update_tx.clone(),
            updates: atomic::AtomicUsize::new(0),
            pool: self.pool,
            tracker: self.tracker.clone(),
        })
    }

    pub(super) async fn close_all_project_clients(&self) {
        let mut res = Vec::new();
        for project in self.projects.iter() {
            if let Some(project) = project.upgrade() {
                res.push(ProjectState::close_all_clients(project));
            }
        }
        futures::future::join_all(res).await;
    }
}

pub(crate) struct ProjectState {
    pub(crate) project_id: ProjectId,
    clients: Mutex<HashMap<String, ClientSender>>,
    awarenesses: Mutex<HashMap<String, Awareness>>,
    pub(crate) doc_box: Mutex<Option<DocBox>>,
    updates: atomic::AtomicUsize,
    doc_update_tx: Sender<DocUpdate>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl ProjectState {
    async fn insert_client(
        &self,
        sender: ClientSender,
    ) -> Result<Option<ClientSender>, (ClientSender, Error)> {
        let mut clients = self.clients.lock().await;
        const MAX_PROJECT_CLIENTS: usize = 100;
        if clients.len() >= MAX_PROJECT_CLIENTS {
            return Err((
                sender,
                anyhow!("Too many ({}) active project clients.", clients.len()),
            ));
        }
        Ok(clients.insert(sender.who.clone(), sender))
    }

    async fn init_doc_box(project: &Arc<ProjectState>) -> Result<StateVector> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.ydoc.transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        tracing::debug!("Initializing new YDoc");
        let (doc, update_count) = storage::load_doc(&project.project_id, project.pool).await?;
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        project.updates.store(update_count, Relaxed);

        // Persist and broadcast update events by subscribing to the callback.
        let observer = DocObserver::new(project.doc_update_tx.clone(), project.tracker.clone());
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

        let db = DocBox { ydoc: doc, sub };
        let sv = db.ydoc.transact().state_vector();
        *doc_box = Some(db);
        Ok(sv)
    }

    pub(super) async fn encode_state_as_update(&self, sv: &StateVector) -> Result<Vec<u8>> {
        let update = DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .ydoc
            .transact()
            .encode_state_as_update_v2(sv);
        Ok(update)
    }
    pub(super) async fn apply_doc_update(&self, origin: YOrigin, update: Update) -> Result<()> {
        if let Err(e) = DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .ydoc
            .transact_mut_with(origin.as_origin())
            .apply_update(update)
        {
            return Err(anyhow!("Failed to apply doc update: {e}"));
        }
        Ok(())
    }
    pub(super) async fn broadcast_msg(&self, data: Vec<u8>, exclude_who: Option<&String>) {
        let mut clients = self.clients.lock().await;

        tracing::debug!("Broadcasting to {} clients", clients.len());
        let mut results = Vec::new();
        for client in clients.values_mut() {
            match exclude_who {
                Some(exclude_who) if client.who == *exclude_who => {}
                _ => results.push(client.send(data.to_owned())),
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

        self.awarenesses.lock().await.remove(who);
        self.broadcast_awarenesses().await;

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

    async fn close_all_clients(project: Arc<ProjectState>) {
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

    pub(super) async fn update_awareness(&self, who: &str, awareness: Awareness) {
        self.awarenesses.lock().await.insert(who.into(), awareness);
        self.broadcast_awarenesses().await;
    }

    async fn broadcast_awarenesses(&self) {
        let Ok(state) =
            serde_json::to_string(&self.awarenesses.lock().await.values().collect::<Vec<_>>())
        else {
            tracing::warn!("Failed to serialize awarenesses");
            return;
        };
        let msg = koso_awareness_state(&state);
        self.broadcast_msg(msg, None).await;
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

pub(crate) struct DocBox {
    pub(crate) ydoc: YDocProxy,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send>,
}

impl DocBox {
    pub(crate) fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}
