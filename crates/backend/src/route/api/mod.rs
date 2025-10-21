use axum::Router;
use common::state::AppState;

pub mod mcp;

pub fn router() -> Router<AppState> {
    axum::Router::new().nest("/mcp", mcp::router())
}
