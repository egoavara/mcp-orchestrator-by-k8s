use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::error::AppError;
use crate::grpc::utils::convert_label_query;
use crate::state::AppState;
use crate::storage::ResourceLimitData;
use crate::storage::scheduling_validation::{validate_node_affinity, validate_node_selector};
use crate::storage::store_authorization::AuthorizationData;
use crate::storage::util_delete::{DeleteOption, DeleteResult};
use crate::storage::util_list::ListOption;
use k8s_openapi::api::core::v1::Affinity;
use std::collections::BTreeMap;

fn from(rl: AuthorizationData) -> AuthorizationResponse {
    AuthorizationResponse {
        namespace: rl.namespace,
        name: rl.name,
        r#type: rl.r#type as i32,
        data: rl.data.to_string(),
        labels: rl.labels,
        created_at: Some(rl.created_at.into()),
        deleted_at: rl.deleted_at.map(|dt| dt.into()),
    }
}

pub async fn create_authorization(
    state: &AppState,
    request: Request<CreateAuthorizationRequest>,
) -> Result<Response<AuthorizationResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.authorization(req.namespace.clone());
    //
    if req.r#type == AuthorizationType::Anonymous as i32 {
        return Err(Status::invalid_argument(
            "Cannot create authorization of type Anonymous",
        ));
    }
    //
    let data = serde_json::from_str(req.data.as_ref().map(AsRef::as_ref).unwrap_or("null"))
        .map_err(|e| Status::invalid_argument(format!("Invalid data format: {}", e)))?;
    let r#type = AuthorizationType::try_from(req.r#type)
        .map_err(|e| Status::invalid_argument(format!("Invalid authorization type: {}", e)))?;
    let auth = store
        .create(&req.name, req.labels.into_iter(), r#type, &data)
        .await
        .map_err(|e| Status::internal(format!("Failed to create resource limit: {}", e)))?;

    Ok(Response::new(from(auth)))
}

pub async fn get_authorization(
    state: &AppState,
    request: Request<GetAuthorizationRequest>,
) -> Result<Response<AuthorizationResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.authorization(req.namespace.clone());
    //

    let rl = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get authorization: {}", e)))?
        .ok_or_else(|| Status::not_found(format!("Authorization {} not found", req.name)))?;

    Ok(Response::new(from(rl)))
}

pub async fn list_authorizations(
    state: &AppState,
    request: Request<ListAuthorizationsRequest>,
) -> Result<Response<ListAuthorizationsResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.authorization(req.namespace.clone());
    //

    let subtype = req
        .r#type
        .map(|t| {
            AuthorizationType::try_from(t)
                .map_err(|e| Status::invalid_argument(format!("Invalid authorization type: {}", e)))
        })
        .transpose()?;
    let label = convert_label_query(req.label.unwrap_or_default());

    let (secrets, continue_token, has_more) = store
        .list(
            subtype,
            label.as_ref(),
            ListOption {
                after: req.after,
                first: req.first,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list authorizations: {}", e)))?;

    let data = secrets.into_iter().map(from).collect::<Vec<_>>();

    Ok(Response::new(ListAuthorizationsResponse {
        data,
        end_cursor: continue_token,
        has_next_page: has_more,
    }))
}

pub async fn delete_authorization(
    state: &AppState,
    request: Request<DeleteAuthorizationRequest>,
) -> Result<Response<DeleteAuthorizationResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.authorization(req.namespace.clone());
    //

    let result = store
        .delete(
            &req.name,
            Some(DeleteOption {
                remove_finalizer: None,
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

    Ok(Response::new(DeleteAuthorizationResponse {
        success,
        message,
    }))
}
