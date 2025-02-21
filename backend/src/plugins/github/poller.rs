use crate::{
    api::{
        ApiResult,
        collab::{
            Collab,
            projects_state::DocBox,
            txn_origin::{Actor, YOrigin},
        },
        yproxy::{YDocProxy, YTaskProxy},
    },
    flags::is_dev,
    plugins::{
        config::{Config, ConfigStorage},
        github::{
            ExternalTask, Kind, PLUGIN_KIND, PR_KIND,
            app::{AppGithub, InstallationRef},
            get_or_create_kind_parent, new_task, resolve_task, update_task,
        },
    },
};
use anyhow::Result;
use axum::{Extension, Router, routing::post};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use yrs::{Origin, ReadTxn};

const INIT_POLL_DELAY: Duration = Duration::from_secs(2 * 60);
const POLL_DELAY: Duration = Duration::from_secs(16 * 60);

#[derive(Clone)]
pub(super) struct Poller {
    collab: Collab,
    client: AppGithub,
    config_storage: ConfigStorage,
}

impl Poller {
    pub(super) fn new(collab: Collab, client: AppGithub, config_storage: ConfigStorage) -> Poller {
        Poller {
            collab,
            client,
            config_storage,
        }
    }

    pub(super) fn router(self) -> Router {
        // TODO: In the future, we could expose this in production with admin authorization
        if is_dev() {
            return Router::new()
                .route("/poll", post(Poller::poll_handler))
                .layer((Extension(self),));
        }
        Router::new()
    }

    #[tracing::instrument(skip(poller))]
    async fn poll_handler(Extension(poller): Extension<Poller>) -> ApiResult<String> {
        poller.poll_all_installations().await?;
        Ok("OK".to_string())
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn poll(self) {
        // Wait awhile before starting polling to avoid
        // competing with client reconnections after a server restart.
        tokio::time::sleep(INIT_POLL_DELAY).await;
        loop {
            if let Err(e) = self.poll_all_installations().await {
                tracing::warn!("Failed poll: {e:?}");
            }
            tokio::time::sleep(POLL_DELAY).await;
        }
    }

    async fn poll_all_installations(&self) -> Result<()> {
        // TODO: Multiple configs can refer to the same installation. If
        // needed, we could optimize the retrieval from GitHub by
        // grouping configs by installation first.
        let configs: Vec<Config> = self.config_storage.list_for_plugin(PLUGIN_KIND.id).await?;
        tracing::trace!("Polling: {configs:?}");

        let now = Instant::now();
        let mut failed = 0;
        let mut successful = 0;
        futures::future::join_all(
            configs
                .into_iter()
                .map(|config| self.poll_installation(config)),
        )
        .await
        .into_iter()
        .for_each(|res| {
            if res.is_ok() {
                successful += 1;
            } else {
                failed += 1;
            }
        });
        tracing::info!(
            "Finished polling in {} ms. Successful: {}, Failed: {}",
            now.elapsed().as_millis(),
            successful,
            failed
        );

        Ok(())
    }

    #[tracing::instrument(
        skip(self, config),
        fields(gh_installation_id=config.external_id, project_id=config.project_id)
    )]
    pub(super) async fn poll_installation(&self, config: Config) -> Result<()> {
        if let Err(e) = self.poll_installation_internal(config).await {
            tracing::warn!("Failed installation poll: {e:?}");
            return Err(e);
        }
        Ok(())
    }

    async fn poll_installation_internal(&self, config: Config) -> Result<()> {
        tracing::debug!("Polling installation");

        let github_tasks_by_url = self.fetch_tasks_from_github(&config).await?;
        tracing::trace!("Fetched Github tasks: {:?}", github_tasks_by_url.values());

        let client = self
            .collab
            .register_local_client(&config.project_id)
            .await?;
        let doc_box = client.project.doc_box.lock().await;
        let doc_box = DocBox::doc_or_error(doc_box.as_ref())?;
        let doc = &doc_box.ydoc;

        let mut txn = doc.transact_mut_with(origin(&config)?);

        let parent = get_or_create_kind_parent(&mut txn, doc, PR_KIND)?;
        let doc_tasks_by_url = self.list_doc_tasks(&txn, doc, &parent, PR_KIND)?;
        tracing::trace!(
            "Found existing tasks in doc: {:?}",
            doc_tasks_by_url
                .iter()
                .map(|(k, v)| Ok(format!("{}:{k}", v.get_id(&txn)?)))
                .collect::<Result<Vec<_>>>()
        );

        // Resolve or update tasks that already exist in the doc.
        for (url, task) in doc_tasks_by_url.iter() {
            match github_tasks_by_url.get(url) {
                Some(github_task) => update_task(&mut txn, task, github_task)?,
                None => resolve_task(&mut txn, task)?,
            }
        }

        // Create any new tasks that don't already exist.
        let mut next_num: u64 = doc.next_num(&txn)?;
        let mut children = parent.get_children(&txn)?;
        for github_task in github_tasks_by_url.values() {
            match doc_tasks_by_url.get(&github_task.url) {
                Some(_) => {}
                None => {
                    let task = new_task(github_task, next_num, PR_KIND)?;
                    next_num += 1;
                    doc.set(&mut txn, &task);
                    children.push(task.id);
                }
            }
        }
        parent.set_children(&mut txn, &children);

        tracing::debug!(
            "Finished polling installation with {} active and {} total tasks",
            github_tasks_by_url.len(),
            children.len()
        );

        Ok(())
    }

    async fn fetch_tasks_from_github(
        &self,
        config: &Config,
    ) -> Result<HashMap<String, ExternalTask>> {
        let client = self
            .client
            .installation_github(InstallationRef::InstallationId {
                id: config.external_id.parse::<u64>()?,
            })
            .await?;
        let prs: Vec<octocrab::models::pulls::PullRequest> =
            client.fetch_install_pull_requests().await?;

        let mut results = HashMap::with_capacity(prs.len());
        for pr in prs {
            match ExternalTask::new(pr) {
                Ok(task) => {
                    if results.insert(task.url.clone(), task).is_some() {
                        tracing::warn!("Found multiple PRs with same url");
                    }
                }
                Err(e) => tracing::warn!("Skipping malformed PR: {e:?}"),
            }
        }
        Ok(results)
    }

    fn list_doc_tasks<T: ReadTxn>(
        &self,
        txn: &T,
        doc: &YDocProxy,
        parent: &YTaskProxy,
        kind: &Kind,
    ) -> Result<HashMap<String, YTaskProxy>> {
        let mut results = HashMap::new();
        for child_id in parent.get_children(txn)? {
            let child = doc.get(txn, &child_id)?;
            if child.get_kind(txn)?.is_some_and(|k| k == kind.id) {
                let url = child.get_url(txn)?.unwrap_or_default();
                if url.is_empty() {
                    tracing::warn!("Omitting doc task with empty URL: {child_id}");
                    continue;
                }
                if results.insert(url, child).is_some() {
                    tracing::warn!("Found multiple tasks with same url: {child_id}");
                }
            }
        }
        Ok(results)
    }
}

fn origin(config: &Config) -> Result<Origin> {
    YOrigin {
        who: "github_poller".to_string(),
        id: format!("install_{}", config.external_id),
        actor: Actor::GitHub,
    }
    .as_origin()
}
