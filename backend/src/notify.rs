use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use uuid::Uuid;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

pub fn start_notifications() -> (Notifier, tokio::task::JoinHandle<()>) {
    let doc = Doc::new();

    // Create a stream of task notifications and a notifier to fan them out to destinations (clients).
    let (tx, rx) = mpsc::channel::<Vec<u8>>(1);
    let notifier = Notifier {
        destinations: Arc::new(Mutex::new(HashMap::new())),
        doc: Arc::new(Mutex::new(doc)),
        tx,
    };
    let notify_task = tokio::spawn(notifier.clone().handle(rx));
    (notifier, notify_task)
}

#[derive(Clone)]
pub struct Notifier {
    destinations: Arc<Mutex<HashMap<String, Destination>>>,
    doc: Arc<Mutex<Doc>>,
    tx: Sender<Vec<u8>>,
}

#[derive(Debug)]
pub struct Destination {
    pub sink: futures::stream::SplitSink<WebSocket, Message>,
    pub who: String,
}

impl Notifier {
    pub async fn register_destination(self, socket: WebSocket, who: SocketAddr) {
        use futures::stream::StreamExt;
        let (mut sender, mut receiver) = socket.split();

        let _ = sender.send(Message::Text("Hello".into())).await;

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

        // High Level Design
        // If this is the very first destination being registered, load everything from the database, and construct the initial graph.
        // For every client that joins, send the current graph as the initial state vector.
        // For every event in the observe_deep, generate a mutation to be applied to the database.
        // When the last client disconnects, consider destroying the graph.

        use yrs::DeepObservable;
        let doc = self.doc.lock().await;
        let graph = doc.get_or_insert_map("graph");
        let sub = graph.observe_deep(|txn, events| {
            events.iter().for_each(|event| match event {
                yrs::types::Event::Text(evt) => {
                    tracing::debug!("Got Text event: {:?}", evt.delta(txn))
                }
                yrs::types::Event::Array(evt) => {
                    tracing::debug!("Got Array event: {:?}", evt.delta(txn))
                }
                yrs::types::Event::Map(evt) => {
                    tracing::debug!("Got Map event: {:?}", evt.keys(txn))
                }
                yrs::types::Event::XmlFragment(evt) => {
                    tracing::debug!("Got XmlFragment event: {:?}", evt.delta(txn))
                }
                yrs::types::Event::XmlText(evt) => {
                    tracing::debug!("Got XmlText event: {:?}", evt.delta(txn))
                }
            })
            // tracing::debug!("got map event: {:?}", events);
        });

        // Listen for messages on the read side of the socket.
        // We don't currently expect any messages other than closures.
        // The idea is to proactively remove clients on closure rather
        // than only sweeping them out while processing a notification.
        tokio::spawn(async move {
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
                        tracing::debug!(
                            "Removing destination for closed client {who}. Reason: {}",
                            if let Some(cf) = &c {
                                format!("code:'{}', detail:'{}'", cf.code, cf.reason)
                            } else {
                                "code:'NONE', detail:'No CloseFrame'".to_string()
                            }
                        );

                        // Remove the destination and close the sink.
                        // If no notification has been processed since registration,
                        // the destination may be in the "new" map, so check both.
                        let dest = self.destinations.lock().await.remove(&who);
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

            drop(sub);
        });
    }

    async fn handle(self, mut rx: Receiver<Vec<u8>>) {
        loop {
            if let Some(update) = rx.recv().await {
                self.doc
                    .lock()
                    .await
                    .transact_mut()
                    .apply_update(Update::decode_v1(&update).unwrap());

                use futures::sink::SinkExt;
                for destination in self.destinations.lock().await.values_mut() {
                    tracing::debug!("Notifying client {}", destination.who);
                    let _ = destination.sink.send(Message::Binary(update.clone())).await;
                }
                tracing::debug!("Got update: {update:?}");
            }
        }
    }
}
