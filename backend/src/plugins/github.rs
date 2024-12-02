use crate::{
    api::{
        collab::Collab,
        model::Task,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::config::{self, ConfigStorage},
};
use anyhow::{anyhow, Context, Result};
use axum::Router;
use kosolib::{AppGithub, AppGithubConfig};
use octocrab::models::pulls::PullRequest;
use poller::Poller;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::SystemTime;
use tokio::task::JoinHandle;
use webhook::{read_webhook_secret, Webhook, WebhookSecret};
use yrs::{ReadTxn, TransactionMut};

mod poller;
mod webhook;

pub(super) const KIND: &str = "github";
const NAME: &str = "Github Plugin";
/// Constant task ID of this plugin's container task.
const PARENT_ID: &str = "plugin_github";

#[derive(Clone)]
pub(crate) struct Plugin {
    collab: Collab,
    config_storage: ConfigStorage,
    secret: WebhookSecret,
    client: AppGithub,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct GithubSpecificConfig {
    pub(crate) project_id: String,
}

type GithubConfig = config::Config<GithubSpecificConfig>;

impl Plugin {
    pub(crate) async fn new(collab: Collab, pool: &'static PgPool) -> Result<Plugin> {
        let secret = read_webhook_secret()?;
        let client = AppGithub::new(&AppGithubConfig::default()).await?;
        let config_storage = ConfigStorage::new(pool)?;
        Ok(Plugin {
            collab,
            secret,
            client,
            config_storage,
        })
    }

    /// Start a background task that polls github periodically.
    /// Return a handle to the task, useful for aborting the task on shutdown.
    pub(crate) fn start_polling(&self) -> JoinHandle<()> {
        tokio::spawn(self.poller().poll())
    }

    /// Returns a router that binds webhook (push) and poll endpoints.
    pub(crate) fn router(&self) -> Router {
        Router::merge(
            Webhook::new(
                self.collab.clone(),
                self.config_storage.clone(),
                self.secret.clone(),
            )
            .router(),
            self.poller().router(),
        )
    }

    fn poller(&self) -> Poller {
        poller::Poller::new(
            self.collab.clone(),
            self.client.clone(),
            self.config_storage.clone(),
        )
    }
}

fn get_plugin_parent<T: ReadTxn>(txn: &T, doc: &YDocProxy) -> Result<YTaskProxy> {
    doc.get(txn, PARENT_ID)
        .with_context(|| "Missing plugin parent")
}

fn get_or_create_plugin_parent(txn: &mut TransactionMut, doc: &YDocProxy) -> Result<YTaskProxy> {
    if let Ok(parent) = doc.get(txn, PARENT_ID) {
        return Ok(parent);
    }

    tracing::debug!("Creating new parent task: {PARENT_ID}");
    let root = doc.get(txn, "root")?;
    let mut root_children = root.get_children(txn)?;
    root_children.push(PARENT_ID.to_string());

    let parent = doc.set(
        txn,
        &Task {
            id: PARENT_ID.to_string(),
            num: doc.next_num(txn)?.to_string(),
            name: NAME.to_string(),
            children: Vec::with_capacity(0),
            assignee: None,
            reporter: None,
            status: None,
            status_time: None,
            url: None,
            kind: None,
        },
    );
    root.set_children(txn, &root_children);

    Ok(parent)
}

#[derive(Debug)]
struct ExternalTask {
    url: String,
    name: String,
}

impl ExternalTask {
    fn new(pr: PullRequest) -> Result<ExternalTask> {
        let name = pr.title.unwrap_or_default();
        let url: String = pr.html_url.map(Into::into).unwrap_or_default();
        if url.is_empty() {
            return Err(anyhow!("Found PR with empty html_url: {}", pr.url));
        }
        Ok(ExternalTask { url, name })
    }
}

fn new_task(external_task: &ExternalTask, num: u64) -> Result<Task> {
    let id = uuid::Uuid::new_v4().to_string();
    tracing::trace!("Creating new task {} ({num}): {}", id, external_task.url);
    Ok(Task {
        id,
        num: num.to_string(),
        name: external_task.name.clone(),
        children: Vec::with_capacity(0),
        assignee: None,
        reporter: None,
        status: Some("In Progress".to_string()),
        status_time: Some(now()?),
        url: Some(external_task.url.clone()),
        kind: Some(KIND.to_string()),
    })
}

fn update_task(
    txn: &mut TransactionMut,
    task: &YTaskProxy,
    external_task: &ExternalTask,
) -> Result<()> {
    tracing::trace!("Updating task {}: {}", task.get_id(txn)?, external_task.url);
    task.set_name(txn, &external_task.name);
    if task.get_status(txn)?.map_or(true, |s| s != "In Progress") {
        task.set_status(txn, Some("In Progress"));
        task.set_status_time(txn, Some(now()?));
    }
    Ok(())
}

fn resolve_task(txn: &mut TransactionMut, task: &YTaskProxy) -> Result<()> {
    tracing::trace!(
        "Resolving task {}: {}",
        task.get_id(txn)?,
        task.get_url(txn)?.unwrap_or_default()
    );
    if task.get_status(txn)?.map_or(true, |s| s != "In Done") {
        task.set_status(txn, Some("Done"));
        task.set_status_time(txn, Some(now()?));
    }
    Ok(())
}

fn now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
