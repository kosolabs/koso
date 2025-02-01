use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::JsonValue};

pub(crate) mod telegram;

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserNotificationConfig {
    pub(crate) email: String,
    pub(crate) notifier: String,
    pub(crate) enabled: bool,
    pub(crate) settings: JsonValue,
}

pub(crate) fn router() -> Router {
    Router::new().nest("/telegram", telegram::router())
}
