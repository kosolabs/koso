use std::net::SocketAddr;

use crate::{
    api::google::User,
    api::{collab::Collab, ApiResult},
};
use axum::{
    body::Body,
    extract::{ConnectInfo, Path, WebSocketUpgrade},
    response::Response,
    routing::get,
    Extension, Router,
};
use axum_extra::{headers, TypedHeader};
use tracing::Instrument as _;

pub(super) fn router() -> Router {
    Router::new().route("/projects/{project_id}", get(ws_handler))
}
/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
#[tracing::instrument(skip(ws, _user_agent, addr, user, collab))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(user): Extension<User>,
    Extension(collab): Extension<Collab>,
) -> ApiResult<Response<Body>> {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let cs: tracing::Span = tracing::Span::current();
    Ok(ws
        .protocols(["bearer"])
        .on_failed_upgrade(|e| tracing::warn!("Failed to upgrade socket: {e:?}"))
        .on_upgrade(move |socket: axum::extract::ws::WebSocket| {
            async move {
                if let Err(e) = collab.register_client(socket, addr, project_id, user).await {
                    tracing::warn!("Failed to register client at {addr}: {e:?}");
                }
            }
            .instrument(cs)
        }))
}
