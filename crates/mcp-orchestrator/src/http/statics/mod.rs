use axum::Router;

use crate::state::AppState;


mod get;

pub fn router() -> Router<AppState> {
    Router::new()
}
