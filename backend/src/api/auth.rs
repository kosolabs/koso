use crate::api::ApiResult;
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
    sqlx::query(
        "
        INSERT INTO users (email, name, picture)
        VALUES ($1, $2, $3)
        ON CONFLICT (email)
        DO UPDATE SET name = EXCLUDED.name, picture = EXCLUDED.picture, login_time = NOW();",
    )
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.picture)
    .execute(pool)
    .await
    .context("Failed to upsert user on login")?;
    Ok(())
}
