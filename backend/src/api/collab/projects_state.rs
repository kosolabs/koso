use super::{
    client::{ClientClosure, ClientSender, CLOSE_RESTART},
    doc_observer::YrsUpdate,
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
        Arc,
    },
};
use tokio::sync::{mpsc::Sender, Mutex};
use yrs::{Doc, ReadTxn as _, StateVector, Transact as _};

pub struct ProjectsState {
    pub projects: DashMap<ProjectId, Arc<ProjectState>>,
    pub doc_update_tx: Sender<YrsUpdate>,
    pub pool: &'static PgPool,
    pub tracker: tokio_util::task::TaskTracker,
}

impl ProjectsState {
    pub fn get_or_insert(&self, project_id: &ProjectId) -> Arc<ProjectState> {
        self.projects
            .entry(project_id.to_string())
            .or_insert_with(|| {
                Arc::new(ProjectState {
                    project_id: project_id.to_string(),
                    clients: Mutex::new(HashMap::new()),
                    doc_box: Mutex::new(None),
                    updates: atomic::AtomicUsize::new(0),
                    doc_update_tx: self.doc_update_tx.clone(),
                    pool: self.pool,
                    tracker: self.tracker.clone(),
                })
            })
            .clone()
    }

    pub fn get(&self, project_id: &ProjectId) -> Option<Arc<ProjectState>> {
        self.projects.get(project_id).map(|p| p.clone())
    }

    pub async fn close(&self) {
        for project in self.projects.iter() {
            project.close_all().await;
        }
        self.projects.clear();
    }
}

pub struct ProjectState {
    pub project_id: ProjectId,
    pub clients: Mutex<HashMap<String, ClientSender>>,
    pub doc_box: Mutex<Option<DocBox>>,
    pub updates: atomic::AtomicUsize,
    pub doc_update_tx: Sender<YrsUpdate>,
    pub pool: &'static PgPool,
    pub tracker: tokio_util::task::TaskTracker,
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
            project: Arc::clone(project),
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
                let mut doc_box = self.doc_box.lock().await;
                *doc_box = None;

                // Set updates back to 0 while holding the doc_box mutex to avoid
                // interleaving with load_graph.
                let updates = self.updates.swap(0, Relaxed);
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
        match client {
            Some(mut client) => {
                client.close(closure.code, closure.reason).await;
            }
            None => {
                tracing::error!(
                    "Unexpectedly, received close for client while no client was registered."
                );
            }
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
        // Drop the doc box to stop observing changes.
        *self.doc_box.lock().await = None;
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
