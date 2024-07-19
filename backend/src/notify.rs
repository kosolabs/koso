use crate::model::Task;
use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use sqlx::PgPool;
use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt,
    net::SocketAddr,
    ops::ControlFlow,
    sync::Arc,
    time::Duration,
};
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

pub fn start(pool: &'static PgPool) -> Notifier {
    let (process_tx, process_rx) = mpsc::channel::<YrsUpdate>(24);
    let (commit_tx, commit_rx) = mpsc::channel::<TaskUpdates>(24);
    let (broadcast_tx, broadcast_rx) = mpsc::channel::<YrsUpdate>(24);
    let notifier = Notifier {
        clients: Arc::new(Mutex::new(Clients {
            map: HashMap::new(),
        })),
        pool,
        doc_boxes: Arc::new(Mutex::new(HashMap::new())),
        process_tx,
        commit_tx,
        broadcast_tx,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    };
    notifier
        .tracker
        .spawn(notifier.clone().receive_updates(process_rx));
    notifier
        .tracker
        .spawn(notifier.clone().receive_task_updates(commit_rx));
    notifier
        .tracker
        .spawn(notifier.clone().receive_broadcasts(broadcast_rx));

    notifier
}

type ProjectId = String;

#[derive(Debug)]
struct Clients {
    map: HashMap<ProjectId, HashMap<String, ClientSender>>,
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
    sub: Box<dyn Send + Sync>,
}

/// An individual task update: insert, delete or update.
#[derive(Debug)]
enum TaskUpdate {
    Delete { id: String },
    Insert { task: Task },
    Update { task: Task },
}

enum UpdateType {
    Insert,
    Update,
}

/// A set of task updates to be applied transactionally.
#[derive(Debug)]
struct TaskUpdates {
    update_id: String,
    project_id: ProjectId,
    updates: Vec<TaskUpdate>,
}

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
    process_tx: Sender<YrsUpdate>,
    commit_tx: Sender<TaskUpdates>,
    broadcast_tx: Sender<YrsUpdate>,
    cancel: CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first client being registered, load everything from the database, and construct the initial graph.
// For every client that joins, send the current graph as the initial state vector.
// For every event in the observe_deep, generate a mutation to be applied to the database.
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
        let mut doc_boxes = self.doc_boxes.lock().await;
        if let Some(doc_box) = doc_boxes.get(project_id) {
            return Ok(doc_box
                .doc
                .transact()
                .encode_state_as_update_v1(&StateVector::default()));
        }

        // Load the doc if it wasn't already loaded by another client.
        let doc = self.load_graph(project_id).await?;

        // Observe changes to the graph and replicate them to the database.
        let observer_notifier = self.clone();
        let project_id_clone = project_id.clone();
        use yrs::DeepObservable;
        let sub = doc
            .get_or_insert_map("graph")
            .observe_deep(move |txn, events: &Events| {
                let update_id = Uuid::new_v4().to_string();
                tracing::Span::current().record("update_id", &update_id);
                if let Err(e) =
                    observer_notifier.commit_doc_event(&update_id, &project_id_clone, txn, events)
                {
                    // TODO: Handle cases where the doc diverges from the DB.
                    tracing::error!("Failed to process doc event: {e}");
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
        tracing::debug!("Initializing new YGraph");
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
                let y_task: MapRef = graph.get_or_init(&mut txn, task.id.as_str());
                y_task.insert(&mut txn, "id", task.id);
                y_task.insert(&mut txn, "name", task.name);
                let y_children: ArrayRef = y_task.get_or_init(&mut txn, "children");
                for child in task.children {
                    y_children.push_back(&mut txn, child);
                }
            }
        }
        tracing::debug!("Initialized new YGraph with {tasks_len} tasks");
        Result::Ok(doc)
    }

    /// Listen for update or close messages sent by a client.
    #[tracing::instrument(skip(self))]
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
        tracing::debug!("Stopped receiving client updates from {}", receiver.who);
    }

    async fn receive_update_from_client(
        &self,
        receiver: &ClientReceiver,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        match msg {
            Ok(Message::Binary(data)) => {
                tracing::debug!("Received update from client");
                if let Err(e) = self
                    .process_tx
                    .send(YrsUpdate {
                        who: receiver.who.clone(),
                        project_id: receiver.project_id.clone(),
                        data,
                    })
                    .await
                {
                    tracing::error!("Error sending update to process channel: {e}");
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
        let client = {
            let clients = &mut self.clients.lock().await;
            let client = clients.remove(project_id, who);

            let remaining_clients = clients.len(project_id);
            tracing::debug!(
                "Removing closed client. {} clients remain. Reason: {}",
                remaining_clients,
                reason
            );

            if remaining_clients == 0 {
                tracing::debug!("Last client disconnected, destroying YGraph");
                self.doc_boxes.lock().await.remove(project_id);
            }
            client
        };
        match client {
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

    /// Receive updates from `process_rx`, apply them to the coc and broadcast to all clients.
    #[tracing::instrument(skip(self, process_rx))]
    async fn receive_updates(self, mut process_rx: Receiver<YrsUpdate>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                update = process_rx.recv() => {
                    let Some(update) = update else {
                        break;
                    };
                    self.process_update(update).await;
                }
            }
        }
        tracing::debug!("Stopped receiving updates");
    }

    #[tracing::instrument(skip(self))]
    async fn process_update(&self, update: YrsUpdate) {
        tracing::debug!("Processing update");
        // Apply the update to the project's doc. This may also trigger a commit to the DB.
        if let Err(e) = self.apply_update_to_doc(&update).await {
            tracing::error!("Failed to apply update to doc: {e}");
            return;
        }

        if let Err(e) = self.broadcast_tx.send(update).await {
            // TODO: Handle cases where clients diverged due to failed sends.
            tracing::error!("Failed send updates to broadcast_tx, dropping update: {e}")
        };
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

    /// Receive updates from `broadcast_rx` and broadcasts them out to all clients.
    #[tracing::instrument(skip(self, broadcast_rx))]
    async fn receive_broadcasts(self, mut broadcast_rx: Receiver<YrsUpdate>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                update = broadcast_rx.recv() => {
                    let Some(update) = update else {
                        break;
                    };
                    self.clients.lock().await.broadcast(&update).await;
                }
            }
        }
        tracing::debug!("Stopped receiving broadcasts");
    }

    /// Apply a yrs event generated by observe_deep asyncronously to the database.
    #[tracing::instrument(skip(self, txn, events))]
    fn commit_doc_event(
        &self,
        update_id: &String,
        project_id: &ProjectId,
        txn: &yrs::TransactionMut,
        events: &Events,
    ) -> Result<(), Box<dyn Error>> {
        let updates = self.events_to_updates(update_id, project_id, txn, events)?;

        // Process the updates in another thread since this function
        // must remain synchronous as it's caller is syncronous.
        async fn _send(n: Notifier, updates: TaskUpdates) {
            if let Err(e) = n.commit_tx.send(updates).await {
                // TODO: Handle cases where the doc diverges from the DB.
                tracing::error!("Failed send updates to commit_tx, dropping update: {e}")
            }
        }
        self.tracker.spawn(_send(self.clone(), updates));
        Ok(())
    }

    /// Receive updates from `commit_rx` and commits them to the DB.
    #[tracing::instrument(skip(self, commit_rx))]
    async fn receive_task_updates(self, mut commit_rx: Receiver<TaskUpdates>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                updates = commit_rx.recv() => {
                    let Some(updates) = updates else {
                        break;
                    };
                    self.commit_task_updates(updates).await;
                }
            }
        }
        tracing::debug!("Stopped receiving task updates");
    }

    #[tracing::instrument(skip(self, updates), fields(update_id=&updates.update_id, project_id=&updates.project_id))]
    async fn commit_task_updates(&self, updates: TaskUpdates) {
        async fn _update(n: &Notifier, updates: &TaskUpdates) -> Result<(), Box<dyn Error>> {
            tracing::debug!("About to commit task updates: {updates:?}");
            let mut txn = n.pool.begin().await?;
            for update in updates.updates.iter() {
                n.commit_task_update(update, &mut txn).await?;
            }
            txn.commit().await?;
            tracing::debug!("Committed task updates");
            Ok(())
        }
        if let Err(e) = _update(self, &updates).await {
            // TODO: Handle cases where the doc diverges from the DB.
            tracing::error!("Failed to commit task updates: {e}")
        };
    }

    async fn commit_task_update(
        &self,
        update: &TaskUpdate,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), Box<dyn Error>> {
        match update {
            TaskUpdate::Delete { id } => {
                match sqlx::query("DELETE FROM tasks WHERE id=$1;")
                    .bind(id)
                    .execute(&mut **txn)
                    .await
                {
                    Err(e) => {
                        return Err(format!("Failed to delete task: {e}, {update:?}").into());
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
            TaskUpdate::Insert { task } => {
                match sqlx::query(
                    "INSERT INTO tasks (id, project_id, name, children)
                          VALUES ($1, $2, $3, $4)",
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
                            format!("Failed to apply insert update: {e}, {update:?}").into()
                        );
                    }
                    Ok(res) => {
                        if res.rows_affected() != 1 {
                            return Err(format!(
                                "unexpectedly modified zero or more than 1 row ({}): '{update:?}'",
                                res.rows_affected()
                            )
                            .into());
                        }
                    }
                }
            }
            TaskUpdate::Update { task } => {
                match sqlx::query(
                    "update tasks
                    SET name=$3, children=$4
                    WHERE id=$1 and project_id=$2;",
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
        update_id: &str,
        project_id: &ProjectId,
        txn: &yrs::TransactionMut<'_>,
        events: &Events<'_>,
    ) -> Result<TaskUpdates, Box<dyn Error>> {
        let mut updates = TaskUpdates {
            update_id: update_id.to_string(),
            project_id: project_id.clone(),
            updates: Vec::new(),
        };
        for evt in events.iter() {
            match evt {
                yrs::types::Event::Array(evt) => {
                    let path = evt.path();
                    tracing::trace!(
                        "Processing Array event: path: {:?}, target: {:?}, delta: {:?}",
                        evt.path(),
                        evt.target().to_json(txn),
                        evt.delta(txn)
                    );
                    let Some(PathSegment::Key(id)) = path.front() else {
                        return Err(
                            format!("Could not get first path segement key: {:?}", path).into()
                        );
                    };
                    updates.updates.push(self.to_task_update(
                        project_id,
                        id.to_string(),
                        UpdateType::Update,
                        txn,
                    )?);
                }
                yrs::types::Event::Map(evt) => {
                    let path = evt.path();
                    let keys = evt.keys(txn);
                    tracing::trace!(
                        "Processing Map event: path: {:?}, target: {:?}, key: {:?}",
                        path,
                        evt.target().to_json(txn),
                        keys
                    );

                    // Edits to an existing task will have a path pointing to the task,
                    // e.g. [Key("3")]
                    // We could use "keys" instead to patch the udpate, but that's just
                    // an optimization.
                    // e.g.: {"name": Updated(Any(String("33")), Any(String("33333")))}
                    if !path.is_empty() {
                        let Some(PathSegment::Key(id)) = path.front() else {
                            return Err(format!(
                                "Could not get first path segement key: {:?}",
                                path
                            )
                            .into());
                        };
                        updates.updates.push(self.to_task_update(
                            project_id,
                            id.to_string(),
                            UpdateType::Update,
                            txn,
                        )?);
                        continue;
                    };

                    // Otherwise, a task is being inserted.
                    if keys.len() > 1 {
                        return Err(format!(
                            "Found map event with empty path and multiple keys: {keys:?}"
                        )
                        .into());
                    }

                    let (id, change) = match keys.iter().next() {
                        Some((id, change)) => (id, change),
                        None => return Err("Found map event with empty path and zero keys".into()),
                    };

                    // The only change should be the insertion.
                    // e.g. {"9": Inserted(YMap(MapRef(<1496944415#11>)))}
                    match change {
                        yrs::types::EntryChange::Inserted(_) => {
                            updates.updates.push(self.to_task_update(
                                project_id,
                                id.to_string(),
                                UpdateType::Insert,
                                txn,
                            )?);
                        }
                        yrs::types::EntryChange::Removed(_) => {
                            updates
                                .updates
                                .push(TaskUpdate::Delete { id: id.to_string() });
                        }
                        yrs::types::EntryChange::Updated(..) => {
                            return Err(
                                format!("Unexpectedly got Updated map event: {change:?}").into()
                            )
                        }
                    };
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
        id: String,
        update_type: UpdateType,
        txn: &yrs::TransactionMut<'_>,
    ) -> Result<TaskUpdate, Box<dyn Error>> {
        let Some(graph) = txn.get_map("graph") else {
            return Err(format!("Could not get map 'graph': {:?}", txn.doc()).into());
        };
        let Some(Out::YMap(task)) = graph.get(txn, &id) else {
            return Err(format!("Could not find graph in: {}", txn.doc()).into());
        };
        let Some(Out::Any(Any::String(task_id))) = task.get(txn, "id") else {
            return Err(format!("Could not find id in: {task:?}").into());
        };
        if *id != *task_id {
            return Err(format!(
                "Id mismatch, expected id ({id}) does not equal 'id' ({task_id}): {task:?}"
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

        match update_type {
            UpdateType::Insert => Ok(TaskUpdate::Insert {
                task: Task {
                    id,
                    project_id: project_id.to_string(),
                    name: name.to_string(),
                    children: children_str,
                },
            }),
            UpdateType::Update => Ok(TaskUpdate::Update {
                task: Task {
                    id,
                    project_id: project_id.to_string(),
                    name: name.to_string(),
                    children: children_str,
                },
            }),
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

        // Close all clients.
        self.clients.lock().await.close_all().await;

        // Drop the doc box to stop observing changes.
        self.doc_boxes.lock().await.clear();
    }
}

impl Clients {
    fn add(&mut self, client: ClientSender) -> Result<(), Box<dyn Error>> {
        let clients = match self.map.entry(client.project_id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(HashMap::new()),
        };
        if let Some(existing) = clients.insert(client.who.clone(), client) {
            return Err(format!("Unexpectedly, client already exists: {existing:?}").into());
        }
        Ok(())
    }

    fn remove(&mut self, project_id: &ProjectId, who: &String) -> Option<ClientSender> {
        let clients = self.map.get_mut(project_id)?;
        clients.remove(who)
    }

    fn len(&self, project_id: &ProjectId) -> usize {
        self.map.get(project_id).map(|c| c.len()).unwrap_or(0)
    }

    async fn broadcast(&mut self, update: &YrsUpdate) {
        let Some(clients) = self.map.get_mut(&update.project_id) else {
            tracing::warn!("No clients for project exist to broadcast to.");
            return;
        };

        tracing::debug!("Broadcasting updates to {} clients", clients.len());
        let mut results = Vec::new();
        for client in clients.values_mut() {
            tracing::debug!("Sending update to {}", client.who);
            results.push(client.send(update.data.to_owned()));
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished broadcasting updates: {res:?}");
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
            .field("project_id", &self.project_id)
            .field("who", &self.who)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
