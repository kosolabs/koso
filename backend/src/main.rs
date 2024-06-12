use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::ConnectOptions;
use std::sync::Arc;
use tower_http::services::ServeFile;
use uuid::Uuid;

use std::time::Duration;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, serde::Serialize)]
struct Task {
    id: String,
    name: String,
}

#[derive(serde::Deserialize)]

struct NewTask {
    name: String,
}

struct AppState {
    pool: PgPool,
}

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

    // Connect to the Postgres database.
    let db_connection_str =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost".to_string());
    let mut opts: PgConnectOptions = db_connection_str.parse().unwrap();
    // Enable query trace logging. Must enable `sqlx=trace`
    opts = opts.log_statements(tracing::log::LevelFilter::Trace);
    tracing::debug!("Connecting to database: {}", db_connection_str);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(opts)
        .await
        .expect("can't connect to database");

    let state = Arc::new(AppState { pool: pool });
    let app = Router::new()
        .route("/task/list", get(list_tasks))
        .route("/task/create", post(create_task))
        .route_service("/", ServeFile::new("assets/index.html"))
        .with_state(state)
        // Enable request tracing. Must enable `tower_http=debug`
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(new_task): Json<NewTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let task = Task {
        id: Uuid::new_v4().to_string(),
        name: new_task.name,
    };

    sqlx::query("INSERT INTO tasks(id, name) VALUES ($1, $2);")
        .bind(&task.id)
        .bind(&task.name)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(Json(task))
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let results: Vec<(String, String)> = sqlx::query_as("SELECT id, name from tasks")
        .fetch_all(&state.pool)
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

/// Utility function for mapping any error into a `500 Internal Server Error` response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
