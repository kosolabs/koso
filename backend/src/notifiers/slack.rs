use crate::{
    api::google::{self, User},
    notifiers::{
        NotifierSettings, SlackSettings, delete_notification_config, insert_notification_config,
    },
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Context as _, Result, anyhow};
use axum::{
    Extension, Form, Json, Router,
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, post},
};
use axum_anyhow::{ApiResult, ResultExt};
use chrono::Utc;
use hmac::{Hmac, Mac};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use sqlx::postgres::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(super) struct SlackClient {
    client: reqwest::Client,
    token: Secret<String>,
}

impl SlackClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            token: read_secret("slack/token")?,
        })
    }

    pub async fn send_message(&self, channel: &str, text: &str) -> Result<()> {
        let url = "https://slack.com/api/chat.postMessage";

        let payload = serde_json::json!({
            "channel": channel,
            "text": text
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.token.data))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

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
    let routes = Router::new()
        .route("/", post(authorize_slack))
        .route("/", delete(deauthorize_slack))
        .layer(middleware::from_fn(google::authenticate));

    let webhooks = Router::new()
        .route("/command", post(handle_command))
        .route("/interact", post(handle_interactivity))
        .layer(middleware::from_fn(verify_slack_signature));

    Router::new().merge(routes).merge(webhooks)
}

const ISSUER: &str = "koso-notifiers-slack";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: u64,
    iss: String,
    user: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizeSlack {
    token: String,
}

#[tracing::instrument(skip(user, pool, key))]
async fn authorize_slack(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(key): Extension<DecodingKey>,
    Json(req): Json<AuthorizeSlack>,
) -> ApiResult<Json<NotifierSettings>> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);
    validation.required_spec_claims.insert("iss".to_string());
    let token = decode::<Claims>(&req.token, &key, &validation).context_status(
        StatusCode::PRECONDITION_FAILED,
        "VALIDATION_FAILED",
        "Invalid token",
    )?;

    let settings = NotifierSettings::Slack(SlackSettings {
        user_id: token.claims.user,
    });

    insert_notification_config(&user.email, &settings, pool).await?;

    Ok(Json(settings))
}

#[tracing::instrument(skip(user, pool))]
async fn deauthorize_slack(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    delete_notification_config(&user.email, "slack", pool).await?;
    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug)]
struct SlashCommandRequest {
    user_id: String,
    command: String,
    response_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SlashCommandResponse {
    response_type: String,
    blocks: serde_json::Value,
}

async fn handle_command(
    Extension(key): Extension<jsonwebtoken::EncodingKey>,
    Form(req): Form<SlashCommandRequest>,
) -> ApiResult<Json<SlashCommandResponse>> {
    tracing::debug!("{:?}", req);

    Ok(Json(SlashCommandResponse {
        response_type: "ephemeral".into(),
        blocks: json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "*Koso Authorization*\n\nClick the button below to connect your Koso account to Slack."
                },
            },
            {
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "Authorize Koso"
                        },
                        "url": get_auth_url(key, &req.user_id)?
                    }
                ]
            },
        ]),
    }))
}

fn get_auth_url(key: EncodingKey, user: &str) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: ISSUER.to_string(),
        user: user.into(),
    };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::debug!("Generated auth token {token} for {user}");
    Ok(format!("{host}/connections/slack?token={token}"))
}

async fn handle_interactivity() -> ApiResult<()> {
    Ok(())
}

const BODY_LIMIT: usize = 10 * 1024 * 1024;

async fn verify_slack_signature(request: Request, next: Next) -> ApiResult<Response> {
    let signing_secret = read_secret::<String>("slack/signing_secret")?;

    let request = verify_signature(request, &signing_secret)
        .await
        .context_forbidden("UNAUTHORIZED", "Failed to verify signature")?;

    Ok(next.run(request).await)
}

async fn verify_signature(request: Request, signing_secret: &Secret<String>) -> Result<Request> {
    let (parts, body) = request.into_parts();

    let expected_signature = hex::decode(
        parts
            .headers
            .get("x-slack-signature")
            .context("Missing x-slack-signature header")?
            .as_bytes()
            .strip_prefix(b"v0=")
            .context("Invalid signature prefix")?,
    )
    .context("Invalid hex in signature")?;

    let timestamp = parts
        .headers
        .get("x-slack-request-timestamp")
        .and_then(|v| v.to_str().ok())
        .context("Missing x-slack-request-timestamp header")?
        .parse::<i64>()
        .context("Failed to parse timestamp")?;

    if (Utc::now().timestamp() - timestamp).abs() > 300 {
        return Err(anyhow!("Timestamp is stale: {timestamp}"));
    }

    let body_bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .context("Invalid body")?;

    let message = format!("v0:{timestamp}:{}", std::str::from_utf8(&body_bytes)?);

    Hmac::<Sha256>::new_from_slice(signing_secret.data.as_bytes())?
        .chain_update(message.as_bytes())
        .verify_slice(&expected_signature)
        .context("Signature verification failed")?;

    Ok(Request::from_parts(parts, Body::from(body_bytes)))
}
