use crate::{
    api::{
        ApiResult, IntoApiResult, collab::Collab, google::User, verify_premium,
        verify_project_access,
    },
    secrets::{Secret, read_secret},
};
use anyhow::{Context, Result, anyhow};
use axum::{Extension, Json, Router, body::Body, extract::Query, response::Response, routing::get};
use futures::{StreamExt, stream};
use serde_json::to_string;
use sqlx::{Pool, Postgres, postgres::PgPool};
use std::collections::BTreeSet;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Task {
    id: String,
    num: String,
    name: String,
    children: Vec<String>,
    assignee: Option<String>,
    status: Option<String>,
    kind: String,
    estimate: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicContent {
    r#type: String,
    text: String,
}

fn text(text: &str) -> AnthropicContent {
    AnthropicContent {
        r#type: "text".into(),
        text: text.into(),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicMessageRequest {
    model: String,
    max_tokens: u32,
    stream: Option<bool>,
    system: Vec<AnthropicContent>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Clone)]
struct AnthropicClient {
    client: reqwest::Client,
    token: Option<Secret<String>>,
}

impl AnthropicClient {
    pub(super) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            token: read_secret("anthropic/token").ok(),
        }
    }

    fn token(&self) -> Result<String> {
        Ok(self
            .token
            .clone()
            .context("anthropic/token is not set")?
            .data)
    }

    async fn message(&self, message: &AnthropicMessageRequest) -> Result<reqwest::Response> {
        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.token()?)
            .header("anthropic-version", "2023-06-01")
            .json(message)
            .send()
            .await?
            .error_for_status()?)
    }

    // async fn breakdown(&self, model: &str, context: &str, task: &Task) -> Result<String> {
}

#[derive(serde::Serialize, serde::Deserialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct ProjectContext {
    project_id: String,
    llm_context: Option<String>,
}

impl ProjectContext {
    async fn fetch(pool: &Pool<Postgres>, project_id: &str) -> Result<ProjectContext> {
        Ok(sqlx::query_as(
            "
            SELECT project_id, llm_context
            FROM projects
            WHERE project_id = $1
            ",
        )
        .bind(project_id)
        .fetch_one(pool)
        .await?)
    }
}

pub(super) fn router() -> Result<Router> {
    let client = AnthropicClient::new();

    Ok(Router::new()
        .route("/summarize", get(summarize_handler))
        .route("/breakdown", get(breakdown_handler))
        .layer(Extension(client))
        .route("/context", get(get_context_handler)))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SummarizeTaskRequest {
    project_id: String,
    task_id: String,
    model: String,
    simulate: Option<bool>,
}

#[tracing::instrument(skip(user, pool, collab, client))]
async fn summarize_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(collab): Extension<Collab>,
    Extension(client): Extension<AnthropicClient>,
    req: Query<SummarizeTaskRequest>,
) -> ApiResult<Response> {
    verify_project_access(pool, &user, &req.project_id).await?;
    verify_premium(pool, &user).await?;

    if req.simulate.unwrap_or(false) {
        return simulate("summarize").await;
    }

    let ydoc = collab.get_doc(&req.project_id).await?;
    let txn = ydoc.transact();

    let tasks = {
        let mut task_ids = BTreeSet::<String>::new();
        let mut stack = vec![req.task_id.clone()];

        while let Some(curr) = stack.pop() {
            let ytask = ydoc.get(&txn, &curr)?;
            task_ids.insert(curr);
            for child_id in ytask.get_children(&txn)? {
                stack.push(child_id);
            }
        }

        task_ids
    }
    .iter()
    .map(|id| {
        let task = ydoc.get(&txn, id)?;
        let children = task.get_children(&txn)?;
        let kind = match task.get_kind(&txn)? {
            Some(kind) => kind,
            None => {
                if children.is_empty() {
                    "Task".into()
                } else {
                    "Rollup".into()
                }
            }
        };
        Ok(Task {
            id: task.get_id(&txn)?,
            num: task.get_num(&txn)?,
            name: task.get_name(&txn)?,
            children: task.get_children(&txn)?,
            assignee: task.get_assignee(&txn)?,
            status: task.get_status(&txn)?,
            kind,
            estimate: task.get_estimate(&txn)?,
        })
    })
    .collect::<Result<Vec<Task>>>()?;

    let response = client
        .message(&AnthropicMessageRequest {
            model: req.model.clone(),
            max_tokens: 8192,
            stream: Some(true),
            system: vec![
                text("Render a one or two sentence summary in Markdown for each of the following sections: Goal, Completed Work, Remaining Work, Key Risks, Next Step")
            ],
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: vec![
                    text("Attached is a JSON document that represents an iteration in a project plan. The plan is represented as a graph of tasks where relationships between tasks are expressed using the children field."),
                    text(&to_string(&tasks)?),
                ],
            }],
        }).await?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(response.bytes_stream()))?)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BreakdownTaskRequest {
    project_id: String,
    task_id: String,
    model: String,
    simulate: Option<bool>,
}

#[tracing::instrument(skip(user, pool, collab, client))]
async fn breakdown_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(collab): Extension<Collab>,
    Extension(client): Extension<AnthropicClient>,
    req: Query<BreakdownTaskRequest>,
) -> ApiResult<Response> {
    verify_project_access(pool, &user, &req.project_id).await?;
    verify_premium(pool, &user).await?;

    if req.simulate.unwrap_or(false) {
        return simulate("breakdown").await;
    }

    let project: ProjectContext = sqlx::query_as(
        "
        SELECT project_id, llm_context
        FROM projects
        WHERE project_id = $1
        ",
    )
    .bind(&req.project_id)
    .fetch_one(pool)
    .await?;

    let ydoc = collab.get_doc(&req.project_id).await?;
    let txn = ydoc.transact();

    let ytask = ydoc.get(&txn, &req.task_id)?;
    let task_name = ytask.get_name(&txn)?;
    let task_desc = ytask.get_desc(&txn)?;

    tracing::info!(project.llm_context);

    let content = {
        let mut content = Vec::<AnthropicContent>::new();

        if let Some(llm_context) = project.llm_context {
            content.push(text("Task break down project description context:"));
            content.push(text(&llm_context));
        }

        content.push(text("Task:"));
        content.push(text(&task_name));
        if let Some(task_desc) = task_desc {
            content.push(text(&task_desc));
        }

        content
    };

    let message = AnthropicMessageRequest {
        model: req.model.clone(),
        max_tokens: 8192,
        stream: Some(true),
        system: vec![text(
            "Break down the task into its first order tasks, one per line, without any preamble.",
        )],
        messages: vec![AnthropicMessage {
            role: "user".into(),
            content,
        }],
    };

    tracing::debug!("AnthropicMessageRequest: {:?}", message);

    let response = client.message(&message).await?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(response.bytes_stream()))?)
}

async fn simulate(method: &str) -> ApiResult<Response> {
    let data = if method == "summarize" {
        include_str!("anthropic/summarize.txt")
    } else if method == "breakdown" {
        include_str!("anthropic/breakdown.txt")
    } else {
        return Err(anyhow!("Invalid method").into());
    };
    let chunks = data
        .split_inclusive("\n\n")
        .map(|chunk| chunk.as_bytes().to_vec())
        .collect::<Vec<Vec<u8>>>();

    let test_stream = stream::iter(chunks).then(move |chunk| async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok::<_, std::io::Error>(chunk)
    });

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(test_stream))?)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ProjectContextRequest {
    project_id: String,
}

#[tracing::instrument(skip(user, pool))]
async fn get_context_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    req: Query<ProjectContextRequest>,
) -> ApiResult<Json<ProjectContext>> {
    verify_project_access(pool, &user, &req.project_id).await?;
    Ok(Json(
        ProjectContext::fetch(pool, &req.project_id)
            .await
            .context_not_found(&format!("Project ID {} not found", req.project_id))?,
    ))
}
