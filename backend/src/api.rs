use anyhow::{anyhow, Error, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use google::User;
use model::{ProjectId, ProjectPermission};
use sqlx::postgres::PgPool;

pub(crate) mod auth;
pub(crate) mod collab;
pub(crate) mod dev;
pub(crate) mod google;
pub(crate) mod model;
pub(crate) mod poll;
pub(crate) mod projects;
pub(crate) mod users;
pub(crate) mod ws;
pub(crate) mod yproxy;

pub(crate) type ApiResult<T> = Result<T, ErrorResponse>;

pub(crate) fn router() -> Router {
    Router::new()
        .nest("/projects", projects::router())
        .nest("/auth", auth::router())
        .nest("/ws", ws::router())
        .nest("/users", users::router())
        .nest("/dev", dev::router())
        .nest("/poll", poll::router())
}

/// Verify that the user is invited and allowed to access Koso.
/// Typically this permission is granted via another invited user sharing
/// one of their projects with another user that has already logged in.
pub(crate) async fn verify_invited(pool: &PgPool, user: &User) -> Result<(), ErrorResponse> {
    match sqlx::query_as(
        "
        SELECT invited
        FROM users
        WHERE email = $1;
        ",
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await
    {
        Ok(Some((true,))) => Ok(()),
        Ok(None | Some((false,))) => Err(not_invited_error(&format!(
            "User {} is not invited",
            user.email
        ))),
        Err(e) => Err(internal_error(&format!(
            "Failed to check user permission: {e}"
        ))),
    }
}

/// Verify that the user has access to the given project.
pub(crate) async fn verify_project_access(
    pool: &PgPool,
    user: User,
    project_id: &ProjectId,
) -> Result<(), ErrorResponse> {
    if project_id.is_empty() {
        return Err(bad_request_error(
            "EMPTY_PROJECT_ID",
            "Project ID must not be empty",
        ));
    }

    let mut txn = match pool.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            return Err(internal_error(&format!(
                "Failed to check user permission: {e}"
            )))
        }
    };

    let permission: Option<ProjectPermission> = match sqlx::query_as(
        "
        SELECT project_id, email
        FROM project_permissions
        WHERE project_id = $1
          AND email = $2;
        ",
    )
    .bind(project_id)
    .bind(&user.email)
    .fetch_optional(&mut *txn)
    .await
    {
        Ok(permission) => permission,
        Err(e) => {
            return Err(internal_error(&format!(
                "Failed to check user permission: {e}"
            )))
        }
    };

    match permission {
        Some(_) => Ok(()),
        None => Err(unauthorized_error(&format!(
            "User {} is not authorized to access {}",
            user.email, project_id
        ))),
    }
}

pub(crate) async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

pub(crate) fn internal_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg)
}

pub(crate) fn unauthenticated_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::UNAUTHORIZED, "UNAUTHENTICATED", msg)
}

pub(crate) fn unauthorized_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::FORBIDDEN, "UNAUTHORIZED", msg)
}

pub(crate) fn not_invited_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::FORBIDDEN, "NOT_INVITED", msg)
}

pub(crate) fn bad_request_error(reason: &'static str, msg: &str) -> ErrorResponse {
    error_response(StatusCode::BAD_REQUEST, reason, msg)
}

pub(crate) fn error_response(status: StatusCode, reason: &'static str, msg: &str) -> ErrorResponse {
    match status {
        StatusCode::INTERNAL_SERVER_ERROR => {
            tracing::error!("Failed: {} ({}): {}", status, reason, msg)
        }
        _ => tracing::warn!("Failed: {} ({}): {}", status, reason, msg),
    }
    ErrorResponse {
        status,
        details: vec![ErrorDetail {
            reason,
            msg: msg.to_string(),
        }],
    }
}

pub(crate) struct ErrorResponse {
    status: StatusCode,
    details: Vec<ErrorDetail>,
}

#[derive(serde::Serialize, Debug)]
pub(crate) struct ErrorDetail {
    // Terse, stable, machine readable error reason.
    // e.g. NO_STOCK
    reason: &'static str,
    // Debug message for developers. Not intended for end users.
    msg: String,
    // Need more details about an error? Consider adding
    // a map of key/values for use in the client.
}

impl ErrorResponse {
    fn as_err(&self) -> Error {
        if self.details.is_empty() {
            anyhow!("({}) <MISSING_ERROR_DETAILS>", self.status)
        } else {
            anyhow!("({}) {:?}", self.status, self.details)
        }
    }
}

#[derive(serde::Serialize)]
struct ErrorResponseBody {
    // StatusCode in number form. e.g. 400, 500
    status: u16,
    details: Vec<ErrorDetail>,
}

/// Converts from ErrorResponse to Response.
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let body = axum::Json(ErrorResponseBody {
            status: self.status.as_u16(),
            details: self.details,
        });

        (self.status, body).into_response()
    }
}

/// Converts from boxed Error to ErrorResponse and logs the error.
impl<E> From<E> for ErrorResponse
where
    E: Into<Box<dyn std::error::Error>>,
{
    fn from(err: E) -> Self {
        internal_error(&format!("{:?}", err.into()))
    }
}
