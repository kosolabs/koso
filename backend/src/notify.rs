use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::SinkExt;
use sqlx::PgPool;
use std::{
    collections::HashMap, error::Error, fmt, net::SocketAddr, ops::ControlFlow, sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Transact, Update};

pub fn start(pool: &'static PgPool) -> Notifier {
    Notifier {
        state: Arc::new(ProjectsState {
            projects: DashMap::new(),
        }),
        pool,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    }
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
    doc: Doc,
}

struct YrsUpdate {
    who: String,
    project: Arc<ProjectState>,
    update_id: String,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct Notifier {
    state: Arc<ProjectsState>,
    pool: &'static PgPool,
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
        if let Err(e) = sender.send(sv).await {
            return Err(format!("Failed to send state vector to client: {e}").into());
        }

        // Store the sender side of the socket in the list of clients.
        if let Some(existing) = project.add_client(sender).await {
            return Err(format!("Unexpectedly, client already exists: {existing:?}").into());
        };

        // Listen for messages on the read side of the socket to broadcast
        // updates to all other clients or get notified about when the
        // client closes the socket.
        self.tracker
            .spawn(self.clone().receive_updates_from_client(receiver, project));

        Ok(())
    }

    async fn init_doc_box(&self, project: &ProjectState) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box
                .doc
                .transact()
                .encode_state_as_update_v2(&StateVector::default()));
        }

        // Load the doc if it wasn't already loaded by another client.
        let doc = self.load_graph(&project.project_id).await?;

        let db = DocBox { doc };
        let sv = db
            .doc
            .transact()
            .encode_state_as_update_v2(&StateVector::default());
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
                let update = Update::decode_v2(&update)?;
                txn.apply_update(update);
            }
        }
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        Result::Ok(doc)
    }

    /// Listen for update or close messages sent by a client.
    #[tracing::instrument(skip(self, project))]
    async fn receive_updates_from_client(
        self,
        mut receiver: ClientReceiver,
        project: Arc<ProjectState>,
    ) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = receiver.next() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    if let ControlFlow::Break(_) = self.receive_update_from_client(&receiver, &project, msg).await {
                        break;
                    }
                }
            }
        }
        tracing::debug!("Stopped receiving client updates from {}", receiver.who);
    }

    async fn receive_update_from_client(
        &self,
        receiver: &ClientReceiver,
        project: &Arc<ProjectState>,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        match msg {
            Ok(Message::Binary(data)) => {
                let update = YrsUpdate {
                    who: receiver.who.clone(),
                    project: project.clone(),
                    update_id: Uuid::new_v4().to_string(),
                    data,
                };
                tracing::debug!("Received update from client: {update:?}");
                if let Err(e) = self.process_update(update).await {
                    tracing::error!("Error processing update: {e}");
                };
                ControlFlow::Continue(())
            }
            Ok(Message::Close(c)) => {
                let reason = if let Some(cf) = &c {
                    format!("code:'{}', detail:'{}'", cf.code, cf.reason)
                } else {
                    "code:'NONE', detail:'No CloseFrame'".to_string()
                };
                self.close_client(project, &receiver.who, &reason).await;
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                self.close_client(project, &receiver.who, &format!("Error: {e}"))
                    .await;
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::warn!("Discarding unsolicited message: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn process_update(&self, update: YrsUpdate) -> Result<(), Box<dyn Error>> {
        self.apply_update_to_doc(&update).await?;
        update.project.broadcast(&update).await;
        self.persist_update(&update).await?;
        Ok(())
    }

    async fn apply_update_to_doc(&self, yrs_update: &YrsUpdate) -> Result<(), Box<dyn Error>> {
        let update = match Update::decode_v2(&yrs_update.data) {
            Ok(update) => update,
            Err(e) => {
                return Err(format!("Could not decode update from client: {e}").into());
            }
        };

        let doc_box = yrs_update.project.doc_box.lock().await;
        let Some(doc_box) = doc_box.as_ref() else {
            // TODO: Aside from short races, if this happens, we've got sockets without a doc.
            // Likely, we should reset all sockets.
            return Err("Tried to apply update but doc_box was None.".into());
        };
        doc_box.doc.transact_mut().apply_update(update);
        Ok(())
    }

    async fn persist_update(&self, update: &YrsUpdate) -> Result<(), Box<dyn Error>> {
        sqlx::query(
            "
            INSERT INTO yupdates (project_id, seq, update_v2)
            VALUES ($1, DEFAULT, $2)",
        )
        .bind(&update.project.project_id)
        .bind(&update.data)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    async fn close_client(&self, project: &ProjectState, who: &String, reason: &String) {
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

    async fn broadcast(&self, update: &YrsUpdate) {
        let mut clients = self.clients.lock().await;

        tracing::debug!("Broadcasting updates to {} clients", clients.len());
        let mut results = Vec::new();
        for client in clients.values_mut() {
            if client.who != update.who {
                results.push(client.send(update.data.to_owned()));
            }
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished broadcasting updates: {res:?}");
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

impl fmt::Debug for DocBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocBox").field("doc", &self.doc).finish()
    }
}

impl fmt::Debug for YrsUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YrsUpdate")
            .field("project_id", &self.project.project_id)
            .field("who", &self.who)
            .field("update_id", &self.update_id)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
