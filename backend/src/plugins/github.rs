use std::path::Path;

use anyhow::{anyhow, Result};
use kosolib::{AppGithub, InstallationRef};

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

pub async fn shad() -> Result<String> {
    let key_path = Path::new(
        &std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string()),
    )
    .join("github/key.pem")
    .into_os_string()
    .into_string()
    .map_err(|e| anyhow!("Path error: {e:?}"))?;
    let client = AppGithub::new(&key_path).await?;
    let client = client
        .installation_github(InstallationRef::Org { owner: "kosolabs" })
        .await?;
    let prs = client.fetch_pull_requests("kosolabs", "secret").await?;

    Ok(serde_json::to_string(&prs)?)
}
