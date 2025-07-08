use crate::{
    api::{
        ApiResult, bad_request_error, error_response,
        google::{self, User},
        unauthorized_error,
    },
    notifiers::{
        DiscordSettings, NotifierSettings, delete_notification_config, fetch_notification_config,
        insert_notification_config,
    },
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Context, Result, anyhow};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, post},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(super) struct DiscordClient {
    client: reqwest::Client,
    token: Secret<String>,
}

impl DiscordClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            token: read_secret("discord/token")?,
        })
    }

    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");

        let payload = json!({
            "content": content
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.token.data))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to send Discord message: {}",
                response.status(),
            ));
        }

        Ok(())
    }
}

pub(super) fn router() -> Router {
    let routes = Router::new()
        .route("/", post(authorize_discord))
        .route("/", delete(deauthorize_discord))
        .route("/test", post(send_test_message_handler))
        .layer(middleware::from_fn(google::authenticate));

    let webhooks = Router::new()
        .route("/interaction", post(handle_interaction))
        .layer(middleware::from_fn(verify_discord_signature));

    Router::new().merge(routes).merge(webhooks)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: u64,
    channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizeDiscord {
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiscordChannel {
    id: String,
}

#[tracing::instrument(skip(user, pool, key))]
async fn authorize_discord(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(key): Extension<DecodingKey>,
    Json(req): Json<AuthorizeDiscord>,
) -> ApiResult<Json<NotifierSettings>> {
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

    let settings = NotifierSettings::Discord(DiscordSettings {
        channel_id: token.claims.channel_id,
    });

    insert_notification_config(&user.email, &settings, pool).await?;

    Ok(Json(settings))
}

#[tracing::instrument(skip(user, pool))]
async fn deauthorize_discord(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    delete_notification_config(&user.email, "discord", pool).await?;
    Ok(Json(()))
}

#[tracing::instrument(skip(user, pool))]
async fn send_test_message_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    let config = fetch_notification_config(&user.email, "discord", pool).await?;

    let NotifierSettings::Discord(settings) = config.settings else {
        return Err(anyhow!("Got a setting config that wasn't Discord").into());
    };

    let client = DiscordClient::new()?;
    client
        .send_message(
            &settings.channel_id,
            "Hello from Koso! This is a test notification.",
        )
        .await?;

    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequest {
    r#type: u8,
    channel_id: Option<String>,
    data: Option<InteractionRequestData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequestData {
    name: Option<String>,
}

#[axum::debug_handler]
async fn handle_interaction(
    Extension(key): Extension<jsonwebtoken::EncodingKey>,
    Json(req): Json<InteractionRequest>,
) -> ApiResult<Json<Value>> {
    tracing::debug!("Discord interaction: {:?}", req);

    match req.r#type {
        1 => {
            return Ok(Json(json!({
                "type": 1
            })));
        }
        2 => {
            // Application Command
            let channel_id = req.channel_id.context("No channel in interaction")?;
            let command = req
                .data
                .context("No data in interaction")?
                .name
                .context("No command in interaction")?;

            match command.as_str() {
                "token" => {
                    let auth_url = get_auth_url(key, &channel_id)?;

                    return Ok(Json(json!({
                        "type": 4, // CHANNEL_MESSAGE_WITH_SOURCE
                        "data": {
                            "content": format!("Click here to authorize Koso: {}", auth_url),
                            "flags": 64 // EPHEMERAL
                        }
                    })));
                }
                _ => {
                    return Err(bad_request_error(
                        "UNKNOWN_DISCORD_COMMAND",
                        &format!("Unknown Discord command: {command}"),
                    ));
                }
            }
        }
        _ => {}
    };

    // Default response for unknown commands
    Ok(Json(json!({
        "type": 4,
        "data": {
            "content": "Use `/token` to start the authorization flow.",
            "flags": 64
        }
    })))
}

fn get_auth_url(key: EncodingKey, channel_id: &str) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        channel_id: channel_id.into(),
    };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::debug!("Generated auth token {token} for {channel_id}");
    Ok(format!("{host}/connections/discord?token={token}"))
}

const BODY_LIMIT: usize = 10 * 1024 * 1024;

async fn verify_discord_signature(request: Request, next: Next) -> ApiResult<Response> {
    let Ok(verifying_key) = get_verifying_key() else {
        return Err(unauthorized_error("Failed to get verifying key"));
    };

    let Ok(request) = verify_signature(request, &verifying_key).await else {
        return Err(unauthorized_error("Failed to verify signature"));
    };

    Ok(next.run(request).await)
}

async fn verify_signature(request: Request, verifying_key: &VerifyingKey) -> Result<Request> {
    let (parts, body) = request.into_parts();

    let signature = &parts
        .headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
        .context("Missing x-signature-ed25519 header")?;

    let timestamp = &parts
        .headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
        .context("Missing x-signature-timestamp header")?;

    let signature_array: [u8; 64] = hex::decode(signature)
        .context("Invalid hex signature")?
        .try_into()
        .map_err(|_| anyhow!("Signature must be 64 bytes"))?;

    let signature = Signature::from_bytes(&signature_array);

    let body_bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .context("Invalid body")?;

    let message = format!("{timestamp}{}", std::str::from_utf8(&body_bytes)?);

    verifying_key
        .verify(message.as_bytes(), &signature)
        .context("Signature verification failed")?;

    Ok(Request::from_parts(parts, Body::from(body_bytes)))
}

fn get_verifying_key() -> Result<VerifyingKey> {
    let public_key_hex = read_secret::<String>("discord/public_key")?;
    let public_key_array: [u8; 32] = hex::decode(public_key_hex.data)
        .context("Invalid hex public key")?
        .try_into()
        .map_err(|_| anyhow!("Public key must be 32 bytes"))?;

    VerifyingKey::from_bytes(&public_key_array).context("Invalid public key")
}
