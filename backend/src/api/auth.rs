use crate::api::{internal_error, ApiResult};
use axum::{routing::post, Extension, Router};
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
    if let Err(e) = sqlx::query(
        "
        INSERT INTO users (email, name, picture, invited)
        VALUES ($1, $2, $3, false)
        ON CONFLICT (email)
        DO UPDATE SET name = EXCLUDED.name, picture = EXCLUDED.picture, login_time = NOW();",
    )
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.picture)
    .execute(pool)
    .await
    {
        return Err(internal_error(&format!(
            "Failed to upsert user on login: {e}"
        )));
    }
    Ok(())
}
