use async_stream::try_stream;
use axum::extract::ws::{Message, WebSocket};
use futures::{stream::Stream, FutureExt, SinkExt};
use sqlx::{postgres::PgListener, Pool, Postgres};
use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn start_notifications(
    pool: sqlx::Pool<sqlx::Postgres>,
) -> (Notifier, tokio::task::JoinHandle<()>) {
    // Create a stream of task notifications and a notifier to fan them out to destinations (clients).
    let stream = stream_task_notifications(pool);
    let notifier = Notifier {
        destinations: Arc::new(Mutex::new(HashMap::new())),
        new_destinations: Arc::new(Mutex::new(HashMap::new())),
    };
    let notify_task = tokio::spawn(notifier.clone().from_stream(stream));
    (notifier, notify_task)
}

#[derive(Clone, Debug)]
pub struct Notifier {
    /// Existing destinations to notify.
    destinations: Arc<Mutex<HashMap<String, Destination>>>,
    /// Newly added destinations that will be moved to `destinations`
    /// when the next notification is processed.
    new_destinations: Arc<Mutex<HashMap<String, Destination>>>,
}

#[derive(Debug)]
pub struct Destination {
    pub sink: futures::stream::SplitSink<WebSocket, Message>,
    pub who: String,
}

impl Notifier {
    pub async fn register_destination(self, socket: WebSocket, who: SocketAddr) {
        use futures::stream::StreamExt;
        let (sender, mut receiver) = socket.split();

        let who = who.to_string() + ":" + &Uuid::new_v4().to_string();
        // Store the sender side of the socket in the list of destinations.
        tracing::debug!("Registering destination for client {who}");
        if let Some(existing) = self.new_destinations.lock().await.insert(
            who.clone(),
            Destination {
                sink: sender,
                who: who.clone(),
            },
        ) {
            tracing::warn!("Unexpectedly, destination {who} already exists: {existing:?}")
        }

        // Listen for messages on the read side of the socket.
        // We don't currently expect any messages other than closures.
        // The idea is to proactively remove clients on closure rather
        // than only sweeping them out while processing a notification.
        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
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
                        let dest = self.new_destinations.lock().await.remove(&who);
                        let other_dest = self.destinations.lock().await.remove(&who);
                        if dest.is_none() && other_dest.is_none() {
                            tracing::warn!("Unexpectedly, received close for client {who} while no destination was registered.")
                        }
                        if let Some(mut dest) = dest {
                            let _ = dest.sink.close().await;
                        }
                        if let Some(mut other_dest) = other_dest {
                            let _ = other_dest.sink.close().await;
                        }
                        break;
                    }
                    _ => {
                        tracing::debug!("Discarding unsolicited message from {who}: {msg:?}");
                    }
                }
            }
        });
    }

    async fn from_stream(self, change_stream: impl Stream<Item = Result<Payload, sqlx::Error>>) {
        use futures::StreamExt;
        futures::pin_mut!(change_stream);

        loop {
            match change_stream.next().await {
                Some(Ok(payload)) => {
                    use futures::sink::SinkExt;

                    let mut destinations = self.destinations.lock().await;

                    // First, move any new destinations into the destinations list.
                    // Doing this reduces contention while inserting new destinations
                    // and fanning out to all destinations below.
                    // For this to be effective, release the lock asap, before fanning out below.
                    {
                        let mut new_dests = self.new_destinations.lock().await;
                        for k in new_dests.keys().cloned().collect::<Vec<String>>() {
                            let new_dest = new_dests.remove(&k).unwrap();
                            if let Some(existing_dest) = destinations.insert(k, new_dest) {
                                tracing::warn!("Unexpectedly found existing dest when adding new dest: {existing_dest:?}")
                            }
                        }
                    }

                    let payload = Message::Text(serde_json::to_string(&payload).unwrap());
                    tracing::debug!("Notifying {} clients with: {payload:?}", destinations.len());

                    // Send the notification to all destinations in parallel.
                    let mut results = Vec::new();
                    for destination in destinations.values_mut() {
                        tracing::debug!("Notifying client {}", destination.who);
                        results.push(destination.sink.send(payload.clone()).map(|r| match r {
                            Ok(_) => {
                                tracing::debug!("Notified {}", destination.who);
                            }
                            Err(e) => {
                                tracing::debug!("Failed to notify {}: {}", destination.who, e);
                            }
                        }));
                    }
                    futures::future::join_all(results).await;
                }
                None => {
                    tracing::debug!("Got None from notify stream; continuing");
                    continue;
                }
                Some(Err(e)) => {
                    // TODO: Make sure all errors should be fatal and prevent the notifier from running further.
                    // On termination, the error `attempted to acquire a connection on a closed pool` is seen here.
                    tracing::debug!("Finishing notifier with error: {}", e);
                    break;
                }
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum Action {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Payload {
    pub timestamp: String,
    pub table: String,
    pub action: Action,
    pub id: String,
    // TODO: Both record and old are json marshalled by the database.
    // It'd be good to massage this into our domain objects here
    // rather than rely on database entirely.
    pub record: String,
    pub old: Option<String>,
}

/// Creates a stream of insert, update and delete task notifications.
fn stream_task_notifications(
    pool: Pool<Postgres>,
) -> impl Stream<Item = Result<Payload, sqlx::Error>> {
    let channels: Vec<&str> = vec!["table_update"];

    try_stream! {
        tracing::debug!("Setting up DB listeners on channels {:?}..", channels);
        let mut listener: PgListener = PgListener::connect_with(&pool).await.unwrap();
        listener.listen_all(channels).await.unwrap();

        loop {
            match listener.try_recv().await? {
                Some(notification) => {
                    tracing::debug!("Yielding notification {:?}", &notification);
                    match serde_json::from_str::<Payload>(notification.payload()) {
                        Ok(payload) => yield payload,
                        Err(e) => tracing::warn!("Discarding unparseable notification ({:?}) due to parse error: {}", notification, e ),
                    };
                },
                None => {
                    tracing::debug!("Notification listener lost database connection. Some notifications may be lost. Reconnecting...");
                    continue;
                },
            }
        }
    }
}
