use crate::{
    secrets::{self},
    settings::settings,
};
use anyhow::{Context, Result, anyhow};
use base64::{Engine, prelude::BASE64_STANDARD};
use futures::StreamExt;
use octocrab::{
    Octocrab, OctocrabBuilder,
    models::{
        self, AppId, InstallationId, InstallationRepositories, Repository, pulls::PullRequest,
        repos::Object,
    },
    params::{Direction, State, pulls::Sort, repos::Reference},
};

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
            .app(AppId::from(settings().plugins.github.app_id), app_key)
            .build()?;
        Ok(AppGithub { app_crab })
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

    pub async fn repo_github(&self, owner: &str, repo: &str) -> Result<RepoGithub> {
        let installation_id = self
            .app_crab
            .apps()
            .get_repository_installation(owner, repo)
            .await?
            .id;

        let (octocrab, _) = self
            .app_crab
            .installation_and_token(installation_id)
            .await?;

        Ok(RepoGithub {
            octocrab,
            owner: owner.into(),
            repo: repo.into(),
        })
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
                    Err(e) => Err(e.context("Failed to fetch prs for {owner}/{name}")),
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
            tracing::warn!(
                "Number of intallation repositories is probably large and we need to paginate: {installed_repos:?}"
            );
        }
        Ok(installed_repos.repositories)
    }
}

pub struct RepoGithub {
    pub octocrab: Octocrab,
    owner: String,
    repo: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Trees {
    sha: String,
    url: String,
    tree: Vec<TreeItem>,
    truncated: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeItem {
    path: String,
    mode: String,
    r#type: String,
    sha: String,
    size: Option<u64>,
    url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Blob {
    sha: String,
    node_id: String,
    size: u64,
    url: String,
    content: String,
    encoding: String,
}

impl RepoGithub {
    pub async fn get_trees(&self) -> Result<Trees> {
        let branch = self
            .octocrab
            .repos(&self.owner, &self.repo)
            .get()
            .await?
            .default_branch
            .context("Unable to determine the default branch")?;

        let commit_hash = match self
            .octocrab
            .repos(&self.owner, &self.repo)
            .get_ref(&Reference::Branch(branch))
            .await?
            .object
        {
            Object::Commit { sha, .. } => sha,
            Object::Tag { sha, .. } => sha,
            _ => return Err(anyhow!("Unknown tag")),
        };

        let resp = serde_json::from_str::<Trees>(
            &self
                .octocrab
                .body_to_string(
                    self.octocrab
                        ._get(format!(
                            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=true",
                            self.owner, self.repo, commit_hash
                        ))
                        .await?,
                )
                .await?,
        )?;

        Ok(resp)
    }

    pub async fn get_blob(&self, url: &str) -> Result<Blob> {
        tracing::trace!("Downloading {url}");
        let blob = serde_json::from_str::<Blob>(
            &self
                .octocrab
                .body_to_string(self.octocrab._get(url).await?)
                .await?,
        )?;
        if blob.encoding != "base64" {
            return Err(anyhow!("Unknown encoding: {}", blob.encoding));
        }
        Ok(blob)
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        let blob = self.get_blob(url).await?;

        Ok(String::from_utf8(
            BASE64_STANDARD.decode(
                blob.content
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>(),
            )?,
        )?)
    }

    pub async fn compile_source_context(&self) -> Result<String> {
        let paths = self
            .get_trees()
            .await?
            .tree
            .into_iter()
            .filter(|item| {
                // Exclude items that aren't files
                if item.r#type != "blob" {
                    tracing::trace!("Exclude non-blob: {}", item.path);
                    return false;
                }

                // Exclude files that don't have a size or whose size is greater than 256kb
                if item.size.is_none_or(|size| size > 262_144) {
                    tracing::trace!("Exclude large file: {}", item.path);
                    return false;
                };

                let Some(name) = item.path.split("/").last() else {
                    tracing::trace!("Exclude no name: {}", item.path);
                    return false;
                };

                // Exclude hidden files
                if name.starts_with(".") {
                    tracing::trace!("Exclude hidden: {}", item.path);
                    return false;
                }

                // Files to exclude
                if [
                    "package-lock.json",
                    "pnpm-lock.yaml",
                    "go.sum",
                    "mix.lock", // Lock files
                ]
                .into_iter()
                .any(|exclude| name == exclude)
                {
                    tracing::trace!("Exclude file: {}", item.path);
                    return false;
                }

                // Extensions to exclude
                if [
                    ".lock", // Text files
                    ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".svg", ".webp", ".ico", ".tiff",
                    ".tif", ".apng", // Image files,
                    ".mp4", ".mov", ".avi", ".wmv", ".flv", ".mkv", ".webm", ".mpeg", ".mpg",
                    ".m4v", ".3gp", ".3g2", ".ts", ".mts", ".m2ts", ".vob",
                    ".ogv", // Video files
                ]
                .into_iter()
                .any(|ext| name.ends_with(ext))
                {
                    tracing::trace!("Exclude extension: {}", item.path);
                    return false;
                }
                true
            })
            .collect::<Vec<TreeItem>>();

        tracing::trace!("Processing: {paths:?}");

        let total_files = paths.len();

        let context = paths
            .into_iter()
            .map(async |item| {
                let text = match self.get_text(&item.url).await {
                    Ok(text) => text,
                    Err(error) => {
                        tracing::warn!("Error reading file {} {}", item.url, error);
                        "".to_string()
                    }
                };
                format!(
                    "## File: {}\n```{}\n{}\n```\n\n",
                    item.path,
                    item.path.split('.').next_back().unwrap_or_default(),
                    text
                )
            })
            .collect::<Vec<_>>();

        let context = futures::stream::iter(context)
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await
            .join("");

        tracing::trace!("Context: {context}");

        tracing::info!(
            "Generated context for: {}/{} ({} files, {} bytes)",
            self.owner,
            self.repo,
            total_files,
            context.len()
        );

        Ok(context)
    }
}
