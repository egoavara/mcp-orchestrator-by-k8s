use axum::{
    routing::{any, get},
    Router,
};
use common::state::AppState;

pub mod api;
pub mod fallback;
pub mod index;
pub mod mcp;
pub mod r#static;

pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/api", api::router())
        .route("/mcp/{session_id}", any(mcp::handler))
        .route("/static/{*path}", get(r#static::handler))
        .route("/", get(index::handler))
        .fallback(fallback::handler)
}
