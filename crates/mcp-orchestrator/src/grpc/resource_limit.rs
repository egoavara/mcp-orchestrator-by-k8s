use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::grpc::utils::convert_label_query;
use crate::state::AppState;
use crate::storage::ResourceLimitData;
use crate::storage::scheduling_validation::{validate_node_affinity, validate_node_selector};
use crate::storage::util_delete::{DeleteOption, DeleteResult};
use crate::storage::util_list::ListOption;
use k8s_openapi::api::core::v1::Affinity;
use std::collections::BTreeMap;

fn from(rl: ResourceLimitData) -> ResourceLimitResponse {
    let node_selector = rl
        .node_selector
        .unwrap_or_default()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let node_affinity = rl
        .node_affinity
        .and_then(|na| serde_json::to_string(&na).ok());

    ResourceLimitResponse {
        name: rl.name,
        description: rl.description,
        limits: Some(ResourceLimit {
            cpu: rl.cpu,
            memory: rl.memory,
            cpu_limit: rl.cpu_limit,
            memory_limit: rl.memory_limit,
            ephemeral_storage: rl.ephemeral_storage,
            volumes: rl.volumes.into_iter().collect(),
            node_selector,
            node_affinity,
        }),
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

    // Validate node_selector
    if !limits.node_selector.is_empty() {
        let btree: BTreeMap<String, String> = limits
            .node_selector
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        validate_node_selector(&btree)
            .map_err(|e| Status::invalid_argument(format!("Invalid node_selector: {}", e)))?;
    }

    // Validate node_affinity
    if let Some(ref json_str) = limits.node_affinity {
        let affinity: Affinity = serde_json::from_str(json_str)
            .map_err(|e| Status::invalid_argument(format!("Invalid node_affinity JSON: {}", e)))?;

        validate_node_affinity(&affinity)
            .map_err(|e| Status::invalid_argument(format!("Invalid node_affinity: {}", e)))?;
    }

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

    let is_deletable = store
        .is_deletable(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to check deletability: {}", e)))?;

    if !is_deletable && !req.force {
        return Err(Status::failed_precondition(
            "ResourceLimit is in use by MCP templates. Use force=true to delete anyway.",
        ));
    }

    let result = store
        .delete(
            &req.name,
            Some(DeleteOption {
                remove_finalizer: Some(req.force),
                timeout: None,
            }),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to delete resource limit: {}", e)))?;

    let (success, message) = match result {
        DeleteResult::Deleted => (true, "ResourceLimit deleted successfully".to_string()),
        DeleteResult::Deleting => (
            true,
            "ResourceLimit is being deleted (finalizers pending)".to_string(),
        ),
    };

    Ok(Response::new(DeleteResourceLimitResponse {
        success,
        message,
    }))
}
