use anyhow::{anyhow, Result};
use jsonwebtoken::EncodingKey;
use octocrab::{
    models::{
        self, pulls::PullRequest, AppId, InstallationId, InstallationRepositories, Repository,
    },
    params::{pulls::Sort, Direction, State},
    Octocrab, OctocrabBuilder,
};
use std::{fs, path::Path};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

const PROD_APP_ID: u64 = 1053272;
const DEV_APP_ID: u64 = 1066302;
const DEFAULT_SECRETS_DIR: &str = "../.secrets";

pub enum InstallationRef<'a> {
    Org { owner: &'a str },
    Repo { owner: &'a str, repo: &'a str },
    InstallationId { id: u64 },
}

#[derive(Default)]
pub struct AppGithubConfig {
    /// Path to the Github application key file in the RSA PEM file format.
    ///
    /// If unspecified, the path defaults to `$secrets_dir/github/key.pem`. The
    /// `$secrets_dir` directory is either he value of the SECRETS_DIR environment
    /// variable or, if that is absent, the DEFAULT_SECRETS_DIR constant.
    ///
    /// See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/managing-private-keys-for-github-apps
    app_key_path: Option<String>,
    /// Github application id.
    ///
    /// If unspecified, the ID defaults to the ID indicated by the environment
    /// given by the GH_APP_ENV environment variable or, if that is absent,
    /// the dev id, DEV_APP_ID.
    app_id: Option<u64>,
}

impl AppGithubConfig {
    fn app_id(&self) -> Result<u64> {
        match self.app_id {
            Some(app_id) => Ok(app_id),
            None => {
                match std::env::var("GH_APP_ENV")
                    .unwrap_or("dev".to_string())
                    .as_str()
                {
                    "prod" => Ok(PROD_APP_ID),
                    "dev" => Ok(DEV_APP_ID),
                    env => Err(anyhow!("Invalid environment: {env}")),
                }
            }
        }
    }

    fn app_key_path(&self) -> Result<String> {
        match self.app_key_path.clone() {
            Some(app_key_path) => Ok(app_key_path),
            None => Path::new(
                &std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string()),
            )
            .join("github/key.pem")
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!("Path error: {e:?}")),
        }
    }

    fn app_key(&self) -> Result<EncodingKey> {
        let key_path = self.app_key_path()?;
        AppGithubConfig::read_app_key(&key_path)
    }

    fn read_app_key(key_path: &str) -> Result<EncodingKey> {
        let pem =
            fs::read(key_path).map_err(|e| anyhow!("Failed to read filed {key_path}: {e}"))?;
        Ok(jsonwebtoken::EncodingKey::from_rsa_pem(&pem)?)
    }
}

#[derive(Clone)]
pub struct AppGithub {
    pub app_crab: Octocrab,
}

impl AppGithub {
    pub async fn new(config: &AppGithubConfig) -> Result<AppGithub> {
        let app_crab = OctocrabBuilder::new()
            .app(AppId::from(config.app_id()?), config.app_key()?)
            .build()?;
        Ok(AppGithub { app_crab })
    }

    /// Authenticate as the given installation.
    pub async fn installation_github(
        &self,
        installation_ref: InstallationRef<'_>,
    ) -> Result<InstallationGithub> {
        let installation_id = match installation_ref {
            InstallationRef::Org { owner } => {
                self.app_crab.apps().get_org_installation(owner).await?.id
            }
            InstallationRef::Repo { owner, repo } => {
                self.app_crab
                    .apps()
                    .get_repository_installation(owner, repo)
                    .await?
                    .id
            }
            InstallationRef::InstallationId { id } => InstallationId::from(id),
        };

        let (installation_crab, _) = self
            .app_crab
            .installation_and_token(installation_id)
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn pulls() {
        let gh = InstallationGithub {
            installation_crab: Octocrab::default(),
        };
        let pulls = gh.fetch_pull_requests("kosolabs", "koso").await.unwrap();
        assert!(!pulls.is_empty());
    }

    #[test_log::test(tokio::test)]
    async fn repos() {
        let client = AppGithub::new(&AppGithubConfig::default()).await.unwrap();
        let gh = client
            .installation_github(InstallationRef::InstallationId { id: 57461190 })
            .await
            .unwrap();
        let repos = gh.fetch_install_repos().await.unwrap();
        assert!(!repos.is_empty());
    }
}
