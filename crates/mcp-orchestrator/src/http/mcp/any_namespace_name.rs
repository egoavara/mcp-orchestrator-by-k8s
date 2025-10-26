use std::{convert::Infallible, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::{Path, Request, State},
    http::Response,
    response::IntoResponse,
};
use http_body_util::combinators::BoxBody;
use rmcp::transport::{
    StreamableHttpServerConfig, StreamableHttpService,
    streamable_http_server::session::local::LocalSessionManager,
};
use tonic::IntoRequest;

use crate::podmcp::PodMcpSessionManager;
use crate::state::AppState;

pub async fn handler_namespace_name(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Response<Body>> {
    tracing::debug!("Handling MCP request for {}/{}", namespace, name);
    let Some(mcp_template) = state
        .kube_store
        .mcp_templates(Some(namespace))
        .get(&name)
        .await
        .map_err(|err| err.into_response())?
    else {
        return Err((axum::http::StatusCode::NOT_FOUND, "MCP Template not found").into_response());
    };
    let manager = state.podmcp.session_manager(mcp_template).await;
    let service = StreamableHttpService::new(
        state.podmcp.factory(),
        Arc::new(manager),
        StreamableHttpServerConfig {
            sse_keep_alive: Some(std::time::Duration::from_secs(15)),
            stateful_mode: true,
        },
    );
    Ok(service.handle(request).await)
}
