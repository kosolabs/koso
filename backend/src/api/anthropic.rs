use crate::{
    api::{ApiResult, collab::Collab, google::User, verify_premium, verify_project_access},
    secrets::{Secret, read_secret},
};
use anyhow::{Result, anyhow};
use axum::{Extension, Router, extract::Query, routing::get};
use serde_json::to_string;
use sqlx::postgres::PgPool;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::{
    collections::{BTreeSet, HashMap},
    hash::Hash,
    sync::Arc,
};
use tokio::sync::RwLock;

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

const SYSTEM: &str = r#"Attached is a JSON document that represents an iteration in a project plan. The plan is represented as a graph of tasks.

Render a one or two sentence summary in Markdown for each of the following sections:

  - Goal
  - Completed Work
  - Remaining Work
  - Key Risks
  - Next Step"#;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AnthropicMessageRequest {
    model: String,
    max_tokens: u32,
    system: Vec<AnthropicContent>,
    messages: Vec<AnthropicMessage>,
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
}

#[derive(Clone)]
pub(super) struct SummaryCache(Arc<RwLock<HashMap<u64, String>>>);

impl SummaryCache {
    pub(super) fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    fn hash(model: &str, tasks: &Vec<Task>) -> u64 {
        let mut hasher = DefaultHasher::new();
        model.hash(&mut hasher);
        tasks.hash(&mut hasher);
        hasher.finish()
    }

    async fn get(&self, key: &u64) -> Option<String> {
        let map = self.0.read().await;
        map.get(key).cloned()
    }

    async fn insert(&self, key: u64, value: &str) {
        let mut map = self.0.write().await;
        map.insert(key, value.into());
    }
}

#[derive(Clone)]
pub(super) struct AnthropicClient {
    pub(super) client: reqwest::Client,
}

impl AnthropicClient {
    pub(super) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn message(&self, message: &AnthropicMessageRequest) -> Result<AnthropicMessageResponse> {
        let token: Secret<String> = read_secret("anthropic/token")?;
        Ok(self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", token.data)
            .header("anthropic-version", "2023-06-01")
            .json(message)
            .send()
            .await?
            .error_for_status()?
            .json::<AnthropicMessageResponse>()
            .await?)
    }
}

pub(super) fn router() -> Result<Router> {
    let cache = SummaryCache::new();
    let client = AnthropicClient::new();

    Ok(Router::new()
        .route("/summarize", get(generate_task_summary_handler))
        .layer((Extension(cache), Extension(client))))
}

#[tracing::instrument(skip(user, pool, collab, cache, client))]
pub(super) async fn generate_task_summary_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(collab): Extension<Collab>,
    Extension(cache): Extension<SummaryCache>,
    Extension(client): Extension<AnthropicClient>,
    req: Query<SummarizeTaskRequest>,
) -> ApiResult<String> {
    verify_project_access(pool, &user, &req.project_id).await?;
    verify_premium(pool, &user).await?;

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

    let key = SummaryCache::hash(&req.model, &tasks);
    if let Some(result) = cache.get(&key).await {
        tracing::trace!("Cache hit!");
        return Ok(result);
    } else {
        tracing::trace!("Cache miss!");
    }

    let resp = client
        .message(&AnthropicMessageRequest {
            model: req.model.clone(),
            max_tokens: 8192,
            system: vec![AnthropicContent {
                r#type: "text".into(),
                text: SYSTEM.into(),
                cache_control: Some({
                    AnthropicCacheControl {
                        r#type: "ephemeral".into(),
                    }
                }),
            }],
            messages: vec![AnthropicMessage {
                role: "user".into(),
                content: vec![AnthropicContent {
                    r#type: "text".into(),
                    text: to_string(&tasks)?,
                    cache_control: None,
                }],
            }],
        })
        .await?;

    tracing::info!("{:?}", resp);

    let content = match resp.content.into_iter().next() {
        Some(content) => Ok(content),
        None => Err(anyhow!("No content in Anthropic response")),
    }?;

    cache.insert(key, &content.text).await;

    Ok(content.text)
}
