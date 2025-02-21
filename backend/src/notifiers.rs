use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, prelude::FromRow};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ParseMode, UserId};

use crate::flags::is_dev;

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

pub(super) struct Notifier {
    pool: &'static Pool<Postgres>,
    bot: Option<teloxide::Bot>,
}

impl Notifier {
    pub(super) fn new(pool: &'static Pool<Postgres>) -> Result<Self> {
        Ok(Self {
            pool,
            bot: match telegram::bot_from_secrets() {
                Ok(bot) => Some(bot),
                Err(e) => {
                    if is_dev() {
                        None
                    } else {
                        return Err(e.context("Failed to initialize telegram bot"));
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
                NotifierSettings::Telegram(settings) => {
                    if let Some(bot) = &self.bot {
                        bot.send_message(UserId(settings.chat_id), message)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
