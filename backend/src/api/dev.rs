use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{User, bad_request_error, google};
use crate::{api::ApiResult, flags::is_dev};
use anyhow::Context as _;
use axum::{Extension, Router, routing::post};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

fn integ_test_user_suffix() -> String {
    format!("-test{}", google::TEST_USER_SUFFIX)
}

pub(super) fn router() -> Router {
    if is_dev() {
        tracing::info!("Enable dev mode. Something is WRONG if you see this in production.");
        return Router::new()
            .route("/cleanup_test_data", post(cleanup_test_data_handler))
            .route("/invite_test_user", post(invite_test_user_handler));
    }

    Router::new()
}

/// Endpoint used by playwright tests to invite test users.
/// This avoids the need to bootstrap some intial user with invite permission.
#[tracing::instrument(skip(pool))]
async fn invite_test_user_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<User>,
) -> ApiResult<()> {
    if !user.email.ends_with(google::TEST_USER_SUFFIX) {
        return Err(bad_request_error(
            "NON_TEST_USER",
            &format!(
                "User {} is not a test user. Expected suffix: {}",
                user.email,
                google::TEST_USER_SUFFIX
            ),
        ));
    }
    sqlx::query(
        "
        UPDATE users
        SET invited=TRUE
        WHERE email = $1 and NOT invited",
    )
    .bind(user.email)
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn cleanup_test_data_handler(Extension(pool): Extension<&'static PgPool>) -> ApiResult<()> {
    let test_user_emails: Vec<(String, DateTime<Utc>)> =
        sqlx::query_as("SELECT email, creation_time FROM users where email LIKE '%'||$1;")
            .bind(integ_test_user_suffix())
            .fetch_all(pool)
            .await?;
    let test_user_emails: Vec<String> = test_user_emails
        .into_iter()
        .filter(|(email, creation_time)| {
            if !email.ends_with(&integ_test_user_suffix()) {
                return false;
            }
            let Ok(d) = SystemTime::now().duration_since(
                UNIX_EPOCH
                    + Duration::from_millis(creation_time.timestamp_millis().try_into().unwrap()),
            ) else {
                return true;
            };

            // Enable post hoc debugging and avoid interfering with other running tests
            // by only deleting users after some time has passed.
            d > Duration::from_secs(3 * 60 * 60)
        })
        .map(|(email, _)| email)
        .collect::<Vec<String>>();
    tracing::debug!("Deleting test users{test_user_emails:?}");

    // Delete any projects with ONLY deletable test users.
    sqlx::query(
        "
        DELETE FROM projects
        WHERE project_id IN (
            SELECT project_id FROM project_permissions
            WHERE email IN (SELECT * FROM unnest($1))
        ) AND project_id NOT IN (
            SELECT project_id FROM project_permissions
            WHERE email NOT IN (SELECT * FROM unnest($1))
        );",
    )
    .bind(&test_user_emails)
    .execute(pool)
    .await
    .context("Failed to delete test projects")?;
    // Delete any orphaned yupdates.
    sqlx::query(
        "
        DELETE FROM yupdates
        WHERE project_id NOT IN (
            SELECT project_id FROM projects
        );",
    )
    .execute(pool)
    .await
    .context("Failed to delete test yupdates")?;
    // Delete any orphaned plugin configs.
    sqlx::query(
        "
        DELETE FROM plugin_configs
        WHERE project_id NOT IN (
            SELECT project_id FROM projects
        );",
    )
    .execute(pool)
    .await
    .context("Failed to delete test yupdates")?;
    // Delete any orphaned project permissions.
    sqlx::query(
        "
        DELETE FROM project_permissions
        WHERE project_id NOT IN (
            SELECT project_id FROM projects
        );",
    )
    .execute(pool)
    .await
    .context("Failed to delete test project_permissions")?;
    // Delete any test users that are no longer part of any project.
    sqlx::query(
        "
        DELETE FROM users
        WHERE email IN (SELECT * FROM unnest($1))
        AND email NOT IN(SELECT email FROM project_permissions);",
    )
    .bind(&test_user_emails)
    .execute(pool)
    .await
    .context("Failed to delete test users")?;
    // Delete any orphaned notification configs.
    sqlx::query(
        "
        DELETE FROM user_notification_configs
        WHERE email NOT IN (
            SELECT email FROM users
        );",
    )
    .execute(pool)
    .await
    .context("Failed to delete test users")?;

    Ok(())
}
