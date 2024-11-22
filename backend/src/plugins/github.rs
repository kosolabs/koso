use anyhow::Result;
use kosolib::{AppGithub, AppGithubConfig, InstallationRef};

pub async fn shad() -> Result<String> {
    let client = AppGithub::new(&AppGithubConfig::default()).await?;
    let client = client
        .installation_github(InstallationRef::Org { owner: "kosolabs" })
        .await?;
    let prs = client.fetch_pull_requests("kosolabs", "secret").await?;

    Ok(serde_json::to_string(&prs)?)
}
