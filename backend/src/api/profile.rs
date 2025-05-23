use crate::api::{ApiResult, google::User};
use crate::notifiers::UserNotificationConfig;
use anyhow::{Context, Result};
use axum::{Extension, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgPool;
use tokio::try_join;

use super::not_found_error;

pub(crate) fn router() -> Router {
    Router::new().route("/", get(get_profile_handler))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Profile {
    notification_configs: Vec<UserNotificationConfig>,
    plugin_mappings: PluginMappings,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
struct PluginMappings {
    github_login: Option<String>,
}

#[tracing::instrument(skip(user, pool))]
async fn get_profile_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Profile>> {
    let (notification_configs, plugin_mappings) = try_join!(
        fetch_notification_configs(&user.email, pool),
        fetch_plugin_mappings(&user.email, pool)
    )?;
    let Some(plugin_mappings) = plugin_mappings else {
        return Err(not_found_error("NOT_FOUND", "User not found"));
    };

    Ok(Json(Profile {
        notification_configs,
        plugin_mappings,
    }))
}

async fn fetch_notification_configs(
    email: &str,
    pool: &PgPool,
) -> Result<Vec<UserNotificationConfig>> {
    sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1",
    )
    .bind(email)
    .fetch_all(pool)
    .await
    .context("Failed to query notification configs")
}

async fn fetch_plugin_mappings(email: &str, pool: &PgPool) -> Result<Option<PluginMappings>> {
    sqlx::query_as(
        "
        SELECT github_login
        FROM users
        WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .context("Failed to query user plugin mappings")
}
