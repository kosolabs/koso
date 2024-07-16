use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    error::Error,
    net::SocketAddr,
    ops::ControlFlow,
    sync::Arc,
    time::Duration,
};

use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use sqlx::PgPool;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use yrs::{
    types::{Events, PathSegment, ToJson},
    updates::decoder::Decode,
    Any, Array, ArrayRef, Doc, Map, MapRef, Out, ReadTxn, StateVector, Transact, Update,
};

use crate::model::Task;

pub fn start(pool: &'static PgPool) -> Notifier {
    let (tx, rx) = mpsc::channel::<YrsUpdate>(1);
    let notifier = Notifier {
        clients: Arc::new(Mutex::new(Clients {
            map: HashMap::new(),
        })),
        pool,
        doc_boxes: Arc::new(Mutex::new(HashMap::new())),
        tx,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    };
    notifier.tracker.spawn(notifier.clone().process_updates(rx));

    notifier
}

type ProjectId = String;

#[derive(Debug)]
struct Clients {
    map: HashMap<ProjectId, HashMap<String, ClientSender>>,
}

#[derive(Debug)]
struct ClientSender {
    ws_sender: futures::stream::SplitSink<WebSocket, Message>,
    who: String,
    project_id: ProjectId,
}

#[derive(Debug)]
struct ClientReceiver {
    ws_receiver: futures::stream::SplitStream<WebSocket>,
    who: String,
    project_id: ProjectId,
}

struct DocBox {
    doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send + Sync>,
}

/// An individual task update: insert, delete or update.
#[derive(Debug)]
enum TaskUpdate {
    #[allow(dead_code)]
    Delete {
        id: String,
    },
    Update {
        task: Task,
    },
}

/// A set of task updates to be applied transactionally.
#[derive(Debug)]
struct TaskUpdates {
    updates: Vec<TaskUpdate>,
}

#[derive(Debug)]
struct YrsUpdate {
    who: String,
    project_id: ProjectId,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct Notifier {
    clients: Arc<Mutex<Clients>>,
    pool: &'static PgPool,
    doc_boxes: Arc<Mutex<HashMap<String, DocBox>>>,
    tx: Sender<YrsUpdate>,
    cancel: CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first client being registered, load everything from the database, and construct the initial graph.
// For every client that joins, send the current graph as the initial state vector.
// For every event in the observe_deep, generate a mutation to be applied to the database.
// When the last client disconnects, consider destroying the graph.
impl Notifier {
    pub async fn register_client(
        self,
        socket: WebSocket,
        who: SocketAddr,
        project_id: ProjectId,
    ) -> Result<(), Box<dyn Error>> {
        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        tracing::debug!("Registering client {who} in {project_id}");

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

        // Init the doc_box, if necessary and grab the state vector.
        let sv = self.init_doc_box(&project_id).await?;

        // Send the entire state vector to the client.
        if let Err(e) = sender.send(sv).await {
            return Err(format!("Failed to send state vector to client: {e}").into());
        }

        // Store the sender side of the socket in the list of clients.
        self.clients.lock().await.add(sender)?;

        // Listen for messages on the read side of the socket to broadcast
        // updates to all other clients or get notified about when the
        // client closes the socket.
        self.tracker
            .spawn(self.clone().receive_updates_from_client(receiver));

        Ok(())
    }

    async fn init_doc_box(&self, project_id: &ProjectId) -> Result<Vec<u8>, Box<dyn Error>> {
        let project_id = project_id.clone();
        let mut doc_boxes = self.doc_boxes.lock().await;
        if let Some(doc_box) = doc_boxes.get(&project_id) {
            return Ok(doc_box
                .doc
                .transact()
                .encode_state_as_update_v1(&StateVector::default()));
        }

        // Load the doc if it wasn't already loaded by another client.
        let doc = self.load_graph(&project_id).await?;

        // Observe changes to the graph and replicate them to the database.
        let observer_notifier = self.clone();
        let project_id_clone = project_id.clone();
        use yrs::DeepObservable;
        let sub = doc
            .get_or_insert_map("graph")
            .observe_deep(move |txn, events: &Events| {
                if let Err(e) = observer_notifier.apply_doc_events(&project_id_clone, txn, events) {
                    // TODO: Handle cases where the doc diverges from the DB.
                    tracing::warn!("Failed to process doc event for {project_id_clone}: {e}");
                }
            });

        let db = DocBox {
            doc,
            sub: Box::new(sub),
        };
        let sv = db
            .doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        doc_boxes.insert(project_id.clone(), db);
        Ok(sv)
    }

    async fn load_graph(&self, project_id: &ProjectId) -> Result<Doc, Box<dyn Error>> {
        tracing::debug!("Initializing new YGraph for {project_id}");
        // TODO: Assert that the project actually exists to avoid conflating that with the absence of tasks.
        let tasks: Vec<Task> =
            sqlx::query_as("SELECT id, project_id, name, children FROM tasks WHERE project_id=$1")
                .bind(project_id)
                .fetch_all(self.pool)
                .await?;
        let tasks_len = tasks.len();

        let doc = Doc::new();
        let graph = doc.get_or_insert_map("graph");
        {
            let mut txn = doc.transact_mut();
            for task in tasks {
                let y_task: MapRef = graph.get_or_init(&mut txn, task.id.clone());
                y_task.insert(&mut txn, "id", task.id);
                y_task.insert(&mut txn, "name", task.name);
                let y_children: ArrayRef = y_task.get_or_init(&mut txn, "children");
                for child in task.children {
                    y_children.push_front(&mut txn, child);
                }
            }
        }
        tracing::debug!("Initialized new YGraph with {tasks_len} tasks in {project_id}");
        Result::Ok(doc)
    }

    /// Take Update V1's in bytes form, applies them to the doc
    /// and then broadcast them to all clients.
    async fn process_updates(self, mut rx: Receiver<YrsUpdate>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                update = rx.recv() => {
                    let Some(update) = update else {
                        break;
                    };
                    self.process_update(&update).await;
                }
            }
        }
        tracing::debug!("Stopped processing updates");
    }

    async fn process_update(&self, update: &YrsUpdate) {
        tracing::debug!("Processing update: {update:?}");
        if let Err(e) = self.apply_update_to_doc(update).await {
            tracing::error!(
                "Failed to apply update to doc for {}: {e}",
                update.project_id
            );
            return;
        }
        self.clients.lock().await.send_to_all(update).await;
    }

    async fn apply_update_to_doc(&self, yrs_update: &YrsUpdate) -> Result<(), Box<dyn Error>> {
        let update = match Update::decode_v1(&yrs_update.data) {
            Ok(update) => update,
            Err(e) => {
                return Err(format!(
                    "Could not decode update from client: {e}, update: {yrs_update:?}"
                )
                .into());
            }
        };

        let doc_boxes = self.doc_boxes.lock().await;
        let doc_box = match doc_boxes.get(&yrs_update.project_id) {
            Some(doc_box) => doc_box,
            None => {
                // TODO: Aside from short races, if this happens, we've got sockets without a doc.
                // Likely, we should reset all sockets.
                return Err(
                    "Tried to broadcast update but doc_box was None. Dropped update: {update:?}"
                        .into(),
                );
            }
        };
        doc_box.doc.transact_mut().apply_update(update);
        Ok(())
    }

    /// Listen for update or close messages sent by a client.
    async fn receive_updates_from_client(self, mut receiver: ClientReceiver) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                msg = receiver.next() => {
                    let Some(msg) = msg else {
                        break;
                    };
                    if let ControlFlow::Break(_) = self.receive_update_from_client(&receiver, msg).await {
                        break;
                    }
                }
            }
        }
        tracing::debug!(
            "Stopped receiving client updates from {} in {}",
            receiver.who,
            receiver.project_id
        );
    }

    async fn receive_update_from_client(
        &self,
        receiver: &ClientReceiver,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        let who = &receiver.who;
        let project_id = &receiver.project_id;
        match msg {
            Ok(Message::Binary(data)) => {
                match self
                    .tx
                    .send(YrsUpdate {
                        who: who.clone(),
                        project_id: project_id.clone(),
                        data,
                    })
                    .await
                {
                    Ok(()) => {
                        tracing::debug!("Received update from {who} in {}", project_id);
                    }
                    Err(err) => {
                        tracing::debug!("Error sending update from {who} in {}: {err}", project_id);
                    }
                };
                ControlFlow::Continue(())
            }
            Ok(Message::Close(c)) => {
                let reason = if let Some(cf) = &c {
                    format!("code:'{}', detail:'{}'", cf.code, cf.reason)
                } else {
                    "code:'NONE', detail:'No CloseFrame'".to_string()
                };
                self.close_client(project_id, who, &reason).await;
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                self.close_client(project_id, who, &format!("Error: {e}"))
                    .await;
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::debug!("Discarding unsolicited message from {who}: {msg:?}");
                ControlFlow::Continue(())
            }
        }
    }

    async fn close_client(&self, project_id: &ProjectId, who: &String, reason: &String) {
        let client = {
            let clients = &mut self.clients.lock().await;
            let client = clients.remove(project_id, who);

            tracing::debug!(
                "Removing closed client {who} in {project_id}. {} clients remain. Reason: {}",
                clients.len(),
                reason
            );

            if clients.len() == 0 {
                tracing::debug!("Last client disconnected in {project_id}, destroying YGraph");
                self.doc_boxes.lock().await.remove(project_id);
            }
            client
        };
        match client {
            Some(mut client) => {
                if let Err(e) = client.close().await {
                    tracing::debug!("Failed to close sink for {who} in {project_id}: {e}");
                }
            }
            None => {
                tracing::warn!(
                    "Unexpectedly, received close for client {who} in {project_id} while no client was registered."
                )
            }
        }
    }

    /// Apply a yrs event generated by observe_deep asyncronously to the database.
    fn apply_doc_events(
        &self,
        project_id: &ProjectId,
        txn: &yrs::TransactionMut,
        events: &Events,
    ) -> Result<(), Box<dyn Error>> {
        let updates = self.events_to_updates(project_id, txn, events)?;

        // Process the updates in another thread since this function
        // must remain synchronous as it's caller is syncronous.
        self.tracker.spawn(self.clone().apply_task_updates(updates));
        Ok(())
    }

    async fn apply_task_updates(self, updates: TaskUpdates) {
        async fn _update(n: Notifier, updates: &TaskUpdates) -> Result<(), Box<dyn Error>> {
            tracing::debug!("About to apply update events: {updates:?}");
            let mut txn = n.pool.begin().await?;
            for update in updates.updates.iter() {
                n.apply_task_update(update, &mut txn).await?;
            }
            txn.commit().await?;
            tracing::debug!("Applied update events: {updates:?}");
            Ok(())
        }
        if let Err(e) = _update(self, &updates).await {
            // TODO: Handle cases where the doc diverges from the DB.
            tracing::warn!("Failed to apply task updates: {e}: {updates:?}")
        };
    }

    async fn apply_task_update(
        &self,
        update: &TaskUpdate,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), Box<dyn Error>> {
        match update {
            TaskUpdate::Delete { id } => {
                match sqlx::query("DELETE tasks WHERE id=$1;")
                    .bind(id)
                    .execute(&mut **txn)
                    .await
                {
                    Err(e) => {
                        return Err(
                            format!("Failed to apply delete update: {e}, {update:?}").into()
                        );
                    }
                    Ok(res) => {
                        if res.rows_affected() == 0 {
                            return Err(format!("task does not exist to delete: {update:?}").into());
                        }
                        // Given the updates where clause should match 0 or 1 rows and never more,
                        // this should never happen.
                        if res.rows_affected() > 1 {
                            return Err(format!(
                                "unexpectedly deleted more than 1 rows ({}): {update:?}",
                                res.rows_affected()
                            )
                            .into());
                        }
                    }
                }
            }
            TaskUpdate::Update { task } => {
                match sqlx::query(
                    "INSERT INTO tasks (id, project_id, name, children)
                          VALUES ($1, $2, $3, $4)
                          ON CONFLICT (id)
                          DO UPDATE SET name = EXCLUDED.name, children = EXCLUDED.children;",
                )
                .bind(&task.id)
                .bind(&task.project_id)
                .bind(&task.name)
                .bind(&task.children)
                .execute(&mut **txn)
                .await
                {
                    Err(e) => {
                        return Err(
                            format!("Failed to apply update update: {e}, {update:?}").into()
                        );
                    }
                    Ok(res) => {
                        if res.rows_affected() == 0 {
                            return Err(format!("task does not exist to update: {task:?}").into());
                        }
                        // Given the updates where clause should match 0 or 1 rows and never more,
                        // this should never happen.
                        if res.rows_affected() > 1 {
                            return Err(format!(
                                "unexpectedly updated more than 1 rows ({}): '{update:?}'",
                                res.rows_affected()
                            )
                            .into());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn events_to_updates(
        &self,
        project_id: &ProjectId,
        txn: &yrs::TransactionMut<'_>,
        events: &Events<'_>,
    ) -> Result<TaskUpdates, Box<dyn Error>> {
        let mut updates = TaskUpdates {
            updates: Vec::new(),
        };
        for evt in events.iter() {
            match evt {
                yrs::types::Event::Array(evt) => {
                    tracing::debug!(
                        "Processing Array event: path: {:?}, target: {:?}, delta: {:?}",
                        evt.path(),
                        evt.target().to_json(txn),
                        evt.delta(txn)
                    );
                    updates
                        .updates
                        .push(self.to_task_update(project_id, evt.path(), txn)?);
                }
                yrs::types::Event::Map(evt) => {
                    tracing::debug!(
                        "Processing Map event: path: {:?}, target: {:?}, key: {:?}",
                        evt.path(),
                        evt.target().to_json(txn),
                        evt.keys(txn)
                    );
                    updates
                        .updates
                        .push(self.to_task_update(project_id, evt.path(), txn)?);
                }
                yrs::types::Event::Text(evt) => {
                    return Err(format!(
                        "Text events are not implemented: path: {:?}, target: {:?}, delta: {:?}",
                        evt.path(),
                        evt.target(),
                        evt.delta(txn)
                    )
                    .into());
                }
                yrs::types::Event::XmlFragment(evt) => {
                    return Err(format!(
                        "XmlFragment events are not implemented: path: {:?}, target: {:?}, delta: {:?}",
                        evt.path(),
                        evt.target(),
                        evt.delta(txn)
                    )
                    .into());
                }
                yrs::types::Event::XmlText(evt) => {
                    return Err(format!(
                        "XMLText events are not implemented: path: {:?}, target: {:?}, delta: {:?}",
                        evt.path(),
                        evt.target(),
                        evt.delta(txn)
                    )
                    .into());
                }
            }
        }

        Ok(updates)
    }

    fn to_task_update(
        &self,
        project_id: &ProjectId,
        path: VecDeque<PathSegment>,
        txn: &yrs::TransactionMut<'_>,
    ) -> Result<TaskUpdate, Box<dyn Error>> {
        let Some(PathSegment::Key(key)) = path.front() else {
            return Err(format!("Could not get first path segement key: {:?}", path).into());
        };
        let Some(graph) = txn.get_map("graph") else {
            return Err(format!("Could not get map 'graph': {:?}", txn.doc()).into());
        };
        let Some(Out::YMap(task)) = graph.get(txn, key) else {
            return Err(format!("Could not find graph in: {}", txn.doc()).into());
        };
        let Some(Out::Any(Any::String(id))) = task.get(txn, "id") else {
            return Err(format!("Could not find id in: {task:?}").into());
        };
        if *key != id {
            return Err(format!(
                "Id mismatch, map key ({key}) does not equal 'id' ({id}): {task:?}"
            )
            .into());
        }
        let Some(Out::Any(Any::String(name))) = task.get(txn, "name") else {
            return Err(format!("Could not find name in: {task:?}").into());
        };
        let Some(Out::YArray(children)) = task.get(txn, "children") else {
            return Err(format!("Could not find children in: {task:?}").into());
        };
        let mut children_str = Vec::new();
        for i in 0..children.len(txn) {
            let Some(Out::Any(Any::String(child))) = children.get(txn, i) else {
                return Err(
                    format!("Could not get child from children at {i}: {children:?}").into(),
                );
            };
            children_str.push(child.to_string());
        }

        Ok(TaskUpdate::Update {
            task: Task {
                id: id.to_string(),
                project_id: project_id.to_string(),
                name: name.to_string(),
                children: children_str,
            },
        })
    }

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

        // Close all clients.
        self.clients.lock().await.close_all().await;

        // Drop the doc box to stop observing changes.
        self.doc_boxes.lock().await.clear();
    }
}

impl Clients {
    fn add(&mut self, client: ClientSender) -> Result<(), Box<dyn Error>> {
        let who = client.who.clone();
        let project_id = client.project_id.clone();
        let clients = match self.map.entry(project_id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(HashMap::new()),
        };
        if let Some(existing) = clients.insert(who.clone(), client) {
            return Err(format!(
                "Unexpectedly, client {who} already exists in {project_id}: {existing:?}"
            )
            .into());
        }
        Ok(())
    }

    fn remove(&mut self, project_id: &ProjectId, who: &String) -> Option<ClientSender> {
        let clients = self.map.get_mut(project_id)?;
        clients.remove(who)
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    async fn send_to_all(&mut self, update: &YrsUpdate) {
        let Some(clients) = self.map.get_mut(&update.project_id) else {
            tracing::warn!(
                "No clients for project {} exist to send to.",
                update.project_id
            );
            return;
        };

        tracing::debug!(
            "Sending updates to {} clients from {} in {}",
            clients.len(),
            update.who,
            update.project_id
        );
        let mut results = Vec::new();
        for client in clients.values_mut() {
            tracing::debug!("Sending update to {}", client.who);
            results.push(client.send(update.data.to_owned()));
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished sending updates: {res:?}");
    }

    async fn close_all(&mut self) {
        for clients in self.map.values_mut() {
            for client in clients.values_mut() {
                let _ = client.close().await;
            }
        }
        self.map.clear();
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

impl ClientReceiver {
    async fn next(&mut self) -> Option<Result<Message, axum::Error>> {
        use futures::stream::StreamExt;
        self.ws_receiver.next().await
    }
}
