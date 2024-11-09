use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::{internal_error, ApiResult};
use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use super::google;

pub(super) fn router() -> Router {
    let enable_dev = std::env::var("TESTONLY_ENABLE_DEV").map_or(false, |v| v == "true");
    if enable_dev {
        tracing::info!("Enable dev mode. Something is WRONG if you see this in production.");
        return Router::new().route("/cleanup_test_data", post(cleanup_test_data_handler));
    }

    Router::new()
}

#[tracing::instrument(skip(pool))]
async fn cleanup_test_data_handler(Extension(pool): Extension<&'static PgPool>) -> ApiResult<()> {
    let test_user_emails: Vec<(String,)> =
        sqlx::query_as("SELECT email FROM users where email LIKE '%'||$1;")
            .bind(google::TEST_USER_SUFFIX)
            .fetch_all(pool)
            .await?;
    let test_user_emails: Vec<String> = test_user_emails
        .into_iter()
        .map(|(email,)| email)
        .filter(|email| email.ends_with(google::TEST_USER_SUFFIX))
        .filter(|email| {
            let parts = email.split("-").collect::<Vec<&str>>();
            let Some(Ok(create_time)) = parts.get(1).map(|t| t.parse::<u64>()) else {
                return true;
            };
            let Ok(d) =
                SystemTime::now().duration_since(UNIX_EPOCH + Duration::from_millis(create_time))
            else {
                return true;
            };
            // Enable post hoc debugging and avoid interfering with other running tests
            // by only deleting users after some time has passed.
            d > Duration::from_secs(3 * 60 * 60)
        })
        .collect::<Vec<String>>();
    tracing::debug!("Deleting test users{test_user_emails:?}");

    if let Err(e) = sqlx::query(
        "
        DELETE FROM projects
        WHERE project_id IN (
            SELECT project_id FROM project_permissions
            WHERE email IN (SELECT * FROM unnest($1))
        );",
    )
    .bind(&test_user_emails)
    .execute(pool)
    .await
    {
        return Err(internal_error(&format!(
            "Failed to delete test projects: {e}"
        )));
    }
    if let Err(e) = sqlx::query(
        "
        DELETE FROM project_permissions
        WHERE email IN (SELECT * FROM unnest($1));",
    )
    .bind(&test_user_emails)
    .execute(pool)
    .await
    {
        return Err(internal_error(&format!(
            "Failed to delete test project_permissions: {e}"
        )));
    }
    if let Err(e) = sqlx::query(
        "
        DELETE FROM users
        WHERE email IN (SELECT * FROM unnest($1));",
    )
    .bind(&test_user_emails)
    .execute(pool)
    .await
    {
        return Err(internal_error(&format!("Failed to delete test users: {e}")));
    }

    Ok(())
}
