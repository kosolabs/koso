use crate::api::{internal_error, ApiResult};
use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use crate::google::User;

pub fn auth_router() -> Router {
    Router::new().route("/login", post(login_handler))
}
#[tracing::instrument(skip(user, pool))]
async fn login_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<()> {
    if let Err(e) = sqlx::query(
        "
        INSERT INTO users (email, name, picture)
        VALUES ($1, $2, $3)
        ON CONFLICT (email)
        DO UPDATE SET name = EXCLUDED.name, picture = EXCLUDED.picture;",
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
