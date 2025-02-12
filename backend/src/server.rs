use crate::{
    api::{
        self,
        collab::Collab,
        google::{self, KeySet},
    },
    healthz,
    plugins::{
        github::{self},
        PluginSettings,
    },
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
    trace::{MakeSpan, OnRequest},
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
    pub plugin_settings: Option<PluginSettings>,
}

#[tracing::instrument(skip(config))]
pub async fn start_main_server(config: Config) -> (SocketAddr, JoinHandle<()>) {
    let pool = match config.pool {
        Some(pool) => pool,
        None => {
            // Connect to the Postgres database.
            let db_connection_str = std::env::var("DATABASE_URL").expect(
                "DATABASE_URL env variable is unset. Try DATABASE_URL=postgresql://localhost/koso",
            );
            tracing::info!("Connecting to database: {}", db_connection_str);

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
                    .expect("Can't connect to database"),
            ))
        }
    };

    let collab = Collab::new(pool).expect("Failed to init collab");
    let key_set = match config.key_set {
        Some(key_set) => key_set,
        None => google::KeySet::new().await.unwrap(),
    };

    let github_plugin = github::Plugin::new(
        config.plugin_settings.unwrap_or_default(),
        collab.clone(),
        pool,
    )
    .await
    .unwrap();
    let github_poll_handle = github_plugin.start_polling();

    let app = Router::new()
        .nest("/api", api::router().fallback(api::handler_404))
        // Apply these layers only to /api routes.
        .layer((middleware::from_fn(google::authenticate),))
        // NOTE: the following routes are not subject to the
        // google authentication middleware above.
        .nest("/healthz", healthz::router())
        .nest("/plugins/github", github_plugin.router().unwrap())
        // Apply these layers to all non-static routes.
        .layer((
            Extension(pool),
            Extension(collab.clone()),
            Extension(key_set),
            middleware::from_fn(emit_request_metrics),
            SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid),
            PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")),
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http()
                .make_span_with(KosoMakeSpan {})
                .on_request(KosoOnRequest {}),
        ))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new("static")
                        .precompressed_gzip()
                        .precompressed_br()
                        .fallback(ServeFile::new("static/index.html")),
                ),
        )
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        .layer((TimeoutLayer::new(Duration::from_secs(10)),));

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
    let serve = tokio::spawn(async move {
        tracing::info!("server listening on {}", listener.local_addr().unwrap());
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal("koso server", config.shutdown_signal))
        .await
        .unwrap();

        // Now that the server is shutdown, it's safe to clean things up.
        github_poll_handle.abort();
        collab.stop().await;
        tracing::info!("Closing database pool...");
        pool.close().await;
        tracing::info!("Database pool closed.");
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
        tracing::info!("Terminating {name} with ctrl-c...");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        tracing::info!("Terminating {name}...");
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
            request_id = request_id,
        )
    }
}

#[derive(Clone, Debug)]
pub struct KosoOnRequest {}

impl<B> OnRequest<B> for KosoOnRequest {
    fn on_request(&mut self, request: &Request<B>, _: &Span) {
        if let Some(client_version) = request
            .headers()
            .get("koso-client-version")
            .map(|h| h.to_str().unwrap_or("INVALID"))
            .or_else(|| version_from_ws_header(request))
        {
            tracing::event!(
                tracing::Level::DEBUG,
                http_version = ?request.version(),
                client_version = client_version,
                "started processing request",
            );
        } else {
            tracing::event!(
                tracing::Level::DEBUG,
                http_version = ?request.version(),
                "started processing request",
            );
        }
    }
}

fn version_from_ws_header<B>(request: &Request<B>) -> Option<&str> {
    let Some(Ok(swp_header)) = request
        .headers()
        .get("sec-websocket-protocol")
        .map(|h| h.to_str())
    else {
        return None;
    };

    // Search the comma separated parts for "koso-client-version"
    // and return the subsequent part containing the version value.
    let mut iter = swp_header.split(", ");
    loop {
        match iter.next() {
            None => break None,
            Some("koso-client-version") => break iter.next(),
            Some(_) => {
                continue;
            }
        }
    }
}

// Built frontend files in /_app/immutable/ are immutable and never change.
// Allow them to be cached as such.
async fn set_static_cache_control(request: Request, next: Next) -> Response {
    let header = if request.uri().path().starts_with("/_app/immutable/") {
        "public, immutable, max-age=31536000"
    } else if request.uri().path() == "/robots.txt" || request.uri().path() == "/favicon.svg" {
        "public, max-age=345600, stale-while-revalidate=345600"
    } else {
        "public, max-age=3600, stale-while-revalidate=3600"
    };

    let mut response = next.run(request).await;
    if response.status().is_success() {
        response.headers_mut().insert(
            reqwest::header::CACHE_CONTROL,
            HeaderValue::from_static(header),
        );
    }
    response
}
