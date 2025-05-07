use crate::api::{ApiResult, google, model::User};
use axum::{Extension, Json, Router, routing::get};
use sqlx::postgres::PgPool;

pub(super) fn router() -> Router {
    Router::new().route("/", get(list_users_handler))
}

#[tracing::instrument(skip(pool))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
) -> ApiResult<Json<Vec<User>>> {
    let mut users: Vec<User> = sqlx::query_as("SELECT email, name, picture FROM users;")
        .fetch_all(pool)
        .await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    Ok(Json(users))
}
