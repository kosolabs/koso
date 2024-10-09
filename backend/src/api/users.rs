use crate::api::{model::User, ApiResult};
use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Extension, Json, Router,
};
use sqlx::postgres::PgPool;

pub(super) fn users_router() -> Router {
    Router::new().route("/", get(list_users_handler))
}

#[tracing::instrument(skip(pool))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<(HeaderMap, Json<Vec<User>>)> {
    let mut users: Vec<User> = sqlx::query_as("SELECT email, name, picture FROM users;")
        .fetch_all(pool)
        .await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600, stale-while-revalidate=300"),
    );

    Ok((headers, Json(users)))
}
