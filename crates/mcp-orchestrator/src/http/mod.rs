pub mod mcp;

pub mod well_known;
pub mod fallback;
pub mod index;
pub mod oauth;
pub mod statics;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(index::handler))
        .route("/health", get(health_handler))
        .nest("/mcp", mcp::router(state))
        .nest("/oauth", oauth::router(state))
        .nest("/statics", statics::router(state))
        .nest("/.well-known", well_known::router(state))
}

async fn health_handler() -> &'static str {
    "OK"
}
