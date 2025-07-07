use crate::{
    api::{
        ApiResult, error_response,
        google::{self, User},
    },
    notifiers::{
        DiscordSettings, NotifierSettings, delete_notification_config, fetch_notification_config,
        insert_notification_config,
    },
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Result, anyhow};
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

    pub async fn send_dm(&self, user_id: &str, content: &str) -> Result<()> {
        // First create a DM channel
        let dm_channel = self.create_dm_channel(user_id).await?;

        // Then send message to that channel
        self.send_message(&dm_channel.id, content).await
    }

    async fn create_dm_channel(&self, user_id: &str) -> Result<DiscordChannel> {
        let url = "https://discord.com/api/v10/users/@me/channels";

        let payload = json!({
            "recipient_id": user_id
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bot {}", self.token.data))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to create DM channel: {}",
                response.status(),
            ));
        }

        let channel: DiscordChannel = response.json().await?;
        Ok(channel)
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
    user_id: String,
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
        user_id: token.claims.user_id,
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
        .send_dm(
            &settings.user_id,
            "Hello from Koso! This is a test notification.",
        )
        .await?;

    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequest {
    r#type: u8,
    data: Option<InteractionRequestData>,
    user: Option<InteractionRequestUser>,
    member: Option<InteractionRequestMember>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequestData {
    custom_id: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequestUser {
    id: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct InteractionRequestMember {
    user: InteractionRequestUser,
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
            let user = req
                .user
                .or_else(|| req.member.map(|m| m.user))
                .ok_or_else(|| anyhow!("No user in interaction"))?;

            if let Some(data) = req.data {
                if data.name.as_deref() == Some("token") {
                    let auth_url = get_auth_url(key, &user.id)?;

                    return Ok(Json(json!({
                        "type": 4, // CHANNEL_MESSAGE_WITH_SOURCE
                        "data": {
                            "content": format!("Click here to authorize Koso: {}", auth_url),
                            "flags": 64 // EPHEMERAL
                        }
                    })));
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

fn get_auth_url(key: EncodingKey, user_id: &str) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        user_id: user_id.into(),
    };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::debug!("Generated auth token {token} for {user_id}");
    Ok(format!("{host}/connections/discord?token={token}"))
}

const BODY_LIMIT: usize = 10 * 1024 * 1024;

async fn verify_discord_signature(request: Request, next: Next) -> Result<Response, StatusCode> {
    let Ok(verifying_key) = get_verifying_key() else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let (parts, body) = request.into_parts();

    let body_bytes = match axum::body::to_bytes(body, BODY_LIMIT).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    if verify_signature(&parts.headers, &body_bytes, &verifying_key).is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let request = Request::from_parts(parts, Body::from(body_bytes));

    Ok(next.run(request).await)
}

fn verify_signature(
    headers: &axum::http::HeaderMap,
    body: &[u8],
    verifying_key: &VerifyingKey,
) -> ApiResult<()> {
    let signature = headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing x-signature-ed25519 header"))?;

    let timestamp = headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing x-signature-timestamp header"))?;

    let signature_array: [u8; 64] = hex::decode(signature)
        .map_err(|e| anyhow!("Invalid signature hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow!("Signature must be 64 bytes"))?;

    let signature = Signature::from_bytes(&signature_array);

    let message = format!("{timestamp}{}", std::str::from_utf8(body)?);

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|e| anyhow!("Signature verification failed: {e}"))?;

    Ok(())
}

fn get_verifying_key() -> Result<VerifyingKey> {
    let public_key_hex = read_secret::<String>("discord/public_key")?;
    let public_key_array: [u8; 32] = hex::decode(public_key_hex.data)
        .map_err(|e| anyhow!("Invalid public key hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow!("Public key must be 32 bytes"))?;

    VerifyingKey::from_bytes(&public_key_array).map_err(|e| anyhow!("Invalid public key: {e}"))
}
