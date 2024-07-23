use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Key {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Certs {
    keys: Vec<Key>,
}

impl Certs {
    pub fn get(&self, kid: &str) -> Result<DecodingKey, Box<dyn Error>> {
        for key in &self.keys {
            if key.kid == *kid {
                return Ok(DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Err("missing".into())
    }
}

pub fn parse(json: &str) -> Result<Certs, Box<dyn Error>> {
    let certs: Certs = serde_json::from_str(json)?;
    Ok(certs)
}

pub async fn fetch() -> Result<Certs, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()
        .await?
        .text()
        .await?;
    let certs: Certs = parse(&resp)?;
    Ok(certs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub name: String,
    pub picture: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use crate::google::Certs;
    use crate::google::{fetch, parse};

    fn certs() -> Certs {
        parse(include_str!("testdata/certs.json")).unwrap()
    }

    #[tokio::test]
    async fn fetch_succeeds() {
        let result = fetch();
        assert!(result.await.is_ok());
    }

    #[test]
    fn get_returns_error_if_kid_is_missing() {
        let certs = certs();
        assert!(certs.get("missing").is_err())
    }

    #[test]
    fn get_returns_key_if_kid_exists() {
        let certs = certs();
        assert!(certs.get("1").is_ok())
    }
}
