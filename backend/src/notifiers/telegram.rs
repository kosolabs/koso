use crate::api::{ApiResult, error_response, google::User};
use crate::notifiers::{NotifierSettings, TelegramSettings, UserNotificationConfig};
use crate::settings::settings;
use crate::{
    secrets::{Secret, read_secret},
    server::shutdown_signal,
};
use anyhow::Result;
use axum::{
    Extension, Json, Router,
    routing::{delete, post},
};
use dptree::case;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use teloxide::{
    Bot,
    dispatching::UpdateFilterExt,
    dptree,
    macros::BotCommands,
    payloads::SendMessageSetters,
    prelude::{Dispatcher, Requester},
    types::{ParseMode, Update, UserId},
};

pub(super) fn router() -> Router {
    Router::new()
        .route("/", post(authorize_telegram))
        .route("/", delete(deauthorize_telegram))
        .route("/test", post(send_test_message_handler))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: u64,
    chat_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizeTelegram {
    token: String,
}

#[tracing::instrument(skip(user, pool))]
async fn authorize_telegram(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Json(req): Json<AuthorizeTelegram>,
) -> ApiResult<Json<NotifierSettings>> {
    let key = decoding_key_from_secrets()?;
    let token = match decode::<Claims>(&req.token, &key, &Validation::default()) {
        Ok(token) => token,
        Err(error) => {
            return Err(error_response(
                StatusCode::PRECONDITION_FAILED,
                "VALIDATION_FAILED",
                Some(&format!("{error}")),
                None,
            ));
        }
    };

    let settings = NotifierSettings::Telegram(TelegramSettings {
        chat_id: token.claims.chat_id,
    });

    sqlx::query(
        "
        INSERT INTO user_notification_configs (email, notifier, enabled, settings)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (email, notifier)
        DO UPDATE SET enabled = EXCLUDED.enabled, settings = EXCLUDED.settings",
    )
    .bind(user.email)
    .bind("telegram")
    .bind(true)
    .bind(sqlx::types::Json(&settings))
    .execute(pool)
    .await?;

    Ok(Json(settings))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Empty {}

#[tracing::instrument(skip(user, pool))]
async fn deauthorize_telegram(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Empty>> {
    sqlx::query(
        "
        DELETE FROM user_notification_configs
        WHERE email = $1 AND notifier = 'telegram'",
    )
    .bind(user.email)
    .execute(pool)
    .await?;

    Ok(Json(Empty {}))
}

#[tracing::instrument(skip(user, pool))]
async fn send_test_message_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Empty>> {
    let config: UserNotificationConfig = sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1 AND notifier = 'telegram'",
    )
    .bind(user.email)
    .fetch_one(pool)
    .await?;

    let NotifierSettings::Telegram(settings) = config.settings;

    let bot = bot_from_secrets()?;
    bot.send_message(
        UserId(settings.chat_id),
        "Hello from Koso! This is a test notification. Change your setting <a href=\"https://koso.app/profile\">here</a>.",
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(Json(Empty {}))
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Token,
}

pub(super) fn bot_from_secrets() -> Result<Bot> {
    let secret: Secret<String> = read_secret("telegram/token")?;
    Ok(Bot::new(secret.data))
}

fn decoding_key_from_secrets() -> Result<DecodingKey> {
    let secret: Secret<String> = read_secret("koso/hmac")?;
    Ok(DecodingKey::from_base64_secret(&secret.data)?)
}

fn encoding_key_from_secrets() -> Result<EncodingKey> {
    let secret: Secret<String> = read_secret("koso/hmac")?;
    Ok(EncodingKey::from_base64_secret(&secret.data)?)
}

pub(crate) async fn start_telegram_server() -> Result<()> {
    let bot = match bot_from_secrets() {
        Ok(bot) => bot,
        Err(error) => {
            if settings().is_dev() {
                tracing::warn!("Telegram bot not started because token is not set.");
                return Ok(());
            } else {
                return Err(error);
            }
        }
    };
    let key = encoding_key_from_secrets()?;
    let schema = Update::filter_message()
        .filter_map(|update: Update| update.from().cloned())
        .branch(
            teloxide::filter_command::<Command, _>()
                .branch(case![Command::Token].endpoint(send_token)),
        )
        .branch(dptree::endpoint(send_usage));
    let mut dis = Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![key])
        .build();

    let token = dis.shutdown_token();
    let abort_token = tokio::spawn(async move { dis.dispatch().await });

    shutdown_signal("telegram bot", None).await;
    match token.shutdown() {
        Err(error) => {
            tracing::warn!("Error while shutting down Teloxide: {error}");
        }
        Ok(f) => {
            if tokio::time::timeout(Duration::from_secs(2), f)
                .await
                .is_err()
            {}
        }
    }
    // Finally, in case we weren't able to cleanly shut down the dispatcher,
    // abort the dispatcher task. This can happen when shutdown races with
    // startup and the call to shutdown() above returns an error, or when
    // waiting for the shutdown future to complete times out.
    abort_token.abort();

    tracing::info!("Telegram bot shutdown.");

    Ok(())
}

async fn send_usage(bot: Bot, user: teloxide::types::User) -> Result<()> {
    tracing::debug!("Sending usage to {user:?}");
    bot.send_message(
        user.id,
        concat!(
            "I can help you authorize Koso to send notifications.\n\n",
            "/token - start the authorization flow"
        ),
    )
    .await?;
    Ok(())
}

async fn send_token(bot: Bot, key: EncodingKey, user: teloxide::types::User) -> Result<()> {
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        chat_id: user.id.0,
    };
    let token = encode(&Header::default(), &claims, &key)?;
    let url = format!("https://koso.app/connections/telegram?token={token}");

    tracing::debug!("Sending auth token {token} to {user:?}");
    let message = format!("Follow this link to authorize Koso: <a href=\"{url}\">{url}</a>");
    bot.send_message(user.id, message)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
