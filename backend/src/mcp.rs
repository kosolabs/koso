use crate::{
    api::{
        collab::{
            Collab,
            projects_state::DocBox,
            txn_origin::{Actor, YOrigin},
        },
        google::User,
        model::{Project, Task},
        projects::{fetch_project, list_projects},
        verify_project_access,
    },
    settings,
};
use anyhow::{Context as _, Result, anyhow};
use axum::Router;
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use regex::Regex;
use rmcp::{
    Error as McpError, RoleServer,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars::{self},
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpService,
        streamable_http_server::session::local::{LocalSessionHandle, LocalSessionManager},
    },
};
use serde_json::Value;
use sqlx::PgPool;
use std::{cell::LazyCell, sync::Arc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CreateTaskParam {
    #[schemars(description = "the ID of the Koso project")]
    project_id: String,
    #[schemars(description = "the name of the task")]
    name: String,
}

#[derive(Clone)]
struct KosoTools {
    inner: Arc<Inner>,
    tool_router: ToolRouter<Self>,
}

struct Inner {
    collab: Collab,
    pool: &'static PgPool,
}

#[tool_router]
impl KosoTools {
    fn new(collab: Collab, pool: &'static PgPool) -> Self {
        Self {
            inner: Arc::new(Inner { collab, pool }),
            tool_router: Self::tool_router(),
        }
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    #[tool(
        name = "create_task",
        description = "Create a new task in a Koso project"
    )]
    async fn create_task(
        &self,
        Parameters(request): Parameters<CreateTaskParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let request_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let user = userr();
        let task = self
            ._create_task(request, context, user, request_id)
            .await
            .map_err(|e| McpError::internal_error("Failed to create task", Some(Value::Null)))?;

        Ok(CallToolResult::success(vec![task]))
    }

    async fn _create_task(
        &self,
        request: CreateTaskParam,
        context: RequestContext<RoleServer>,
        user: User,
        request_id: String,
    ) -> Result<Content> {
        verify_project_access(self.inner.pool, &user, &request.project_id)
            .await
            .map_err(|e| anyhow!("No access"))?;

        let client = self
            .inner
            .collab
            .register_local_client(&request.project_id)
            .await?;

        let doc = client.project.doc_box.lock().await;
        let doc = DocBox::doc_or_error(doc.as_ref())?;
        let doc = &doc.ydoc;
        let mut txn = doc.transact_mut_with(
            YOrigin {
                who: format!("mcp-session-{}", context.id),
                id: request_id,
                actor: Actor::User(user),
            }
            .as_origin()?,
        );

        let id = BASE64_URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4());
        let num = doc.next_num(&txn)?.to_string();

        let parent = doc.get(&txn, "root")?;
        let mut children: Vec<String> = parent.get_children(&txn)?;

        let task = doc.set(
            &mut txn,
            &Task {
                id: id.clone(),
                num,
                name: request.name,
                kind: None,
                ..Task::default()
            },
        );
        children.push(id);
        parent.set_children(&mut txn, &children);

        let task = task.to_task(&txn)?;

        Ok(Content::resource(ResourceContents::TextResourceContents {
            uri: format!("task://projects/{}/tasks/{}", request.project_id, task.id),
            mime_type: Some("application/json".to_string()),
            text: serde_json::to_string(&task)?,
        }))
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    #[tool(
        name = "list_projects",
        description = "List my Koso projects",
        annotations(read_only_hint = true)
    )]
    async fn list_projects(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());

        let user = userr();
        let projects: Vec<Project> = list_projects(&user.email, self.inner.pool)
            .await
            .map_err(|e| McpError::internal_error("Failed to list projects", Some(Value::Null)))?;
        let projects = projects
            .into_iter()
            .map(to_resource_contents)
            .collect::<Result<Vec<_>>>()
            .map_err(|e| {
                McpError::internal_error("Failed to serialize projects", Some(Value::Null))
            })?;
        Ok(CallToolResult::success(projects))
    }
}

fn to_resource_contents(project: Project) -> Result<Content> {
    Ok(Content::resource(ResourceContents::TextResourceContents {
        uri: format!("projects:///projects/{}", project.project_id),
        mime_type: Some(
            "application/json
"
            .to_string(),
        ),
        text: serde_json::to_string(&project)?,
    }))
}

// Implement the server handler
#[tool_handler]
impl rmcp::ServerHandler for KosoTools {
    #[tracing::instrument(skip(self))]
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides access to Koso projects and tasks".into()),
            capabilities: ServerCapabilities::builder()
                // .enable_completions()
                .enable_logging()
                //.enable_prompts()
                .enable_tools()
                .enable_resources()
                // .enable_tool_list_changed()
                .build(),
            ..Default::default()
        }
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());

        let user = userr();
        let projects: Vec<Project> = list_projects(&user.email, self.inner.pool)
            .await
            .map_err(|e| McpError::internal_error("Failed to list projects", Some(Value::Null)))?;
        let projects = projects
            .into_iter()
            .map(|p| {
                Resource::new(
                    RawResource::new(format!("projects:///projects/{}", p.project_id), p.name),
                    None,
                )
            })
            .collect::<Vec<_>>();

        Ok(ListResourcesResult {
            resources: projects,
            next_cursor: None,
        })
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());

        let resource_templates = vec![
            ResourceTemplate::new(
                RawResourceTemplate {
                    uri_template: "projects:///projects/{project_id}".to_string(),
                    name: "Koso Project".to_string(),
                    description: Some(
                        "A Koso project has multiple collaborators and contains Koso tasks"
                            .to_string(),
                    ),
                    mime_type: Some("application/json".to_string()),
                },
                None,
            ),
            ResourceTemplate::new(
                RawResourceTemplate {
                    uri_template: "tasks:///projects/{project_id}/tasks/{task_id}".to_string(),
                    name: "Koso Task".to_string(),
                    description: Some(
                        "A Koso task exists in a Koso project, may be assigned to a collaborator and may have children"
                            .to_string(),
                    ),
                    mime_type: Some("application/json".to_string()),
                },
                None,
            ),
        ];

        Ok(ListResourceTemplatesResult {
            resource_templates,
            next_cursor: None,
        })
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());

        match self._read_resource(&uri, userr()).await {
            Ok(None) => Err(McpError::resource_not_found("resource_not_found", None)),
            Ok(Some(resource)) => Ok(resource),
            Err(e) => {
                tracing::warn!("Failed to read resource: {e:#}");
                Err(McpError::internal_error(
                    "Failed to read resource",
                    Some(Value::Null),
                ))
            }
        }
    }
}

thread_local! {
    static PROJECT_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^/projects/([^/]+)$").unwrap());
    static TASK_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^/projects/([^/]+)/tasks/([^/]+)$").unwrap());
}

impl KosoTools {
    async fn _read_resource(&self, uri: &str, user: User) -> Result<Option<ReadResourceResult>> {
        let uri = url::Url::parse(uri)?;
        match uri.scheme() {
            "projects" => {
                let project_id = PROJECT_RE.with(|re| {
                    re.captures(uri.path())
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str())
                });
                let Some(project_id) = project_id else {
                    return Ok(None);
                };
                verify_project_access(self.inner.pool, &user, &project_id.to_string())
                    .await
                    .map_err(|e| anyhow!("No access"))?;

                match fetch_project(self.inner.pool, project_id).await {
                    Ok(None) => Ok(None),
                    Ok(Some(project)) => Ok(Some(ReadResourceResult {
                        contents: vec![ResourceContents::TextResourceContents {
                            uri: format!("projects:///projects/{}", project.project_id),
                            mime_type: Some("application/json".to_string()),
                            text: serde_json::to_string(&project)?,
                        }],
                    })),
                    Err(e) => Err(e),
                }
            }
            "tasks" => {
                let Some((project_id, task_id)) = TASK_RE.with(|re| {
                    re.captures(uri.path())
                        .map(|c| (c.get(1).map(|m| m.as_str()), c.get(2).map(|m| m.as_str())))
                        .and_then(|(project_id, task_id)| {
                            if let Some(project_id) = project_id
                                && let Some(task_id) = task_id
                            {
                                Some((project_id, task_id))
                            } else {
                                None
                            }
                        })
                }) else {
                    return Ok(None);
                };

                verify_project_access(self.inner.pool, &user, &project_id.to_string())
                    .await
                    .map_err(|e| anyhow!("No access"))?;

                let task = {
                    let client = self
                        .inner
                        .collab
                        .register_local_client(&project_id.to_string())
                        .await?;
                    let doc = client.project.doc_box.lock().await;
                    let doc = DocBox::doc_or_error(doc.as_ref())?;
                    let doc = &doc.ydoc;
                    let txn = doc.transact();
                    doc.get(&txn, task_id).map(|task| task.to_task(&txn))
                };

                match task {
                    Ok(Ok(task)) => Ok(Some(ReadResourceResult {
                        contents: vec![ResourceContents::TextResourceContents {
                            uri: format!("projects:///projects/{}/tasks/{}", project_id, task.id),
                            mime_type: Some("application/json".to_string()),
                            text: serde_json::to_string(&task)?,
                        }],
                    })),
                    Err(e) | Ok(Err(e)) => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}

// pub(crate) async fn start_server(cancel_token: CancellationToken) -> Result<()> {
//     // Create and run the server with STDIO transport
//     let service = Counter::new()
//         .serve_with_ct(stdio(), cancel_token.child_token())
//         .await
//         .context("Error starting MCP server")?;
//     service.waiting().await?;
//     Ok(())
// }

pub(super) fn router(
    collab: Collab,
    pool: &'static PgPool,
    cancel: CancellationToken,
) -> Result<Router> {
    // TODO: Enable this outside of dev when complete.
    if !settings::settings().is_dev() {
        return Ok(Router::new());
    }

    let session_manager = Arc::new(LocalSessionManager::default());
    let service = StreamableHttpService::new(
        move || Ok(KosoTools::new(collab.clone(), pool)),
        Arc::clone(&session_manager),
        Default::default(),
    );

    tokio::spawn(async move {
        cancel.cancelled().await;
        if let Err(e) = shutdown_mcp_server(session_manager).await {
            tracing::warn!("Failed to shutdown MCP server: {e:#}")
        }
    });
    Ok(Router::new().route_service("/sse", service))
}

async fn shutdown_mcp_server(session_manager: Arc<LocalSessionManager>) -> Result<()> {
    let mut sessions = session_manager.sessions.write().await;
    let res = futures::future::join_all(sessions.drain().map(|(_, s)| close_session(s))).await;
    tracing::info!("Shutdown MCP server: {res:?}");

    Ok(())
}

async fn close_session(session: LocalSessionHandle) -> Result<()> {
    session.close().await.context("Failed to close session")
}

// TODO: Completions
// TODO: Prompts

// TODO
fn userr() -> User {
    User {
        email: "leonhard.kyle@gmail.com".to_string(),
        name: "Kyle".to_string(),
        picture: "".to_string(),
        exp: 1,
    }
}
