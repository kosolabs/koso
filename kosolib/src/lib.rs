use std::fs;

use anyhow::Result;
use octocrab::{
    models::{pulls::PullRequest, AppId},
    params::{pulls::Sort, Direction, State},
    OctocrabBuilder,
};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub async fn fetch_pull_requests() -> Result<Vec<PullRequest>> {
    let pem = fs::read("koso-github.2024-11-14.private-key.pem").unwrap();
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(&pem).unwrap();
    let crab = OctocrabBuilder::new()
        .app(AppId::from(1053272), key)
        .build()
        .unwrap();
    let installation = crab
        .apps()
        .get_repository_installation("kosolabs", "koso")
        .await
        .unwrap();
    let crab = crab.installation(installation.id).unwrap();

    Ok(crab
        .pulls("kosolabs", "secret")
        .list()
        .state(State::Closed)
        .sort(Sort::Updated)
        .direction(Direction::Descending)
        .send()
        .await?
        .items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn pulls() {
        let pulls = fetch_pull_requests().await.unwrap();
        println!("Got pulls: {pulls:#?}");
    }
}
