
use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::error::AppError;
use crate::grpc::utils::convert_label_query;
use crate::state::AppState;
use crate::storage::NamespaceData;
use crate::storage::util_delete::DeleteOption;

fn from(ns: NamespaceData) -> NamespaceResponse {
    NamespaceResponse {
        name: ns.name,
        labels: ns.labels,
        created_at: ns.created_at.to_rfc3339(),
        deleted_at: ns.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

pub async fn create_namespace(
    state: &AppState,
    request: Request<CreateNamespaceRequest>,
) -> Result<Response<NamespaceResponse>, Status> {
    let store = state.kube_store.namespaces();
    let req = request.into_inner();

    let ns = store
        .create(&req.name, req.labels.into_iter())
        .await
        .map_err(|e| Status::internal(format!("Failed to create namespace: {}", e)))?;

    Ok(Response::new(from(ns)))
}

pub async fn get_namespace(
    state: &AppState,
    request: Request<GetNamespaceRequest>,
) -> Result<Response<NamespaceResponse>, Status> {
    let store = state.kube_store.namespaces();
    let req = request.into_inner();

    let ns = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get namespace: {}", e)))?
        .ok_or_else(|| Status::not_found(format!("Namespace {} not found", req.name)))?;

    Ok(Response::new(from(ns)))
}

pub async fn list_namespaces(
    state: &AppState,
    request: Request<ListNamespacesRequest>,
) -> Result<Response<ListNamespacesResponse>, Status> {
    let store = state.kube_store.namespaces();
    let req = request.into_inner();

    let label_query = convert_label_query(req.label.unwrap_or_default());

    let namespaces = store
        .list(label_query.as_ref())
        .await
        .map_err(|e| Status::internal(format!("Failed to list namespaces: {}", e)))?;

    let responses = namespaces.into_iter().map(from).collect::<Vec<_>>();

    let total = responses.len() as i32;

    Ok(Response::new(ListNamespacesResponse {
        namespaces: responses,
        next_page_token: String::new(),
        total_count: total,
    }))
}

pub async fn delete_namespace(
    state: &AppState,
    request: Request<DeleteNamespaceRequest>,
) -> Result<Response<DeleteNamespaceResponse>, Status> {
    let store = state.kube_store.namespaces();
    let req = request.into_inner();

    store
        .delete(&req.name, Some(DeleteOption::timeout_millis(1500)))
        .await
        .map_err(|e| match e {
            AppError::NotFound(msg) => Status::not_found(msg),
            _ => Status::internal(format!("Failed to delete namespace: {}", e)),
        })?;

    Ok(Response::new(DeleteNamespaceResponse {
        success: true,
        message: format!("Namespace {} deleted successfully", req.name),
    }))
}
