use crate::secrets;
use anyhow::{anyhow, Context, Result};
use octocrab::{
    models::{
        self, pulls::PullRequest, AppId, InstallationId, InstallationRepositories, Repository,
    },
    params::{pulls::Sort, Direction, State},
    Octocrab, OctocrabBuilder,
};

const PROD_APP_ID: u64 = 1053272;
const DEV_APP_ID: u64 = 1066302;

pub enum InstallationRef {
    InstallationId { id: u64 },
}

#[derive(Clone)]
pub struct AppGithub {
    pub app_crab: Octocrab,
}

impl AppGithub {
    pub async fn new() -> Result<AppGithub> {
        // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps
        let app_key = jsonwebtoken::EncodingKey::from_rsa_pem(
            secrets::read_secret::<String>("github/key.pem")?
                .data
                .as_bytes(),
        )?;

        let app_crab = OctocrabBuilder::new()
            .app(AppId::from(Self::app_id()?), app_key)
            .build()?;
        Ok(AppGithub { app_crab })
    }

    fn app_id() -> Result<u64> {
        match std::env::var("GH_APP_ENV")
            .context("GH_APP_ENV is unset. Try GH_APP_ENV=dev")?
            .as_str()
        {
            "prod" => Ok(PROD_APP_ID),
            "dev" => Ok(DEV_APP_ID),
            env => Err(anyhow!("Invalid environment: {env}")),
        }
    }

    /// Authenticate as the given installation.
    pub async fn installation_github(
        &self,
        installation_ref: InstallationRef,
    ) -> Result<InstallationGithub> {
        let installation_id = match installation_ref {
            InstallationRef::InstallationId { id } => InstallationId::from(id),
        };

        let (installation_crab, _) = self
            .app_crab
            .installation_and_token(installation_id)
            .await
            .with_context(|| {
                format!("failed authenticating as installation '{installation_id}'")
            })?;
        Ok(InstallationGithub { installation_crab })
    }
}

pub struct InstallationGithub {
    pub installation_crab: Octocrab,
}

impl InstallationGithub {
    pub async fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        let mut page = self
            .installation_crab
            .pulls(owner, repo)
            .list()
            .state(State::Open)
            .sort(Sort::Updated)
            .direction(Direction::Descending)
            .per_page(100)
            .send()
            .await?;

        // Paginate through additional pages, if any, collecting all results.
        let mut prs = Vec::with_capacity(page.total_count.unwrap_or(0).try_into()?);
        loop {
            prs.append(&mut page.items);
            page = match self
                .installation_crab
                .get_page::<models::pulls::PullRequest>(&page.next)
                .await?
            {
                Some(next_page) => next_page,
                None => break,
            }
        }
        Ok(prs)
    }

    /// Returns all open PRs from all of the installation's repositories.
    pub async fn fetch_install_pull_requests(&self) -> Result<Vec<PullRequest>> {
        let installed_repos = self.fetch_install_repos().await?;
        let mut results = Vec::new();
        for repo in installed_repos {
            let owner = match repo.owner {
                Some(owner) => owner.login,
                None => return Err(anyhow!("No owner set for repo: {repo:?}")),
            };
            let name = repo.name;
            tracing::trace!("Fetching PRs for {owner}/{name}");
            results.push(async move {
                match self.fetch_pull_requests(&owner, &name).await {
                    Ok(prs) => {
                        tracing::trace!("Fetched {} PRs for {owner}/{name}", prs.len());
                        Ok(prs)
                    }
                    Err(e) => Err(anyhow!("Failed to fetch prs for {owner}/{name}: {e}")),
                }
            });
        }
        let results = futures::future::join_all(results).await;
        Ok(results
            .into_iter()
            .collect::<Result<Vec<Vec<PullRequest>>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// Returns all of this installation's repositories.
    pub async fn fetch_install_repos(&self) -> Result<Vec<Repository>> {
        let installed_repos: InstallationRepositories = self
            .installation_crab
            .get("/installation/repositories", None::<&()>)
            .await?;
        let len: i64 = installed_repos.repositories.len().try_into()?;
        if len != installed_repos.total_count {
            tracing::warn!("Number of intallation repositories is probably large and we need to paginate: {installed_repos:?}");
        }
        Ok(installed_repos.repositories)
    }
}
