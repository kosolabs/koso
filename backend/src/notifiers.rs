use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Pool, Postgres};
use teloxide::prelude::Requester;
use teloxide::types::UserId;

pub(crate) mod telegram;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct TelegramSettings {
    pub(super) chat_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(super) enum NotifierSettings {
    Telegram(TelegramSettings),
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserNotificationConfig {
    pub(super) email: String,
    pub(super) notifier: String,
    pub(super) enabled: bool,
    #[sqlx(json)]
    pub(super) settings: NotifierSettings,
}

pub(super) fn router() -> Router {
    Router::new().nest("/telegram", telegram::router())
}

pub(super) async fn notify(
    pool: &Pool<Postgres>,
    telegram_bot: &teloxide::Bot,
    recipient: &str,
    message: &str,
) -> Result<()> {
    let configs: Vec<UserNotificationConfig> = sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1",
    )
    .bind(recipient)
    .fetch_all(pool)
    .await?;

    for config in configs {
        match config.settings {
            NotifierSettings::Telegram(settings) => {
                telegram_bot
                    .send_message(UserId(settings.chat_id), message)
                    .await?;
            }
        }
    }

    Ok(())
}
