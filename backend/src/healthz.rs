use crate::api::ApiResult;
use axum::{Json, Router, routing::get};

#[derive(serde::Serialize)]
struct Healthz {
    status: String,
}

pub(super) fn router() -> Router {
    Router::new().route("/", get(healthz_handler))
}

async fn healthz_handler() -> ApiResult<Json<Healthz>> {
    Ok(Json(Healthz {
        status: "ok".to_string(),
    }))
}
