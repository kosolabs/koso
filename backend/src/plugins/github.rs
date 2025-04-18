use crate::{
    api::{
        collab::Collab,
        google,
        model::Task,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::{PluginSettings, config::ConfigStorage, github::app::AppGithub},
};
use anyhow::{Result, anyhow};
use auth::Auth;
use axum::{Router, middleware};
use connect::ConnectHandler;
use octocrab::models::pulls::PullRequest;
use poller::Poller;
use regex::Regex;
use sqlx::PgPool;
use std::{cell::LazyCell, collections::HashSet, time::SystemTime};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use webhook::Webhook;
use yrs::{ReadTxn, TransactionMut};

mod app;
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
    cancel: CancellationToken,
}

impl Plugin {
    pub(crate) async fn new(
        settings: PluginSettings,
        collab: Collab,
        pool: &'static PgPool,
        cancel: CancellationToken,
    ) -> Result<Plugin> {
        PLUGIN_KIND.validate()?;
        PR_KIND.validate()?;
        let client: AppGithub = AppGithub::new().await?;
        let config_storage = ConfigStorage::new(pool)?;
        Ok(Plugin {
            collab,
            client,
            config_storage,
            pool,
            settings,
            cancel,
        })
    }

    /// Start a background task that polls github periodically.
    /// Return a handle to the task, useful for aborting the task on shutdown.
    pub(crate) fn start_polling(&self) -> JoinHandle<()> {
        if !self.settings.disable_polling {
            tokio::spawn(self.poller().poll(self.cancel.clone()))
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
                )?
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
            desc: None,
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
    description: String,
}

impl ExternalTask {
    fn new(pr: PullRequest) -> Result<ExternalTask> {
        let name = pr.title.unwrap_or_default();
        let url: String = pr.html_url.map(Into::into).unwrap_or_default();
        if url.is_empty() {
            return Err(anyhow!("Found PR with empty html_url: {}", pr.url));
        }
        let description = pr.body.unwrap_or_default();
        Ok(ExternalTask {
            url,
            name,
            description,
        })
    }
}

fn new_task(external_task: &ExternalTask, num: u64, kind: &Kind) -> Result<Task> {
    let id = uuid::Uuid::new_v4().to_string();
    tracing::trace!("Creating new task {} ({num}): {}", id, external_task.url);
    Ok(Task {
        id,
        num: num.to_string(),
        name: external_task.name.clone(),
        desc: None,
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
    if task.get_status(txn)?.is_none_or(|s| s != "In Progress") {
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
    if task.get_status(txn)?.is_none_or(|s| s != "Done") {
        task.set_status(txn, Some("Done"));
        task.set_status_time(txn, Some(now()?));
    }
    Ok(())
}

/// Adds the given task ID as a child of any tasks referenced in the github task.
fn add_referenced_task_links(
    txn: &mut TransactionMut,
    doc: &YDocProxy,
    task_id: &str,
    github_task: &ExternalTask,
) -> Result<()> {
    for link_task in doc.get_by_nums(txn, &find_referenced_task_nums(github_task))? {
        // Disallow linking to managed links this, additionally, prevents circular links
        // because the given task is itself always managed.
        if is_managed_task(txn, &link_task)? {
            continue;
        }

        link_task.push_child(txn, task_id)?;
    }
    Ok(())
}

thread_local! {
    static RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"(?i)(?-u:\b)koso[#_-](\d+)").unwrap());
}

/// Searches the external task's name and description for references to Koso Tasks
/// of the form: koso#<num>, koso_<num> or koso-<num>
fn find_referenced_task_nums(github_task: &ExternalTask) -> HashSet<String> {
    RE.with(|re| {
        re.captures_iter(&github_task.description)
            .chain(re.captures_iter(&github_task.name))
            .map(|g| g[1].to_owned())
            .collect()
    })
}

fn is_managed_task<T: ReadTxn>(txn: &T, link: &YTaskProxy) -> Result<bool> {
    Ok(link
        .get_kind(txn)?
        .map(|kind| kind != "Task" || kind != "Rollup")
        .unwrap_or(false))
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

    #[test_log::test]
    fn find_referenced_task_nums_matches_name() {
        assert_eq!(
            find_referenced_task_nums(&ExternalTask {
                url: "https://github.com/kosolabs/koso/pull/121".into(),
                name: "koso-15: Something else".into(),
                description: "Something something".into(),
            }),
            HashSet::from_iter(vec!["15".to_string()].into_iter())
        );
    }

    #[test_log::test]
    fn find_referenced_task_nums_matches_description() {
        assert_eq!(
            find_referenced_task_nums(&ExternalTask {
                url: "https://github.com/kosolabs/koso/pull/121".into(),
                name: "Something else".into(),
                description: "Something something koso#17, koso#19".into(),
            }),
            HashSet::from_iter(vec!["17".to_string(), "19".to_string()].into_iter())
        );
    }

    #[test_log::test]
    fn find_referenced_task_nums_matches_description_and_name() {
        assert_eq!(
            find_referenced_task_nums(&ExternalTask {
                url: "https://github.com/kosolabs/koso/pull/121".into(),
                name: "Something else KoSo_18".into(),
                description: "Somethingkoso#14 something KOSO-17, koso#19".into(),
            }),
            HashSet::from_iter(
                vec!["17".to_string(), "18".to_string(), "19".to_string()].into_iter()
            )
        );
    }
}
