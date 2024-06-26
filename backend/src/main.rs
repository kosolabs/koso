use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        MatchedPath, Request,
    },
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json},
    routing::{get, post},
    Extension, Router,
};
use axum_extra::{headers, TypedHeader};
use listenfd::ListenFd;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::ConnectOptions;
use std::net::SocketAddr;
use std::{borrow::BorrowMut, sync::Arc};
use std::{
    future::ready,
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::Mutex;
use tower_http::services::ServeFile;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use axum_streams::StreamBodyAsOptions;

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

mod db_listener;
use db_listener::{stream_task_notifications, Payload};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Task {
    id: String,
    name: String,
}

#[derive(serde::Deserialize, Debug)]
struct NewTask {
    name: String,
}

struct AppState {}

#[derive(Clone)]
struct ChangeRouter {
    /// Existing routes to notify.
    routes: Arc<Mutex<Vec<ClientRoute>>>,
    /// Newly added routes that will be moved to `routes`
    /// when the next notification is processed.
    new_routes: Arc<Mutex<Vec<ClientRoute>>>,
}

#[derive(Debug)]
struct ClientRoute {
    sink: futures::stream::SplitSink<WebSocket, Message>,
    who: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "yotei=trace,tower_http=trace,axum::rejection=trace,sqlx=trace,axum=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (_main_server, _metrics_server) = tokio::join!(start_main_server(), start_metrics_server());
}

async fn start_main_server() {
    // Connect to the Postgres database.
    let db_connection_str =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost".to_string());
    tracing::debug!("Connecting to database: {}", db_connection_str);
    let pool: &'static PgPool = Box::leak(Box::new(
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(
                db_connection_str
                    .parse::<PgConnectOptions>()
                    .unwrap()
                    // Enable query trace logging. Must enable `sqlx=trace`
                    .log_statements(tracing::log::LevelFilter::Trace),
            )
            .await
            .expect("can't connect to database"),
    ));

    // Create a stream of task notifications and a router to fan them out to client routes.
    let change_router = ChangeRouter {
        routes: Arc::new(Mutex::new(Vec::new())),
        new_routes: Arc::new(Mutex::new(Vec::new())),
    };
    let notify_task = tokio::spawn(handle_notify(
        change_router.clone(),
        stream_task_notifications(pool),
    ));

    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/task/list", get(list_tasks))
        .route("/task/create", post(create_task))
        .route("/task/update", post(update_task))
        .route("/task/stream", get(stream_tasks))
        .route("/ws", get(ws_handler))
        .route_service("/", ServeFile::new("assets/index.html"))
        .route_service("/script.js", ServeFile::new("assets/script.js"))
        .route_layer(middleware::from_fn(track_metrics))
        .fallback(handler_404)
        .with_state(state)
        .layer((
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            Extension(pool),
            Extension(change_router),
        ));

    // We can either use a listener provided by the environment by ListenFd or
    // listen on a local port. The former is convenient when using `cargo watch`
    // with systemd.
    // For example: `systemfd --no-pid -s http::3000 -- cargo watch -x run``
    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind("0.0.0.0:3000").await.unwrap(),
    };

    tracing::debug!("server listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal("server"))
    .await
    .unwrap();

    // Now that the server is shutdown, it's safe to clean things up.
    notify_task.abort();
    tracing::debug!("Closing database pool...");
    pool.close().await;
}

async fn create_task(
    Extension(pool): Extension<&'static PgPool>,
    Json(new_task): Json<NewTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = Task {
        id: Uuid::new_v4().to_string(),
        name: new_task.name,
    };

    sqlx::query("INSERT INTO tasks(id, name) VALUES ($1, $2);")
        .bind(&task.id)
        .bind(&task.name)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    Ok(Json(task))
}

async fn update_task(
    Extension(pool): Extension<&'static PgPool>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let res = sqlx::query("UPDATE tasks SET name = $1 WHERE id=$2;")
        .bind(&task.name)
        .bind(&task.id)
        .execute(pool)
        .await
        .map_err(internal_error)?;
    if res.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            format!("task '{}' does not exist!", &task.id),
        ));
    }
    // Given the updates where clause should match 0 or 1 rows and never more,
    // this should never happen.
    if res.rows_affected() > 1 {
        tracing::error!(
            "unexpectedly updated more than 1 rows ({}) for id '{}'",
            res.rows_affected(),
            &task.id
        )
    }

    Ok(Json(task))
}

async fn list_tasks(
    Extension(pool): Extension<&'static PgPool>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let results: Vec<(String, String)> = sqlx::query_as("SELECT id, name from tasks")
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

    let mut tasks = Vec::new();
    for row in results {
        tasks.push(Task {
            id: row.0,
            name: row.1,
        })
    }

    Ok(Json(tasks))
}

async fn stream_tasks(Extension(pool): Extension<&'static PgPool>) -> impl IntoResponse {
    use tokio_stream::StreamExt;

    let tasks = sqlx::query_as("SELECT id, name from tasks")
        .fetch(pool)
        .map(|r: Result<(String, String), sqlx::Error>| match r {
            Ok(t) => Ok(Task { id: t.0, name: t.1 }),
            Err(e) => {
                tracing::warn!("stream_tasks query failed: {}", e);
                Err(axum::Error::new(e))
            }
        });

    StreamBodyAsOptions::new()
        .buffering_ready_items(5)
        // An error mid stream will break the connection and cause
        // most clients to produce an ERR_INCOMPLETE_CHUNKED_ENCODING error.
        // Server side, you'll see the log above and a message like:
        // "axum::serve: failed to serve connection: error from user's Body stream"
        // An alternative is to return what amouts to a Result of Task rather
        // than simply Task to clients and require clients handle errors explicitly.
        // See https://github.com/abdolence/axum-streams-rs/issues/54
        .json_array_with_errors(tasks)
}

/// Utility function for mapping any error into a `500 Internal Server Error` response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

// Default 404 handler.
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

// Completion of this function signals to a server,
// via graceful_shutdown, to begin shutdown.
// As such, avoid doing cleanup work here.
async fn shutdown_signal(name: &str) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::debug!("Terminating {name} with ctrl-c...");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        tracing::debug!("Terminating {name}...");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn start_metrics_server() {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
    ];
    let recorder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap();

    let app = Router::new().route("/metrics", get(move || ready(recorder.render())));
    // The `/metrics` endpoint should not be publicly available.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    tracing::debug!(
        "metrics server listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal("metrics server"))
        .await
        .unwrap();
}

async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().to_string();

    // Run the next handler.
    let response = next.run(req).await;

    // Emit metrics.
    let labels = [
        ("method", method),
        ("path", path),
        ("status", response.status().as_u16().to_string()),
    ];
    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels)
        .record(start.elapsed().as_secs_f64());

    response
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(change_router): Extension<ChangeRouter>,
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, change_router))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, change_router: ChangeRouter) {
    use futures::stream::StreamExt;
    let (sender, mut receiver) = socket.split();

    let who = who.to_string();
    // Store the sender side of the socket in the list of client routes.
    tracing::debug!("Adding route for client {who}");
    change_router.new_routes.lock().await.push(ClientRoute {
        sink: sender,
        who: who.clone(),
    });

    // Listen for messages on the read side of the socket.
    // We don't currently expect any messages other than closures.
    // The idea is to proactively remove clients on closure rather
    // than only sweeping them out while processing a notification.
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(c) => {
                    if let Some(cf) = c {
                        tracing::debug!(
                            ">>> {who} sent close with code {} and reason `{}`",
                            cf.code,
                            cf.reason
                        );
                    } else {
                        tracing::debug!(">>> {who} somehow sent close message without CloseFrame");
                    }
                    // TODO: Close and remove the client route from the change_router.
                    break;
                }
                _ => {
                    tracing::debug!(">>> {who} sent unsolicited message: {msg:?}");
                }
            }
        }
    });
}

use futures::stream::Stream;

async fn handle_notify(
    change_router: ChangeRouter,
    change_stream: impl Stream<Item = Result<Payload, sqlx::Error>>,
) {
    use futures::StreamExt;
    futures::pin_mut!(change_stream);

    loop {
        match change_stream.next().await {
            Some(Ok(payload)) => {
                // First, move any new routes into the routes list.
                // Doing this reduces contention while inserting new routes
                // and fanning out to all routes below.
                change_router
                    .routes
                    .lock()
                    .await
                    .append(change_router.new_routes.lock().await.borrow_mut());

                let payload = Message::Text(serde_json::to_string(&payload).unwrap());
                let mut routes = change_router.routes.lock().await;
                tracing::debug!(
                    "Notifying {} clients ({}) with: {:?}",
                    routes.len(),
                    routes
                        .iter()
                        .map(|cr| &*cr.who)
                        .collect::<Vec<&str>>()
                        .join(", "),
                    payload
                );

                // Route the notification to all clients.
                // Along the way, prune any dead clients.
                // TODO: Doing this serially won't scale to many clients.
                let mut i = 0;
                while i < routes.len() {
                    let client_route = routes[i].borrow_mut();

                    tracing::debug!("Notifying client {}", client_route.who);
                    use futures::sink::SinkExt;
                    match client_route.sink.send(payload.clone()).await {
                        Ok(_) => {
                            tracing::debug!("Sent to {}", client_route.who);
                            i += 1;
                        }
                        Err(e) => {
                            tracing::debug!(
                                "Failed to send to {}, removing route: {}",
                                client_route.who,
                                e
                            );
                            // TODO: Only remove the route when the error indicates closure.
                            routes.remove(i);
                        }
                    }
                }
            }
            None => {
                tracing::debug!("Got None from notify stream; continuing");
                continue;
            }
            Some(Err(e)) => {
                // TODO: Make sure all errors should be fatal and prevent the notifier from running further.
                // On termination, the error `attempted to acquire a connection on a closed pool` is seen here.
                tracing::debug!("Finishing notifier with error: {}", e);
                break;
            }
        }
    }
}
