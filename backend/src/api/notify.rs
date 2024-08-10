use super::collab::doc_update_processor::DocUpdateProcessor;
use super::collab::yrs_message_processor::YrsMessageProcessor;
use super::collab::{client::ClientSender, storage};
use crate::{
    api::{
        self,
        collab::{
            client::{
                from_socket, ClientClosure, ClientReceiver, CLOSE_ERROR, CLOSE_NORMAL,
                CLOSE_RESTART, CLOSE_UNAUTHORIZED,
            },
            doc_observer::DocObserver,
            msg_sync::sync_request,
        },
        google::User,
        model::ProjectId,
    },
    postgres::compact,
};
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use sqlx::PgPool;
use std::{
    collections::HashMap,
    fmt,
    net::SocketAddr,
    ops::ControlFlow,
    sync::{
        atomic::{self, Ordering::Relaxed},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::types::ToJson;
use yrs::{Doc, ReadTxn, StateVector, Transact};

pub fn start(pool: &'static PgPool) -> Notifier {
    let (process_tx, process_rx) = mpsc::channel::<YrsMessage>(1);
    let (doc_update_tx, doc_update_rx) = mpsc::channel::<YrsUpdate>(50);
    let notifier = Notifier {
        state: Arc::new(ProjectsState {
            projects: DashMap::new(),
        }),
        pool,
        process_tx,
        doc_update_tx,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    };

    let doc_update_processor = DocUpdateProcessor {
        state: Arc::clone(&notifier.state),
        pool,
        doc_update_rx,
        cancel: notifier.cancel.clone(),
    };
    notifier
        .tracker
        .spawn(doc_update_processor.process_doc_updates());

    let yrs_message_processor = YrsMessageProcessor {
        process_rx,
        state: Arc::clone(&notifier.state),
        cancel: notifier.cancel.clone(),
    };
    notifier
        .tracker
        .spawn(yrs_message_processor.process_messages());

    notifier
}

pub struct ProjectsState {
    projects: DashMap<ProjectId, Arc<ProjectState>>,
}

pub struct ProjectState {
    pub project_id: ProjectId,
    pub clients: Mutex<HashMap<String, ClientSender>>,
    pub doc_box: Mutex<Option<DocBox>>,
    pub updates: atomic::AtomicUsize,
}

pub struct DocBox {
    pub doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send>,
}

pub struct YrsMessage {
    pub who: String,
    pub project_id: ProjectId,
    pub id: String,
    pub data: Vec<u8>,
}

pub struct YrsUpdate {
    pub who: String,
    pub project_id: ProjectId,
    pub id: String,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct Notifier {
    state: Arc<ProjectsState>,
    pool: &'static PgPool,
    process_tx: Sender<YrsMessage>,
    doc_update_tx: Sender<YrsUpdate>,
    cancel: CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first client being registered, load everything from the database, and construct the initial doc.
// For every client that joins, send the current graph as the initial state vector.
// When a client sends an update, apply the update to the doc, store it in the database
// and broadcast it to other clients.
// When the last client disconnects, consider destroying the graph.
impl Notifier {
    #[tracing::instrument(skip(self, socket, who, user), fields(who))]
    pub async fn register_client(
        self,
        socket: WebSocket,
        who: SocketAddr,
        project_id: ProjectId,
        user: User,
    ) -> Result<()> {
        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        tracing::Span::current().record("who", &who);
        tracing::debug!("Registering client");

        let (mut sender, receiver) = from_socket(socket, &who, &project_id);

        // Before doing anything else, make sure the user has access to the project.
        if let Err(e) = api::verify_access(self.pool, user, &project_id).await {
            sender.close(CLOSE_UNAUTHORIZED, "Unauthorized.").await;
            return Err(e.as_err());
        }

        // Get of insert the project state.
        let project = self.state.get_or_insert(&project_id);

        // Store the sender side of the socket in the list of clients.
        // It's important to add the client before calling init_doc_box to avoid
        // races with remove_and_close_client. e.g. a client present without an initialized doc box.
        if let Some(mut existing) = project.add_client(sender).await {
            existing
                .close(CLOSE_ERROR, "Unexpected duplicate connection.")
                .await;
            tracing::error!("Unexpectedly, client already exists: {existing:?}");
            // Intentionally fall through below and start listening for messages on the new sender.
        };

        // Init the doc_box, if necessary and grab the state vector.
        let sv = match self.init_doc_box(&project).await {
            Ok(sv) => sv,
            Err(e) => {
                self.remove_and_close_client(
                    &project_id,
                    &who,
                    ClientClosure {
                        code: CLOSE_ERROR,
                        reason: "Failed to init doc.",
                        details: format!("Failed to init doc: {e}"),
                    },
                )
                .await;
                return Err(anyhow!("Failed to init doc: {e}"));
            }
        };

        // Send the entire state vector to the client.
        tracing::debug!("Sending sync_request message to client");
        if let Err(e) = project.send_msg(&who, sync_request(sv)).await {
            self.remove_and_close_client(
                &project_id,
                &who,
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
        self.tracker
            .spawn(self.clone().receive_messages_from_client(receiver));

        Ok(())
    }

    async fn init_doc_box(&self, project: &Arc<ProjectState>) -> Result<StateVector> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.doc.transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        tracing::debug!("Initializing new YDoc");
        let (doc, update_count) = storage::load_doc(&project.project_id, self.pool).await?;
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        project.updates.store(update_count, Relaxed);

        // Persist and broadcast update events by subscribing to the callback.
        let observer = DocObserver {
            project: Arc::clone(project),
            tracker: self.tracker.clone(),
            doc_update_tx: self.doc_update_tx.clone(),
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

    /// Listen for update or close messages sent by a client.
    #[tracing::instrument(skip(self))]
    async fn receive_messages_from_client(self, mut receiver: ClientReceiver) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = receiver.next() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    if let ControlFlow::Break(_) = self.receive_message_from_client(&receiver, msg).await {
                        break;
                    }
                }
            }
        }
        tracing::debug!("Stopped receiving messages from client");
    }

    async fn receive_message_from_client(
        &self,
        receiver: &ClientReceiver,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        match msg {
            Ok(Message::Binary(data)) => {
                if let Err(e) = self
                    .process_tx
                    .send(YrsMessage {
                        who: receiver.who.clone(),
                        project_id: receiver.project_id.clone(),
                        id: Uuid::new_v4().to_string(),
                        data,
                    })
                    .await
                {
                    tracing::error!("Error sending message to process channel: {e}");
                };
                ControlFlow::Continue(())
            }
            Ok(Message::Close(c)) => {
                let details = if let Some(cf) = &c {
                    format!(
                        "Client closed connection: code:'{}', detail:'{}'",
                        cf.code, cf.reason
                    )
                } else {
                    "Client closed connection: code:'NONE', detail:'No CloseFrame'".to_string()
                };
                self.remove_and_close_client(
                    &receiver.project_id,
                    &receiver.who,
                    ClientClosure {
                        code: CLOSE_NORMAL,
                        reason: "Client closed connection.",
                        details,
                    },
                )
                .await;
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                self.remove_and_close_client(
                    &receiver.project_id,
                    &receiver.who,
                    ClientClosure {
                        code: CLOSE_ERROR,
                        reason: "Failed to read from client socket.",
                        details: format!("Failed to read from client socket: {e}"),
                    },
                )
                .await;
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::warn!("Discarding unsolicited message: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }

    async fn remove_and_close_client(
        &self,
        project_id: &ProjectId,
        who: &String,
        closure: ClientClosure,
    ) {
        let Some(project) = self.state.get(project_id) else {
            tracing::warn!("Tried to remove client but project was None.");
            return;
        };

        let client = {
            let clients = &mut project.clients.lock().await;
            let client = clients.remove(who);

            let remaining_clients = clients.len();
            tracing::debug!(
                "Removed client. {} clients remain. Reason: {}",
                remaining_clients,
                closure.details,
            );

            if remaining_clients == 0 {
                tracing::debug!("Last client disconnected, destroying YGraph");
                let mut doc_box = project.doc_box.lock().await;
                *doc_box = None;

                // Set updates back to 0 while holding the doc_box mutex to avoid
                // interleaving with load_graph.
                let updates = project.updates.swap(0, Relaxed);
                if updates > 10 {
                    self.tracker
                        .spawn(compact(self.pool, project.project_id.clone()));
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

    #[tracing::instrument(skip(self))]
    pub async fn stop(&self) {
        // Cancel background tasks and wait for them to complete.
        tracing::info!(
            "Waiting for {} outstanding task(s) to finish..",
            self.tracker.len()
        );
        self.cancel.cancel();
        self.tracker.close();
        if let Err(e) = tokio::time::timeout(Duration::from_secs(15), self.tracker.wait()).await {
            tracing::warn!(
                "Timed out waiting for tasks. {} remain: {e}",
                self.tracker.len()
            );
        }

        // Close all the project state, including client connections.
        self.state.close().await;
    }

    pub async fn get_doc(&self, project_id: &ProjectId) -> Result<yrs::Any, Error> {
        let (doc, _) = storage::load_doc(project_id, self.pool).await?;
        let txn = doc.transact();
        let Some(graph) = txn.get_map("graph") else {
            return Err(anyhow!("No graph present in doc"));
        };
        Ok(graph.to_json(&txn))
    }
}

impl ProjectsState {
    fn get_or_insert(&self, project_id: &ProjectId) -> Arc<ProjectState> {
        self.projects
            .entry(project_id.to_string())
            .or_insert_with(|| {
                Arc::new(ProjectState {
                    project_id: project_id.to_string(),
                    clients: Mutex::new(HashMap::new()),
                    doc_box: Mutex::new(None),
                    updates: atomic::AtomicUsize::new(0),
                })
            })
            .clone()
    }

    pub fn get(&self, project_id: &ProjectId) -> Option<Arc<ProjectState>> {
        self.projects.get(project_id).map(|p| p.clone())
    }

    async fn close(&self) {
        for project in self.projects.iter() {
            project.close_all().await;
        }
        self.projects.clear();
    }
}

impl ProjectState {
    async fn add_client(&self, sender: ClientSender) -> Option<ClientSender> {
        self.clients.lock().await.insert(sender.who.clone(), sender)
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
    async fn close_all(&self) {
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

impl DocBox {
    pub fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}

impl fmt::Debug for YrsMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsMessage")
            .field("project_id", &self.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}

impl fmt::Debug for YrsUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsUpdate")
            .field("project_id", &self.project_id)
            .field("who", &self.who)
            .field("id", &self.id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
