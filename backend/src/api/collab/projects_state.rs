use super::{
    client::{ClientClosure, ClientSender, CLOSE_RESTART},
    doc_observer::YrsUpdate,
    txn_origin::YOrigin,
};
use crate::{
    api::{
        collab::{doc_observer::DocObserver, storage},
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

pub struct ProjectsState {
    pub projects: DashMap<ProjectId, Weak<ProjectState>>,
    pub doc_update_tx: Sender<YrsUpdate>,
    pub pool: &'static PgPool,
    pub tracker: tokio_util::task::TaskTracker,
}

impl ProjectsState {
    pub fn get_or_insert(&self, project_id: &ProjectId) -> Arc<ProjectState> {
        let entry = self.projects.entry(project_id.to_string());

        let project;
        match entry {
            dashmap::Entry::Occupied(mut o) => match o.get().upgrade() {
                Some(p) => project = p,
                None => {
                    project = Arc::new(ProjectState {
                        project_id: project_id.to_string(),
                        clients: Mutex::new(HashMap::new()),
                        doc_box: Mutex::new(None),
                        doc_update_tx: self.doc_update_tx.clone(),
                        updates: atomic::AtomicUsize::new(0),
                        pool: self.pool,
                        tracker: self.tracker.clone(),
                    });
                    o.insert(Arc::downgrade(&project));
                }
            },
            dashmap::Entry::Vacant(v) => {
                project = Arc::new(ProjectState {
                    project_id: project_id.to_string(),
                    clients: Mutex::new(HashMap::new()),
                    doc_box: Mutex::new(None),
                    doc_update_tx: self.doc_update_tx.clone(),
                    updates: atomic::AtomicUsize::new(0),
                    pool: self.pool,
                    tracker: self.tracker.clone(),
                });
                v.insert(Arc::downgrade(&project));
            }
        }
        project
    }

    pub async fn close(&self) {
        for project in self.projects.iter() {
            if let Some(project) = project.upgrade() {
                project.close_all().await;
            }
        }
        self.projects.clear();
    }
}

pub struct ProjectState {
    pub project_id: ProjectId,
    clients: Mutex<HashMap<String, ClientSender>>,
    doc_box: Mutex<Option<DocBox>>,
    updates: atomic::AtomicUsize,
    doc_update_tx: Sender<YrsUpdate>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl ProjectState {
    pub async fn add_client(&self, sender: ClientSender) -> Option<ClientSender> {
        self.clients.lock().await.insert(sender.who.clone(), sender)
    }
    pub async fn init_doc_box(project: &Arc<ProjectState>) -> Result<StateVector> {
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
            project: Arc::downgrade(project),
            doc_update_tx: project.doc_update_tx.clone(),
            tracker: project.tracker.clone(),
        };
        let res = doc.observe_update_v2(move |txn, update| {
            observer.handle_doc_update_v2_event(txn, update);
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

    pub async fn encode_state_as_update(&self, sv: &StateVector) -> Result<Vec<u8>> {
        let update = DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .doc
            .transact()
            .encode_state_as_update_v2(sv);
        Ok(update)
    }
    pub async fn apply_doc_update(&self, origin: YOrigin, update: Update) -> Result<()> {
        DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .doc
            .transact_mut_with(origin.as_origin())
            .apply_update(update);
        Ok(())
    }
    pub async fn broadcast_msg(&self, from_who: &String, data: Vec<u8>) {
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

    pub async fn send_msg(&self, to_who: &String, data: Vec<u8>) -> Result<()> {
        let mut clients = self.clients.lock().await;
        let Some(client) = clients.get_mut(to_who) else {
            return Err(anyhow!("Unexpectedly found no client to send to"));
        };
        if let Err(e) = client.send(data).await {
            return Err(anyhow!("Failed to send to client: {e}"));
        };
        Ok(())
    }

    pub async fn remove_and_close_client(&self, who: &String, closure: ClientClosure) {
        let client = {
            let clients = &mut self.clients.lock().await;
            let client = clients.remove(who);

            let remaining_clients = clients.len();
            tracing::debug!(
                "Removed client. {} clients remain. Reason: {}",
                remaining_clients,
                closure.details,
            );

            if remaining_clients == 0 {
                tracing::debug!("Last client disconnected, destroying YGraph");
                // Set updates back to 0 while holding the doc_box mutex to avoid
                // interleaving with load_graph.
                let updates = self.updates.load(Relaxed);
                if updates > 10 {
                    self.tracker
                        .spawn(compact(self.pool, self.project_id.clone()));
                } else {
                    tracing::debug!("Skipping compacting, only {updates} updates exist")
                }
            }
            client
        };

        // Close the client after releasing locks.
        if let Some(mut client) = client {
            client.close(closure.code, closure.reason).await;
        }
    }

    pub async fn close_all(&self) {
        // Close all clients.
        let mut clients = self.clients.lock().await;
        for client in clients.values_mut() {
            client
                .close(CLOSE_RESTART, "The server is shutting down.")
                .await;
        }
        clients.clear();
    }
}

impl Drop for ProjectState {
    fn drop(&mut self) {
        tracing::debug!("Destroying project state: {}", self.project_id);
    }
}

pub struct DocBox {
    pub doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    pub sub: Box<dyn Send>,
}

impl DocBox {
    pub fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}
