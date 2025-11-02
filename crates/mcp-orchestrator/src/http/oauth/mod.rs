use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use axum_qs::Qs;
use oidc_auth::{CallbackQuery, RegisterRequest};

use crate::state::AppState;

pub fn router(state: &AppState) -> Router<AppState> {
    let mut router = Router::new();
    if let Some(_oidc_manager) = &state.oidc_manager {
        router = router
            .route("/authorize", get(handler_authorize))
            .route("/callback", get(handler_callback))
            .route("/register", post(handler_register))
    }
    router
}

async fn handler_authorize(
    State(state): State<AppState>,
    cookiejar: CookieJar,
    Qs(query): Qs<serde_json::Map<String, serde_json::Value>>,
) -> impl IntoResponse {
    let oidc_manager = state.oidc_manager.unwrap();
    oidc_manager.authorize(cookiejar, query).await
}

async fn handler_callback(
    State(state): State<AppState>,
    cookiejar: CookieJar,
    query: Query<CallbackQuery>,
) -> impl IntoResponse {
    let oidc_manager = state.oidc_manager.unwrap();
    oidc_manager.callback(cookiejar, query).await
}
#[axum::debug_handler]
async fn handler_register(
    State(state): State<AppState>,
    body: Json<RegisterRequest>,
) -> impl IntoResponse {
    let oidc_manager = state.oidc_manager.unwrap();
    oidc_manager.register(body).await
}
