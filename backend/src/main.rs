use axum::{
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade, MatchedPath, Path, Request},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::FutureExt;
use google::{Certs, Claims};
use jsonwebtoken::Validation;
use listenfd::ListenFd;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    ConnectOptions, Pool, Postgres,
};
use std::{
    error::Error,
    future::ready,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, signal};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod google;
mod model;
mod notify;

struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "koso=trace,tower_http=trace,axum::rejection=trace,sqlx=trace,axum=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (_main_server, _metrics_server) = tokio::join!(start_main_server(), start_metrics_server());
}

#[tracing::instrument()]
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

    let notifier = notify::start(pool);
    let certs = google::fetch().await.unwrap();

    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/api/auth/login", post(login_handler))
        .route("/ws/projects/:project_id", get(ws_handler))
        .nest_service("/", ServeDir::new("static"))
        .fallback_service(ServeFile::new("static/index.html"))
        .route_layer(middleware::from_fn(emit_request_metrics))
        .with_state(state)
        .layer((
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            Extension(pool),
            Extension(notifier.clone()),
            Extension(certs),
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
    notifier.stop().await;
    tracing::debug!("Closing database pool...");
    pool.close().await;
}

async fn login_handler(
    headers: HeaderMap,
    Extension(certs): Extension<Certs>,
    Extension(pool): Extension<&'static PgPool>,
) -> StatusCode {
    let Some(auth_header) = headers.get("Authorization") else {
        return StatusCode::UNAUTHORIZED;
    };
    let Ok(auth) = auth_header.to_str() else {
        return StatusCode::UNAUTHORIZED;
    };
    let parts: Vec<&str> = auth.split(" ").collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return StatusCode::UNAUTHORIZED;
    }
    let bearer = parts[1];
    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return StatusCode::UNAUTHORIZED;
    };
    let Some(kid) = header.kid else {
        return StatusCode::UNAUTHORIZED;
    };
    let Ok(key) = certs.get(&kid) else {
        return StatusCode::UNAUTHORIZED;
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let Ok(token) = jsonwebtoken::decode::<Claims>(bearer, &key, &validation) else {
        return StatusCode::UNAUTHORIZED;
    };

    if let Err(e) = sqlx::query(
        "
        INSERT INTO users (email, name, picture)
        VALUES ($1, $2, $3)
        ON CONFLICT (email)
        DO UPDATE SET name = EXCLUDED.name, picture = EXCLUDED.picture;",
    )
    .bind(&token.claims.email)
    .bind(&token.claims.name)
    .bind(&token.claims.picture)
    .execute(pool)
    .await
    {
        tracing::warn!("Failed: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::NO_CONTENT
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(notifier): Extension<notify::Notifier>,
    Extension(certs): Extension<Certs>,
) -> impl IntoResponse {
    if project_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "projects segment must not be empty",
        )
            .into_response();
    }
    let Some(swp_header) = headers.get("sec-websocket-protocol") else {
        return (
            StatusCode::UNAUTHORIZED,
            "sec-websocket-protocol must be set",
        )
            .into_response();
    };
    let Ok(swp) = swp_header.to_str() else {
        return (
            StatusCode::UNAUTHORIZED,
            "sec-websocket-protocol must be only visible ASCII chars",
        )
            .into_response();
    };
    let parts: Vec<&str> = swp.split(", ").collect();
    if parts.len() != 2 || parts[0] != "bearer" {
        return (
            StatusCode::UNAUTHORIZED,
            "sec-websocket-protocol must contain a bearer token",
        )
            .into_response();
    }
    let bearer = parts[1];
    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return (StatusCode::UNAUTHORIZED, "failed to decode the jwt header").into_response();
    };
    let Some(kid) = header.kid else {
        return (
            StatusCode::UNAUTHORIZED,
            "failed to extract kid from jwt header",
        )
            .into_response();
    };
    let Ok(key) = certs.get(&kid) else {
        return (StatusCode::UNAUTHORIZED, "invalid kid").into_response();
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let Ok(token) = jsonwebtoken::decode::<Claims>(bearer, &key, &validation) else {
        return (StatusCode::UNAUTHORIZED, "failed validation").into_response();
    };
    tracing::debug!("Bearer token: {:?}", token);

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.protocols(["bearer"]).on_upgrade(move |socket| {
        notifier
            .register_client(socket, addr, project_id)
            .map(move |res| {
                if let Err(e) = res {
                    tracing::warn!("Failed to register destination for {addr}: {e}");
                }
            })
    })
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

/// Starts a prometheus metrics server and returns a future that completes on termination.
#[tracing::instrument()]
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

async fn emit_request_metrics(req: Request, next: Next) -> impl IntoResponse {
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
