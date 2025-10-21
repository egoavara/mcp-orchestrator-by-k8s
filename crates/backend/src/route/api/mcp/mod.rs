use axum::{routing::get, Router};
use common::state::AppState;

pub mod delete_id;
pub mod get;
pub mod get_id;
pub mod post;

pub fn router() -> Router<AppState> {
    axum::Router::new()
        .route("/", get(get::handler).post(post::handler))
        .route("/{id}", get(get_id::handler).delete(delete_id::handler))
}
