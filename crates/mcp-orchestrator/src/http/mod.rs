pub mod mcp;

pub mod fallback;
pub mod index;
pub mod statics;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index::handler))
        .nest("/mcp", mcp::router())
        .nest("/statics", statics::router())
        .route("/health", get(health_handler))
}

async fn health_handler() -> &'static str {
    "OK"
}
