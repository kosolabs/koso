use anyhow::{Context, Error, Result, anyhow};
use axum::{
    Router,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
};
use axum_extra::headers;
use google::User;
use model::{ProjectId, ProjectPermission};
use rmcp::model::ErrorCode;
use sqlx::postgres::PgPool;
use std::backtrace::{Backtrace, BacktraceStatus};

use crate::notifiers;

pub(crate) mod anthropic;
pub(crate) mod auth;
pub(crate) mod billing;
pub(crate) mod collab;
pub(crate) mod dev;
pub(crate) mod gemini;
pub(crate) mod google;
pub(crate) mod model;
pub(crate) mod profile;
pub(crate) mod projects;
pub(crate) mod simulate;
pub(crate) mod users;
pub(crate) mod ws;
pub(crate) mod yproxy;

pub(crate) type ApiResult<T> = Result<T, ErrorResponse>;

pub(crate) fn router() -> Result<Router> {
    Ok(Router::new()
        .nest("/projects", projects::router())
        .nest("/profile", profile::router())
        .nest("/auth", auth::router())
        .nest("/ws", ws::router())
        .nest("/users", users::router())
        .nest("/dev", dev::router())
        .nest("/anthropic", anthropic::router()?)
        .nest("/gemini", gemini::router()?)
        .layer(middleware::from_fn(google::authenticate))
        .nest("/notifiers", notifiers::router()?)
        .nest("/billing", billing::router()?))
}

/// Verify that the user is premium.
pub(crate) async fn verify_premium(pool: &PgPool, user: &User) -> Result<(), ErrorResponse> {
    match sqlx::query_as(
        "
        SELECT subscription_end_time IS NOT NULL AND subscription_end_time > now() AS premium
        FROM users
        WHERE email = $1;
        ",
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await
    .context("Failed to check user premium status")?
    {
        Some((true,)) => Ok(()),
        None | Some((false,)) => Err(not_premium_error(&format!(
            "User {} is not premium",
            user.email
        ))),
    }
}

/// Verify that the user has access to the given project.
pub(crate) async fn verify_project_access(
    pool: &PgPool,
    user: &User,
    project_id: &ProjectId,
) -> Result<(), ErrorResponse> {
    if project_id.is_empty() {
        return Err(bad_request_error(
            "EMPTY_PROJECT_ID",
            "Project ID must not be empty",
        ));
    }

    let permission: Option<ProjectPermission> = sqlx::query_as(
        "
        SELECT project_id, email
        FROM project_permissions
        WHERE project_id = $1
          AND email = $2;",
    )
    .bind(project_id)
    .bind(&user.email)
    .fetch_optional(pool)
    .await
    .context("Failed to check user permission")?;

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

pub(crate) fn unauthenticated_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::UNAUTHORIZED, "UNAUTHENTICATED", msg, None)
}

pub(crate) fn unauthorized_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::FORBIDDEN, "UNAUTHORIZED", msg, None)
}

pub(crate) fn not_premium_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::FORBIDDEN, "NOT_PREMIUM", msg, None)
}

pub(crate) fn bad_request_error(reason: &'static str, msg: &str) -> ErrorResponse {
    error_response(StatusCode::BAD_REQUEST, reason, msg, None)
}

pub(crate) fn not_found_error(reason: &'static str, msg: &str) -> ErrorResponse {
    error_response(StatusCode::NOT_FOUND, reason, msg, None)
}

pub(crate) fn error_response(
    status: StatusCode,
    reason: &'static str,
    msg: &str,
    err: Option<Error>,
) -> ErrorResponse {
    let err = ErrorRender { err, msg };

    match status {
        StatusCode::INTERNAL_SERVER_ERROR => {
            tracing::error!("Failed: {} ({}): {:?}", status, reason, err)
        }
        _ => tracing::warn!("Failed: {} ({}): {:?}", status, reason, err),
    }
    let msg = format!("{err}");
    ErrorResponse {
        status,
        error: reason,
        error_description: msg.clone(),
        details: vec![ErrorDetail { reason, msg }],
    }
}

struct ErrorRender<'a> {
    err: Option<Error>,
    msg: &'a str,
}

impl std::fmt::Display for ErrorRender<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl std::fmt::Debug for ErrorRender<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.err {
            None => {
                f.write_str(self.msg)?;
                Self::fmt_backtrace(&Backtrace::capture(), f)
            }
            Some(err) => {
                write!(f, "[{}]: ", self.msg)?;
                write!(f, "{err}")?;
                for cause in err.chain().skip(1) {
                    write!(f, ": {cause}")?;
                }

                Self::fmt_backtrace(err.backtrace(), f)
            }
        }
    }
}

impl ErrorRender<'_> {
    fn fmt_backtrace(backtrace: &Backtrace, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match backtrace.status() {
            BacktraceStatus::Captured => {}
            _ => return Ok(()),
        }

        let mut backtrace = backtrace.to_string();
        backtrace.truncate(backtrace.trim_end().len());

        write!(f, "\n Stack backtrace:")?;

        // Backtrace frames usually contain two lines: function and file. There are a few
        // that only have a function line and not file line, for example: ___rust_try.
        // To handle this we use a peekable iterator to pop off frames with the second
        // line of the frame, the file, being optional.
        let mut iter: std::iter::Peekable<std::str::Split<'_, &str>> =
            backtrace.split("\n").peekable();
        let mut skipped_frames = 0;
        loop {
            // The function line. For example:
            //   5: koso::server::emit_request_metrics::{{closure}}
            let Some(function_line) = iter.next() else {
                break;
            };
            // The optional file line. For example:
            //       at /some/file/path/lib.rs:520:23
            let file_line = match iter.peek() {
                Some(fl) if fl.trim().starts_with("at ") => iter.next().unwrap_or(""),
                _ => "",
            };

            // Trim everything before the function name itself, leaving, for example:
            // koso::server::emit_request_metrics::{{closure}}
            let function_name = function_line
                .trim_start_matches(|c: char| c.is_numeric() || c.is_whitespace() || c == ':');
            if !function_name.starts_with("koso::") {
                skipped_frames += 1;
                continue;
            }

            if skipped_frames > 0 {
                skipped_frames = 0;
            }
            write!(f, "\n {function_line}")?;
            if !file_line.is_empty() {
                write!(f, "\n{file_line}")?;
            }
        }
        if skipped_frames > 0 {
            write!(f, "\n       [Skipped {skipped_frames} frames]")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ErrorResponse {
    pub(crate) status: StatusCode,
    pub(crate) error: &'static str,
    pub(crate) error_description: String,
    pub(crate) details: Vec<ErrorDetail>,
}

#[derive(serde::Serialize, Debug)]
pub(crate) struct ErrorDetail {
    // Terse, stable, machine readable error reason.
    // e.g. NO_STOCK
    pub(crate) reason: &'static str,
    // Debug message for developers. Not intended for end users.
    pub(crate) msg: String,
    // Need more details about an error? Consider adding
    // a map of key/values for use in the client.
}

impl ErrorResponse {
    pub(crate) fn as_err(&self) -> Error {
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
    error: &'static str,
    error_description: String,
    details: Vec<ErrorDetail>,
}

/// Converts from ErrorResponse to Response.
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let body = axum::Json(ErrorResponseBody {
            status: self.status.as_u16(),
            error: self.error,
            error_description: self.error_description,
            details: self.details,
        });

        (self.status, body).into_response()
    }
}

/// Converts from boxed Error to ErrorResponse and logs the error.
impl<E> From<E> for ErrorResponse
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        err.context_internal("Internal error, something went wrong")
    }
}

/// Converts from ErrorResponse to rmcp::Error.
impl From<ErrorResponse> for rmcp::ErrorData {
    fn from(err: ErrorResponse) -> Self {
        let code = match err.status {
            StatusCode::INTERNAL_SERVER_ERROR => ErrorCode::INTERNAL_ERROR,
            StatusCode::NOT_FOUND => ErrorCode::RESOURCE_NOT_FOUND,
            StatusCode::BAD_REQUEST => ErrorCode::INVALID_REQUEST,
            StatusCode::UNAUTHORIZED => ErrorCode::INVALID_REQUEST,
            StatusCode::FORBIDDEN => ErrorCode::INVALID_REQUEST,
            _ => ErrorCode::INTERNAL_ERROR,
        };

        let msg = err
            .details
            .first()
            .map(|details| details.reason)
            .unwrap_or("[missing]");
        let data = match serde_json::to_value(&err.details) {
            Ok(value) => Some(value),
            Err(e) => {
                tracing::error!("Failed to serialize error details: {e:#}: {:?}", err);
                None
            }
        };
        rmcp::ErrorData::new(code, msg, data)
    }
}

pub struct XForwardedFor {
    pub client_ip: String,
}

impl headers::Header for XForwardedFor {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("x-forwarded-for");
        &NAME
    }

    fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(
        values: &mut I,
    ) -> Result<Self, headers::Error> {
        let val = values.next().ok_or_else(headers::Error::invalid)?;
        let val = val.to_str().map_err(|_| headers::Error::invalid())?;
        let first = val.split(',').next().ok_or_else(headers::Error::invalid)?;
        let client_ip = first.trim().to_string();
        Ok(XForwardedFor { client_ip })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend([self.client_ip.to_string().try_into().unwrap()])
    }
}

#[allow(dead_code)]
pub(crate) trait IntoApiResult<T, E> {
    fn context_status(self, status: StatusCode, reason: &'static str, msg: &str) -> ApiResult<T>;
    fn context_bad_request(self, reason: &'static str, msg: &str) -> ApiResult<T>;
    fn context_unauthenticated(self, msg: &str) -> ApiResult<T>;
    fn context_unauthorized(self, msg: &str) -> ApiResult<T>;
    fn context_not_found(self, msg: &str) -> ApiResult<T>;
    fn context_internal(self, msg: &str) -> ApiResult<T>;
}

impl<T, E> IntoApiResult<T, E> for Result<T, E>
where
    E: IntoErrorResponse<E>,
{
    fn context_status(self, status: StatusCode, reason: &'static str, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_status(status, reason, msg)),
        }
    }

    fn context_bad_request(self, reason: &'static str, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_bad_request(reason, msg)),
        }
    }

    fn context_unauthenticated(self, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_unauthenticated(msg)),
        }
    }

    fn context_unauthorized(self, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_unauthorized(msg)),
        }
    }

    fn context_not_found(self, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_not_found(msg)),
        }
    }

    fn context_internal(self, msg: &str) -> ApiResult<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.context_internal(msg)),
        }
    }
}

pub(crate) trait IntoErrorResponse<E> {
    fn context_status(self, status: StatusCode, reason: &'static str, msg: &str) -> ErrorResponse;
    fn context_bad_request(self, reason: &'static str, msg: &str) -> ErrorResponse;
    fn context_unauthenticated(self, msg: &str) -> ErrorResponse;
    fn context_unauthorized(self, msg: &str) -> ErrorResponse;
    fn context_not_found(self, msg: &str) -> ErrorResponse;
    fn context_internal(self, msg: &str) -> ErrorResponse;
}

impl<E> IntoErrorResponse<E> for E
where
    E: Into<anyhow::Error>,
{
    fn context_status(self, status: StatusCode, reason: &'static str, msg: &str) -> ErrorResponse {
        error_response(status, reason, msg, Some(self.into()))
    }

    fn context_bad_request(self, reason: &'static str, msg: &str) -> ErrorResponse {
        self.context_status(StatusCode::BAD_REQUEST, reason, msg)
    }

    fn context_unauthenticated(self, msg: &str) -> ErrorResponse {
        self.context_status(StatusCode::UNAUTHORIZED, "UNAUTHENTICATED", msg)
    }

    fn context_unauthorized(self, msg: &str) -> ErrorResponse {
        self.context_status(StatusCode::FORBIDDEN, "UNAUTHORIZED", msg)
    }

    fn context_not_found(self, msg: &str) -> ErrorResponse {
        self.context_status(StatusCode::NOT_FOUND, "NOT_FOUND", msg)
    }

    fn context_internal(self, msg: &str) -> ErrorResponse {
        self.context_status(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg)
    }
}
