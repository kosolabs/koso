use crate::api::google;
use crate::api::{ApiResult, error_response, google::User};
use crate::notifiers::{
    NotifierSettings, TelegramSettings, delete_notification_config, insert_notification_config,
};
use crate::secrets::{Secret, read_secret};
use crate::settings::settings;
use anyhow::{Context, Result};
use axum::middleware;
use axum::{
    Extension, Json, Router,
    routing::{delete, post},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(super) struct TelegramClient {
    client: reqwest::Client,
    token: Secret<String>,
}

impl TelegramClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            token: read_secret("telegram/token")?,
        })
    }

    pub async fn send_message(&self, chat_id: u64, markdown: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.token.data
        );

        tracing::debug!("{:?}", url);

        let req = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&json!( {
                "chat_id": chat_id,
                "text": markdown,
                "parse_mode": "Markdown",
            }));

        tracing::debug!("{:?}", req);

        let response = req.send().await?;

        tracing::debug!("{:?}", response);

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to send message: {}",
                response.status()
            ));
        }

        Ok(())
    }
}

pub(super) fn router() -> Router {
    Router::new()
        .route("/", post(authorize_telegram))
        .route("/", delete(deauthorize_telegram))
        .layer(middleware::from_fn(google::authenticate))
        .route("/webhook", post(handle_webhook))
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

#[tracing::instrument(skip(user, pool, key))]
async fn authorize_telegram(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(key): Extension<DecodingKey>,
    Json(req): Json<AuthorizeTelegram>,
) -> ApiResult<Json<NotifierSettings>> {
    let token = match decode::<Claims>(&req.token, &key, &Validation::default())
        .context("Failed to decode token")
    {
        Ok(token) => token,
        Err(error) => {
            return Err(error_response(
                StatusCode::PRECONDITION_FAILED,
                "VALIDATION_FAILED",
                "Invalid token",
                Some(error),
            ));
        }
    };

    let settings = NotifierSettings::Telegram(TelegramSettings {
        chat_id: token.claims.chat_id,
    });

    insert_notification_config(&user.email, &settings, pool).await?;

    Ok(Json(settings))
}

#[tracing::instrument(skip(user, pool))]
async fn deauthorize_telegram(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    delete_notification_config(&user.email, "telegram", pool).await?;
    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramUpdate {
    update_id: u64,
    message: TelegramMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramMessage {
    message_id: u64,
    from: TelegramUser,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramUser {
    id: u64,
}

async fn handle_webhook(
    Extension(key): Extension<jsonwebtoken::EncodingKey>,
    Json(req): Json<TelegramUpdate>,
) -> ApiResult<Json<()>> {
    let client = TelegramClient::new()?;

    match req.message.text.as_str() {
        "/token" => send_token(&client, key, req.message.from.id).await?,
        _ => send_usage(&client, req.message.from.id).await?,
    }
    tracing::debug!("{:?}", req);
    Ok(Json(()))
}

async fn send_usage(client: &TelegramClient, user_id: u64) -> Result<()> {
    tracing::debug!("Sending usage to {user_id}");
    client
        .send_message(
            user_id,
            concat!(
                "I can help you authorize Koso to send notifications.\n\n",
                "/token - start the authorization flow"
            ),
        )
        .await?;
    Ok(())
}

async fn send_token(client: &TelegramClient, key: EncodingKey, chat_id: u64) -> Result<()> {
    let url = get_auth_url(key, chat_id)?;
    let message = format!("Follow this link to authorize Koso: <a href=\"{url}\">{url}</a>");
    client.send_message(chat_id, &message).await?;
    Ok(())
}

fn get_auth_url(key: EncodingKey, chat_id: u64) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        chat_id,
    };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::debug!("Generated auth token {token} for {chat_id}");
    Ok(format!("{host}/connections/telegram?token={token}"))
}
