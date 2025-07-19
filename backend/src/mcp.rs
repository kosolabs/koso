use crate::{
    api::{
        ApiResult, IntoApiResult,
        collab::{
            Collab,
            projects_state::DocBox,
            txn_origin::{Actor, YOrigin},
        },
        google::User,
        model::{Project, Task},
        not_found_error,
        projects::{fetch_project, list_projects},
        verify_project_access,
    },
    oauth,
};
use anyhow::{Context as _, Result};
use axum::{Extension, Router, extract::FromRequestParts, middleware};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use regex::Regex;
use rmcp::{
    ErrorData, RoleServer,
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
use sqlx::PgPool;
use std::{cell::LazyCell, sync::Arc};
use tokio_util::sync::CancellationToken;
use url::Url;
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
    ) -> Result<CallToolResult, ErrorData> {
        let request_id = Uuid::new_v4().to_string();
        tracing::Span::current().record("request_id", &request_id);

        let task = self._create_task(request, context, request_id).await?;

        Ok(CallToolResult::success(vec![task]))
    }

    async fn _create_task(
        &self,
        request: CreateTaskParam,
        mut context: RequestContext<RoleServer>,
        request_id: String,
    ) -> ApiResult<Content> {
        let user = user_extension(&mut context).await?;
        verify_project_access(self.inner.pool, &user, &request.project_id).await?;

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
            uri: format!("tasks://projects/{}/tasks/{}", request.project_id, task.id),
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
    ) -> Result<CallToolResult, ErrorData> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());
        Ok(self._list_projects(context).await?)
    }

    async fn _list_projects(
        &self,
        mut context: RequestContext<RoleServer>,
    ) -> ApiResult<CallToolResult> {
        let user = user_extension(&mut context).await?;
        let projects: Vec<Project> = list_projects(&user.email, self.inner.pool).await?;
        let projects = projects
            .into_iter()
            .map(Self::project_to_resource_content)
            .collect::<Result<Vec<_>>>()
            .context("Failed to serialize projects")?;
        Ok(CallToolResult::success(projects))
    }

    fn project_to_resource_content(project: Project) -> Result<Content> {
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
}

// Implement the server handler
#[tool_handler]
impl rmcp::ServerHandler for KosoTools {
    #[tracing::instrument(skip(self))]
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides access to Koso projects and tasks".into()),
            capabilities: ServerCapabilities::builder()
                .enable_logging()
                .enable_prompts()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());
        Ok(self._list_resources(context).await?)
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        tracing::Span::current().record("request_id", Uuid::new_v4().to_string());
        Ok(self._read_resource(&uri, context).await?)
    }

    #[tracing::instrument(skip(self, context), fields(request_id, session_id=context.id.to_string()))]
    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
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
}

thread_local! {
    static PROJECT_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^/projects/([^/]+)$").unwrap());
    static TASK_RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^/projects/([^/]+)/tasks/([^/]+)$").unwrap());
}

impl KosoTools {
    async fn _list_resources(
        &self,
        mut context: RequestContext<RoleServer>,
    ) -> ApiResult<ListResourcesResult> {
        let user = user_extension(&mut context).await?;
        let projects: Vec<Project> = list_projects(&user.email, self.inner.pool)
            .await
            .context_internal("Failed to list projects")?;
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

    async fn _read_resource(
        &self,
        uri: &str,
        mut context: RequestContext<RoleServer>,
    ) -> ApiResult<ReadResourceResult> {
        let user = user_extension(&mut context).await?;
        let uri = url::Url::parse(uri).context_bad_request("invalid_uri", "Invalid URI")?;
        match uri.scheme() {
            "projects" => self.read_project(uri, user).await,
            "tasks" => self.read_task(uri, user).await,
            scheme => Err(not_found_error(
                "resource_not_found",
                &format!("Invalid scheme: {scheme}"),
            )),
        }
    }

    async fn read_project(&self, uri: Url, user: User) -> ApiResult<ReadResourceResult> {
        let project_id = PROJECT_RE
            .with(|re| {
                re.captures(uri.path())
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str())
            })
            .context("Invalid project path")
            .context_bad_request("invalid_uri", "Invalid project path")?;

        verify_project_access(self.inner.pool, &user, &project_id.to_string()).await?;

        match fetch_project(self.inner.pool, project_id).await {
            Ok(None) => Err(not_found_error(
                "resource_not_found",
                &format!("Project {project_id} not found"),
            )),
            Ok(Some(project)) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: format!("projects:///projects/{}", project.project_id),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string(&project)?,
                }],
            }),
            Err(e) => Err(e.into()),
        }
    }

    async fn read_task(&self, uri: Url, user: User) -> ApiResult<ReadResourceResult> {
        let (project_id, task_id) = TASK_RE
            .with(|re| {
                re.captures(uri.path())
                    .map(|c| (c.get(1).map(|m| m.as_str()), c.get(2).map(|m| m.as_str())))
                    .and_then(|res| match res {
                        (Some(project_id), Some(task_id)) => Some((project_id, task_id)),
                        _ => None,
                    })
            })
            .context("Invalid task path")
            .context_bad_request("invalid_uri", "Invalid task path")?;

        verify_project_access(self.inner.pool, &user, &project_id.to_string()).await?;

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
            let task = doc.get(&txn, task_id).context_not_found("Task not found")?;
            task.to_task(&txn)?
        };

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: format!("projects:///projects/{}/tasks/{}", project_id, task.id),
                mime_type: Some("application/json".to_string()),
                text: serde_json::to_string(&task)?,
            }],
        })
    }
}

pub(super) fn router(
    collab: Collab,
    pool: &'static PgPool,
    cancel: CancellationToken,
) -> Result<Router> {
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
    Ok(Router::new()
        .route_service("/sse", service)
        .layer(middleware::from_fn(oauth::authenticate)))
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

async fn user_extension(context: &mut RequestContext<RoleServer>) -> Result<User> {
    let parts = context
        .extensions
        .get_mut::<axum::http::request::Parts>()
        .context("Missing axum extension")?;
    let Extension(user): Extension<User> = Extension::from_request_parts(parts, &())
        .await
        .context("Missing user extension")?;
    Ok(user)
}
