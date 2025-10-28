use crate::api::{google, model::User, verify_premium};
use axum::{Extension, Json, Router, extract::Path, routing::get};
use axum_anyhow::{ApiResult, OptionExt, bad_request, forbidden};
use sqlx::postgres::PgPool;

pub(super) fn router() -> Router {
    Router::new()
        .route("/", get(list_users_handler))
        .route("/{email}", get(get_user_handler))
}

#[tracing::instrument(skip(pool, user))]
async fn list_users_handler(
    Extension(pool): Extension<&'static PgPool>,
    Extension(user): Extension<google::User>,
) -> ApiResult<Json<Vec<User>>> {
    verify_premium(pool, &user).await?;

    let mut users: Vec<User> = sqlx::query_as(
        "
        SELECT email, name, picture, premium
        FROM (
            SELECT email, name, picture, (subscription_end_time IS NOT NULL AND subscription_end_time > now()) AS premium
            FROM users
        ) WHERE premium;",
    )
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
    verify_user_access(&user, &email)?;

    let user: User = sqlx::query_as(
        "
        SELECT email, name, picture, (subscription_end_time IS NOT NULL AND subscription_end_time > now()) AS premium
        FROM users
        WHERE email=$1;",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await?
    .context_not_found("NOT_FOUND",
            &format!("User {email} not found"))?;
    Ok(Json(user))
}

fn verify_user_access(user: &google::User, email: &str) -> ApiResult<()> {
    if email.is_empty() {
        return Err(bad_request("EMPTY_EMAIL", "Email must not be empty"));
    }
    if email != user.email {
        return Err(forbidden(
            "UNAUTHORIZED",
            &format!("User {} is not authorized to access {}", user.email, email),
        ));
    }
    Ok(())
}
