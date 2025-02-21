use crate::api::{ApiResult, google::User};
use crate::notifiers::UserNotificationConfig;
use axum::{Extension, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

pub(crate) fn router() -> Router {
    Router::new().route("/", get(get_profile_handler))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Profile {
    notification_configs: Vec<UserNotificationConfig>,
}

#[tracing::instrument(skip(user, pool))]
async fn get_profile_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Profile>> {
    let notification_configs: Vec<UserNotificationConfig> = sqlx::query_as(
        "
        SELECT email, notifier, enabled, settings
        FROM user_notification_configs
        WHERE email = $1",
    )
    .bind(user.email)
    .fetch_all(pool)
    .await?;

    Ok(Json(Profile {
        notification_configs,
    }))
}
