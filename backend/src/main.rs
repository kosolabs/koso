use crate::ApiResult::AErr;
use crate::ApiResult::AOk;
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade, MatchedPath, Path, Request},
    http::{HeaderName, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures::FutureExt;
use google::{Certs, Claims};
use jsonwebtoken::Validation;
use listenfd::ListenFd;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use model::Project;
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    ConnectOptions,
};
use std::{
    error::Error,
    fmt::{self, Display},
    future::ready,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, signal};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod google;
mod model;
mod notify;

// type ApiResult<T> = Result<T, ErrorResponse>;

pub enum ApiResult<T, E = ErrorResponse> {
    AOk(T),

    /// Contains the error value
    AErr(E),
}

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
        .nest(
            "/api",
            Router::new()
                .route("/auth/login", post(login_handler))
                .route("/projects", get(list_projects_handler))
                .fallback(handler_404),
        )
        .nest(
            "/ws",
            Router::new()
                .route("/projects/:project_id", get(ws_handler))
                .fallback(handler_404),
        )
        // IMPORTANT - any routes subsequent to the auth layer allow
        // unauthenticated access. e.g. static content.
        .layer(middleware::from_fn(authenticate))
        .nest_service(
            "/",
            ServeDir::new("static").fallback(ServeFile::new("static/index.html")),
        )
        // This is unreachable as the service above matches all routes.
        .fallback(handler_404)
        .route_layer(middleware::from_fn(emit_request_metrics))
        .with_state(state)
        .layer((
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
            SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid),
            PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")),
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

#[tracing::instrument(skip(request, next), fields(email))]
async fn authenticate(mut request: Request, next: Next) -> ApiResult<Response<Body>> {
    let certs = request.extensions().get::<Certs>().unwrap();
    let headers = request.headers();

    // return Err(internal_error("HACKINGGGGG Failure"));

    let bearer = if let Some(auth_header) = headers.get("Authorization") {
        let Ok(auth) = auth_header.to_str() else {
            return AErr(unauthorized_error(format!(
                "Could not convert auth header to string: {auth_header:?}"
            )));
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return AErr(unauthorized_error(format!(
                "Could not split bearer parts: {parts:?}"
            )));
        }
        parts[1]
    } else if let Some(swp_header) = headers.get("sec-websocket-protocol") {
        let Ok(swp) = swp_header.to_str() else {
            return AErr(unauthorized_error(format!(
                "sec-websocket-protocol must be only visible ASCII chars: {swp_header:?}"
            )));
        };
        let parts: Vec<&str> = swp.split(", ").collect();
        if parts.len() != 2 || parts[0] != "bearer" {
            return AErr(unauthorized_error(format!(
                "sec-websocket-protocol must contain a bearer token: {parts:?}"
            )));
        }
        parts[1]
    } else {
        return AErr(unauthorized_error("Authorization header is absent."));
    };

    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return AErr(unauthorized_error(format!(
            "Could not decode header: {bearer:?}"
        )));
    };
    let Some(kid) = header.kid else {
        return AErr(unauthorized_error(format!(
            "header.kid is absent: {header:?}"
        )));
    };
    let key = match certs.get(&kid) {
        Ok(key) => key,
        Err(e) => {
            return AErr(unauthorized_error(format!(
                "certs is absent for {kid:?}: {e}"
            )));
        }
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    ]);
    validation.set_issuer(&["https://accounts.google.com"]);
    let token = match jsonwebtoken::decode::<Claims>(bearer, &key, &validation) {
        Ok(token) => token,
        Err(e) => {
            return AErr(unauthorized_error(format!("Failed validation: {e}")));
        }
    };

    tracing::Span::current().record("email", token.claims.email.clone());
    assert!(request.extensions_mut().insert(token.claims).is_none());

    AOk(next.run(request).await)
}

#[tracing::instrument(skip(claims, pool))]
async fn list_projects_handler(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<Project>>> {
    let tasks: Vec<Project> = sqlx::query_as(
        "
        SELECT
          project_permissions.project_id,
          projects.name
        FROM project_permissions 
        JOIN projects ON (project_permissions.project_id = projects.id)
        WHERE email = $1",
    )
    .bind(claims.email)
    .fetch_all(pool)
    .await?;
    AOk(Json(tasks))
}

#[tracing::instrument(skip(claims, pool))]
async fn login_handler(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<()> {
    if let Err(e) = sqlx::query(
        "
        INSERT INTO users (email, name, picture)
        VALUES ($1, $2, $3)",
    )
    .bind(&claims.email)
    .bind(&claims.name)
    .bind(&claims.picture)
    .execute(pool)
    .await
    {
        return AErr(internal_error(format!(
            "Failed to upsert user on login: {e}"
        )));
    }
    AOk(())
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
#[tracing::instrument(skip(ws, project_id, _user_agent, addr, notifier))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(notifier): Extension<notify::Notifier>,
) -> ApiResult<Response<Body>> {
    if project_id.is_empty() {
        return AErr(internal_error("projects segment must not be empty"));
    }

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let cs = tracing::Span::current();
    AOk(ws.protocols(["bearer"]).on_upgrade(move |socket| {
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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

fn internal_error<T: std::fmt::Debug>(msg: T) -> ErrorResponse {
    ErrorResponse(
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("{:?}", msg)))
            .unwrap(),
    )
}

fn unauthorized_error<T: std::fmt::Debug>(msg: T) -> ErrorResponse {
    ErrorResponse(
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(format!("{:?}", msg)))
            .unwrap(),
    )
}

fn bad_request_error<T: std::fmt::Debug>(msg: T) -> ErrorResponse {
    ErrorResponse(
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!("{:?}", msg)))
            .unwrap(),
    )
}

impl<T> IntoResponse for ApiResult<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            AOk(ok) => ok.into_response(),
            AErr(err) => err.0,
        }
    }
}

impl std::ops::FromResidual for ApiResult<T> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        todo!()
    }
}

pub struct ErrorResponse(Response);

impl<T> From<T> for ErrorResponse
where
    T: IntoResponse,
{
    fn from(value: T) -> Self {
        Self(value.into_response())
    }
}

// impl<E> From<E> for Response<Body>
// where
//     E: Into<Box<dyn Error>>,
// {
//     fn from(e: E) -> Self {
//         internal_error(format!("{:?}", e.into()))
//     }
// }

// struct AppError {
//     error: String,
//     code: StatusCode,
// }

// // Tell axum how to convert `AppError` into a response.
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         match self.code {
//             StatusCode::INTERNAL_SERVER_ERROR => tracing::error!("{} -- {}", self.code, self.error),
//             _ => tracing::warn!("{} -- {}", self.code, self.error),
//         }
//         if dev_mode() {
//             (self.code, format!("{} -- {}", self.code, self.error)).into_response()
//         } else {
//             (self.code, format!("{}", self.code)).into_response()
//         }
//     }
// }

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
// impl<E> From<E> for Response<String>
// where
//     E: Into<Box<dyn Error>>,
// {
//     fn from(err: E) -> Self {
//         Response::builder()
//             .status(StatusCode::INTERNAL_SERVER_ERROR)
//             .body(format!("{}", err.into()))
//             .unwrap()
//     }
// }

// impl<E> From<E> for AppError
// where
//     E: Into<(String, StatusCode)>,
// {
//     fn from(t: E) -> Self {
//         let tt: (String, StatusCode) = t.into();
//         AppError {
//             error: tt.0,
//             code: tt.1,
//         }
//     }
// }

fn dev_mode() -> bool {
    true // TODO
}
