pub mod handlers;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn create_http_router() -> Router<AppState> {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler() -> &'static str {
    "OK"
}
