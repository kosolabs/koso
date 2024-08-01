use crate::postgres::compact;
use anyhow::anyhow;
use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
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
use yrs::Origin;
use yrs::{
    encoding::{read::Read as _, write::Write as _},
    updates::{
        decoder::{Decode, DecoderV1},
        encoder::{Encode, Encoder as _, EncoderV1},
    },
    Doc, ReadTxn, StateVector, Transact, Update,
};

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

const MSG_SYNC: u8 = 0;
// const MSG_AWARENESS: u8 = 1;
// const MSG_AUTH: u8 = 2;
// const MSG_QUERY_AWARENESS: u8 = 3;

const MSG_SYNC_REQUEST: u8 = 0;
const MSG_SYNC_RESPONSE: u8 = 1;
const MSG_SYNC_UPDATE: u8 = 2;

pub type ProjectId = String;

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
    #[tracing::instrument(skip(self, socket, who), fields(who))]
    pub async fn register_client(
        self,
        socket: WebSocket,
        who: SocketAddr,
        project_id: ProjectId,
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

        // Get of insert the project state.
        let project = self.state.get_or_insert(&project_id);

        // Init the doc_box, if necessary and grab the state vector.
        let sv = self.init_doc_box(&project).await?;

        // Send the entire state vector to the client.
        tracing::debug!("Sending sync_request message to client");
        let sync_request_msg = {
            let mut encoder = EncoderV1::new();
            encoder.write_var(MSG_SYNC);
            encoder.write_var(MSG_SYNC_REQUEST);
            encoder.write_buf(sv.encode_v1());
            encoder.to_vec()
        };
        if let Err(e) = sender.send(sync_request_msg).await {
            return Err(anyhow!("Failed to send state vector to client: {e}"));
        }

        // Store the sender side of the socket in the list of clients.
        if let Some(existing) = project.add_client(sender).await {
            return Err(anyhow!("Unexpectedly, client already exists: {existing:?}"));
        };

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
        let doc = self.load_graph(project).await?;

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

    async fn load_graph(&self, project: &ProjectState) -> Result<Doc> {
        tracing::debug!("Initializing new YDoc");
        let updates: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT update_v2 FROM yupdates WHERE project_id=$1")
                .bind(&project.project_id)
                .fetch_all(self.pool)
                .await?;
        let update_count = updates.len();
        project.updates.store(update_count, Relaxed);

        let doc = Doc::new();
        {
            let mut txn = doc.transact_mut();
            for (update,) in updates {
                txn.apply_update(Update::decode_v2(&update)?);
            }
        }
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        Result::Ok(doc)
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
                let reason = if let Some(cf) = &c {
                    format!("code:'{}', detail:'{}'", cf.code, cf.reason)
                } else {
                    "code:'NONE', detail:'No CloseFrame'".to_string()
                };
                self.close_client(&receiver.project_id, &receiver.who, &reason)
                    .await;
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                self.close_client(&receiver.project_id, &receiver.who, &format!("Error: {e}"))
                    .await;
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::warn!("Discarding unsolicited message: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }

    async fn close_client(&self, project_id: &ProjectId, who: &String, reason: &String) {
        let Some(project) = self.state.get(project_id) else {
            tracing::error!("Unexpectedly, received close for client but the project is missing.");
            return;
        };

        match project.remove_client(who, reason).await {
            Some(mut client) => {
                if let Err(e) = client.close().await {
                    tracing::debug!("Failed to close client: {e}");
                }
            }
            None => {
                tracing::error!(
                    "Unexpectedly, received close for client while no client was registered."
                )
            }
        }

        let updates = project.updates.load(Relaxed);
        if updates > 10 && project.clients.lock().await.len() == 0 {
            project.updates.store(0, Relaxed);
            self.tracker.spawn(compact(self.pool, project_id.clone()));
        } else {
            tracing::debug!("Skipping compacting, only {updates} updates exist")
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
                        let sync_response_msg = {
                            let mut encoder = EncoderV1::new();
                            encoder.write_var(MSG_SYNC);
                            encoder.write_var(MSG_SYNC_RESPONSE);
                            encoder.write_buf(update);
                            encoder.to_vec()
                        };
                        {
                            let mut clients = project.clients.lock().await;
                            let Some(client) = clients.get_mut(&msg.who) else {
                                return Err(anyhow!("Unexpectedly found no client to reply to"));
                            };
                            if let Err(e) = client.send(sync_response_msg).await {
                                return Err(anyhow!("Failed to send sync_response to client: {e}"));
                            };
                        }

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

        if let Err(e) = self.persist_update(&project, &update.data).await {
            return Err(anyhow!("Failed to persist update: {e}"));
        }
        let sync_update_msg = {
            let mut encoder = EncoderV1::new();
            encoder.write_var(MSG_SYNC);
            encoder.write_var(MSG_SYNC_UPDATE);
            encoder.write_buf(update.data);
            encoder.to_vec()
        };
        project.broadcast_msg(&update.who, sync_update_msg).await;

        Ok(())
    }

    async fn persist_update(&self, project: &ProjectState, data: &Vec<u8>) -> Result<()> {
        sqlx::query(
            "
            INSERT INTO yupdates (project_id, seq, update_v2)
            VALUES ($1, DEFAULT, $2)",
        )
        .bind(&project.project_id)
        .bind(data)
        .execute(self.pool)
        .await?;
        project.updates.fetch_add(1, Relaxed);
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

    async fn remove_client(&self, who: &String, reason: &String) -> Option<ClientSender> {
        let clients = &mut self.clients.lock().await;
        let client = clients.remove(who);

        let remaining_clients = clients.len();
        tracing::debug!(
            "Removing closed client. {} clients remain. Reason: {}",
            remaining_clients,
            reason
        );

        if remaining_clients == 0 {
            tracing::debug!("Last client disconnected, destroying YGraph");
            *self.doc_box.lock().await = None;
        }
        client
    }

    async fn broadcast_msg(&self, who: &String, data: Vec<u8>) {
        let mut clients = self.clients.lock().await;

        tracing::debug!("Broadcasting to {} clients", clients.len());
        let mut results = Vec::new();
        for client in clients.values_mut() {
            if client.who != *who {
                results.push(client.send(data.to_owned()));
            }
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished broadcasting: {res:?}");
    }

    async fn close_all(&self) {
        // Close all clients.
        let mut clients = self.clients.lock().await;
        for client in clients.values_mut() {
            let _ = client.close().await;
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

impl ClientSender {
    async fn send(&mut self, data: Vec<u8>) -> Result<(), axum::Error> {
        self.ws_sender.send(Message::Binary(data)).await
    }

    async fn close(&mut self) -> Result<(), axum::Error> {
        self.ws_sender.close().await
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
