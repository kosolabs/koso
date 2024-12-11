use crate::{
    api::{self, google::User, unauthorized_error, ApiResult},
    plugins::{
        config::{Config, ConfigStorage},
        github::{self, auth::Auth, GithubSpecificConfig, Poller},
    },
};
use axum::{routing::post, Extension, Json, Router};
use octocrab::{models::Installation, OctocrabBuilder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::Instrument;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConnectRequest {
    project_id: String,
    installation_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectResponse {}

#[derive(Clone)]
pub(super) struct ConnectHandler {
    auth: Auth,
    pool: &'static PgPool,
    storage: ConfigStorage,
    poller: Poller,
}

impl ConnectHandler {
    pub(super) fn new(
        auth: Auth,
        pool: &'static PgPool,
        storage: ConfigStorage,
        poller: Poller,
    ) -> ConnectHandler {
        ConnectHandler {
            auth,
            pool,
            storage,
            poller,
        }
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/connect", post(Self::connect_project_handler))
            .layer((Extension(self),))
    }

    #[tracing::instrument(skip(user, handler))]
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
        api::verify_project_access(self.pool, user.clone(), &request.project_id).await?;
        self.verify_installation_access(&user, &request.installation_id)
            .await?;

        tracing::info!(
            "Connecting project {} to installation {}",
            request.project_id,
            request.installation_id
        );
        let config = Config {
            plugin_id: github::PLUGIN_KIND.id.to_string(),
            external_id: request.installation_id,
            config: GithubSpecificConfig {
                project_id: request.project_id,
            },
        };
        self.storage.insert_or_update(&config).await?;

        // Trigger an initial poll in the background.
        let poller = self.poller.clone();
        tokio::spawn(async move { poller.poll_installation(config).await }.in_current_span());

        Ok(Json(ConnectResponse {}))
    }

    async fn verify_installation_access(
        &self,
        user: &User,
        installation_id: &str,
    ) -> ApiResult<()> {
        let installations = self.fetch_installations(user).await?;
        let installation_authorized = installations
            .into_iter()
            .any(|installation| installation.id.0.to_string() == installation_id);
        if !installation_authorized {
            return Err(unauthorized_error(&format!(
                "Not authorized to access installation {}",
                installation_id
            )));
        }
        Ok(())
    }

    async fn fetch_installations(&self, user: &User) -> ApiResult<Vec<Installation>> {
        let token = self.auth.user_access_token(user).await?;
        // TODO: It'd be nice to reuse the underlying connection with the specific access token
        // rather than connecting from scratch on every call.
        let crab = OctocrabBuilder::new()
            .user_access_token(token.access_token.as_str())
            .build()?;

        let installations = crab
            .current()
            .list_app_installations_accessible_to_user()
            .per_page(100)
            .send()
            .await?;
        if installations.total_count.unwrap_or_default() > installations.items.len().try_into()? {
            tracing::warn!("Need to paginate installations");
        }
        Ok(installations.items)
    }
}
