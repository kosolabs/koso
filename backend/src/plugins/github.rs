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
use kosolib::{AppGithub, AppGithubConfig};
use octocrab::models::pulls::PullRequest;
use poller::Poller;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{fmt::Debug, time::SystemTime};
use tokio::task::JoinHandle;
use webhook::Webhook;
use yrs::TransactionMut;

mod auth;
mod connect;
mod poller;
mod webhook;

const PLUGIN_KIND: &Kind = &Kind::new("github", "GitHub");
const PR_KIND: &Kind = &Kind::new_nested(PLUGIN_KIND, "github_pr", "GitHub PR");

#[derive(Clone)]
pub(crate) struct Plugin {
    collab: Collab,
    config_storage: ConfigStorage,
    client: AppGithub,
    pool: &'static PgPool,
    settings: PluginSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct GithubSpecificConfig {}

type GithubConfig = config::Config<GithubSpecificConfig>;

impl Plugin {
    pub(crate) async fn new(
        settings: PluginSettings,
        collab: Collab,
        pool: &'static PgPool,
    ) -> Result<Plugin> {
        PLUGIN_KIND.validate()?;
        PR_KIND.validate()?;
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
    kind: &Kind,
) -> Result<YTaskProxy> {
    let parent_kind = kind.parent_kind.unwrap_or(kind);
    let plugin_parent = match doc.get(txn, parent_kind.id) {
        Ok(parent) => parent,
        Err(_) => create_container(txn, &doc.get(txn, "root")?, doc, parent_kind)?,
    };
    if kind.parent_kind.is_none() {
        return Ok(plugin_parent);
    }
    match doc.get(txn, kind.id) {
        Ok(kind_parent) => Ok(kind_parent),
        Err(_) => create_container(txn, &plugin_parent, doc, kind),
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

#[derive(Clone, Debug)]
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

struct Kind<'a> {
    id: &'a str,
    name: &'a str,
    parent_kind: Option<&'a Kind<'a>>,
}

impl Kind<'_> {
    /// Creates a new kind.
    /// `id` must NOT contain underscores as they're used
    /// as separators in the kind hierarchy. For example, 'github_pr'
    /// is a child kind of 'github'.
    const fn new<'a>(id: &'a str, name: &'a str) -> Kind<'a> {
        Kind {
            id,
            name,
            parent_kind: None,
        }
    }

    /// Creates a new nested kind.
    /// For consistent namespacing, `id` must have `{parent_kind.id}_` as a prefix.
    /// The remaining suffix must NOT contain underscores.
    /// For example, a kind of 'github_pr' should have a parent_kind of 'github'.
    const fn new_nested<'a>(parent_kind: &'a Kind, id: &'a str, name: &'a str) -> Kind<'a> {
        Kind {
            id,
            name,
            parent_kind: Some(parent_kind),
        }
    }

    fn validate(&self) -> Result<()> {
        let sub_id = match self.parent_kind {
            Some(parent_kind) => {
                parent_kind.validate()?;
                let Some(sub_id) = self.id.strip_prefix(&format!("{}_", parent_kind.id)) else {
                    return Err(anyhow!(
                        "Kind id ({}) does not start with parent kind id ({})",
                        self.id,
                        parent_kind.id
                    ));
                };
                sub_id
            }
            None => self.id,
        };
        if sub_id.contains("_") {
            return Err(anyhow!(
                "Kind id ({}) must not contain underscores which separate parent and children kinds",
                sub_id,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn validate_pr_kind() {
        let res = PR_KIND.validate();
        assert!(res.is_ok(), "PR_KIND is invalid {res:?}");
    }

    #[test_log::test(tokio::test)]
    async fn validate_plugin_kind() {
        let res = PLUGIN_KIND.validate();
        assert!(res.is_ok(), "PLUGIN_KIND is invalid {res:?}");
    }
}
