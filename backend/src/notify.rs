use std::{
    collections::VecDeque, error::Error, net::SocketAddr, ops::ControlFlow, sync::Arc,
    time::Duration,
};

use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use sqlx::PgPool;
use tokio::sync::{
    broadcast::{self, error::RecvError, Receiver, Sender},
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
    let (broadcast_tx, broadcast_rx) = broadcast::channel(1);
    let notifier = Notifier {
        pool,
        doc_box: Arc::new(Mutex::new(Option::None)),
        broadcast_tx,
        cancel: CancellationToken::new(),
        tracker: tokio_util::task::TaskTracker::new(),
    };
    notifier
        .tracker
        .spawn(notifier.clone().process_updates(broadcast_rx));

    notifier
}

#[derive(Debug)]
pub struct Destination {
    pub who: String,
    broadcast_rx: Receiver<YrsUpdate>,
    ws_sender: futures::stream::SplitSink<WebSocket, Message>,
    ws_receiver: futures::stream::SplitStream<WebSocket>,
}

pub struct DocBox {
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

#[derive(Debug, Clone)]
struct YrsUpdate {
    #[allow(dead_code)]
    who: String,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct Notifier {
    pool: &'static PgPool,
    doc_box: Arc<Mutex<Option<DocBox>>>,
    broadcast_tx: Sender<YrsUpdate>,
    cancel: CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

// High Level Design
// If this is the very first destination being registered, load everything from the database, and construct the initial graph.
// For every client that joins, send the current graph as the initial state vector.
// For every event in the observe_deep, generate a mutation to be applied to the database.
// When the last client disconnects, consider destroying the graph.
impl Notifier {
    pub async fn register_destination(
        self,
        socket: WebSocket,
        who: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        tracing::debug!("Registering destination for client {who}");

        use futures::stream::StreamExt;
        let (mut ws_sender, ws_receiver) = socket.split();

        // Send a welcome message just for fun!
        ws_sender.send(Message::Text("Hello!".into())).await?;

        // Init the doc_box, if necessary and grab the state vector.
        let sv = self.init_doc_box().await?;

        // Send the entire state vector to the client.
        ws_sender.send(Message::Binary(sv)).await?;

        // Tie everything together in a Destination.
        let dest = Destination {
            ws_sender,
            ws_receiver,
            who,
            broadcast_rx: self.broadcast_tx.subscribe(),
        };

        // Listen for messages on the read side of the socket to broadcast
        // updates to all other clients or get notified about when the
        // client closes the socket.
        self.tracker
            .spawn(self.clone().receive_updates_from_client(dest));

        Ok(())
    }

    async fn init_doc_box(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut doc_box = self.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box
                .doc
                .transact()
                .encode_state_as_update_v1(&StateVector::default()));
        }

        // Load the doc if it wasn't already loaded by another client.
        let doc = self.load_graph().await?;

        // Observe changes to the graph and replicate them to the database.
        let observer_notifier = self.clone();
        use yrs::DeepObservable;
        let sub = doc
            .get_or_insert_map("graph")
            .observe_deep(move |txn, events: &Events| {
                if let Err(e) = observer_notifier.apply_doc_events(txn, events) {
                    // TODO: Handle cases where the doc diverges from the DB.
                    tracing::warn!("Failed to process doc event: {e}");
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
        *doc_box = Some(db);
        Ok(sv)
    }

    async fn load_graph(&self) -> Result<Doc, Box<dyn Error>> {
        tracing::debug!("Initializing new YGraph");
        let tasks: Vec<Task> = sqlx::query_as("SELECT id, name, children FROM tasks")
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
        tracing::debug!("Initialized new YGraph with {} tasks", tasks_len);
        Result::Ok(doc)
    }

    /// Take Update V1's in bytes form, applies them to the doc.
    async fn process_updates(self, mut rx: Receiver<YrsUpdate>) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                update = rx.recv() => {
                    match update {
                        Ok(update) => self.process_update(&update).await,
                        Err(RecvError::Closed) => { break; },
                        Err(RecvError::Lagged(skipped)) => {
                            // TODO: Deal with dropped updates in some way.
                            tracing::warn!("Update receiver lagged. Dropped {skipped} updates.");
                        },
                    }
                }
            }
        }
        tracing::debug!("Stopped processing updates");
    }

    async fn process_update(&self, update: &YrsUpdate) {
        tracing::debug!("Processing update: {update:?}");
        if let Err(e) = self.apply_update_to_doc(update).await {
            tracing::error!("Failed to apply update to doc: {e}");
        }
    }

    async fn apply_update_to_doc(&self, update: &YrsUpdate) -> Result<(), Box<dyn Error>> {
        let update = match Update::decode_v1(&update.data) {
            Ok(update) => update,
            Err(e) => {
                return Err(format!(
                    "Could not decode update from client: {e}, update: {update:?}"
                )
                .into());
            }
        };

        let doc_box = self.doc_box.lock().await;
        let doc_box = match doc_box.as_ref() {
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
    async fn receive_updates_from_client(self, mut dest: Destination) {
        use futures::stream::StreamExt;
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => { break; }
                // Read updates from this client and broadcast them to others.
                msg = dest.ws_receiver.next() => {
                    match msg {
                        Some(msg) => {
                            if let ControlFlow::Break(_) = self.receive_update_from_client(&dest, msg).await {
                                break;
                            }
                        },
                        None => {
                            tracing::debug!("Read None from socket. Will close.");
                            break;
                        },
                    }
                }
                // Read updates from others and send them to the client.
                update = dest.broadcast_rx.recv() => {
                    match update {
                        Ok(update) => {
                            // No need to send the clients own update back to itself.
                            if update.who == dest.who {
                                break;
                            }
                            tracing::debug!("Sending update to {}", dest.who);
                            if let Err(e) = dest.ws_sender.send(Message::Binary(update.data.to_owned())).await {
                                tracing::warn!("Failed to send update to {}: {e}", dest.who);
                            }
                        }
                        Err(RecvError::Closed) => { break; },
                        Err(RecvError::Lagged(skipped)) => {
                            // TODO: Deal with dropped updates in some way.
                            tracing::warn!("Broadcast receiver for {} lagged. Dropped {skipped} updates.", dest.who);
                        }
                    }
                }
            }
        }

        tracing::debug!(
            "Stopped receiving client updates from {}. Closing socket.",
            dest.who
        );
        if let Err(e) = dest.ws_sender.close().await {
            tracing::debug!("Error closing socket for {}: {e}", dest.who);
        }
    }

    async fn receive_update_from_client(
        &self,
        dest: &Destination,
        msg: Result<Message, axum::Error>,
    ) -> ControlFlow<()> {
        match msg {
            Ok(Message::Binary(data)) => {
                match self.broadcast_tx.send(YrsUpdate {
                    who: dest.who.clone(),
                    data,
                }) {
                    Ok(_) => {
                        tracing::debug!("Received update from {}", dest.who);
                    }
                    Err(err) => {
                        tracing::debug!("Error sending update from {}: {err}", dest.who);
                    }
                };
                ControlFlow::Continue(())
            }
            Ok(Message::Close(_)) => {
                tracing::debug!("Client closed socket {}", dest.who);
                ControlFlow::Break(())
            }
            Err(e) => {
                tracing::warn!("Got error reading from client socket. Will close socket. {e}");
                ControlFlow::Break(())
            }
            Ok(_) => {
                tracing::debug!("Discarding unsolicited message from {}: {msg:?}", dest.who);
                ControlFlow::Continue(())
            }
        }
    }

    /// Apply a yrs event generated by observe_deep asyncronously to the database.
    fn apply_doc_events(
        &self,
        txn: &yrs::TransactionMut,
        events: &Events,
    ) -> Result<(), Box<dyn Error>> {
        let updates = self.events_to_updates(txn, events)?;

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
                    "INSERT INTO tasks (id, name, children)
                          VALUES ($1, $2, $3)
                          ON CONFLICT (id)
                          DO UPDATE SET name = EXCLUDED.name, children = EXCLUDED.children;",
                )
                .bind(&task.id)
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
                    updates.updates.push(self.to_task_update(evt.path(), txn)?);
                }
                yrs::types::Event::Map(evt) => {
                    tracing::debug!(
                        "Processing Map event: path: {:?}, target: {:?}, key: {:?}",
                        evt.path(),
                        evt.target().to_json(txn),
                        evt.keys(txn)
                    );
                    updates.updates.push(self.to_task_update(evt.path(), txn)?);
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

        // Drop the doc box to stop observing changes.
        *self.doc_box.lock().await = None;
    }
}
