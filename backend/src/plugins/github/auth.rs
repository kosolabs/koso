use self::google::User;
use crate::api::{bad_request_error, google, ApiResult};
use anyhow::{anyhow, Result};
use axum::{extract::Query, middleware, routing::post, Extension, Json, Router};
use octocrab::auth;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::Serialize;
use std::{fs, path::Path};

const DEFAULT_SECRETS_DIR: &str = "../.secrets";

#[derive(Serialize)]
struct AuthResult {
    token: String,
}

#[tracing::instrument(skip(code, auth))]
async fn auth_handler(
    Query(code): Query<String>,
    Extension(user): Extension<User>,
    Extension(auth): Extension<Auth>,
) -> ApiResult<Json<AuthResult>> {
    if code.is_empty() {
        return Err(bad_request_error("EMPTY_CODE", "code must be present"));
    }
    let oauth = auth.generate_access_token(&code).await?;
    Ok(Json(AuthResult {
        token: oauth.access_token.expose_secret().to_string(),
    }))
}

#[derive(Clone)]
pub(super) struct Auth {
    client_id: String,
    client_secret: ClientSecret,
    client: Client,
}

/// Contains the Github app's client secret.
#[derive(Clone)]
struct ClientSecret {
    secret: String,
}

//   const appId = "1053272";
const PROD_CLIENT_ID: &str = "Iv23lioB8K1C62NP3UbV";
//   const devAppId = "1066302";
const DEV_CLIENT_ID: &str = "Iv23lif5pPjNjiQVtgPH";

impl Auth {
    pub(super) fn new() -> Result<Auth> {
        let client_id = match std::env::var("GH_APP_ENV")
            .unwrap_or("dev".to_string())
            .as_str()
        {
            "prod" => PROD_CLIENT_ID,
            "dev" => DEV_CLIENT_ID,
            env => return Err(anyhow!("Invalid environment: {env}")),
        }
        .to_string();
        let client_secret = Self::read_client_secret()?;
        Ok(Auth {
            client_id,
            client_secret,
            client: Client::default(),
        })
    }

    /// Read the client secret from $secrets_dir/github/client_secret.
    /// The default is `../.secrets/github/client_secret`, unless `SECRETS_DIR` is set.
    fn read_client_secret() -> Result<ClientSecret> {
        let dir = std::env::var("SECRETS_DIR").unwrap_or_else(|_| DEFAULT_SECRETS_DIR.to_string());
        let path = Path::new(&dir)
            .join("github/client_")
            .into_os_string()
            .into_string()
            .map_err(|e| anyhow!("Invalid github secret path in {dir}: {e:?}"))?;
        tracing::info!("Using github webhook secret at {path}");
        let secret = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read github secret from {path}: {e}"))?
            .trim()
            .to_owned();
        Ok(ClientSecret { secret })
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/auth", post(auth_handler))
            .layer((Extension(self), middleware::from_fn(google::authenticate)))
    }

    async fn generate_access_token(&self, code: &String) -> Result<auth::OAuth> {
        let res = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/json")
            .query(&[
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret.secret),
                ("code", code),
            ])
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(anyhow!("Access token post failed: {}", res.status()));
        }
        Ok(res.json().await?)
    }
}
