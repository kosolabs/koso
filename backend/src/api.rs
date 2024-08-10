use anyhow::{anyhow, Error, Result};
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use google::User;
use model::{ProjectId, ProjectPermission};
use sqlx::postgres::PgPool;

pub mod auth;
pub mod collab;
pub mod google;
pub mod model;
pub mod notify;
pub mod projects;
pub mod ws;

pub type ApiResult<T> = Result<T, ErrorResponse>;

pub fn api_router() -> Router {
    Router::new()
        .nest("/projects", projects::projects_router())
        .nest("/auth", auth::auth_router())
        .nest("/ws", ws::ws_router())
}

pub async fn verify_access(
    pool: &PgPool,
    user: User,
    project_id: &ProjectId,
) -> Result<(), ErrorResponse> {
    if project_id.is_empty() {
        return Err(bad_request_error("Project ID must not be empty"));
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

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

pub fn internal_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

pub fn unauthorized_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::UNAUTHORIZED, msg)
}

pub fn bad_request_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::BAD_REQUEST, msg)
}

pub fn error_response(code: StatusCode, msg: &str) -> ErrorResponse {
    tracing::error!("Failed: {}: {}", code, msg);
    ErrorResponse {
        code,
        msg: msg.to_string(),
    }
}

pub struct ErrorResponse {
    code: StatusCode,
    msg: String,
}

impl ErrorResponse {
    fn as_err(&self) -> Error {
        anyhow!("{} ({})", self.msg, self.code)
    }
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
        Response::builder()
            .status(self.code)
            .body(Body::from(format!("{}: {}", self.code, msg)))
            .unwrap()
    }
}

/// Converts from boxed Error to ErrorResponse and logs the error.
impl<E> From<E> for ErrorResponse
where
    E: Into<Box<dyn std::error::Error>>,
{
    fn from(err: E) -> Self {
        let err = err.into();
        let code = StatusCode::INTERNAL_SERVER_ERROR;
        let msg = format!("{:?}", err);
        tracing::error!("Failed: {}: {}", code, msg);
        ErrorResponse { code, msg }
    }
}

fn dev_mode() -> bool {
    // TODO: Decide on this based on an environment variable or the build.
    true
}
