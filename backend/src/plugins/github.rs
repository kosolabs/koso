use anyhow::Result;

const USER_AGENT: &str = "koso - https://github.com/kosolabs/koso";

pub async fn shad() -> Result<String> {
    let client = reqwest::Client::new();
    Ok(client
        .get("https://api.github.com/repos/kosolabs/koso/pulls")
        .query(&[
            ("state", "closed"),
            ("sort", "updated"),
            ("direction", "desc"),
        ])
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .text()
        .await?)
}
