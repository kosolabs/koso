use anyhow::{Context as _, Result};
use axum::routing::post;
use axum::{Extension, Json, Router, middleware};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};

use crate::api::google::User;
use crate::api::{ApiResult, google};
use crate::notifiers::slack::SlackClient;
use crate::notifiers::teams::TeamsClient;
use crate::notifiers::telegram::TelegramClient;
use crate::settings::settings;

pub(crate) mod discord;
pub(crate) mod slack;
pub(crate) mod teams;
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
#[serde(rename_all = "camelCase")]
pub(super) struct TeamsSettings {
    pub(super) bot_token: String,
    pub(super) channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(super) enum NotifierSettings {
    Discord(DiscordSettings),
    Slack(SlackSettings),
    Telegram(TelegramSettings),
    Teams(TeamsSettings),
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
        .route("/", post(send))
        .layer(middleware::from_fn(google::authenticate))
        .nest("/discord", discord::router())
        .nest("/slack", slack::router())
        .nest("/telegram", telegram::router())
        .nest("/teams", teams::router()))
}

#[derive(Serialize, Deserialize, Debug)]
struct SendMessageRequest {
    message: String,
    notifiers: Option<Vec<String>>,
}

#[tracing::instrument(skip(user, pool))]
async fn send(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Json(req): Json<SendMessageRequest>,
) -> ApiResult<Json<()>> {
    let notifier = Notifier::new(pool)?;
    notifier
        .notify(&user.email, &req.message, req.notifiers)
        .await?;
    Ok(Json(()))
}

pub(super) struct Notifier {
    pool: &'static PgPool,
    discord: Option<discord::DiscordClient>,
    slack: Option<slack::SlackClient>,
    telegram: Option<telegram::TelegramClient>,
    teams: Option<teams::TeamsClient>,
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
            teams: match TeamsClient::new() {
                Ok(client) => Some(client),
                Err(e) => {
                    if settings().is_dev() {
                        None
                    } else {
                        return Err(e.context("Failed to initialize Teams client"));
                    }
                }
            },
        })
    }

    pub(super) async fn notify(
        &self,
        recipient: &str,
        message: &str,
        notifiers: Option<Vec<String>>,
    ) -> Result<()> {
        let notifiers = notifiers.unwrap_or(
            vec!["discord", "slack", "telegram", "teams"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        );
        let configs: Vec<UserNotificationConfig> = sqlx::query_as(
            "
            SELECT email, notifier, enabled, settings
            FROM user_notification_configs
            WHERE email = $1
            AND notifier = ANY($2)",
        )
        .bind(recipient)
        .bind(notifiers)
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
                NotifierSettings::Teams(settings) => {
                    if let Some(client) = &self.teams {
                        client
                            .send_message(&settings.bot_token, &settings.channel_id, message)
                            .await?;
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

pub(crate) async fn insert_notification_config(
    email: &str,
    settings: &NotifierSettings,
    pool: &PgPool,
) -> Result<()> {
    let notifier = match settings {
        NotifierSettings::Discord(_) => "discord",
        NotifierSettings::Slack(_) => "slack",
        NotifierSettings::Telegram(_) => "telegram",
        NotifierSettings::Teams(_) => "teams",
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
