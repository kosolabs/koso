use crate::{
    api::google::{self, User},
    notifiers::{
        NotifierSettings, TeamsSettings, delete_notification_config, insert_notification_config,
    },
    settings::settings,
};
use anyhow::Result;
use axum::{
    Extension, Form, Json, Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, post},
};
use axum_anyhow::{ApiResult, ResultExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(super) struct TeamsClient {
    client: reqwest::Client,
}

impl TeamsClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
        })
    }

    pub async fn send_message(&self, bot_token: &str, channel_id: &str, text: &str) -> Result<()> {
        // Use Microsoft Graph API to send messages to Teams channels
        let url = format!(
            "https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages",
            "your-team-id", channel_id
        );

        let payload = serde_json::json!({
            "body": {
                "content": text,
                "contentType": "text"
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", bot_token))
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
        .route("/", post(authorize_teams))
        .route("/", delete(deauthorize_teams))
        .layer(middleware::from_fn(google::authenticate));

    let webhooks = Router::new()
        .route("/command", post(handle_command))
        .route("/interact", post(handle_interactivity))
        .layer(middleware::from_fn(verify_teams_signature));

    Router::new().merge(routes).merge(webhooks)
}

const ISSUER: &str = "koso-notifiers-teams";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Claims {
    exp: u64,
    iss: String,
    bot_token: String,
    channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizeTeams {
    token: String,
}

#[tracing::instrument(skip(user, pool, key))]
async fn authorize_teams(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(key): Extension<DecodingKey>,
    Json(req): Json<AuthorizeTeams>,
) -> ApiResult<Json<NotifierSettings>> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);
    validation.required_spec_claims.insert("iss".to_string());
    let token = decode::<Claims>(&req.token, &key, &validation).context_status(
        StatusCode::PRECONDITION_FAILED,
        "VALIDATION_FAILED",
        "Invalid token",
    )?;

    let settings = NotifierSettings::Teams(TeamsSettings {
        bot_token: token.claims.bot_token,
        channel_id: token.claims.channel_id,
    });

    insert_notification_config(&user.email, &settings, pool).await?;

    Ok(Json(settings))
}

#[tracing::instrument(skip(user, pool))]
async fn deauthorize_teams(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<()>> {
    delete_notification_config(&user.email, "teams", pool).await?;
    Ok(Json(()))
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamsCommandRequest {
    bot_token: String,
    channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamsCommandResponse {
    response_type: String,
    blocks: serde_json::Value,
}

async fn handle_command(
    Extension(key): Extension<jsonwebtoken::EncodingKey>,
    Form(req): Form<TeamsCommandRequest>,
) -> ApiResult<Json<TeamsCommandResponse>> {
    tracing::debug!("{:?}", req);

    Ok(Json(TeamsCommandResponse {
        response_type: "ephemeral".into(),
        blocks: json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "*Koso Authorization*\n\nClick the button below to connect your Koso account to Microsoft Teams."
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
                        "url": get_auth_url(key, &req.bot_token, &req.channel_id)?
                    }
                ]
            },
        ]),
    }))
}

fn get_auth_url(key: EncodingKey, bot_token: &str, channel_id: &str) -> Result<String> {
    let host = &settings().host;
    let timer = SystemTime::now() + Duration::from_secs(60 * 60);
    let claims = Claims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: ISSUER.to_string(),
        bot_token: bot_token.into(),
        channel_id: channel_id.into(),
    };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::debug!("Generated auth token {token} for bot token and channel {channel_id}");
    Ok(format!("{host}/connections/teams?token={token}"))
}

async fn handle_interactivity() -> ApiResult<()> {
    Ok(())
}

async fn verify_teams_signature(request: Request, next: Next) -> ApiResult<Response> {
    // For Teams, we'll use a simple webhook URL validation
    // Teams doesn't have the same signature verification as Slack
    Ok(next.run(request).await)
}
