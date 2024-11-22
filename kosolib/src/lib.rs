use anyhow::{anyhow, Result};
use jsonwebtoken::EncodingKey;
use octocrab::{
    models::{pulls::PullRequest, AppId, InstallationId},
    params::{pulls::Sort, Direction, State},
    Octocrab, OctocrabBuilder,
};
use std::fs;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

const APP_ID: u64 = 1053272;

pub enum InstallationRef<'a> {
    Org { owner: &'a str },
    Repo { owner: &'a str, repo: &'a str },
    InstallationId { id: u64 },
}

pub struct AppGithub {
    pub app_crab: Octocrab,
}

impl AppGithub {
    pub async fn new(app_key_path: &str) -> Result<AppGithub> {
        let app_crab = OctocrabBuilder::new()
            .app(AppId::from(APP_ID), AppGithub::read_app_key(app_key_path)?)
            .build()?;
        Ok(AppGithub { app_crab })
    }

    fn read_app_key(key_path: &str) -> Result<EncodingKey> {
        let pem =
            fs::read(key_path).map_err(|e| anyhow!("Failed to read filed {key_path}: {e}"))?;
        Ok(jsonwebtoken::EncodingKey::from_rsa_pem(&pem)?)
    }

    pub async fn installation_github(
        self,
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
    pub async fn fetch_pull_requests(self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        Ok(self
            .installation_crab
            .pulls(owner, repo)
            .list()
            .state(State::Closed)
            .sort(Sort::Updated)
            .direction(Direction::Descending)
            .send()
            .await?
            .items)
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
}
