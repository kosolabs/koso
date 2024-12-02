use crate::{
    api::{
        bad_request_error,
        collab::{projects_state::DocBox, txn_origin::YOrigin, Collab},
        unauthorized_error,
        yproxy::{YDocProxy, YTaskProxy},
        ApiResult,
    },
    plugins::{
        config::ConfigStorage,
        github::{
            get_or_create_plugin_parent, get_plugin_parent, new_task, resolve_task, update_task,
            ExternalTask, GithubConfig, KIND,
        },
    },
};
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
use octocrab::models::webhook_events::payload::PullRequestWebhookEventAction;
use octocrab::models::webhook_events::WebhookEvent;
use octocrab::models::webhook_events::WebhookEventPayload;
use sha2::Sha256;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tower_http::request_id::RequestId;
use tracing::Instrument as _;
use yrs::{Origin, ReadTxn, TransactionMut};

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

/// Maximum size of request body in bytes.
const BODY_LIMIT: usize = 10 * 1024 * 1024;

/// Contains the secret used to validate webhook deliveries.
/// See https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#creating-a-secret-token
#[derive(Clone)]
pub(super) struct WebhookSecret {
    secret: Arc<Vec<u8>>,
}

/// Encapsulates several Github webhook headers.
/// See https://docs.github.com/en/webhooks/webhook-events-and-payloads#delivery-headers
struct WebhookHeaders<'a> {
    delivery_id: &'a str,
    event: &'a str,
    signature: &'a [u8],
    installation_id: &'a str,
}

#[derive(Debug)]
struct KosoGithubEvent {
    request_id: String,
    installation_id: u64,
    pr_url: String,
    pr_title: String,
    action: KosoGithubEventAction,
}

#[derive(Debug)]
enum KosoGithubEventAction {
    Opened,
    Closed,
    Edited,
}

#[derive(Clone)]
pub(super) struct Webhook {
    collab: Collab,
    config_storage: ConfigStorage,
    secret: WebhookSecret,
}

impl Webhook {
    pub(super) fn new(
        collab: Collab,
        config_storage: ConfigStorage,
        secret: WebhookSecret,
    ) -> Webhook {
        Webhook {
            collab,
            config_storage,
            secret,
        }
    }

    pub(super) fn router(self) -> Router {
        Router::new().route(
            "/app/webhook",
            post(github_webhook).layer((Extension(self),)),
        )
    }
}

#[tracing::instrument(
    skip(parts, body, webhook, request_id),
    fields(gh_delivery_id, gh_event, gh_installation_id)
)]
async fn github_webhook(
    Extension(webhook): Extension<Webhook>,
    Extension(request_id): Extension<RequestId>,
    parts: Parts,
    body: Body,
) -> ApiResult<String> {
    let headers = parse_headers(&parts.headers)?;
    let body: Bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .map_err(|_| bad_request_error("INVALID_BODY", "Invalid body"))?;
    validate_signature(headers.signature, &body, &webhook.secret)?;

    tracing::Span::current().record("gh_delivery_id", headers.delivery_id);
    tracing::Span::current().record("gh_event", headers.event);
    tracing::Span::current().record("gh_installation_id", headers.installation_id);

    webhook
        .process_event(
            WebhookEvent::try_from_header_and_body(headers.event, &body)?,
            request_id
                .header_value()
                .to_str()
                .unwrap_or("INVALID")
                .to_string(),
        )
        .await?;

    Ok("OK".to_string())
}

// See https://docs.github.com/en/webhooks/webhook-events-and-payloads#delivery-headers.
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

    let Some(header) = headers.get("X-GitHub-Hook-Installation-Target-ID") else {
        return Err(bad_request_error(
            "MISSING_HEADER",
            "Missing X-GitHub-Hook-Installation-Target-ID header",
        ));
    };
    let installation_id = header.to_str()?;

    Ok(WebhookHeaders {
        delivery_id,
        event,
        signature,
        installation_id,
    })
}

type HmacSha256 = Hmac<Sha256>;

/// Validate the authenticity of the event.
/// See https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#validating-webhook-deliveries
fn validate_signature(
    signature_header: &[u8],
    body: &[u8],
    secret: &WebhookSecret,
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

/// Read the webhook secret from $secrets_dir/github/webhook_secret.
/// The default is `../.secrets/github/webhook_secret`, unless `SECRETS_DIR` is set.
pub(super) fn read_webhook_secret() -> Result<WebhookSecret> {
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

impl Webhook {
    #[tracing::instrument(skip(self, event, request_id), fields(target))]
    async fn process_event(self, event: WebhookEvent, request_id: String) -> ApiResult<()> {
        match event.specific {
            WebhookEventPayload::PullRequest(pr_event) => {
                let installation_id: u64 = match event
                    .installation
                    .ok_or_else(|| anyhow!("Missing installation field."))?
                {
                    octocrab::models::webhook_events::EventInstallation::Full(installation) => {
                        *installation.id
                    }
                    octocrab::models::webhook_events::EventInstallation::Minimal(
                        installation_id,
                    ) => *installation_id.id,
                };
                let pr = pr_event.pull_request;
                let Some(pr_title) = pr.title else {
                    return Err(bad_request_error(
                        "BAD_EVENT",
                        "Event missing pr.title field",
                    ));
                };
                let Some(pr_url) = pr.html_url.map(|u| u.to_string()) else {
                    return Err(bad_request_error(
                        "BAD_EVENT",
                        "Event missing pr.html_url field",
                    ));
                };
                let action = match pr_event.action {
                    PullRequestWebhookEventAction::Opened
                    | PullRequestWebhookEventAction::Reopened => KosoGithubEventAction::Opened,
                    PullRequestWebhookEventAction::Closed => KosoGithubEventAction::Closed,
                    PullRequestWebhookEventAction::Edited => KosoGithubEventAction::Edited,
                    _ => {
                        tracing::trace!(
                            "Discarding unhandled PR action type: {:?}",
                            pr_event.action
                        );
                        return Ok(());
                    }
                };
                tracing::Span::current().record("target", &pr_url);

                let event = KosoGithubEvent {
                    installation_id,
                    pr_url,
                    pr_title,
                    action,
                    request_id,
                };

                tokio::spawn(
                    async move {
                        if let Err(e) = self.process_koso_event(event).await {
                            tracing::warn!("Failed to process koso event: {e}")
                        }
                    }
                    .in_current_span(),
                );
            }
            _ => tracing::trace!("Discarding unhandled event."),
        };

        Ok(())
    }

    async fn process_koso_event(&self, event: KosoGithubEvent) -> Result<()> {
        tracing::debug!("Processing Koso event: {event:?}");
        let config: GithubConfig = self
            .config_storage
            .get(KIND, &event.installation_id.to_string())
            .await?;

        let client = self
            .collab
            .register_local_client(&config.config.project_id)
            .await?;
        let doc_box = client.project.doc_box.lock().await;
        let doc_box = DocBox::doc_or_error(doc_box.as_ref())?;
        let doc = &doc_box.ydoc;

        let mut txn = doc.transact_mut_with(origin(&event));
        match (get_doc_task(&txn, doc, &event.pr_url)?, &event.action) {
            (Some(task), KosoGithubEventAction::Opened | KosoGithubEventAction::Edited) => {
                update_task(&mut txn, &task, &to_external_task(event))?;
            }
            (None, KosoGithubEventAction::Opened | KosoGithubEventAction::Edited) => {
                create_task(&mut txn, doc, &to_external_task(event))?;
            }
            (Some(task), KosoGithubEventAction::Closed) => {
                resolve_task(&mut txn, &task)?;
            }
            (None, KosoGithubEventAction::Closed) => {
                tracing::trace!("Discarding close event without associated task");
            }
        }
        Ok(())
    }
}

fn get_doc_task<T: ReadTxn>(txn: &T, doc: &YDocProxy, url: &str) -> Result<Option<YTaskProxy>> {
    let Ok(parent) = get_plugin_parent(txn, doc) else {
        return Ok(None);
    };

    for child in parent.get_children(txn)? {
        let child = doc.get(txn, &child)?;
        if child.get_url(txn)?.unwrap_or_default() == url
            && child.get_kind(txn)?.unwrap_or_default() == KIND
        {
            return Ok(Some(child));
        }
    }
    Ok(None)
}

fn to_external_task(event: KosoGithubEvent) -> ExternalTask {
    ExternalTask {
        url: event.pr_url,
        name: event.pr_title,
    }
}

fn create_task(
    txn: &mut TransactionMut,
    doc: &YDocProxy,
    external_task: &ExternalTask,
) -> Result<()> {
    let parent = get_or_create_plugin_parent(txn, doc)?;
    let mut children: Vec<String> = parent.get_children(txn)?;

    let task = new_task(external_task, doc.next_num(txn)?)?;
    doc.set(txn, &task);

    // Add the new task as a child of the plugin parent.
    children.push(task.id.clone());
    parent.set_children(txn, &children);

    Ok(())
}

fn origin(event: &KosoGithubEvent) -> Origin {
    YOrigin {
        who: "github_webhook".to_string(),
        id: format!(
            "install_{}_request_{}",
            event.installation_id, event.request_id
        ),
    }
    .as_origin()
}
