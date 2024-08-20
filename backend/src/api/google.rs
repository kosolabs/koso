use crate::api::{unauthenticated_error, ApiResult};
use anyhow::{anyhow, Result};
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub(crate) struct KeySet {
    keys: Arc<DashMap<String, LoadedKey>>,
    client: reqwest::Client,
}

#[derive(Debug)]
enum LoadedKey {
    Present { key: Key },
    Absent { fetch_time: Instant },
}

struct Key {
    decoding_key: DecodingKey,
    raw_key: RawKey,
}

impl KeySet {
    pub(crate) async fn new() -> Result<KeySet> {
        let key_set = KeySet {
            keys: Arc::new(DashMap::new()),
            client: reqwest::Client::new(),
        };
        // Avoid a cold start by preloading keys initially.
        key_set.load_keys().await?;
        Ok(key_set)
    }

    pub(crate) async fn get(&self, kid: &str) -> Result<DecodingKey> {
        if let Some(loaded_key) = self.keys.get(kid) {
            match loaded_key.value() {
                LoadedKey::Present { key } => {
                    // This is the happy path. The key is already cached and ready to go.
                    return Ok(key.decoding_key.clone());
                }
                LoadedKey::Absent { fetch_time } => {
                    const NOT_FOUND_CACHE_DURATION: Duration = Duration::from_secs(360);
                    if Instant::now() <= *fetch_time + NOT_FOUND_CACHE_DURATION {
                        return Err(anyhow!("Key {kid} not found (cached at {:?}).", fetch_time));
                    }
                }
            }
        }

        if let Err(e) = self.load_keys().await {
            return Err(anyhow!("Failed to load keys: {e}"));
        }

        match self.keys.entry(kid.to_string().clone()) {
            dashmap::Entry::Occupied(o) => match &o.get() {
                LoadedKey::Present { key } => Ok(key.decoding_key.clone()),
                LoadedKey::Absent { fetch_time } => {
                    let mut fetch_time = *fetch_time;
                    if Instant::now() <= fetch_time + Duration::from_secs(60) {
                        fetch_time = Instant::now();
                        o.replace_entry(LoadedKey::Absent { fetch_time });
                    }
                    Err(anyhow!(
                        "Key {kid} not found (after load, cached at {:?}).",
                        fetch_time
                    ))
                }
            },
            dashmap::Entry::Vacant(v) => {
                let fetch_time = Instant::now();
                v.insert(LoadedKey::Absent { fetch_time });
                Err(anyhow!(
                    "Key {kid} not found (uncached at {:?}).",
                    fetch_time
                ))
            }
        }
    }

    async fn load_keys(&self) -> Result<()> {
        let certs = parse(
            &self
                .client
                .get("https://www.googleapis.com/oauth2/v3/certs")
                .send()
                .await?
                .text()
                .await?,
        )?;
        for key in certs.keys {
            self.keys.insert(
                key.kid.clone(),
                LoadedKey::Present {
                    key: Key {
                        decoding_key: DecodingKey::from_rsa_components(&key.n, &key.e)?,
                        raw_key: key,
                    },
                },
            );
        }
        Ok(())
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw_key.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RawKey {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Certs {
    keys: Vec<RawKey>,
}

fn parse(json: &str) -> Result<Certs> {
    let certs: Certs = serde_json::from_str(json)?;
    Ok(certs)
}

#[tracing::instrument(skip(request, next), fields(email))]
pub(crate) async fn authenticate(mut request: Request, next: Next) -> ApiResult<Response<Body>> {
    let key_set = request.extensions().get::<KeySet>().unwrap();
    let headers = request.headers();

    let bearer = if let Some(auth_header) = headers.get("Authorization") {
        let Ok(auth) = auth_header.to_str() else {
            return Err(unauthenticated_error(&format!(
                "Could not convert auth header to string: {auth_header:?}"
            )));
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(unauthenticated_error(&format!(
                "Could not split bearer parts: {parts:?}"
            )));
        }
        parts[1]
    } else if let Some(swp_header) = headers.get("sec-websocket-protocol") {
        let Ok(swp) = swp_header.to_str() else {
            return Err(unauthenticated_error(&format!(
                "sec-websocket-protocol must be only visible ASCII chars: {swp_header:?}"
            )));
        };
        let parts: Vec<&str> = swp.split(", ").collect();
        if parts.len() != 2 || parts[0] != "bearer" {
            return Err(unauthenticated_error(&format!(
                "sec-websocket-protocol must contain a bearer token: {parts:?}"
            )));
        }
        parts[1]
    } else {
        return Err(unauthenticated_error("Authorization header is absent."));
    };

    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return Err(unauthenticated_error(&format!(
            "Could not decode header: {bearer:?}"
        )));
    };
    let Some(kid) = header.kid else {
        return Err(unauthenticated_error(&format!(
            "header.kid is absent: {header:?}"
        )));
    };
    let key = match key_set.get(&kid).await {
        Ok(key) => key,
        Err(e) => {
            return Err(unauthenticated_error(&format!(
                "certs is absent for {kid:?}: {e}"
            )));
        }
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    // Allow the token to last seven days longer than the given expiry.
    // This number matches the clients's validation in auth.ts.
    const SEVEN_DAYS_SECS: u64 = 7 * 24 * 60 * 60;
    validation.leeway = SEVEN_DAYS_SECS;
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let token = match jsonwebtoken::decode::<User>(bearer, &key, &validation) {
        Ok(token) => token,
        Err(e) => {
            return Err(unauthenticated_error(&format!("Failed validation: {e}")));
        }
    };
    if token.claims.email.is_empty() {
        return Err(unauthenticated_error(&format!(
            "Claims email is empty: {token:?}"
        )));
    }

    tracing::Span::current().record("email", token.claims.email.clone());
    assert!(request.extensions_mut().insert(token.claims).is_none());

    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct User {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: String,
    pub(crate) exp: usize,
}

#[cfg(test)]
mod tests {
    use crate::api::google::KeySet;

    #[tokio::test]
    async fn fetch_succeeds() {
        let result: Result<KeySet, anyhow::Error> = KeySet::new().await;
        assert!(result.is_ok());
    }
}
