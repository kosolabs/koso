use crate::api::{unauthorized_error, ApiResult};
use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation};
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
pub(crate) struct Certs {
    keys: Vec<Key>,
}

impl Certs {
    fn get(&self, kid: &str) -> Result<DecodingKey, Box<dyn Error>> {
        for key in &self.keys {
            if key.kid == *kid {
                return Ok(DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Err("missing".into())
    }
}

fn parse(json: &str) -> Result<Certs, Box<dyn Error>> {
    let certs: Certs = serde_json::from_str(json)?;
    Ok(certs)
}

pub(crate) async fn fetch() -> Result<Certs, Box<dyn Error>> {
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

#[tracing::instrument(skip(request, next), fields(email))]
pub(crate) async fn authenticate(mut request: Request, next: Next) -> ApiResult<Response<Body>> {
    let certs = request.extensions().get::<Certs>().unwrap();
    let headers = request.headers();

    let bearer = if let Some(auth_header) = headers.get("Authorization") {
        let Ok(auth) = auth_header.to_str() else {
            return Err(unauthorized_error(&format!(
                "Could not convert auth header to string: {auth_header:?}"
            )));
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(unauthorized_error(&format!(
                "Could not split bearer parts: {parts:?}"
            )));
        }
        parts[1]
    } else if let Some(swp_header) = headers.get("sec-websocket-protocol") {
        let Ok(swp) = swp_header.to_str() else {
            return Err(unauthorized_error(&format!(
                "sec-websocket-protocol must be only visible ASCII chars: {swp_header:?}"
            )));
        };
        let parts: Vec<&str> = swp.split(", ").collect();
        if parts.len() != 2 || parts[0] != "bearer" {
            return Err(unauthorized_error(&format!(
                "sec-websocket-protocol must contain a bearer token: {parts:?}"
            )));
        }
        parts[1]
    } else {
        return Err(unauthorized_error("Authorization header is absent."));
    };

    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return Err(unauthorized_error(&format!(
            "Could not decode header: {bearer:?}"
        )));
    };
    let Some(kid) = header.kid else {
        return Err(unauthorized_error(&format!(
            "header.kid is absent: {header:?}"
        )));
    };
    let key = match certs.get(&kid) {
        Ok(key) => key,
        Err(e) => {
            return Err(unauthorized_error(&format!(
                "certs is absent for {kid:?}: {e}"
            )));
        }
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let token = match jsonwebtoken::decode::<User>(bearer, &key, &validation) {
        Ok(token) => token,
        Err(e) => {
            return Err(unauthorized_error(&format!("Failed validation: {e}")));
        }
    };
    if token.claims.email.is_empty() {
        return Err(unauthorized_error(&format!(
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
    use crate::api::google::Certs;
    use crate::api::google::{fetch, parse};

    fn certs() -> Certs {
        parse(include_str!("../testdata/certs.json")).unwrap()
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
