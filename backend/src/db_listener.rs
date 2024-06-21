use std::fmt::Debug;

use serde::Deserialize;

use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::{Pool, Postgres};

#[derive(Deserialize, Debug)]
pub enum Action {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub timestamp: String,
    pub table: String,
    pub action: Action,
    pub id: String,
    pub record: String,
    pub old: Option<String>,
}

pub async fn start_listening<T: DeserializeOwned + Sized + Debug>(
    pool: &Pool<Postgres>,
    channels: Vec<&str>,
    call_back: impl Fn(T),
) -> Result<(), Error> {
    tracing::debug!("Setting up DB listeners on channels {:?}..", channels);
    let mut listener = PgListener::connect_with(pool).await.unwrap();
    listener.listen_all(channels).await?;
    loop {
        tracing::debug!("Waiting for DB notification..");
        while let Some(notification) = listener.try_recv().await? {
            tracing::debug!("Getting notification {:#?}", notification);

            match serde_json::from_str::<T>(notification.payload()) {
                Ok(payload) => call_back(payload),
                Err(e) => tracing::warn!(
                    "Failed to parse payload: {} from notification {:#?}",
                    e,
                    notification
                ),
            };
        }
    }
}

pub async fn listen_for_notifications(pool: &Pool<Postgres>) -> Result<(), Error> {
    let call_back = |payload: Payload| {
        match payload.action {
            Action::INSERT => {
                tracing::debug!("Processing insert event for payload '{:#?}'", payload);
            }
            Action::UPDATE => {
                tracing::debug!("Processing update event for payload '{:#?}'", payload);
            }
            Action::DELETE => {
                tracing::debug!("Processing delete event for payload '{:#?}'", payload);
            }
        };
    };

    let channels = vec!["table_update"];
    let res = start_listening(&pool, channels, call_back).await;
    tracing::debug!("Finished listening with result {:#?}", res);
    return res;
}
