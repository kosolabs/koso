use anyhow::Result;
use kosolib::{AppGithub, InstallationRef};

pub async fn shad() -> Result<String> {
    let key_path = std::env::var("GH_APP_KEY_PATH")
        .unwrap_or_else(|_| "../.secrets/koso-github.2024-11-14.private-key.pem".to_string());
    let client = AppGithub::new(&key_path).await?;
    let client = client
        .installation_github(&InstallationRef::Org { owner: "kosolabs" })
        .await?;
    let prs = client.fetch_pull_requests("kosolabs", "secret").await?;

    Ok(serde_json::to_string(&prs)?)
}
