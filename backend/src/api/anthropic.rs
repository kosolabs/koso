use crate::{
    api::{ApiResult, collab::Collab, google::User, verify_premium, verify_project_access},
    secrets::{Secret, read_secret},
};
use anyhow::{Context, Result, anyhow};
use axum::{Extension, Router, body::Body, extract::Query, response::Response, routing::get};
use futures::{StreamExt, stream};
use reqwest::RequestBuilder;
use serde_json::to_string;
use sqlx::postgres::PgPool;
use std::{collections::BTreeSet, hash::Hash};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default, Hash)]
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
struct AnthropicCacheControl {
    r#type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicContent {
    r#type: String,
    text: String,
    cache_control: Option<AnthropicCacheControl>,
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
    system: Vec<AnthropicContent>,
    messages: Vec<AnthropicMessage>,
    stream: Option<bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicUsage {
    cache_creation_input_tokens: u32,
    cache_read_input_tokens: u32,
    output_tokens: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicMessageResponse {
    id: String,
    r#type: String,
    role: String,
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct SummarizeTaskRequest {
    project_id: String,
    task_id: String,
    model: String,
    simulate: Option<bool>,
}

#[derive(Clone)]
pub(super) struct AnthropicClient {
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

    fn message_builder(&self) -> Result<RequestBuilder> {
        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.token()?)
            .header("anthropic-version", "2023-06-01"))
    }

    // async fn breakdown(&self, model: &str, context: &str, task: &Task) -> Result<String> {
}

pub(super) fn router() -> Result<Router> {
    let client = AnthropicClient::new();

    Ok(Router::new()
        .route("/summarize", get(summarize_handler))
        .layer(Extension(client)))
}

#[tracing::instrument(skip(user, pool, collab, client))]
pub(super) async fn summarize_handler(
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
        .message_builder()?
        .json(&AnthropicMessageRequest {
            model: req.model.clone(),
            max_tokens: 8192,
            stream: Some(true),
            system: vec![AnthropicContent {
                r#type: "text".into(),
                text: "Attached is a JSON document that represents an iteration in a project plan. The plan is represented as a graph of tasks where relationships between tasks are expressed using the children field.".into(),
                cache_control: Some({
                    AnthropicCacheControl {
                        r#type: "ephemeral".into(),
                    }
                }),
            }],
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: vec![
                    AnthropicContent {
                        r#type: "text".into(),
                        text: to_string(&tasks)?,
                        cache_control: None,
                    },
                    AnthropicContent {
                        r#type: "text".into(),
                        text: "Render a one or two sentence summary in Markdown for each of the following sections: Goal, Completed Work, Remaining Work, Key Risks, Next Step".into(),
                        cache_control: Some({
                            AnthropicCacheControl {
                                r#type: "ephemeral".into(),
                            }
                        }),
                    },
                ],
            }],
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(response.bytes_stream()))?)
}

async fn simulate(method: &str) -> ApiResult<Response> {
    let data = if method == "summarize" {
        include_str!("summarize.txt")
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
