use axum::{Router, routing};

use crate::state::AppState;

mod get;

pub fn router() -> Router<AppState> {
    Router::new().route("/{*path}", routing::get(get::handler))
}
