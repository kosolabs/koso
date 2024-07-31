use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::SinkExt;
use sqlx::PgPool;
use std::{
    collections::HashMap, error::Error, fmt, net::SocketAddr, ops::ControlFlow, sync::Arc,
    time::Duration,
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::{
    sync::{self, Awareness, DefaultProtocol, Protocol},
    updates::{decoder::Decode, encoder::Encode},
    Doc, ReadTxn, StateVector, Transact, Update,
};

pub fn start(pool: &'static PgPool) -> Notifier {
    let (process_tx, process_rx) = mpsc::channel::<YrsMessage>(1);
    let notifier = Notifier {
        state: Arc::new(ProjectsState {
            projects: DashMap::new(),
        }),
        pool,
        process_tx,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    };
    notifier
        .tracker
        .spawn(notifier.clone().process_messages(process_rx));

    notifier
}

type ProjectId = String;

struct ProjectsState {
    projects: DashMap<ProjectId, Arc<ProjectState>>,
}

struct ProjectState {
    project_id: ProjectId,
    clients: Mutex<HashMap<String, ClientSender>>,
    doc_box: Mutex<Option<DocBox>>,
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
    awareness: Awareness,
}

struct YrsMessage {
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
    ) -> Result<(), Box<dyn Error>> {
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
        let sv = sync::Message::Sync(sync::SyncMessage::SyncStep1(sv)).encode_v1();
        tracing::debug!("Sending SyncStep1 message to client");
        if let Err(e) = sender.send(sv).await {
            return Err(format!("Failed to send state vector to client: {e}").into());
        }

        // Store the sender side of the socket in the list of clients.
        if let Some(existing) = project.add_client(sender).await {
            return Err(format!("Unexpectedly, client already exists: {existing:?}").into());
        };

        // Listen for messages on the read side of the socket.
        self.tracker
            .spawn(self.clone().receive_messages_from_client(receiver));

        Ok(())
    }

    async fn init_doc_box(&self, project: &ProjectState) -> Result<StateVector, Box<dyn Error>> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.awareness.doc().transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        let awareness = Awareness::new(self.load_graph(&project.project_id).await?);

        let db = DocBox { awareness };
        let sv = db.awareness.doc().transact().state_vector();
        *doc_box = Some(db);
        Ok(sv)
    }

    async fn load_graph(&self, project_id: &ProjectId) -> Result<Doc, Box<dyn Error>> {
        tracing::debug!("Initializing new YDoc");
        let updates: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT update_v2 FROM yupdates WHERE project_id=$1")
                .bind(project_id)
                .fetch_all(self.pool)
                .await?;
        let update_count = updates.len();

        let doc = Doc::new();
        {
            let mut txn = doc.transact_mut();
            for (update,) in updates {
                txn.apply_update(Update::decode_v1(&update)?);
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
    }

    /// Receive messages from `process_rx`, apply them to the doc and broadcast to all clients.
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
        tracing::debug!("Stopped processing messages");
    }

    #[tracing::instrument(skip(self))]
    async fn process_message(&self, msg: YrsMessage) {
        if let Err(e) = self.process_message_internal(msg).await {
            tracing::warn!("Failed to process message: {e}");
        }
    }

    async fn process_message_internal(&self, msg: YrsMessage) -> Result<(), Box<dyn Error>> {
        let Some(project) = self.state.get(&msg.project_id) else {
            return Err("Tried to handle message but project was None."
                .to_string()
                .into());
        };

        let reply = {
            let protocol = DefaultProtocol {};
            let sync_message = match sync::Message::decode_v1(&msg.data)? {
                sync::Message::Sync(m) => m,
                m => return Err(format!("Unimplemented message protocol type: {m:?}").into()),
            };

            let mut doc_box = project.doc_box.lock().await;
            let Some(doc_box) = doc_box.as_mut() else {
                return Err("Tried to handle message but doc_box was None.".into());
            };
            match sync_message {
                sync::SyncMessage::SyncStep1(sv) => {
                    tracing::debug!("Handling SyncStep1 message");
                    protocol.handle_sync_step1(&doc_box.awareness, sv)?
                }
                sync::SyncMessage::SyncStep2(update) => {
                    tracing::debug!("Handling SyncStep2 message");
                    if let Some(reply) = protocol
                        .handle_sync_step2(&mut doc_box.awareness, Update::decode_v1(&update)?)?
                    {
                        return Err(
                            format!("Unexpectedly got reply on SyncStep2: {reply:?}").into()
                        );
                    }
                    // This update contains things the client has but the server didn't have.
                    // Handle it in the same way we would any other update.
                    Some(sync::Message::Sync(sync::SyncMessage::Update(update)))
                }
                sync::SyncMessage::Update(update) => {
                    tracing::debug!("Handling Update message");
                    if let Some(reply) = protocol
                        .handle_update(&mut doc_box.awareness, Update::decode_v1(&update)?)?
                    {
                        return Err(format!("Unexpectedly got reply on Update: {reply:?}").into());
                    }
                    Some(sync::Message::Sync(sync::SyncMessage::Update(update)))
                }
            }
        };
        let Some(reply) = reply else {
            return Ok(());
        };

        match &reply {
            // This is the response to a SyncStep1 client message containing
            // changes the server has but the client doesn't.
            // Send it ONLY to the requesting client. Do not broadcast or persist it
            sync::Message::Sync(sync::SyncMessage::SyncStep2(_)) => {
                tracing::debug!("Sending SyncStep2 message to client.");
                let mut clients = project.clients.lock().await;
                let Some(client) = clients.get_mut(&msg.who) else {
                    return Err("Unexpectedly found no client to reply to"
                        .to_string()
                        .into());
                };
                if let Err(e) = client.send(reply.encode_v1()).await {
                    return Err(format!("Failed to send reply to client: {e}").into());
                };
            }
            // This is the response to a SyncStep2 OR update client message containing
            // changes the client has but the server doesn't.
            // Broadcast the change to everyone except the given client AND persist it.
            sync::Message::Sync(sync::SyncMessage::Update(m)) => {
                if let Err(e) = self.persist_update(&project.project_id, m).await {
                    return Err(format!("Failed to persist update: {e}").into());
                }
                project.broadcast_msg(&msg.who, reply.encode_v1()).await;
            }
            _ => {
                return Err(format!("Unexpected type of reply to client: {reply:?}").into());
            }
        }
        Ok(())
    }

    async fn persist_update(
        &self,
        project_id: &ProjectId,
        data: &Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "
            INSERT INTO yupdates (project_id, seq, update_v2)
            VALUES ($1, DEFAULT, $2)",
        )
        .bind(project_id)
        .bind(data)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn stop(&self) {
        // Cancel background tasks and wait for them to complete.
        tracing::debug!(
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
