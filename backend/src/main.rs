use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use axum_extra::{headers, TypedHeader};
use listenfd::ListenFd;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::ConnectOptions;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeFile;
use tower_http::timeout::TimeoutLayer;
use uuid::Uuid;

use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use std::{
    future::ready,
    time::{Duration, Instant},
};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum_streams::StreamBodyAsOptions;

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

#[derive(serde::Serialize)]
struct Task {
    id: String,
    name: String,
}

#[derive(serde::Deserialize)]

struct NewTask {
    name: String,
}

struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "yotei=trace,tower_http=trace,axum::rejection=trace,sqlx=trace".into()
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

    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/task/list", get(list_tasks))
        .route("/task/create", post(create_task))
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
        .filter_map(|t: Result<(String, String), sqlx::Error>| t.ok())
        .map(|t| Task { id: t.0, name: t.1 });

    StreamBodyAsOptions::new()
        .buffering_ready_items(5)
        .json_array(tasks)
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
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    tracing::debug!("opening web socket with client {addr}");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if !socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        tracing::debug!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }
    tracing::debug!("Pinged {who}...");

    // receive single message from a client (we can either receive or send with socket).
    // this will likely be the Pong for our Ping or a hello message from client.
    // waiting for message from a client will block this task, but will not block other client's
    // connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return;
            }
        } else {
            tracing::debug!("client {who} abruptly disconnected");
            return;
        }
    }

    // Since each client gets individual statemachine, we can pause handling
    // when necessary to wait for some external event (in this case illustrated by sleeping).
    // Waiting for this client to finish getting its greetings does not prevent other clients from
    // connecting to server and receiving their greetings.
    for i in 1..5 {
        tracing::debug!("sending 'Hi {i} times!' to client {who}");
        if socket
            .send(Message::Text(format!("Hi {i} times!")))
            .await
            .is_err()
        {
            tracing::debug!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    use futures::{sink::SinkExt, stream::StreamExt};
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            tracing::debug!("sending 'Server message {i} ...' to client {who}");
            if sender
                .send(Message::Text(format!("Server message {i} ...")))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        tracing::debug!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Cow::from("Goodbye"),
            })))
            .await
        {
            tracing::debug!("Could not send Close due to {e}, probably it is ok?");
        }
        n_msg
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => tracing::debug!("{a} messages sent to {who}"),
                Err(a) => tracing::debug!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => tracing::debug!("Received {b} messages"),
                Err(b) => tracing::debug!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    tracing::debug!("Websocket context {who} destroyed");
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            tracing::debug!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            tracing::debug!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who,
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::debug!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            tracing::debug!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            tracing::debug!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
