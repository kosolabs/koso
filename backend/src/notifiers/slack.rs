use crate::{
    api::{
        ApiResult, error_response,
        google::{self, User},
    },
    notifiers::{
        NotifierSettings, SlackSettings, delete_notification_config, fetch_notification_config,
        insert_notification_config,
    },
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Result, anyhow};
use axum::{
    Extension, Form, Json, Router, middleware,
    routing::{delete, post},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

pub(super) fn router() -> Result<Router> {
    Ok(Router::new()
        .route("/", post(authorize_slack))
        .route("/", delete(deauthorize_slack))
        .route("/test", post(send_test_message_handler))
        .layer(middleware::from_fn(google::authenticate))
        .route("/command", post(handle_slash_command))
        .route("/interact", post(handle_interactivity)))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: u64,
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

#[tracing::instrument(skip(user, pool))]
async fn send_test_message_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    let config = fetch_notification_config(&user.email, "slack", pool).await?;

    let NotifierSettings::Slack(settings) = config.settings else {
        return Err(anyhow!("Got a setting config that wasn't Slack").into());
    };

    let client = SlackClient::new()?;
    client
        .send_message(
            &settings.user_id,
            "Hello from Koso! This is a test notification.",
        )
        .await?;

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

async fn handle_slash_command(
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

// get_auth_url(key, &req.user_id)?,
fn get_auth_url(key: EncodingKey, user: &str) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        user: user.into(),
    };
    let token = encode(&Header::default(), &claims, &key)?;
    Ok(format!("{host}/connections/slack?token={token}"))
}

async fn handle_interactivity() -> ApiResult<()> {
    Ok(())
}
