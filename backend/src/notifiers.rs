use anyhow::{Context as _, Result};
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};

use crate::notifiers::slack::SlackClient;
use crate::notifiers::telegram::TelegramClient;
use crate::settings::settings;

pub(crate) mod discord;
pub(crate) mod slack;
pub(crate) mod telegram;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct DiscordSettings {
    pub(super) channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct SlackSettings {
    pub(super) user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct TelegramSettings {
    pub(super) chat_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(super) enum NotifierSettings {
    Discord(DiscordSettings),
    Slack(SlackSettings),
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

pub(super) fn router() -> Result<Router> {
    Ok(Router::new()
        .nest("/discord", discord::router())
        .nest("/slack", slack::router())
        .nest("/telegram", telegram::router()))
}

pub(super) struct Notifier {
    pool: &'static PgPool,
    discord: Option<discord::DiscordClient>,
    slack: Option<slack::SlackClient>,
    telegram: Option<telegram::TelegramClient>,
}

impl Notifier {
    pub(super) fn new(pool: &'static PgPool) -> Result<Self> {
        Ok(Self {
            pool,
            discord: match discord::DiscordClient::new() {
                Ok(client) => Some(client),
                Err(e) => {
                    if settings().is_dev() {
                        None
                    } else {
                        return Err(e.context("Failed to initialize Discord client"));
                    }
                }
            },
            slack: match SlackClient::new() {
                Ok(client) => Some(client),
                Err(e) => {
                    if settings().is_dev() {
                        None
                    } else {
                        return Err(e.context("Failed to initialize Slack client"));
                    }
                }
            },
            telegram: match TelegramClient::new() {
                Ok(client) => Some(client),
                Err(e) => {
                    if settings().is_dev() {
                        None
                    } else {
                        return Err(e.context("Failed to initialize Telegram bot"));
                    }
                }
            },
        })
    }

    pub(super) async fn notify(&self, recipient: &str, message: &str) -> Result<()> {
        let configs: Vec<UserNotificationConfig> = sqlx::query_as(
            "
            SELECT email, notifier, enabled, settings
            FROM user_notification_configs
            WHERE email = $1",
        )
        .bind(recipient)
        .fetch_all(self.pool)
        .await?;

        for config in configs {
            match config.settings {
                NotifierSettings::Discord(settings) => {
                    if let Some(discord) = &self.discord {
                        discord.send_message(&settings.channel_id, message).await?;
                    }
                }
                NotifierSettings::Slack(settings) => {
                    if let Some(slack) = &self.slack {
                        slack.send_message(&settings.user_id, message).await?;
                    }
                }
                NotifierSettings::Telegram(settings) => {
                    if let Some(client) = &self.telegram {
                        client.send_message(settings.chat_id, message).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub(crate) async fn fetch_notification_configs(
    email: &str,
    pool: &PgPool,
) -> Result<Vec<UserNotificationConfig>> {
    sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1",
    )
    .bind(email)
    .fetch_all(pool)
    .await
    .context("Failed to query notification configs")
}

pub(crate) async fn fetch_notification_config(
    email: &str,
    notifier: &str,
    pool: &PgPool,
) -> Result<UserNotificationConfig> {
    sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1 AND notifier = $2",
    )
    .bind(email)
    .bind(notifier)
    .fetch_one(pool)
    .await
    .context("Failed to query notification config")
}

pub(crate) async fn insert_notification_config(
    email: &str,
    settings: &NotifierSettings,
    pool: &PgPool,
) -> Result<()> {
    let notifier = match settings {
        NotifierSettings::Discord(_) => "discord",
        NotifierSettings::Slack(_) => "slack",
        NotifierSettings::Telegram(_) => "telegram",
    };
    sqlx::query(
        "
        INSERT INTO user_notification_configs (email, notifier, enabled, settings)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (email, notifier)
        DO UPDATE SET enabled = EXCLUDED.enabled, settings = EXCLUDED.settings",
    )
    .bind(email)
    .bind(notifier)
    .bind(true)
    .bind(sqlx::types::Json(settings))
    .execute(pool)
    .await
    .context("Failed to insert notification config")
    .map(|_| ())
}

pub(crate) async fn delete_notification_config(
    email: &str,
    notifier: &str,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query(
        "
        DELETE FROM user_notification_configs
        WHERE email = $1 AND notifier = $2",
    )
    .bind(email)
    .bind(notifier)
    .execute(pool)
    .await
    .context("Failed to delete notification config")
    .map(|_| ())
}
