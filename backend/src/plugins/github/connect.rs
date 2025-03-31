use crate::{
    api::{self, ApiResult, google::User, unauthorized_error},
    plugins::{
        config::{Config, ConfigStorage, GithubSettings, Settings},
        github::{self, Poller, auth::Auth},
    },
    settings::settings,
};
use anyhow::Result;
use axum::{
    Extension, Json, Router,
    routing::{get, post},
};
use octocrab::{Octocrab, OctocrabBuilder, models::Installation};
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitResponse {
    app_name: String,
    client_id: String,
}

#[derive(Clone)]
pub(super) struct ConnectHandler {
    auth: Auth,
    pool: &'static PgPool,
    storage: ConfigStorage,
    poller: Poller,
    crab: Octocrab,
    client_id: String,
    app_name: String,
}

impl ConnectHandler {
    pub(super) fn new(
        auth: Auth,
        pool: &'static PgPool,
        storage: ConfigStorage,
        poller: Poller,
    ) -> Result<ConnectHandler> {
        Ok(ConnectHandler {
            auth,
            pool,
            storage,
            poller,
            crab: OctocrabBuilder::new().build()?,
            client_id: settings().plugins.github.client_id.clone(),
            app_name: settings().plugins.github.app_name.clone(),
        })
    }

    pub(super) fn router(self) -> Router {
        Router::new()
            .route("/connect", post(Self::connect_project_handler))
            .route("/init", get(Self::init_handler))
            .layer((Extension(self),))
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
}
