use crate::{
    api::{
        bad_request_error,
        google::{self, User},
        ApiResult,
    },
    plugins::github::{read_secret, Secret},
};
use anyhow::{anyhow, Result};
use axum::{extract::Query, middleware, routing::post, Extension, Json, Router};
use octocrab::auth;
use reqwest::Client;
use secrecy::ExposeSecret as _;
use serde::Serialize;

const PROD_CLIENT_ID: &str = "Iv23lioB8K1C62NP3UbV";
const DEV_CLIENT_ID: &str = "Iv23lif5pPjNjiQVtgPH";

#[derive(Serialize)]
struct AuthResult {
    token: String,
}

#[derive(Clone)]
pub(super) struct Auth {
    client_id: String,
    client_secret: ClientSecret,
    client: Client,
}

/// Contains the Github app's client secret.
type ClientSecret = Secret<String>;

impl Auth {
    pub(super) fn new() -> Result<Auth> {
        Ok(Auth {
            client_id: Self::client_id()?,
            client_secret: read_secret("github/client_secret")?,
            client: Client::default(),
        })
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/auth", post(Self::auth_handler))
            .layer((Extension(self), middleware::from_fn(google::authenticate)))
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

    async fn generate_access_token(&self, code: &String) -> Result<auth::OAuth> {
        let res = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/json")
            .query(&[
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret.data),
                ("code", code),
            ])
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(anyhow!("Access token post failed: {}", res.status()));
        }
        Ok(res.json().await?)
    }

    fn client_id() -> Result<String> {
        Ok(match std::env::var("GH_APP_ENV")
            .unwrap_or("dev".to_string())
            .as_str()
        {
            "prod" => PROD_CLIENT_ID,
            "dev" => DEV_CLIENT_ID,
            env => return Err(anyhow!("Invalid environment: {env}")),
        }
        .to_string())
    }
}
