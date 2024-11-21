use axum::{routing::get, Router};

use crate::plugins::github::shad;

use super::ApiResult;

pub(super) fn router() -> Router {
    Router::new().route("/github/prs", get(github_prs))
}

#[tracing::instrument()]
async fn github_prs() -> ApiResult<String> {
    Ok(shad().await?)
}
