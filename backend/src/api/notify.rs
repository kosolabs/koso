use crate::api;
use crate::api::collab::msg_sync::{
    sync_request, sync_response, sync_update, MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE,
    MSG_SYNC_UPDATE,
};
use crate::{api::google::User, api::model::ProjectId, postgres::compact};
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket};
use dashmap::DashMap;
use futures::SinkExt;
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
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::types::ToJson;
use yrs::Origin;
use yrs::{
    encoding::read::Read as _,
    updates::decoder::{Decode, DecoderV1},
    Doc, ReadTxn, StateVector, Transact, Update,
};

use super::collab::storage;

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
    notifier
        .tracker
        .spawn(notifier.clone().receive_doc_updates(doc_update_rx));
    notifier
        .tracker
        .spawn(notifier.clone().process_messages(process_rx));

    notifier
}

struct ProjectsState {
    projects: DashMap<ProjectId, Arc<ProjectState>>,
}

struct ProjectState {
    project_id: ProjectId,
    clients: Mutex<HashMap<String, ClientSender>>,
    doc_box: Mutex<Option<DocBox>>,
    updates: atomic::AtomicUsize,
}

struct ClientSender {
    ws_sender: futures::stream::SplitSink<WebSocket, Message>,
    who: String,
    project_id: ProjectId,
}

struct ClientClosure {
    code: CloseCode,
    /// Reason sent to the client.
    /// Must not contain anything sensitive.
    reason: &'static str,
    /// Additional details for internal logging.
    details: String,
}

struct ClientReceiver {
    ws_receiver: futures::stream::SplitStream<WebSocket>,
    who: String,
    project_id: ProjectId,
}

struct DocBox {
    doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send>,
}

struct YrsMessage {
    who: String,
    project_id: ProjectId,
    id: String,
    data: Vec<u8>,
}

struct YrsUpdate {
    who: String,
    project_id: ProjectId,
    id: String,
    data: Vec<u8>,
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

        use futures::stream::StreamExt;
        let (ws_sender, ws_receiver) = socket.split();
        let mut sender = ClientSender {
            ws_sender,
            who: who.clone(),
            project_id: project_id.clone(),
        };
        let receiver = ClientReceiver {
            ws_receiver,
            who: who.clone(),
            project_id: project_id.clone(),
        };

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
        let observer_notifier = self.clone();
        let observer_project = Arc::clone(project);
        let res = doc.observe_update_v2(move |txn, update| {
            observer_notifier.handle_doc_update_v2_event(&observer_project, txn, update);
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

    #[tracing::instrument(skip(self, process_rx))]
    async fn process_messages(self, mut process_rx: Receiver<YrsMessage>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = process_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    self.process_message(msg).await;
                }
            }
        }
        tracing::info!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: YrsMessage) {
        if let Err(e) = self.process_message_internal(msg).await {
            tracing::warn!("Failed to process message: {e}");
        }
    }

    async fn process_message_internal(&self, msg: YrsMessage) -> Result<()> {
        let Some(project) = self.state.get(&msg.project_id) else {
            return Err(anyhow!("Tried to handle message but project was None."));
        };

        let mut decoder = DecoderV1::from(msg.data.as_slice());
        match decoder.read_var()? {
            MSG_SYNC => {
                match decoder.read_var()? {
                    MSG_SYNC_REQUEST => {
                        tracing::debug!("Handling sync_request message");
                        let update = {
                            let sv: StateVector = StateVector::decode_v1(decoder.read_buf()?)?;
                            DocBox::doc_or_error(project.doc_box.lock().await.as_ref())?
                                .doc
                                .transact()
                                .encode_state_as_update_v2(&sv)
                        };

                        // Respond to the client with a sync_response message containing
                        // changes known to the server but not the client.
                        // There's no need to broadcast such updates to others or perist them.
                        tracing::debug!("Sending synce_response message to client.");
                        project.send_msg(&msg.who, sync_response(update)).await?;

                        Ok(())
                    }
                    MSG_SYNC_RESPONSE | MSG_SYNC_UPDATE => {
                        tracing::debug!("Handling sync_update|sync_response message");
                        let update = decoder.read_buf()?.to_vec();
                        {
                            let update = Update::decode_v2(&update)?;
                            DocBox::doc_or_error(project.doc_box.lock().await.as_ref())?
                                .doc
                                .transact_mut_with(as_origin(&msg.who, &msg.id))
                                .apply_update(update);
                        }

                        Ok(())
                    }
                    invalid_type => Err(anyhow!("Invalid sync type: {invalid_type}")),
                }
            }
            invalid_type => Err(anyhow!("Invalid message protocol type: {invalid_type}")),
        }
    }

    /// Callback invoked on update_v2 doc events, triggered by calls to "apply_update" in process_message_internal.
    /// observe_update_v2 only accepts synchronous callbacks thus requiring this function be synchronous
    /// and any async operations, including sendin to a channel, to occur in a spawned task.
    fn handle_doc_update_v2_event(
        &self,
        observer_project: &ProjectState,
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
            project_id: observer_project.project_id.clone(),
            id: origin.id,
            data: event.update.clone(),
        };
        let nn = self.clone();
        self.tracker.spawn(async move {
            if let Err(e) = nn.doc_update_tx.send(update).await {
                tracing::error!("failed to send to broadcast channel: {e}");
            }
        });
    }

    #[tracing::instrument(skip(self, doc_update_rx))]
    async fn receive_doc_updates(self, mut doc_update_rx: Receiver<YrsUpdate>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = doc_update_rx.recv() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    self.process_doc_update(msg).await;
                }
            }
        }
        tracing::info!("Stopped processing doc updates");
    }

    #[tracing::instrument(skip(self))]
    async fn process_doc_update(&self, update: YrsUpdate) {
        if let Err(e) = self.process_doc_update_internal(update).await {
            tracing::warn!("Failed to process doc update: {e}");
        }
    }

    async fn process_doc_update_internal(&self, update: YrsUpdate) -> Result<()> {
        let Some(project) = self.state.get(&update.project_id) else {
            return Err(anyhow!(
                "Unexpectedly, received close for client but the project is missing."
            ));
        };

        if let Err(e) = storage::persist_update(&project.project_id, &update.data, self.pool).await
        {
            return Err(anyhow!("Failed to persist update: {e}"));
        }
        project
            .broadcast_msg(&update.who, sync_update(update.data))
            .await;

        Ok(())
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

    fn get(&self, project_id: &ProjectId) -> Option<Arc<ProjectState>> {
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

    async fn broadcast_msg(&self, from_who: &String, data: Vec<u8>) {
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

    async fn send_msg(&self, to_who: &String, data: Vec<u8>) -> Result<()> {
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
    fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}

// https://www.rfc-editor.org/rfc/rfc6455.html#section-7.4.1
// https://www.iana.org/assignments/websocket/websocket.xhtml#close-code-number
const CLOSE_NORMAL: u16 = 1000;
const CLOSE_ERROR: u16 = 1011;
const CLOSE_RESTART: u16 = 1012;
const CLOSE_UNAUTHORIZED: u16 = 3000;

impl ClientSender {
    async fn send(&mut self, data: Vec<u8>) -> Result<(), axum::Error> {
        self.ws_sender.send(Message::Binary(data)).await
    }

    async fn close(&mut self, code: CloseCode, reason: &'static str) {
        let _ = self
            .ws_sender
            .send(Message::Close(Some(CloseFrame {
                code,
                reason: reason.into(),
            })))
            .await;
        let _ = self.ws_sender.close().await;
    }
}

impl fmt::Debug for ClientSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientSender")
            .field("who", &self.who)
            .field("project_id", &self.project_id)
            .finish()
    }
}

impl ClientReceiver {
    async fn next(&mut self) -> Option<Result<Message, axum::Error>> {
        use futures::stream::StreamExt;
        self.ws_receiver.next().await
    }
}

impl fmt::Debug for ClientReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientReceiver")
            .field("who", &self.who)
            .field("project_id", &self.project_id)
            .finish()
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

struct YOrigin {
    who: String,
    id: String,
}

fn from_origin(origin: Option<&Origin>) -> Result<YOrigin> {
    origin
        .map(|o| match String::from_utf8(o.as_ref().to_vec()) {
            Ok(v) => {
                let mut parts = v.split("@@");
                let (Some(who), Some(id)) = (parts.next(), parts.next()) else {
                    return Err(anyhow!("Could not split origin into parts: {v}"));
                };
                Ok(YOrigin {
                    who: who.to_string(),
                    id: id.to_string(),
                })
            }
            Err(e) => Err(anyhow!("Failed to parse origin bytes to string: {o}: {e}")),
        })
        .unwrap_or_else(|| Err(anyhow!("Missing origin")))
}

fn as_origin(who: &str, id: &str) -> Origin {
    format!("{who}@@{id}").into()
}
