use crate::{
    api::{self, google::User, unauthorized_error, ApiResult},
    plugins::github::{auth::Auth, GithubSpecificConfig},
};
use axum::{routing::post, Extension, Json, Router};
use octocrab::{models::Installation, OctocrabBuilder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectRequest {
    project_id: String,
    installation_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectResponse {}

pub(super) fn router() -> Router {
    Router::new().route("/connect", post(connect_project_handler))
}

async fn connect_project_handler(
    Extension(user): Extension<User>,
    Extension(auth): Extension<Auth>,
    Extension(pool): Extension<&'static PgPool>,
    Json(request): Json<ConnectRequest>,
) -> ApiResult<Json<ConnectResponse>> {
    verify_installation_access(&auth, &user, &request.installation_id).await?;
    api::verify_project_access(pool, user, &request.project_id).await?;

    tracing::info!(
        "Connecting project {} to installation {}",
        request.project_id,
        request.installation_id
    );

    let config = GithubSpecificConfig {
        project_id: request.project_id,
    };
    sqlx::query(
        "
        INSERT INTO plugin_configs (plugin_id, external_id, config)
        VALUES ($1, $2, $3)
        ON CONFLICT (plugin_id, external_id)
        DO UPDATE SET config = EXCLUDED.config;",
    )
    .bind("github")
    .bind(request.installation_id)
    .bind(sqlx::types::Json(&config))
    .execute(pool)
    .await?;

    Ok(Json(ConnectResponse {}))
}

async fn verify_installation_access(
    auth: &Auth,
    user: &User,
    installation_id: &str,
) -> ApiResult<()> {
    let installations = fetch_installations(auth, user).await?;
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

async fn fetch_installations(auth: &Auth, user: &User) -> ApiResult<Vec<Installation>> {
    let token = auth.user_github_token(user).await?;
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
