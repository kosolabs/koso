use crate::{
    api::{
        ApiResult, bad_request_error,
        collab::{
            Collab,
            projects_state::DocBox,
            txn_origin::{Actor, YOrigin},
        },
        unauthorized_error,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::{
        config::{Config, ConfigStorage},
        github::{
            self, ExternalTask, Kind, PLUGIN_KIND, PR_KIND, add_referenced_task_links,
            get_or_create_kind_parent, lookup_by_github_user_id, new_task,
        },
    },
    secrets::{Secret, read_secret},
};
use anyhow::{Result, anyhow};
use axum::{
    Extension, Router,
    body::{Body, Bytes},
    http::{HeaderMap, request::Parts},
    routing::post,
};
use hmac::{Hmac, Mac};
use octocrab::models::webhook_events::{
    WebhookEvent, WebhookEventPayload, payload::PullRequestWebhookEventAction,
};
use sha2::Sha256;
use sqlx::PgPool;
use tower_http::request_id::RequestId;
use tracing::Instrument as _;
use yrs::{Origin, ReadTxn, TransactionMut};

/// Maximum size of request body in bytes.
const BODY_LIMIT: usize = 10 * 1024 * 1024;

/// Contains the secret used to validate webhook deliveries.
/// See https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#creating-a-secret-token
type WebhookSecret = Secret<Vec<u8>>;

/// Encapsulates several Github webhook headers.
/// See https://docs.github.com/en/webhooks/webhook-events-and-payloads#delivery-headers
struct WebhookHeaders<'a> {
    delivery_id: &'a str,
    event: &'a str,
    signature: &'a [u8],
    installation_id: &'a str,
}

#[derive(Clone, Debug)]
struct KosoGithubEvent {
    request_id: String,
    installation_id: u64,
    task: ExternalTask,
}

#[derive(Clone)]
pub(super) struct Webhook {
    collab: Collab,
    config_storage: ConfigStorage,
    secret: WebhookSecret,
    pool: &'static PgPool,
}

impl Webhook {
    pub(super) fn new(
        collab: Collab,
        config_storage: ConfigStorage,
        pool: &'static PgPool,
    ) -> Result<Webhook> {
        Ok(Webhook {
            collab,
            config_storage,
            secret: read_secret("github/webhook_secret")?,
            pool,
        })
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
        .process_webhook_event(
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

    let mut mac = HmacSha256::new_from_slice(&secret.data)?;
    mac.update(body);
    match mac.verify_slice(&signature) {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::warn!("Received webhook event with invalid signature: {err:?}");
            Err(unauthorized_error(
                format!("Invalid signature: {}", hex::encode(signature)).as_str(),
            ))
        }
    }
}

impl Webhook {
    #[tracing::instrument(skip(self, event, request_id), fields(target))]
    async fn process_webhook_event(self, event: WebhookEvent, request_id: String) -> ApiResult<()> {
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
                let task = ExternalTask::new(pr_event.pull_request)?;
                match pr_event.action {
                    PullRequestWebhookEventAction::Opened
                    | PullRequestWebhookEventAction::Reopened
                    | PullRequestWebhookEventAction::Closed
                    | PullRequestWebhookEventAction::Edited => {}
                    _ => {
                        tracing::trace!(
                            "Discarding unhandled PR action type: {:?}",
                            pr_event.action
                        );
                        return Ok(());
                    }
                }
                tracing::Span::current().record("target", &task.url);

                let event = KosoGithubEvent {
                    request_id,
                    installation_id,
                    task,
                };

                tokio::spawn(
                    async move {
                        if let Err(e) = self.process_koso_event(event).await {
                            tracing::warn!("Failed to process koso event: {e:?}")
                        }
                    }
                    .in_current_span(),
                );
            }
            _ => tracing::trace!("Discarding unhandled event."),
        };

        Ok(())
    }

    async fn process_koso_event(&self, mut event: KosoGithubEvent) -> Result<()> {
        tracing::debug!("Processing Koso event: {event:?}");

        // Populate the email of the author if we're able to.
        if let Some(user_id) = &event.task.user_id {
            if let Some(email) = lookup_by_github_user_id(user_id, self.pool).await? {
                event.task.koso_user_email = Some(email);
            }
        }

        let configs = self
            .config_storage
            .list_for_external_id(PLUGIN_KIND.id, &event.installation_id.to_string())
            .await?;
        if configs.is_empty() {
            tracing::debug!(
                "No config registered for installation '{}'. Discarding event.",
                event.installation_id
            );
            return Ok(());
        };

        futures::future::join_all(
            configs
                .into_iter()
                .map(|config| self.merge_task(event.clone(), config)),
        )
        .await;

        Ok(())
    }

    #[tracing::instrument(
        skip(self, event, config),
        fields(project_id=config.project_id)
    )]
    async fn merge_task(&self, event: KosoGithubEvent, config: Config) {
        if let Err(e) = self.merge_task_internal(event, config).await {
            tracing::warn!("Failed to process event for config: {e:?}");
        }
    }

    async fn merge_task_internal(&self, event: KosoGithubEvent, config: Config) -> Result<()> {
        let client = self
            .collab
            .register_local_client(&config.project_id)
            .await?;

        // Avoid any expensive, async work while holding the doc_box lock.
        {
            let doc_box = client.project.doc_box.lock().await;
            self.apply_task_changes(&event, &DocBox::doc_or_error(doc_box.as_ref())?.ydoc)
        }
    }

    // Note: This function should remain synchronous to avoid blocking the doc_box lock.
    fn apply_task_changes(&self, event: &KosoGithubEvent, doc: &YDocProxy) -> Result<()> {
        let mut txn = doc.transact_mut_with(origin(event)?);
        match get_doc_task(&txn, doc, &event.task.url, PR_KIND)? {
            None => {
                create_task(&mut txn, doc, &event.task)?;
            }
            Some(task) => update_task(&mut txn, doc, &task, &event.task)?,
        }
        Ok(())
    }
}

fn get_doc_task<T: ReadTxn>(
    txn: &T,
    doc: &YDocProxy,
    url: &str,
    kind: &Kind,
) -> Result<Option<YTaskProxy>> {
    let Ok(parent) = doc.get(txn, kind.id) else {
        return Ok(None);
    };

    for child in parent.get_children(txn)? {
        let child = doc.get(txn, &child)?;
        if child.get_url(txn)?.is_some_and(|u| u == url)
            && child.get_kind(txn)?.is_some_and(|k| k == kind.id)
        {
            return Ok(Some(child));
        }
    }
    Ok(None)
}

fn create_task(
    txn: &mut TransactionMut,
    doc: &YDocProxy,
    external_task: &ExternalTask,
) -> Result<()> {
    let parent = get_or_create_kind_parent(txn, doc, PR_KIND)?;
    let mut children: Vec<String> = parent.get_children(txn)?;

    let task = new_task(external_task, doc.next_num(txn)?, PR_KIND)?;
    doc.set(txn, &task);

    // Add the new task as a child of the plugin parent.
    children.push(task.id.clone());
    parent.set_children(txn, &children);

    add_referenced_task_links(txn, doc, &task.id, external_task)?;

    Ok(())
}

fn update_task(
    txn: &mut TransactionMut,
    doc: &YDocProxy,
    task: &YTaskProxy,
    external_task: &ExternalTask,
) -> Result<()> {
    add_referenced_task_links(txn, doc, &task.get_id(txn)?, external_task)?;
    github::update_task(txn, task, external_task)?;
    Ok(())
}

fn origin(event: &KosoGithubEvent) -> Result<Origin> {
    YOrigin {
        who: "github_webhook".to_string(),
        id: format!(
            "install_{}_request_{}",
            event.installation_id, event.request_id
        ),
        actor: Actor::GitHub,
    }
    .as_origin()
}
