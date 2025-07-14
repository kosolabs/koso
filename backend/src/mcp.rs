use anyhow::Result;
use axum::Router;
use rmcp::{
    Error as McpError,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpService, streamable_http_server::session::local::LocalSessionManager,
    },
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Counter {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

// Implement the server handler
#[tool_handler]
impl rmcp::ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
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

pub(super) fn router() -> Result<Router> {
    let service = StreamableHttpService::new(
        move || Ok(Counter::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );
    Ok(Router::new().route_service("/sse", service))
}
