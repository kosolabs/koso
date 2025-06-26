use crate::api::{ApiResult, billing::update_user_subscription_end_time};
use anyhow::Context as _;
use axum::{Extension, Router, routing::post};
use sqlx::PgPool;

use crate::api::google::User;

pub(super) fn router() -> Router {
    Router::new().route("/login", post(login_handler))
}
#[tracing::instrument(skip(user, pool))]
async fn login_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<()> {
    // We optimistically assume the user has already logged in.
    let res = sqlx::query(
        "
        UPDATE users
        SET name = $2, picture = $3, login_time = NOW()
        WHERE email=$1;",
    )
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.picture)
    .execute(pool)
    .await
    .context("Failed to update user on login")?;

    // On the first login, run the insert.
    // No need to run this in a transaction. If it fails because
    // another login has already happened, that's fine, just continue.
    if res.rows_affected() == 0 {
        sqlx::query(
            "
            INSERT INTO users (email, name, picture, login_time)
            VALUES ($1, $2, $3, NOW());",
        )
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.picture)
        .execute(pool)
        .await
        .context("Failed to upsert user on login")?;

        // The new user might already be a member of a subscription.
        // Handle this by updating their subscription end time.
        // No need for this to be in the same transaction as the insert.
        // If it races with another login or subscription update,
        // the users subscription end time will end up in the right place.
        update_user_subscription_end_time(&user.email, pool).await?;
    }

    Ok(())
}
