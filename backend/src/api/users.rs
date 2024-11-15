use crate::api::{google, model::User, verify_invited, ApiResult};
use axum::{routing::get, Extension, Json, Router};
use sqlx::postgres::PgPool;

pub(super) fn router() -> Router {
    Router::new().route("/", get(list_users_handler))
}

#[tracing::instrument(skip(pool))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
) -> ApiResult<Json<Vec<User>>> {
    verify_invited(pool, &user).await?;

    let mut users: Vec<User> = sqlx::query_as("SELECT email, name, picture FROM users;")
        .fetch_all(pool)
        .await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    Ok(Json(users))
}
