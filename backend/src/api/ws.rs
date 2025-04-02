use crate::api::{ApiResult, collab::Collab, google::User};
use axum::{
    Extension, Router,
    body::Body,
    extract::{Path, WebSocketUpgrade},
    response::Response,
    routing::get,
};
use tracing::Instrument as _;
use uuid::Uuid;

pub(super) fn router() -> Router {
    Router::new().route("/projects/{project_id}", get(ws_handler))
}
/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
#[tracing::instrument(skip(ws, user, collab), fields(who))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    Extension(user): Extension<User>,
    Extension(collab): Extension<Collab>,
) -> ApiResult<Response<Body>> {
    let who = Uuid::new_v4().to_string();
    let cs: tracing::Span = tracing::Span::current();
    cs.record("who", &who);

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    Ok(ws
        .protocols(["bearer"])
        .on_failed_upgrade(|e| tracing::warn!("Failed to upgrade socket: {e:?}"))
        .on_upgrade(move |socket: axum::extract::ws::WebSocket| {
            async move {
                if let Err(e) = collab.register_client(socket, who, project_id, user).await {
                    tracing::warn!("Failed to register client: {e:?}");
                }
            }
            .instrument(cs)
        }))
}
