use axum::{routing::get, Json, Router};

use crate::api::ApiResult;

#[derive(serde::Serialize)]
struct Healthz {
    status: String,
}

pub(super) fn healthz_router() -> Router {
    Router::new().route("/", get(healthz_handler))
}

async fn healthz_handler() -> ApiResult<Json<Healthz>> {
    Ok(Json(Healthz {
        status: "ok".to_string(),
    }))
}
