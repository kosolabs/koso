use crate::api::{bad_request_error, unauthorized_error, ApiResult};
use axum::{body::Bytes, routing::post, RequestExt as _, Router};
use hmac::{Hmac, Mac};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use sha2::Sha256;

pub(crate) fn router() -> Router {
    Router::new().route("/github/webhook", post(github_webhook))
}

const BODY_LIMIT: usize = 10 * 1024 * 1024;

#[tracing::instrument()]
async fn github_webhook(request: axum::extract::Request) -> ApiResult<String> {
    let (parts, body) = request.with_limited_body().into_parts();
    let body: Bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .map_err(|_| bad_request_error("INVALID_BODY", "Invalid body"))?;

    // request_from_github is the HTTP request your webhook handler received
    let Some(header) = parts.headers.get("X-GitHub-Event") else {
        return Err(bad_request_error(
            "MISSING_EVENT_HEADER",
            "Missing X-GitHub-Event header",
        ));
    };
    let event_header = header.to_str()?;

    let Some(header) = parts.headers.get("X-Hub-Signature-256") else {
        return Err(bad_request_error(
            "MISSING_SIG_HEADER",
            "Missing X-Hub-Signature-256 header",
        ));
    };
    let signature_header = header.as_bytes();

    let secret = "TODO".as_bytes();
    if !verify_gh_signature(signature_header, &body, secret)? {
        return Err(unauthorized_error(
            format!("Invalid signature: {signature_header:?}").as_str(),
        ));
    }

    let event = WebhookEvent::try_from_header_and_body(event_header, &body).unwrap();
    match event.kind {
        WebhookEventType::Ping => return Ok("PONG".to_string()),
        WebhookEventType::PullRequest => {
            tracing::info!("Received PR event: {event:?}");
        }
        // ...
        _ => tracing::warn!("Discarding unhandled event: {event:?}"),
    };

    Ok("OK".to_string())
}

type HmacSha256 = Hmac<Sha256>;

fn verify_gh_signature(signature: &[u8], body: &[u8], secret: &[u8]) -> ApiResult<bool> {
    let Some(signature) = signature
        .get(b"sha256=".len()..)
        .and_then(|v| hex::decode(v).ok())
    else {
        return Err(unauthorized_error("Invalid signature."));
    };

    let mut mac = HmacSha256::new_from_slice(secret)?;
    mac.update(body);
    Ok(mac.verify_slice(&signature).is_ok())
}
