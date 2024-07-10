use std::{collections::HashMap, error::Error, fmt::Debug, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitStream, SinkExt};
use sqlx::PgPool;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use uuid::Uuid;
use yrs::{
    types::{Events, PathSegment},
    updates::decoder::Decode,
    Any, Array, ArrayRef, Doc, Map, MapRef, Out, ReadTxn, StateVector, Transact, Update,
};

use crate::model::Task;

pub fn start_notifications(pool: &'static PgPool) -> (Notifier, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(1);
    let notifier = Notifier {
        destinations: Arc::new(Mutex::new(HashMap::new())),
        pool,
        doc_box: Arc::new(Mutex::new(Option::None)),
        tx,
    };
    let notify_task = tokio::spawn(notifier.clone().broadcast_updates(rx));
    (notifier, notify_task)
}

#[derive(Debug)]
pub struct Destination {
    pub sink: futures::stream::SplitSink<WebSocket, Message>,
    pub who: String,
}

pub struct DocBox {
    doc: Doc,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    sub: Box<dyn Send + Sync>,
}

#[derive(Clone)]
pub struct Notifier {
    destinations: Arc<Mutex<HashMap<String, Destination>>>,
    pool: &'static PgPool,
    doc_box: Arc<Mutex<Option<DocBox>>>,
    tx: Sender<Vec<u8>>,
}

// High Level Design
// If this is the very first destination being registered, load everything from the database, and construct the initial graph.
// For every client that joins, send the current graph as the initial state vector.
// For every event in the observe_deep, generate a mutation to be applied to the database.
// When the last client disconnects, consider destroying the graph.
impl Notifier {
    pub async fn register_destination(self, socket: WebSocket, who: SocketAddr) {
        use futures::stream::StreamExt;
        let (mut sender, receiver) = socket.split();

        // Send a welcome message just for fun!
        let _ = sender.send(Message::Text("Hello!".into())).await;

        let doc_box = self.doc_box.clone();
        let mut doc_box = doc_box.lock().await;
        // Load the doc if it wasn't already loaded by another client.
        if doc_box.is_none() {
            let doc = match self.load_graph().await {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to load graph: {}", e);
                    return;
                }
            };

            // Observe changes to the graph and replicate them to the database.
            let observer_notifier = self.clone();
            use yrs::DeepObservable;
            let sub = doc
                .get_or_insert_map("graph")
                .observe_deep(move |txn, events| {
                    let rows = observer_notifier.events_to_rows(txn, events);
                    let o = observer_notifier.clone();
                    tokio::spawn(async move {
                        o.write_events(&rows).await;
                    });
                });

            // Store the doc and sub for subsequent clients.
            *doc_box = Some(DocBox {
                doc,
                sub: Box::new(sub),
            });
        }

        let doc = doc_box.as_ref().unwrap();

        // Send the entire state vector to the client.
        let sv = doc
            .doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        if let Err(e) = sender.send(Message::Binary(sv)).await {
            tracing::warn!("Failed to send state vector to client: {e}");
            return;
        }

        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        // Store the sender side of the socket in the list of destinations.
        tracing::debug!("Registering destination for client {who}");
        if let Some(existing) = self.destinations.lock().await.insert(
            who.clone(),
            Destination {
                sink: sender,
                who: who.clone(),
            },
        ) {
            tracing::warn!("Unexpectedly, destination {who} already exists: {existing:?}")
        }

        // Listen for messages on the read side of the socket to broadcast
        // updates to all other clients or get notified about when the
        // client closes the socket.
        tokio::spawn(async move {
            self.receive_updates(who, receiver).await;
        });
    }

    async fn load_graph(&self) -> Result<Doc, Box<dyn Error>> {
        tracing::debug!("Initializing new YGraph");
        let tasks: Vec<Task> = sqlx::query_as("SELECT id, name, children FROM tasks")
            .fetch_all(self.pool)
            .await?;
        let doc = Doc::new();
        let graph = doc.get_or_insert_map("graph");
        {
            let mut txn = doc.transact_mut();
            for task in tasks.iter() {
                let y_task: MapRef = graph.get_or_init(&mut txn, task.id.clone());
                y_task.insert(&mut txn, "id", task.id.clone());
                y_task.insert(&mut txn, "name", task.name.clone());
                let y_children: ArrayRef = y_task.get_or_init(&mut txn, "children");
                for child in task.children.iter() {
                    y_children.push_front(&mut txn, child.clone());
                }
            }
        }
        tracing::debug!("Initialized new YGraph with {} tasks", tasks.len());
        Result::Ok(doc)
    }

    /// Take Update V1's in bytes form and broadcast them to all destinations.
    async fn broadcast_updates(self, mut rx: Receiver<Vec<u8>>) {
        loop {
            if let Some(update) = rx.recv().await {
                self.doc_box
                    .lock()
                    .await
                    .as_mut()
                    .unwrap()
                    .doc
                    .transact_mut()
                    .apply_update(Update::decode_v1(&update).unwrap());

                use futures::sink::SinkExt;
                use futures::FutureExt;
                let mut results = Vec::new();
                let mut destinations = self.destinations.lock().await;
                for destination in destinations.values_mut() {
                    tracing::debug!("Notifying client {}", destination.who);
                    results.push(
                        destination
                            .sink
                            .send(Message::Binary(update.clone()))
                            .map(|r| match r {
                                Ok(_) => {
                                    tracing::debug!("Sent update to {}", destination.who);
                                }
                                Err(e) => {
                                    tracing::debug!(
                                        "Failed to send update to {}: {e}",
                                        destination.who
                                    );
                                }
                            }),
                    );
                }
                futures::future::join_all(results).await;

                tracing::debug!("Got update: {update:?}");
            }
        }
    }

    /// Listen for update or close messages sent by a client.
    async fn receive_updates(self, who: String, mut receiver: SplitStream<WebSocket>) {
        use futures::stream::StreamExt;
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Binary(vec) => {
                    match self.tx.send(vec).await {
                        Ok(()) => {
                            tracing::debug!("Broadcasting update from {who}");
                        }
                        Err(err) => {
                            tracing::debug!("Error broadcasting: {err}");
                        }
                    };
                }
                Message::Close(c) => {
                    // Remove the destination and close the sink.
                    let dest = {
                        let destinations = &mut self.destinations.lock().await;
                        tracing::debug!(
                            "Removing destination for closed client {who}, {} remain. Reason: {}",
                            destinations.len() - 1,
                            if let Some(cf) = &c {
                                format!("code:'{}', detail:'{}'", cf.code, cf.reason)
                            } else {
                                "code:'NONE', detail:'No CloseFrame'".to_string()
                            }
                        );

                        let dest = destinations.remove(&who);
                        if destinations.len() == 0 {
                            tracing::debug!("Last client disconnected, destroying YGraph");
                            *self.doc_box.lock().await = None;
                        }
                        dest
                    };
                    if dest.is_none() {
                        tracing::warn!("Unexpectedly, received close for client {who} while no destination was registered.")
                    }
                    if let Some(mut dest) = dest {
                        let _ = dest.sink.close().await;
                    }
                    break;
                }
                _ => {
                    tracing::debug!("Discarding unsolicited message from {who}: {msg:?}");
                }
            }
        }
    }

    async fn write_events(&self, rows: &Vec<(String, String, Vec<String>)>) {
        // TODO: should wrap all of these in a transaction.
        for row in rows {
            self.write_event(row).await;
        }
    }
    async fn write_event(&self, row: &(String, String, Vec<String>)) {
        tracing::debug!("About to write_event row: {row:?}");
        let (id, name, children) = row;
        match sqlx::query("UPDATE tasks SET name = $2, children = $3 WHERE id=$1;")
            .bind(id)
            .bind(name)
            .bind(children)
            .execute(self.pool)
            .await
        {
            Err(e) => {
                tracing::warn!("Failed to write event {row:?}: {e}");
            }
            Ok(res) => {
                if res.rows_affected() == 0 {
                    tracing::warn!("task '{id}' does not exist!");
                }
                // Given the updates where clause should match 0 or 1 rows and never more,
                // this should never happen.
                if res.rows_affected() > 1 {
                    tracing::warn!(
                        "unexpectedly updated more than 1 rows ({}) for id '{id}'",
                        res.rows_affected()
                    );
                }
            }
        }
    }

    fn events_to_rows(
        &self,
        txn: &yrs::TransactionMut<'_>,
        events: &Events<'_>,
    ) -> Vec<(String, String, Vec<String>)> {
        let mut results = Vec::new();
        for evt in events.iter() {
            match evt {
                yrs::types::Event::Text(evt) => {
                    tracing::debug!("Got Text event: {:?}", evt.delta(txn));
                    continue;
                }
                yrs::types::Event::Array(evt) => {
                    tracing::debug!(
                        "Got Array event: {:?}, path: {:?}",
                        evt.delta(txn),
                        evt.path()
                    );
                    let path = evt.path();
                    let PathSegment::Key(thing) = path.back().unwrap() else {
                        tracing::warn!("Path segment is not key type: {:?}", path.back().unwrap());
                        continue;
                    };

                    if *thing != "children".into() {
                        tracing::warn!("Path segment is not 'children': {thing}");
                        continue;
                    }

                    if let PathSegment::Key(id) = path.front().unwrap() {
                        let Out::YMap(task) = txn.get_map("graph").unwrap().get(txn, id).unwrap()
                        else {
                            tracing::warn!("Could not find graph in: {}", txn.doc());
                            continue;
                        };

                        let Out::Any(Any::String(id)) = task.get(txn, "id").unwrap() else {
                            tracing::warn!("Could not find id in: {task:?}");
                            continue;
                        };
                        let Out::Any(Any::String(name)) = task.get(txn, "name").unwrap() else {
                            tracing::warn!("Could not find name in: {task:?}");
                            continue;
                        };
                        let Out::YArray(children) = task.get(txn, "children").unwrap() else {
                            tracing::warn!("Could not find children in: {task:?}");
                            continue;
                        };

                        let mut children_str = Vec::new();
                        for i in 0..children.len(txn) {
                            let Out::Any(Any::String(child)) = children.get(txn, i).unwrap() else {
                                tracing::warn!(
                                    "Could not get child from children at {i}: {children:?}"
                                );

                                continue;
                            };
                            children_str.push(child.to_string());
                        }

                        results.push((id.to_string(), name.to_string(), children_str));
                    }
                }
                yrs::types::Event::Map(evt) => {
                    tracing::debug!("Got Map event: {:?}", evt.keys(txn));
                    continue;
                }
                yrs::types::Event::XmlFragment(evt) => {
                    tracing::debug!("Got XmlFragment event: {:?}", evt.delta(txn));
                    continue;
                }
                yrs::types::Event::XmlText(evt) => {
                    tracing::debug!("Got XmlText event: {:?}", evt.delta(txn));
                    continue;
                }
            }
        }
        results
    }
}
