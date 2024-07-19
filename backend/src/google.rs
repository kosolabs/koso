use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Key {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub name: String,
    pub exp: usize,
}
