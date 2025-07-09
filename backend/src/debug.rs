use anyhow::Result;
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use regex::Regex;

use crate::{api::ApiResult, settings::settings};

const BODY_LIMIT: usize = 10 * 1024 * 1024;

pub(super) async fn log_request_response(
    request: Request,
    next: Next,
) -> ApiResult<impl IntoResponse> {
    if !matches(&settings().debug_path, &request) {
        return Ok(next.run(request).await);
    }

    let request = log_request(request).await?;
    let response = next.run(request).await;
    let response = log_response(response).await?;

    Ok(response)
}

async fn log_request(request: Request) -> Result<Request> {
    let (parts, body) = request.into_parts();
    let headers = parts
        .headers
        .iter()
        .map(|(name, value)| format!("{}: {}", name, String::from_utf8_lossy(value.as_bytes())))
        .collect::<Vec<String>>();
    let bytes = axum::body::to_bytes(body, BODY_LIMIT).await?;
    tracing::debug!(
        "Request: \n{} {}\n{}\n\n{}",
        parts.method,
        parts.uri,
        headers.join("\n"),
        String::from_utf8_lossy(&bytes),
    );
    Ok(Request::from_parts(parts, Body::from(bytes)))
}

async fn log_response(response: Response) -> Result<Response> {
    let (parts, body) = response.into_parts();
    let headers = parts
        .headers
        .iter()
        .map(|(name, value)| format!("{}: {}", name, String::from_utf8_lossy(value.as_bytes())))
        .collect::<Vec<String>>();
    let bytes = axum::body::to_bytes(body, BODY_LIMIT).await?;
    tracing::debug!(
        "Response: \n{}\n\n{}",
        headers.join("\n"),
        String::from_utf8_lossy(&bytes),
    );
    Ok(Response::from_parts(parts, Body::from(bytes)))
}

fn matches(pattern: &Option<Regex>, request: &Request) -> bool {
    match pattern {
        Some(pattern) => pattern.is_match(request.uri().path()),
        None => false,
    }
}
