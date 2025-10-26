pub mod mcp;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/mcp", mcp::router())
        .route("/health", get(health_handler))
}

async fn health_handler() -> &'static str {
    "OK"
}
