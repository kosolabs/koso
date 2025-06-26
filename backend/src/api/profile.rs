use crate::api::{ApiResult, google::User};
use crate::notifiers::UserNotificationConfig;
use anyhow::{Context, Result};
use axum::{Extension, Json, Router, routing::get};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgPool;
use sqlx::types::chrono;
use tokio::try_join;

use super::not_found_error;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Subscription {
    status: SubscriptionStatus,
    seats: i32,
    end_time: DateTime<Utc>,
    member_emails: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
enum SubscriptionStatus {
    None,
    Active,
    Expired,
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
    let Some(plugin_connections) = plugin_connections else {
        return Err(not_found_error("NOT_FOUND", "User not found"));
    };

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

async fn fetch_owned_subscription(email: &str, pool: &PgPool) -> Result<Option<Subscription>> {
    Ok(sqlx::query_as(
        "
        SELECT seats, end_time, member_emails
        FROM subscriptions
        WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .context("Failed to query user subscriptions")?
    .map(
        |(seats, end_time, mut member_emails): (i32, DateTime<Utc>, Vec<String>)| {
            // Sort for consistent ordering in the UI.
            member_emails.sort();
            Subscription {
                seats,
                end_time,
                member_emails,
                status: if end_time.timestamp() <= chrono::Utc::now().timestamp() {
                    SubscriptionStatus::Expired
                } else {
                    SubscriptionStatus::Active
                },
            }
        },
    ))
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
