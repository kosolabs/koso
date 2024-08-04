use std::net::SocketAddr;

use crate::{
    api::{self, ApiResult},
    google::{self, User},
    notify,
};
use axum::{
    body::Body,
    extract::{ConnectInfo, Path, WebSocketUpgrade},
    middleware,
    response::Response,
    routing::get,
    Extension, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::FutureExt as _;
use sqlx::PgPool;
use tracing::Instrument as _;

pub fn ws_router() -> Router {
    Router::new()
        .route("/projects/:project_id", get(ws_handler))
        .layer(middleware::from_fn(google::authenticate))
        .fallback(api::handler_404)
}
/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
#[tracing::instrument(skip(ws, _user_agent, addr, user, notifier, pool))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(user): Extension<User>,
    Extension(notifier): Extension<notify::Notifier>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Response<Body>> {
    api::verify_access(pool, user, &project_id).await?;

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let cs = tracing::Span::current();
    Ok(ws.protocols(["bearer"]).on_upgrade(move |socket| {
        notifier
            .register_client(socket, addr, project_id)
            .map(move |res| {
                if let Err(e) = res {
                    tracing::warn!("Failed to register destination for {addr}: {e}");
                }
            })
            .instrument(cs)
    }))
}
