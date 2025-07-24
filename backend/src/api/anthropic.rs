use crate::{
    api::{
        ApiResult, collab::Collab, google::User, simulate::simulate, verify_premium,
        verify_project_access,
    },
    secrets::{Secret, read_secret},
};
use anyhow::{Context, Result};
use axum::{Extension, Router, body::Body, extract::Query, response::Response, routing::get};
use serde_json::to_string;
use sqlx::postgres::PgPool;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

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
}

pub(super) fn router() -> Result<Router> {
    let client = AnthropicClient::new();

    Ok(Router::new()
        .route("/summarize", get(summarize_handler))
        .route("/breakdown", get(breakdown_handler))
        .layer(Extension(client)))
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

    let ydoc = collab.get_doc(&req.project_id).await?;
    let txn = ydoc.transact();

    let mut parents: HashMap<String, Vec<String>> = HashMap::new();
    for task in ydoc.tasks(&txn)? {
        let parent_id = task.get_id(&txn)?;
        for child_id in task.get_children(&txn)? {
            parents.entry(child_id).or_default().push(parent_id.clone());
        }
    }

    let content = {
        let mut content = VecDeque::<AnthropicContent>::new();

        let mut processed: HashSet<String> = HashSet::new();
        let mut remaining: VecDeque<String> = VecDeque::from([req.task_id.to_string()]);
        while let Some(task_id) = remaining.pop_front() {
            if let Some(parents) = parents.get(&task_id) {
                for parent in parents {
                    remaining.push_back(parent.into());
                }
            }
            if !processed.contains(&task_id) {
                let ytask = ydoc.get(&txn, &task_id)?;
                if let Some(task_desc) = ytask.get_desc(&txn)? {
                    content.push_front(text(&task_desc));
                }
                content.push_front(text(&ytask.get_name(&txn)?));
                if req.task_id == task_id {
                    content.push_front(text("Task:"));
                }
                processed.insert(task_id);
            }
        }
        content.push_front(text("Context:"));

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
            content: content.into(),
        }],
    };

    tracing::debug!(
        "AnthropicMessageRequest: {}",
        serde_json::to_string(&message)?
    );

    let response = client.message(&message).await?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(response.bytes_stream()))?)
    // Ok(Response::default())
}
