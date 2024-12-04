use crate::{
    api::{
        bad_request_error,
        google::{self, User},
        ApiResult,
    },
    plugins::github::{read_secret, Secret},
};
use anyhow::{anyhow, Context, Result};
use axum::{middleware, routing::post, Extension, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const PROD_CLIENT_ID: &str = "Iv23lioB8K1C62NP3UbV";
const DEV_CLIENT_ID: &str = "Iv23lif5pPjNjiQVtgPH";

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
}

#[derive(Serialize)]
struct AuthResult {
    access_token: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum GithubOAuthResponse {
    Success(OAuth),
    Error(OAuthError),
}

#[derive(Deserialize)]
struct OAuth {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: Option<usize>,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<usize>,
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

    #[tracing::instrument(skip(request, user, auth))]
    async fn auth_handler(
        Extension(user): Extension<User>,
        Extension(auth): Extension<Auth>,
        Json(request): Json<AuthRequest>,
    ) -> ApiResult<Json<AuthResult>> {
        if request.code.is_empty() {
            return Err(bad_request_error("EMPTY_CODE", "code must be present"));
        }
        let oauth = auth.generate_access_token(&request.code).await?;
        Ok(Json(AuthResult {
            access_token: oauth.access_token,
        }))
    }

    async fn generate_access_token(&self, code: &String) -> Result<OAuth> {
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
            return Err(anyhow!("Access token post failed: {}", res.status()));
        }
        let res: GithubOAuthResponse = res
            .json()
            .await
            .with_context(|| "Failed to decode login response")?;
        let oauth = match res {
            GithubOAuthResponse::Success(oauth) => oauth,
            GithubOAuthResponse::Error(e) => {
                return Err(anyhow!(
                    "Access token post failed with error: '{}' - '{}'",
                    e.error,
                    e.error_description
                ));
            }
        };

        Ok(oauth)
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
