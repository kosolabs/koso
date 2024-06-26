use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use sqlx::postgres::PgListener;
use sqlx::{Pool, Postgres};

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Serialize, Debug)]
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

use futures::stream::Stream;

/// Creates a stream of insert, update and delete task notifications.
pub fn stream_task_notifications(
    pool: &Pool<Postgres>,
) -> impl Stream<Item = Result<Payload, sqlx::Error>> + '_ {
    let channels: Vec<&str> = vec!["table_update"];
    use async_stream::try_stream;

    try_stream! {
        tracing::debug!("Setting up DB listeners on channels {:?}..", channels);
        let mut listener: PgListener = PgListener::connect_with(pool).await.unwrap();
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
