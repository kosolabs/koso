use crate::api::{ApiResult, google, model::User, not_found_error, verify_premium};
use axum::{
    Extension, Json, Router,
    extract::Path,
    routing::{get, patch},
};
use sqlx::postgres::PgPool;

use super::{bad_request_error, model::UpdateUser, unauthorized_error};

pub(super) fn router() -> Router {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/{email}", get(get_user_handler))
        .route("/{email}", patch(update_user_handler))
}

#[tracing::instrument(skip(pool, user))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
) -> ApiResult<Json<Vec<User>>> {
    verify_premium(pool, &user).await?;

    let mut users: Vec<User> =
        sqlx::query_as("SELECT email, name, picture, premium FROM users WHERE premium;")
            .fetch_all(pool)
            .await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    Ok(Json(users))
}

#[tracing::instrument(skip(pool, user))]
async fn get_user_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
    Path(email): Path<String>,
) -> ApiResult<Json<User>> {
    authorize_user(&user, &email)?;

    let user: Option<User> =
        sqlx::query_as("SELECT email, name, picture, premium FROM users WHERE email=$1;")
            .bind(&email)
            .fetch_optional(pool)
            .await?;
    match user {
        Some(user) => Ok(Json(user)),
        None => Err(not_found_error(
            "NOT_FOUND",
            &format!("User {email} not found"),
        )),
    }
}

#[tracing::instrument(skip(pool, user))]
async fn update_user_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
    Path(email): Path<String>,
    Json(update_user): Json<UpdateUser>,
) -> ApiResult<()> {
    authorize_user(&user, &email)?;

    let res = sqlx::query(
        "
        UPDATE users
        SET github_login = $2
        WHERE email = $1",
    )
    .bind(&user.email)
    .bind(&update_user.github_login)
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(not_found_error("NOT_FOUND", "User does not exist."));
    }
    Ok(())
}

fn authorize_user(user: &google::User, email: &str) -> ApiResult<()> {
    if email.is_empty() {
        return Err(bad_request_error("EMPTY_EMAIL", "Email must not be empty"));
    }
    if email != user.email {
        return Err(unauthorized_error(&format!(
            "User {} is not authorized to access {}",
            user.email, email
        )));
    }
    Ok(())
}
