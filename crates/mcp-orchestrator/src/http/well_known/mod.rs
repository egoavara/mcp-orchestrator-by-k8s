use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};

use crate::state::AppState;

pub fn router(state: &AppState) -> Router<AppState> {
    let mut router = Router::new();
    if let Some(oidc_manager) = &state.oidc_manager {
        router = router
            .route("/oauth-protected-resource", get(protected_resource_handler))
            .route(
                "/oauth-authorization-server",
                get(authorization_server_handler),
            );
    }
    router
}

async fn protected_resource_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.oidc_manager.unwrap().protected_resource())
}

async fn authorization_server_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.oidc_manager.unwrap().authorization_server())
}
