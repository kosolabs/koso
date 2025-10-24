use crate::{
    api::{self, google::User},
    plugins::{
        config::{Config, ConfigStorage, GithubSettings, Settings},
        github::{self, Poller},
    },
    secrets::{Secret, read_secret},
    settings::settings,
};
use anyhow::{Context as _, Result, anyhow};
use axum::{
    Extension, Json, Router,
    routing::{delete, get, post},
};
use axum_anyhow::{ApiResult, bad_request, forbidden, not_found};
use octocrab::{Octocrab, OctocrabBuilder, models::Installation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::Instrument;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConnectRequest {
    project_id: String,
    installation_id: String,
    code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectResponse {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConnectUserRequest {
    code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectUserResponse {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitResponse {
    app_name: String,
    client_id: String,
}

#[derive(Serialize)]
struct GithubOAuthRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
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
}

#[derive(Deserialize)]
struct OAuthError {
    error: String,
    error_description: String,
}

/// Contains the Github app's client secret.
type ClientSecret = Secret<String>;

#[derive(Clone)]
pub(super) struct ConnectHandler {
    pool: &'static PgPool,
    storage: ConfigStorage,
    poller: Poller,
    crab: Octocrab,
    client_id: String,
    client_secret: ClientSecret,
    app_name: String,
    client: Client,
}

impl ConnectHandler {
    pub(super) fn new(
        pool: &'static PgPool,
        storage: ConfigStorage,
        poller: Poller,
    ) -> Result<ConnectHandler> {
        Ok(ConnectHandler {
            pool,
            storage,
            poller,
            crab: OctocrabBuilder::new().build()?,
            client_id: settings().plugins.github.client_id.clone(),
            client_secret: read_secret("github/client_secret")?,
            app_name: settings().plugins.github.app_name.clone(),
            client: Client::default(),
        })
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/connect", post(Self::connect_project_handler))
            .route("/init", get(Self::init_handler))
            .route("/userConnections", post(Self::connect_user_handler))
            .route(
                "/userConnections",
                delete(Self::delete_user_connection_handler),
            )
            .layer(Extension(self))
    }

    #[tracing::instrument(skip(handler))]
    async fn init_handler(
        Extension(handler): Extension<ConnectHandler>,
    ) -> ApiResult<Json<InitResponse>> {
        Ok(Json(InitResponse {
            app_name: handler.app_name,
            client_id: handler.client_id,
        }))
    }

    #[tracing::instrument(
        skip(user, handler, request),
        fields(project_id=request.project_id, installation_id=request.installation_id)
    )]
    async fn connect_project_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<ConnectRequest>,
    ) -> ApiResult<Json<ConnectResponse>> {
        handler.connect_project(request, user).await
    }

    async fn connect_project(
        &self,
        request: ConnectRequest,
        user: User,
    ) -> ApiResult<Json<ConnectResponse>> {
        api::verify_project_access(self.pool, &user, &request.project_id).await?;
        self.verify_installation_access(&request).await?;

        tracing::debug!(
            "Connecting project {} to installation {}",
            request.project_id,
            request.installation_id
        );
        let config = Config {
            project_id: request.project_id,
            plugin_id: github::PLUGIN_KIND.id.to_string(),
            external_id: request.installation_id,
            settings: Settings::Github(GithubSettings {}),
        };
        self.storage.insert_or_update(&config).await?;

        // Trigger an initial poll in the background.
        let poller = self.poller.clone();
        tokio::spawn(async move { poller.poll_installation(config).await }.in_current_span());

        Ok(Json(ConnectResponse {}))
    }

    async fn verify_installation_access(&self, request: &ConnectRequest) -> ApiResult<()> {
        if request.code.is_empty() {
            return Err(bad_request("EMPTY_CODE", "Code is blank"));
        }

        let installations = self.fetch_installations(&request.code).await?;
        let installation_authorized = installations
            .into_iter()
            .any(|installation| installation.id.0.to_string() == request.installation_id);
        if !installation_authorized {
            return Err(forbidden(
                "UNAUTHORIZED",
                &format!(
                    "Not authorized to access installation {}",
                    request.installation_id
                ),
            ));
        }
        Ok(())
    }

    async fn fetch_installations(&self, code: &str) -> ApiResult<Vec<Installation>> {
        let token = self.generate_access_token(code).await?;
        let crab = self.crab.user_access_token(token.access_token.as_str())?;

        let installations = crab
            .current()
            .list_app_installations_accessible_to_user()
            .per_page(100)
            .send()
            .await?;
        if installations.total_count.unwrap_or_default() > u64::try_from(installations.items.len())?
        {
            tracing::warn!("Need to paginate installations");
        }
        Ok(installations.items)
    }

    #[tracing::instrument(skip(user, handler, request))]
    async fn connect_user_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<ConnectUserRequest>,
    ) -> ApiResult<Json<ConnectUserResponse>> {
        handler.connect_user(request, user).await
    }

    async fn connect_user(
        &self,
        request: ConnectUserRequest,
        user: User,
    ) -> ApiResult<Json<ConnectUserResponse>> {
        let octocrab::models::Author { url, id, .. } = self.fetch_user(&request).await?;

        tracing::debug!("Connecting user {} to github user {id} ({url})", user.email);
        self.update_user_connection(&user, Some(&id.to_string()))
            .await?;

        Ok(Json(ConnectUserResponse {}))
    }

    async fn fetch_user(
        &self,
        request: &ConnectUserRequest,
    ) -> ApiResult<octocrab::models::Author> {
        if request.code.is_empty() {
            return Err(bad_request("EMPTY_CODE", "Code is blank"));
        }

        let token = self.generate_access_token(&request.code).await?;
        let crab = self.crab.user_access_token(token.access_token.as_str())?;
        Ok(crab.current().user().await?)
    }

    #[tracing::instrument(skip(user, handler))]
    async fn delete_user_connection_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
    ) -> ApiResult<Json<()>> {
        handler.update_user_connection(&user, None).await?;
        Ok(Json(()))
    }

    async fn update_user_connection(
        &self,
        user: &User,
        github_user_id: Option<&str>,
    ) -> ApiResult<()> {
        let res = sqlx::query(
            "
            UPDATE users
            SET github_user_id = $2
            WHERE email = $1",
        )
        .bind(&user.email)
        .bind(github_user_id)
        .execute(self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(not_found("NOT_FOUND", "User does not exist."));
        }
        Ok(())
    }

    /// Exchange the Github OAuth code for a user access token.
    ///
    /// https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#generating-a-user-access-token-when-a-user-installs-your-app
    async fn generate_access_token(&self, code: &str) -> ApiResult<OAuth> {
        let res = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("ACCEPT", "application/json")
            .header("Content-Type", "application/json")
            .json(&GithubOAuthRequest {
                client_id: self.client_id.as_str(),
                client_secret: self.client_secret.data.as_str(),
                code,
            })
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
                return Err(bad_request(
                    "GITHUB_AUTH_REJECTED",
                    &format!("Login rejected: '{}' - '{}'", e.error, e.error_description),
                ));
            }
        };

        Ok(oauth)
    }
}
