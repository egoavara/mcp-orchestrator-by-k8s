
use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::state::AppState;

pub async fn create_mcp_template(
    state: &AppState,
    request: Request<CreateMcpTemplateRequest>,
) -> Result<Response<McpTemplateResponse>, Status> {
    todo!()
}

pub async fn get_mcp_template(
    state: &AppState,
    request: Request<GetMcpTemplateRequest>,
) -> Result<Response<McpTemplateResponse>, Status> {
    todo!()
}

pub async fn list_mcp_templates(
    state: &AppState,
    request: Request<ListMcpTemplatesRequest>,
) -> Result<Response<ListMcpTemplatesResponse>, Status> {
    todo!()
}

pub async fn delete_mcp_template(
    state: &AppState,
    request: Request<DeleteMcpTemplateRequest>,
) -> Result<Response<DeleteMcpTemplateResponse>, Status> {
    todo!()
}
