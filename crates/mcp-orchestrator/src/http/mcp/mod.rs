use axum::{Router, routing::get};

mod delete_namespace_name;
mod get_namespace_name;
mod post_namespace_name;
pub(crate) mod utils;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/{namespace}/{name}",
        get(get_namespace_name::handler)
            .post(post_namespace_name::handler)
            .delete(delete_namespace_name::handler),
    )
}
