use axum::{extract::State, http::StatusCode, response::IntoResponse};
use common::{failure::Failure, state::AppState};

pub async fn handler(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, Failure> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}
