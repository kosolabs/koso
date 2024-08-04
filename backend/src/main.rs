use anyhow::Result;
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade, MatchedPath, Path, Request},
    http::{HeaderName, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use axum_extra::{headers, TypedHeader};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use futures::FutureExt;
use google::User;
use listenfd::ListenFd;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use model::{Project, ProjectPermission, ProjectUser};
use notify::ProjectId;
use postgres::list_project_users;
use sqlx::{
    postgres::{PgConnectOptions, PgPool, PgPoolOptions},
    ConnectOptions,
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
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::MakeSpan,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{Instrument, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod google;
mod model;
mod notify;
mod postgres;

type ApiResult<T> = Result<T, ErrorResponse>;

struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "koso=debug,tower_http=trace,axum::rejection=trace,sqlx=trace,axum=trace".into()
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
                .route("/projects", post(create_project_handler))
                .route("/projects/:project_id", patch(update_project_handler))
                .route(
                    "/projects/:project_id/permissions",
                    post(add_project_permission_handler),
                )
                .route(
                    "/projects/:project_id/users",
                    get(list_project_users_handler),
                )
                .layer(middleware::from_fn(google::authenticate))
                .fallback(handler_404),
        )
        .nest(
            "/ws",
            Router::new()
                .route("/projects/:project_id", get(ws_handler))
                .layer(middleware::from_fn(google::authenticate))
                .fallback(handler_404),
        )
        .route_layer(middleware::from_fn(emit_request_metrics))
        .with_state(state)
        .layer((
            SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), MakeRequestUuid),
            PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")),
            // Enable request tracing. Must enable `tower_http=debug`
            TraceLayer::new_for_http().make_span_with(KosoMakeSpan {}),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
            Extension(pool),
            Extension(notifier.clone()),
            Extension(certs),
        ))
        .nest_service(
            "/",
            ServeDir::new("static").fallback(ServeFile::new("static/index.html")),
        )
        // This is unreachable as the service above matches all routes.
        .fallback(handler_404);

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

#[tracing::instrument(skip(user, pool))]
async fn list_projects_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<Project>>> {
    let projects = list_projects(&user.email, pool).await?;
    Ok(Json(projects))
}

async fn list_projects(email: &String, pool: &PgPool) -> Result<Vec<Project>> {
    let projects: Vec<Project> = sqlx::query_as(
        "
        SELECT
          project_permissions.project_id,
          projects.name
        FROM project_permissions 
        JOIN projects ON (project_permissions.project_id = projects.id)
        WHERE email = $1",
    )
    .bind(email)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

#[tracing::instrument(skip(user, pool))]
async fn create_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Json(mut project): Json<Project>,
) -> ApiResult<Json<Project>> {
    let projects = list_projects(&user.email, pool).await?;
    if projects.len() >= 5 {
        return Err(bad_request_error(&format!(
            "User has more than 5 projects ({})",
            projects.len()
        )));
    }

    project.project_id = BASE64_URL_SAFE_NO_PAD.encode(Uuid::new_v4());

    let mut txn = pool.begin().await?;
    sqlx::query("INSERT INTO projects (id, name) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(&mut *txn)
        .await?;
    sqlx::query("INSERT INTO project_permissions (project_id, email) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&user.email)
        .execute(&mut *txn)
        .await?;
    txn.commit().await?;

    tracing::debug!(
        "Created project '{}' with id '{}'",
        project.name,
        project.project_id
    );

    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn list_project_users_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Vec<ProjectUser>>> {
    verify_access(pool, user, &project_id).await?;
    let users = list_project_users(pool, &project_id).await?;
    Ok(Json(users))
}

#[tracing::instrument(skip(user, pool))]
async fn update_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(project): Json<Project>,
) -> ApiResult<Json<Project>> {
    verify_access(pool, user, &project_id).await?;

    if project_id != project.project_id {
        return Err(bad_request_error(&format!(
            "Path project id ({project_id} is different than body project id {}",
            project.project_id
        )));
    }

    if project.name.is_empty() {
        return Err(bad_request_error("Project name is empty"));
    }

    sqlx::query("UPDATE projects SET name=$2 WHERE id=$1")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(pool)
        .await?;
    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn add_project_permission_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(permission): Json<ProjectPermission>,
) -> ApiResult<()> {
    verify_access(pool, user, &project_id).await?;

    if project_id != permission.project_id {
        return Err(bad_request_error(&format!(
            "Path project id ({project_id} is different than body project id {}",
            permission.project_id
        )));
    }

    if permission.email.is_empty() {
        return Err(bad_request_error("Permission email is empty"));
    }

    sqlx::query("INSERT INTO project_permissions (project_id, email) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&permission.project_id)
            .bind(&permission.email)
            .execute(pool)
            .await?;
    Ok(())
}

#[tracing::instrument(skip(user, pool))]
async fn login_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<()> {
    if let Err(e) = sqlx::query(
        "
        INSERT INTO users (email, name, picture)
        VALUES ($1, $2, $3)
        ON CONFLICT (email)
        DO UPDATE SET name = EXCLUDED.name, picture = EXCLUDED.picture;",
    )
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.picture)
    .execute(pool)
    .await
    {
        return Err(internal_error(&format!(
            "Failed to upsert user on login: {e}"
        )));
    }
    Ok(())
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
    verify_access(pool, user, &project_id).await?;

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

async fn verify_access(
    pool: &PgPool,
    user: User,
    project_id: &ProjectId,
) -> Result<(), ErrorResponse> {
    if project_id.is_empty() {
        return Err(bad_request_error("Project ID must not be empty"));
    }

    let mut txn = match pool.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            return Err(internal_error(&format!(
                "Failed to check user permission: {e}"
            )))
        }
    };

    let permission: Option<ProjectPermission> = match sqlx::query_as(
        "
        SELECT project_id, email
        FROM project_permissions
        WHERE project_id = $1
          AND email = $2;
        ",
    )
    .bind(project_id)
    .bind(&user.email)
    .fetch_optional(&mut *txn)
    .await
    {
        Ok(permission) => permission,
        Err(e) => {
            return Err(internal_error(&format!(
                "Failed to check user permission: {e}"
            )))
        }
    };

    match permission {
        Some(_) => Ok(()),
        None => Err(unauthorized_error(&format!(
            "User {} is not authorized to access {}",
            user.email, project_id
        ))),
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404! Nothing to see here")
}

fn internal_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

fn unauthorized_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::UNAUTHORIZED, msg)
}

fn bad_request_error(msg: &str) -> ErrorResponse {
    error_response(StatusCode::BAD_REQUEST, msg)
}

fn error_response(code: StatusCode, msg: &str) -> ErrorResponse {
    tracing::error!("Failed: {}: {}", code, msg);
    ErrorResponse {
        code,
        msg: msg.to_string(),
    }
}

struct ErrorResponse {
    code: StatusCode,
    msg: String,
}

/// Converts from ErrorResponse to Response.
impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let msg = if dev_mode() {
            self.msg
        } else {
            // Redact the the error message outside of dev.
            "See server logs for details.".to_string()
        };
        Response::builder()
            .status(self.code)
            .body(Body::from(format!("{}: {}", self.code, msg)))
            .unwrap()
    }
}

/// Converts from boxed Error to ErrorResponse and logs the error.
impl<E> From<E> for ErrorResponse
where
    E: Into<Box<dyn Error>>,
{
    fn from(err: E) -> Self {
        let err = err.into();
        let code = StatusCode::INTERNAL_SERVER_ERROR;
        let msg = format!("{:?}", err);
        tracing::error!("Failed: {}: {}", code, msg);
        ErrorResponse { code, msg }
    }
}

fn dev_mode() -> bool {
    // TODO: Decide on this based on an environment variable or the build.
    true
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
