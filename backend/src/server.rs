use crate::api::{
    self,
    collab::Collab,
    google::{self, KeySet},
};
use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderName, HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Extension, Router,
};
use listenfd::ListenFd;
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    ConnectOptions,
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, signal, sync::oneshot::Receiver, task::JoinHandle};
use tower::builder::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::MakeSpan,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{Level, Span};

#[derive(Default)]
pub struct Config {
    pub pool: Option<&'static PgPool>,
    pub port: Option<u16>,
    pub shutdown_signal: Option<Receiver<()>>,
    pub key_set: Option<KeySet>,
}

#[tracing::instrument(skip(config))]
pub async fn start_main_server(config: Config) -> (SocketAddr, JoinHandle<()>) {
    let pool = match config.pool {
        Some(pool) => pool,
        None => {
            // Connect to the Postgres database.
            let db_connection_str = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost".to_string());
            tracing::debug!("Connecting to database: {}", db_connection_str);

            Box::leak(Box::new(
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
            ))
        }
    };

    let collab = Collab::new(pool);
    let key_set = match config.key_set {
        Some(key_set) => key_set,
        None => google::KeySet::new().await.unwrap(),
    };

    let app = Router::new()
        .nest("/api", api::api_router().fallback(api::handler_404))
        .layer((
            middleware::from_fn(emit_request_metrics),
            SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid),
            PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")),
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http().make_span_with(KosoMakeSpan {}),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            Extension(pool),
            Extension(collab.clone()),
            Extension(key_set),
            middleware::from_fn(google::authenticate),
        ))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_immutable_cache_control))
                .service(ServeDir::new("static").fallback(ServeFile::new("static/index.html"))),
        );

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
        None => {
            let port = config.port.unwrap_or(3000);
            TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap()
        }
    };

    let addr = listener.local_addr().unwrap();
    let serve = tokio::spawn(async {
        tracing::debug!("server listening on {}", listener.local_addr().unwrap());
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal("server", config.shutdown_signal))
        .await
        .unwrap();

        // Now that the server is shutdown, it's safe to clean things up.
        collab.stop().await;
        tracing::debug!("Closing database pool...");
        pool.close().await;
    });

    return (addr, serve);
}

// Completion of this function signals to a server,
// via graceful_shutdown, to begin shutdown.
// As such, avoid doing cleanup work here.
pub(super) async fn shutdown_signal(name: &str, signal: Option<Receiver<()>>) {
    if let Some(signal) = signal {
        if let Err(e) = signal.await {
            tracing::error!("Reading shutdown signal failed: {e:?}");
        }
        return;
    }

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

async fn emit_request_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        "404_UNMATCHED_PATH".to_string()
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

#[derive(Clone)]
struct KosoMakeSpan {}

/// Forked from tracing's DefaultMakeSpan in order to add request_id
impl<B> MakeSpan<B> for KosoMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .extensions()
            .get::<RequestId>()
            .map(|h| h.header_value().to_str().unwrap_or("INVALID"))
            .unwrap_or("MISSING");

        tracing::span!(
            Level::DEBUG,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = request_id,
        )
    }
}

// Built frontend files in /_app/immutable/ are immutable and never change.
// Allow them to be cached as such.
async fn set_immutable_cache_control(request: Request, next: Next) -> Response {
    let is_immutable = request.uri().path().starts_with("/_app/immutable/");
    let mut response = next.run(request).await;
    if is_immutable && response.status().is_success() {
        response.headers_mut().insert(
            reqwest::header::CACHE_CONTROL,
            HeaderValue::from_static("public, immutable, max-age=31536000"),
        );
    }
    response
}
