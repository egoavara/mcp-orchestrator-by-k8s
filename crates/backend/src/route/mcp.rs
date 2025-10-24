use crate::{passmcp::PassthroughMcpService, state::AppState};
use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::{Html, IntoResponse},
};
use rmcp::transport::StreamableHttpService;
use tokio::runtime::Handle;

pub async fn handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    request: Request<Body>,
) -> impl IntoResponse {
    let mcp_service = StreamableHttpService::new(
        move || {
            let kube_client = state.kube_client.clone();
            let session_id = session_id.clone();
            let client = tokio::task::block_in_place(move || {
                Handle::current().block_on(async move {
                    PassthroughMcpService::new(session_id, kube_client).await
                })
            })
            .unwrap();
            Ok(client)
        },
        state.local_session_manager.clone(),
        Default::default(),
    );
    mcp_service.handle(request).await
}
