use axum::{Router, routing::any};

mod any_namespace_name;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/{namespace}/{name}",
        any(any_namespace_name::handler_namespace_name),
    )
}
