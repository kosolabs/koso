use anyhow::{Context, Error, Result};
use axum::{
    Router,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware,
    response::IntoResponse,
};
use axum_anyhow::{ApiError, ApiResult, OptionExt, bad_request};
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
pub(crate) mod dupes;
pub(crate) mod gemini;
pub(crate) mod google;
pub(crate) mod model;
pub(crate) mod profile;
pub(crate) mod projects;
pub(crate) mod simulate;
pub(crate) mod users;
pub(crate) mod ws;
pub(crate) mod yproxy;

pub(crate) fn router() -> Result<Router> {
    configure_error_logging();

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
pub(crate) async fn verify_premium(pool: &PgPool, user: &User) -> ApiResult<()> {
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
) -> ApiResult<()> {
    if project_id.is_empty() {
        return Err(bad_request(
            "EMPTY_PROJECT_ID",
            "Project ID must not be empty",
        ));
    }

    sqlx::query_as::<_, ProjectPermission>(
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
    .context("Failed to check user permission")?
    .context_forbidden(
        "UNAUTHORIZED",
        &format!(
            "User {} is not authorized to access {}",
            user.email, project_id
        ),
    )?;

    Ok(())
}

pub(crate) async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

pub(crate) fn not_premium_error(msg: &str) -> ApiError {
    ApiError::builder()
        .status(StatusCode::FORBIDDEN)
        .title("NOT_PREMIUM")
        .detail(msg)
        .build()
}

fn configure_error_logging() {
    axum_anyhow::on_error(|err| {
        let detail = ErrorRender {
            err: &err.error,
            msg: &err.detail,
        };
        if err.status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(status = %err.status, title = %err.title, ?detail, "Failed");
        } else {
            tracing::warn!(status = %err.status, title = %err.title, ?detail, "Failed");
        }
    });
}

struct ErrorRender<'a> {
    err: &'a Option<Error>,
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

pub(crate) struct RmcpErrorData(rmcp::ErrorData);

impl<E> From<E> for RmcpErrorData
where
    E: Into<anyhow::Error>,
{
    fn from(_err: E) -> Self {
        RmcpErrorData(rmcp::ErrorData::new(
            ErrorCode::INTERNAL_ERROR,
            "Internal error",
            None,
        ))
    }
}

impl From<RmcpErrorData> for rmcp::ErrorData {
    fn from(err: RmcpErrorData) -> Self {
        err.0
    }
}

fn rmcp_error(code: ErrorCode, title: &'static str, detail: &str) -> RmcpErrorData {
    RmcpErrorData(rmcp::ErrorData::new(
        code,
        title,
        match serde_json::to_value(detail) {
            Ok(value) => Some(value),
            Err(e) => {
                tracing::error!("Failed to serialize error details: {e:#}: {:?}", detail);
                None
            }
        },
    ))
}

pub(crate) fn resource_not_found(title: &'static str, detail: &str) -> RmcpErrorData {
    rmcp_error(ErrorCode::RESOURCE_NOT_FOUND, title, detail)
}

pub(crate) fn invalid_request(title: &'static str, detail: &str) -> RmcpErrorData {
    rmcp_error(ErrorCode::INVALID_REQUEST, title, detail)
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
