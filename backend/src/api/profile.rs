use crate::api::billing::fetch_owned_subscription;
use crate::api::billing::model::{Subscription, SubscriptionStatus};
use crate::api::google::User;
use crate::notifiers::{UserNotificationConfig, fetch_notification_configs};
use anyhow::{Context, Result};
use axum::{Extension, Json, Router, routing::get};
use axum_anyhow::{ApiResult, OptionExt};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgPool;
use sqlx::types::chrono;
use tokio::try_join;

pub(crate) fn router() -> Router {
    Router::new().route("/", get(get_profile_handler))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Profile {
    notification_configs: Vec<UserNotificationConfig>,
    plugin_connections: PluginConnections,
    subscriptions: Subscriptions,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
struct PluginConnections {
    github_user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Subscriptions {
    owned_subscription: Option<Subscription>,
    end_time: Option<DateTime<Utc>>,
    status: SubscriptionStatus,
}

#[tracing::instrument(skip(user, pool))]
async fn get_profile_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Profile>> {
    let (notification_configs, plugin_connections, owned_subscription, subscription_end_time) = try_join!(
        fetch_notification_configs(&user.email, pool),
        fetch_plugin_connections(&user.email, pool),
        fetch_owned_subscription(&user.email, pool),
        fetch_subscription_end_time(&user.email, pool),
    )?;
    let plugin_connections = plugin_connections.context_not_found("NOT_FOUND", "User not found")?;

    Ok(Json(Profile {
        notification_configs,
        plugin_connections,
        subscriptions: Subscriptions {
            owned_subscription,
            end_time: subscription_end_time,
            status: match subscription_end_time {
                Some(end_time) => {
                    if end_time.timestamp() <= chrono::Utc::now().timestamp() {
                        SubscriptionStatus::Expired
                    } else {
                        SubscriptionStatus::Active
                    }
                }
                None => SubscriptionStatus::None,
            },
        },
    }))
}

async fn fetch_plugin_connections(email: &str, pool: &PgPool) -> Result<Option<PluginConnections>> {
    sqlx::query_as(
        "
        SELECT github_user_id
        FROM users
        WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .context("Failed to query user plugin connections")
}

async fn fetch_subscription_end_time(email: &str, pool: &PgPool) -> Result<Option<DateTime<Utc>>> {
    let (end_time,): (Option<DateTime<Utc>>,) = sqlx::query_as(
        "
        SELECT subscription_end_time
        FROM users
        WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .context("Failed to query user subscription")?
    .context("User not found")?;
    Ok(end_time)
}
