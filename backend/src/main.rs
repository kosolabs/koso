use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeFile;
use uuid::Uuid;

#[derive(Clone, serde::Serialize)]
struct Task {
    id: String,
    name: String,
}

#[derive(serde::Deserialize)]

struct NewTask {
    name: String,
}

struct TaskStore {
    tasks: Vec<Task>,
}

struct AppState {
    task_store: Mutex<TaskStore>,
}

#[tokio::main]
async fn main() {
    let store = Arc::new(AppState {
        task_store: Mutex::new(TaskStore { tasks: Vec::new() }),
    });
    let app = Router::new()
        .route("/task/list", get(list_tasks))
        .route("/task/create", post(create_task))
        .route_service("/", ServeFile::new("assets/index.html"))
        .with_state(store);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
    state.task_store.lock().unwrap().tasks.push(task.clone());
    Ok(Json(task))
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    Ok(Json(state.task_store.lock().unwrap().tasks.to_vec()))
}
