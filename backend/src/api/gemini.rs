use crate::{
    api::{google::User, simulate::simulate, verify_premium, verify_project_access},
    plugins::github::app::AppGithub,
    secrets::{Secret, read_secret},
};
use anyhow::{Context, Result};
use axum::{Extension, Router, body::Body, extract::Query, response::Response, routing::get};
use axum_anyhow::ApiResult;
use sqlx::postgres::PgPool;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct GenerateContentRequest {
    system_instruction: Content,
    contents: Vec<Content>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Content {
    parts: Vec<Part>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Part {
    text: String,
}

#[derive(Clone)]
struct GeminiClient {
    client: reqwest::Client,
    token: Option<Secret<String>>,
}

impl GeminiClient {
    pub(super) fn new() -> Self {
        GeminiClient {
            client: reqwest::Client::new(),
            token: read_secret("gemini/token").ok(),
        }
    }

    fn token(&self) -> Result<String> {
        Ok(self.token.clone().context("gemini/token is not set")?.data)
    }

    async fn generate_content_stream(
        &self,
        req: &GenerateContentRequest,
    ) -> Result<reqwest::Response> {
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:streamGenerateContent?alt=sse";
        Ok(self
            .client
            .post(url)
            .header("x-goog-api-key", self.token()?)
            .json(&req)
            .send()
            .await?
            .error_for_status()?)
    }
}

pub(super) fn router() -> Result<Router> {
    let client = GeminiClient::new();

    Ok(Router::new()
        .route("/context", get(generate_context_handler))
        .layer(Extension(client)))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GenerateRepoContextRequest {
    project_id: String,
    owner: String,
    repo: String,
    simulate: Option<bool>,
}

#[tracing::instrument(skip(user, pool, client))]
async fn generate_context_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(client): Extension<GeminiClient>,
    req: Query<GenerateRepoContextRequest>,
) -> ApiResult<Response> {
    verify_project_access(pool, &user, &req.project_id).await?;
    verify_premium(pool, &user).await?;

    if req.simulate.unwrap_or(false) {
        return simulate("context").await;
    }

    let repo_github = AppGithub::new()
        .await?
        .repo_github(&req.owner, &req.repo)
        .await?;
    let source_context = repo_github.compile_source_context().await?;

    let prompt =
        "Generate a design doc for the attached codebase. Write confidently and do not hedge.";
    let resp = client
        .generate_content_stream(&GenerateContentRequest {
            system_instruction: Content {
                parts: vec![Part {
                    text: prompt.into(),
                }],
            },
            contents: vec![Content {
                parts: vec![Part {
                    text: source_context,
                }],
            }],
        })
        .await?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(resp.bytes_stream()))?)
}
