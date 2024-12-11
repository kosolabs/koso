use crate::{
    api::{
        collab::Collab,
        google,
        model::Task,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::{
        config::{self, ConfigStorage},
        PluginSettings,
    },
};
use anyhow::{anyhow, Result};
use auth::Auth;
use axum::{middleware, Router};
use connect::ConnectHandler;
use core::fmt;
use kosolib::{AppGithub, AppGithubConfig};
use octocrab::models::pulls::PullRequest;
use poller::Poller;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{fmt::Debug, fs, path::Path, time::SystemTime};
use tokio::task::JoinHandle;
use webhook::Webhook;
use yrs::TransactionMut;

mod auth;
mod connect;
mod poller;
mod webhook;

const PLUGIN_KIND: &Kind = &Kind::new("github", "Github Plugin");
const PR_KIND: &Kind = &Kind::new("github_pr", "Pull Requests");

#[derive(Clone)]
pub(crate) struct Plugin {
    collab: Collab,
    config_storage: ConfigStorage,
    client: AppGithub,
    pool: &'static PgPool,
    settings: PluginSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct GithubSpecificConfig {
    pub(crate) project_id: String,
}

type GithubConfig = config::Config<GithubSpecificConfig>;

impl Plugin {
    pub(crate) async fn new(
        settings: PluginSettings,
        collab: Collab,
        pool: &'static PgPool,
    ) -> Result<Plugin> {
        let client: AppGithub = AppGithub::new(&AppGithubConfig::default()).await?;
        let config_storage = ConfigStorage::new(pool)?;
        Ok(Plugin {
            collab,
            client,
            config_storage,
            pool,
            settings,
        })
    }

    /// Start a background task that polls github periodically.
    /// Return a handle to the task, useful for aborting the task on shutdown.
    pub(crate) fn start_polling(&self) -> JoinHandle<()> {
        if !self.settings.disable_polling {
            tokio::spawn(self.poller().poll())
        } else {
            tokio::spawn(async { tracing::debug!("Plugin polling disabled") })
        }
    }

    /// Returns a router that binds webhook (push) and poll endpoints.
    pub(crate) fn router(&self) -> Result<Router> {
        let auth = Auth::new()?;
        Ok(Router::new()
            .merge(auth.clone().router())
            .merge(
                ConnectHandler::new(
                    auth.clone(),
                    self.pool,
                    self.config_storage.clone(),
                    self.poller().clone(),
                )
                .router(),
            )
            .layer((middleware::from_fn(google::authenticate),))
            // Webhook and poller are unauthenticated, so add it AFTER adding the authentication layers.
            .merge(Webhook::new(self.collab.clone(), self.config_storage.clone())?.router())
            .merge(self.poller().router()))
    }

    fn poller(&self) -> Poller {
        poller::Poller::new(
            self.collab.clone(),
            self.client.clone(),
            self.config_storage.clone(),
        )
    }
}

fn get_or_create_kind_parent(
    txn: &mut TransactionMut,
    doc: &YDocProxy,
    plugin_kind: &Kind,
    sub_kind: Option<&Kind>,
) -> Result<YTaskProxy> {
    let plugin_parent = match doc.get(txn, plugin_kind.id) {
        Ok(parent) => parent,
        Err(_) => create_container(txn, &doc.get(txn, "root")?, doc, plugin_kind)?,
    };
    let Some(sub_kind) = sub_kind else {
        return Ok(plugin_parent);
    };
    match doc.get(txn, sub_kind.id) {
        Ok(kind_parent) => Ok(kind_parent),
        Err(_) => create_container(txn, &plugin_parent, doc, sub_kind),
    }
}

fn create_container(
    txn: &mut TransactionMut,
    container_parent: &YTaskProxy,
    doc: &YDocProxy,
    kind: &Kind,
) -> Result<YTaskProxy> {
    tracing::debug!("Creating new kind container: {}", kind.id);
    let mut plugin_children = container_parent.get_children(txn)?;
    plugin_children.push(kind.id.to_string());

    let kind_parent = doc.set(
        txn,
        &Task {
            id: kind.id.to_string(),
            num: doc.next_num(txn)?.to_string(),
            name: kind.name.to_string(),
            children: Vec::with_capacity(0),
            assignee: None,
            reporter: None,
            status: None,
            status_time: None,
            url: None,
            kind: Some(kind.id.to_string()),
        },
    );
    container_parent.set_children(txn, &plugin_children);
    Ok(kind_parent)
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

fn new_task(external_task: &ExternalTask, num: u64, kind: &Kind) -> Result<Task> {
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
        kind: Some(kind.id.to_string()),
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

#[derive(Clone)]
struct Secret<T> {
    data: T,
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

/// Read the secret from $secrets_dir/$sub_path.
/// The default is `../.secrets/$sub_path`, unless `SECRETS_DIR` is set.
fn read_secret<T: std::convert::From<String>>(sub_path: &str) -> Result<Secret<T>> {
    let dir = std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string());
    let path = Path::new(&dir)
        .join(sub_path)
        .into_os_string()
        .into_string()
        .map_err(|e| anyhow!("Invalid secret path in {dir}: {e:?}"))?;
    tracing::info!("Using {sub_path} secret at {path}");
    let secret: String = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read secret from {path}: {e}"))?
        .trim()
        .to_owned();
    Ok(Secret {
        data: secret.into(),
    })
}

struct Kind<'a> {
    name: &'a str,
    id: &'a str,
}

impl Kind<'_> {
    const fn new<'a>(id: &'a str, name: &'a str) -> Kind<'a> {
        Kind { name, id }
    }
}
