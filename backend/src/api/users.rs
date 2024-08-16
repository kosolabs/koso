use crate::api::{model::User, ApiResult};
use axum::{routing::get, Extension, Json, Router};
use sqlx::postgres::PgPool;

pub(super) fn users_router() -> Router {
    Router::new().route("/", get(list_users_handler))
}

#[tracing::instrument(skip(pool))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<User>>> {
    Ok(Json(
        sqlx::query_as("SELECT email, name, picture FROM users;")
            .fetch_all(pool)
            .await?,
    ))
}
