use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::api::{bad_request_error, unauthorized_error, ApiResult};
use anyhow::anyhow;
use anyhow::Result;
use axum::http::HeaderMap;
use axum::Extension;
use axum::{
    body::{Body, Bytes},
    http::request::Parts,
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use kosolib::{AppGithub, AppGithubConfig, InstallationRef};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use sha2::Sha256;

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

/// Maximum size of request body in bytes.
const BODY_LIMIT: usize = 10 * 1024 * 1024;

#[derive(Clone)]
struct WebhookSecret {
    secret: Arc<Vec<u8>>,
}

struct WebhookHeaders<'a> {
    delivery_id: &'a str,
    event: &'a str,
    signature: &'a [u8],
}

pub async fn shad() -> Result<String> {
    let client = AppGithub::new(&AppGithubConfig::default()).await?;
    let client = client
        .installation_github(InstallationRef::Org { owner: "kosolabs" })
        .await?;
    let prs = client.fetch_pull_requests("kosolabs", "secret").await?;

    Ok(serde_json::to_string(&prs)?)
}

pub(crate) fn router() -> Result<Router> {
    let secret = read_webhook_secret()?;
    Ok(Router::new()
        .route("/app/webhook", post(github_webhook))
        .layer((Extension(secret),)))
}

#[tracing::instrument(skip(parts, body, secret), fields(gh_delivery_id, gh_event))]
async fn github_webhook(
    Extension(secret): Extension<WebhookSecret>,
    parts: Parts,
    body: Body,
) -> ApiResult<String> {
    let headers = parse_headers(&parts.headers)?;
    tracing::Span::current().record("gh_event", headers.event);
    tracing::Span::current().record("gh_delivery_id", headers.delivery_id);

    let body: Bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .map_err(|_| bad_request_error("INVALID_BODY", "Invalid body"))?;
    validate_signature(headers.signature, &body, secret)?;

    // TODO: Do great things with the event!
    let event = WebhookEvent::try_from_header_and_body(headers.event, &body).unwrap();
    match event.kind {
        WebhookEventType::Ping => tracing::info!("Received Ping event: {event:?}"),
        WebhookEventType::PullRequest => {
            tracing::info!("Received PR event: {event:?}");
        }
        _ => tracing::warn!("Discarding unhandled event: {event:?}"),
    };

    Ok("OK".to_string())
}

fn parse_headers(headers: &HeaderMap) -> ApiResult<WebhookHeaders> {
    let Some(header) = headers.get("X-GitHub-Event") else {
        return Err(bad_request_error(
            "MISSING_HEADER",
            "Missing X-GitHub-Event header",
        ));
    };
    let event = header.to_str()?;

    let Some(header) = headers.get("X-Hub-Signature-256") else {
        return Err(bad_request_error(
            "MISSING_HEADER",
            "Missing X-Hub-Signature-256 header",
        ));
    };
    let signature: &[u8] = header.as_bytes();

    let Some(header) = headers.get("X-GitHub-Delivery") else {
        return Err(bad_request_error(
            "MISSING_HEADER",
            "Missing X-GitHub-Delivery header",
        ));
    };
    let delivery_id = header.to_str()?;

    Ok(WebhookHeaders {
        delivery_id,
        event,
        signature,
    })
}

type HmacSha256 = Hmac<Sha256>;

/// Validate the authenticity of the event.
/// See https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#validating-webhook-deliveries
fn validate_signature(
    signature_header: &[u8],
    body: &[u8],
    secret: WebhookSecret,
) -> ApiResult<()> {
    let Some(signature) = signature_header
        .get(b"sha256=".len()..)
        .and_then(|v| hex::decode(v).ok())
    else {
        return Err(unauthorized_error("Invalid signature."));
    };

    let mut mac = HmacSha256::new_from_slice(&secret.secret)?;
    mac.update(body);
    match mac.verify_slice(&signature) {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::warn!("Received webhook event with invalid signature: {err}");
            Err(unauthorized_error(
                format!("Invalid signature: {signature_header:?}").as_str(),
            ))
        }
    }
}

fn read_webhook_secret() -> Result<WebhookSecret> {
    let dir = std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string());
    let path = Path::new(&dir)
        .join("github/webhook_secret")
        .into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Invalid github secret path in {dir}: {e:?}"))?;
    tracing::info!("Using github webhook secret at {path}");
    let secret =
        fs::read(&path).map_err(|e| anyhow!("Failed to read github secret from {path}: {e}"))?;
    Ok(WebhookSecret {
        secret: Arc::new(secret),
    })
}
