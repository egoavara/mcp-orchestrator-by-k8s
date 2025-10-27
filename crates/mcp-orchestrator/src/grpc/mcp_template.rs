use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::{
    error::AppError,
    grpc::utils::convert_label_query,
    state::AppState,
    storage::{
        McpTemplateCreate, McpTemplateData,
        util_delete::{DeleteOption, DeleteResult},
        util_list::ListOption,
    },
};

fn from(rl: McpTemplateData) -> McpTemplateResponse {
    McpTemplateResponse {
        namespace: rl.namespace,
        name: rl.name,
        labels: rl.labels,
        image: rl.image,
        command: rl.command,
        args: rl.args,
        envs: rl.envs,
        secret_envs: rl.secret_envs,
        resource_limit_name: rl.resource_limit_name,
        volume_mounts: rl.volume_mounts,
        secret_mounts: rl.secret_mounts,
        created_at: rl.created_at.to_rfc3339(),
        deleted_at: rl.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

pub async fn create_mcp_template(
    state: &AppState,
    request: Request<CreateMcpTemplateRequest>,
) -> Result<Response<McpTemplateResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.mcp_templates(req.namespace.clone());
    tracing::info!("Creating MCP template: {:?}", req.name);
    tracing::debug!("MCP template request: {:?}", req);

    let mt = store
        .create(
            &req.name,
            req.labels.into_iter(),
            McpTemplateCreate {
                image: req.image,
                command: req.command,
                args: req.args,
                envs: req.envs,
                secret_envs: req.secret_envs,
                resource_limit_name: req.resource_limit_name,
                volume_mounts: req.volume_mounts,
                secret_mounts: req.secret_mounts,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to create MCP template: {}", e)))?;

    Ok(Response::new(from(mt)))
}

pub async fn get_mcp_template(
    state: &AppState,
    request: Request<GetMcpTemplateRequest>,
) -> Result<Response<McpTemplateResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.mcp_templates(req.namespace.clone());

    let mt = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get MCP template: {}", e)))?
        .ok_or_else(|| Status::not_found("MCP template not found".to_string()))?;

    Ok(Response::new(from(mt)))
}

pub async fn list_mcp_templates(
    state: &AppState,
    request: Request<ListMcpTemplatesRequest>,
) -> Result<Response<ListMcpTemplatesResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.mcp_templates(req.namespace.clone());

    let label = convert_label_query(req.label.unwrap_or_default());
    let (responses, continue_token, has_more) = store
        .list(
            label.as_ref(),
            ListOption {
                after: req.after,
                first: req.first,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list resource limits: {}", e)))?;

    let data = responses.into_iter().map(from).collect::<Vec<_>>();

    Ok(Response::new(ListMcpTemplatesResponse {
        data,
        end_cursor: continue_token,
        has_next_page: has_more,
    }))
}

pub async fn delete_mcp_template(
    state: &AppState,
    request: Request<DeleteMcpTemplateRequest>,
) -> Result<Response<DeleteMcpTemplateResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.mcp_templates(req.namespace.clone());

    let result = store
        .delete(&req.name, Some(DeleteOption::timeout_millis(1500)))
        .await
        .map_err(|e| match e {
            AppError::NotFound(msg) => Status::not_found(msg),
            _ => Status::internal(format!("Failed to delete resource limit: {}", e)),
        })?;

    let (success, message) = match result {
        DeleteResult::Deleted => (
            true,
            format!("ResourceLimit {} deleted successfully", req.name),
        ),
        DeleteResult::Deleting => (
            false,
            format!("ResourceLimit {} is being deleted", req.name),
        ),
    };
    Ok(Response::new(DeleteMcpTemplateResponse {
        success,
        message,
    }))
}
