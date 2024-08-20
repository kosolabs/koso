use crate::api::{unauthenticated_error, ApiResult};
use anyhow::{anyhow, Result};
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct KeySet {
    inner: Arc<KeySetInner>,
}

pub(crate) struct KeySetInner {
    certs: Mutex<Certs>,
    client: reqwest::Client,
    last_load: Mutex<Instant>,
}

impl KeySet {
    const LOAD_DEBOUNCE_DURATION: Duration = Duration::from_secs(306);

    pub(crate) async fn new() -> Result<KeySet> {
        let key_set = KeySet {
            inner: Arc::new(KeySetInner {
                certs: Mutex::new(Certs {
                    keys: Vec::with_capacity(0),
                }),
                client: reqwest::Client::new(),
                last_load: Mutex::new(Instant::now()),
            }),
        };
        // Avoid a cold start by preloading keys initially.
        let _ = key_set.load_keys(None).await?;
        Ok(key_set)
    }

    pub(crate) async fn get(&self, kid: &str) -> Result<DecodingKey> {
        // This is the happy path. The key is already cached and ready to go.
        if let Some(key) = self.inner.certs.lock().await.get(kid) {
            return Ok(key.key.clone());
        }
        // Maybe the key is new, so try reloading from Google.
        match self.load_keys(Some(kid)).await? {
            Some(key) => Ok(key),
            None => Err(anyhow!("Key not found")),
        }
    }

    async fn load_keys(&self, kid: Option<&str>) -> Result<Option<DecodingKey>> {
        let mut last_load = self.inner.last_load.lock().await;
        // Limit how often we make the remote call to avoid being DOS'd.
        // If we didn't find the cert a minute ago, it probably still doesn't exist.
        if Instant::now() - *last_load < KeySet::LOAD_DEBOUNCE_DURATION {
            return Ok(None);
        }

        tracing::debug!("Fetching google certs");
        let certs = parse(
            &self
                .inner
                .client
                .get("https://www.googleapis.com/oauth2/v3/certs")
                .send()
                .await?
                .text()
                .await?,
        )?;
        tracing::debug!("Fetched google certs: {certs:?}");

        let key = kid.and_then(|kid| certs.get(kid).map(|key| key.key.clone()));
        *last_load = Instant::now();
        *self.inner.certs.lock().await = certs;

        Ok(key)
    }
}

struct Key {
    kid: String,
    key: DecodingKey,
}

#[derive(Debug)]
struct Certs {
    keys: Vec<Key>,
}

impl Certs {
    fn get(&self, kid: &str) -> Option<&Key> {
        self.keys.iter().find(|&key| key.kid == *kid)
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Key").field("kid", &self.kid).finish()
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
struct RawCerts {
    keys: Vec<RawKey>,
}

fn parse(json: &str) -> Result<Certs> {
    let raw_certs: RawCerts = serde_json::from_str(json)?;

    let mut certs = Certs {
        keys: Vec::with_capacity(raw_certs.keys.len()),
    };
    for key in raw_certs.keys {
        certs.keys.push(Key {
            kid: key.kid,
            key: DecodingKey::from_rsa_components(&key.n, &key.e)?,
        })
    }
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

        assert!(result.unwrap().get("does_not_exist_kid").await.is_err());
    }
}
