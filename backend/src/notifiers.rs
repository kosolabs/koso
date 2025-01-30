use axum::Router;

pub(crate) mod telegram;

pub(crate) fn router() -> Router {
    Router::new().nest("/telegram", telegram::router())
}
