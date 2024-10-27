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
pub(crate) mod projects;
pub(crate) mod users;
pub(crate) mod ws;

pub(crate) type ApiResult<T> = Result<T, ErrorResponse>;

pub(crate) fn api_router() -> Router {
    Router::new()
        .nest("/projects", projects::projects_router())
        .nest("/auth", auth::auth_router())
        .nest("/ws", ws::ws_router())
        .nest("/users", users::users_router())
        .nest("/dev", dev::dev_router())
}

pub(crate) async fn verify_access(
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
        reason,
        msg: msg.to_string(),
    }
}

pub(crate) struct ErrorResponse {
    status: StatusCode,
    // Terse, stable, machine readable error reason.
    // e.g. NO_STOCK
    reason: &'static str,
    // Debug message for developers. Not intended for end users.
    msg: String,
}

impl ErrorResponse {
    fn as_err(&self) -> Error {
        anyhow!("{} ({}-{})", self.msg, self.status, self.reason)
    }
}

#[derive(serde::Serialize)]
struct ErrorResponseBody {
    // StatusCode in number form. e.g. 400, 500
    status: u16,
    // Terse, stable, machine readable error reason.
    // e.g. NO_STOCK
    reason: &'static str,
    // Debug message for developers. Not intended for end users.
    msg: String,
}

/// Converts from ErrorResponse to Response.
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let msg = if dev_mode() {
            self.msg
        } else {
            // Redact the the error message outside of dev.
            "See server logs for details.".to_string()
        };
        let body = axum::Json(ErrorResponseBody {
            status: self.status.as_u16(),
            reason: self.reason,
            msg,
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

fn dev_mode() -> bool {
    // TODO: Decide on this based on an environment variable or the build.
    true
}
