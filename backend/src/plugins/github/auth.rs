use crate::{
    api::{ApiResult, bad_request_error, google::User},
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Context, Result, anyhow};
use axum::{Extension, Json, Router, routing::post};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthRequest {
    code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthResult {
    expires_in: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum GithubOAuthResponse {
    Success(OAuth),
    Error(OAuthError),
}

#[derive(Deserialize, Clone)]
pub(super) struct OAuth {
    pub(super) access_token: String,
    // token_type: String,
    // scope: String,
    expires_in: Option<u64>,
    // refresh_token: Option<String>,
    // refresh_token_expires_in: Option<u64>,
    #[serde(skip_deserializing, default = "Instant::now")]
    created_at: Instant,
}

#[derive(Deserialize)]
struct OAuthError {
    error: String,
    error_description: String,
}

#[derive(Clone)]
pub(super) struct Auth {
    client_id: String,
    client_secret: ClientSecret,
    client: Client,
    tokens: Arc<Mutex<HashMap<String, OAuth>>>,
}

/// Contains the Github app's client secret.
type ClientSecret = Secret<String>;

impl Auth {
    pub(super) fn new() -> Result<Auth> {
        Ok(Auth {
            client_id: settings().plugins.github.client_id.clone(),
            client_secret: read_secret("github/client_secret")?,
            client: Client::default(),
            tokens: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/auth", post(Self::auth_handler))
            .layer((Extension(self),))
    }

    #[tracing::instrument(skip(request, user, auth))]
    async fn auth_handler(
        Extension(user): Extension<User>,
        Extension(auth): Extension<Auth>,
        Json(request): Json<AuthRequest>,
    ) -> ApiResult<Json<AuthResult>> {
        if request.code.is_empty() {
            return Err(bad_request_error("EMPTY_CODE", "Code is blank"));
        }
        let oauth = auth.generate_access_token(&request.code).await?;
        let expires_in = oauth.expires_in.unwrap_or(60 * 60 * 4);
        auth.tokens.lock().await.insert(user.email, oauth);
        Ok(Json(AuthResult { expires_in }))
    }

    async fn generate_access_token(&self, code: &String) -> ApiResult<OAuth> {
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
        let status = res.status();
        if !status.is_success() {
            return Err(anyhow!("Access token post failed: {}", res.status()).into());
        }
        let res: GithubOAuthResponse = res
            .json()
            .await
            .with_context(|| "Failed to decode login response")?;
        let oauth = match res {
            GithubOAuthResponse::Success(oauth) => oauth,
            GithubOAuthResponse::Error(e) => {
                return Err(bad_request_error(
                    "GITHUB_AUTH_REJECTED",
                    &format!("Login rejected: '{}' - '{}'", e.error, e.error_description),
                ));
            }
        };

        Ok(oauth)
    }

    pub(super) async fn user_access_token(&self, user: &User) -> ApiResult<OAuth> {
        let token = {
            let tokens = self.tokens.lock().await;
            tokens.get(&user.email).cloned()
        };
        let Some(token) = token else {
            return Err(bad_request_error(
                "GITHUB_UNAUTHENTICATED",
                "User is not authenticated with Github.",
            ));
        };

        if token.expires_in.is_some_and(|expires_in| {
            token.created_at.elapsed() > Duration::from_secs(expires_in - 60 * 60)
        }) {
            return Err(bad_request_error(
                "GITHUB_UNAUTHENTICATED",
                "User's Github authentication expired.",
            ));
        }
        Ok(token)
    }
}
