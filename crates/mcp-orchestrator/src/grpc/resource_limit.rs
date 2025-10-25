use kube::ResourceExt;
use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::error::AppError;
use crate::grpc::utils::convert_label_query;
use crate::state::AppState;
use crate::storage::ResourceLimitData;
use crate::storage::util_delete::{DeleteOption, DeleteResult};
use crate::storage::util_list::ListOption;

fn from(rl: ResourceLimitData) -> ResourceLimitResponse {
    ResourceLimitResponse {
        name: rl.name,
        description: rl.description,
        limits: Some(rl.limits),
        labels: rl.labels,
        created_at: rl.created_at.to_rfc3339(),
        deleted_at: rl.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

pub async fn create_resource_limit(
    state: &AppState,
    request: Request<CreateResourceLimitRequest>,
) -> Result<Response<ResourceLimitResponse>, Status> {
    let store = state.kube_store.resource_limits();
    let req = request.into_inner();

    let limits = req
        .limits
        .ok_or_else(|| Status::invalid_argument("limits field is required"))?;

    let cm = store
        .create(&req.name, req.labels.into_iter(), &req.description, &limits)
        .await
        .map_err(|e| Status::internal(format!("Failed to create resource limit: {}", e)))?;

    Ok(Response::new(from(cm)))
}

pub async fn get_resource_limit(
    state: &AppState,
    request: Request<GetResourceLimitRequest>,
) -> Result<Response<ResourceLimitResponse>, Status> {
    let store = state.kube_store.resource_limits();
    let req = request.into_inner();

    let rl = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get resource limit: {}", e)))?
        .ok_or_else(|| Status::not_found(format!("ResourceLimit {} not found", req.name)))?;

    Ok(Response::new(from(rl)))
}

pub async fn list_resource_limits(
    state: &AppState,
    request: Request<ListResourceLimitsRequest>,
) -> Result<Response<ListResourceLimitsResponse>, Status> {
    let store = state.kube_store.resource_limits();
    let req = request.into_inner();

    let label = convert_label_query(req.label.unwrap_or_default());

    let (secrets, continue_token, has_more) = store
        .list(
            label.as_ref(),
            ListOption {
                after: req.after,
                first: req.first,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list resource limits: {}", e)))?;

    let data = secrets.into_iter().map(from).collect::<Vec<_>>();

    Ok(Response::new(ListResourceLimitsResponse {
        data,
        end_cursor: continue_token,
        has_next_page: has_more,
    }))
}

pub async fn delete_resource_limit(
    state: &AppState,
    request: Request<DeleteResourceLimitRequest>,
) -> Result<Response<DeleteResourceLimitResponse>, Status> {
    let store = state.kube_store.resource_limits();
    let req = request.into_inner();

    let result = store
        .delete(&req.name, Some(DeleteOption::timeout_millis(1500)))
        .await
        .map_err(|e| match e {
            AppError::NotFound(msg) => Status::not_found(msg),
            AppError::Conflict(msg) => Status::failed_precondition(msg),
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
    Ok(Response::new(DeleteResourceLimitResponse {
        success,
        message,
    }))
}
