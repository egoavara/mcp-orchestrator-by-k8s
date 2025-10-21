use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::{
    failure::Failure,
    request::CreateMcpPodRequest,
    state::AppState,
};
use crate::service::mcp_pod_service::McpPodService;

pub async fn handler(
    State(state): State<AppState>,
    Json(req): Json<CreateMcpPodRequest>,
) -> Result<impl IntoResponse, Failure> {
    let service = McpPodService::new(state.kube_client);
    let response = service.create_pod(req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
