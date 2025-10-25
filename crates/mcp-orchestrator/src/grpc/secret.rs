use proto::mcp::orchestrator::v1::*;
use tonic::{Request, Response, Status};

use crate::error::AppError;
use crate::grpc::utils::convert_label_query;
use crate::state::AppState;
use crate::storage::SecretData;
use crate::storage::util_delete::{DeleteOption, DeleteResult};
use crate::storage::util_list::ListOption;

fn from(secret: SecretData) -> SecretResponse {
    SecretResponse {
        namespace: secret.namespace,
        name: secret.name,
        labels: secret.labels,
        keys: secret
            .raw
            .data
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect(),
        created_at: secret.created_at.to_rfc3339(),
        deleted_at: secret.deleted_at.map(|dt| dt.to_rfc3339()),
    }
}

pub async fn create_secret(
    state: &AppState,
    request: Request<CreateSecretRequest>,
) -> Result<Response<SecretResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.secrets(req.namespace.clone());

    let secret = store
        .create(
            &req.name,
            None,
            req.labels.into_iter(),
            req.data.into_iter(),
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to create secret: {}", e)))?;

    Ok(Response::new(from(secret)))
}

pub async fn get_secret(
    state: &AppState,
    request: Request<GetSecretRequest>,
) -> Result<Response<SecretResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.secrets(req.namespace);

    let secret = store
        .get(&req.name)
        .await
        .map_err(|e| Status::internal(format!("Failed to get secret: {}", e)))?
        .ok_or_else(|| Status::not_found("Secret not found".to_string()))?;

    Ok(Response::new(from(secret)))
}

pub async fn list_secrets(
    state: &AppState,
    request: Request<ListSecretsRequest>,
) -> Result<Response<ListSecretsResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.secrets(req.namespace.clone());

    let label_query = convert_label_query(req.label.unwrap_or_default());

    let (secrets, continue_token, has_more) = store
        .list(
            &label_query,
            ListOption {
                after: req.after,
                first: req.first,
            },
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to list secrets: {}", e)))?;

    let data = secrets.into_iter().map(from).collect::<Vec<_>>();

    Ok(Response::new(ListSecretsResponse {
        data,
        end_cursor: continue_token,
        has_next_page: has_more,
    }))
}

pub async fn update_secret(
    _state: &AppState,
    _request: Request<UpdateSecretRequest>,
) -> Result<Response<SecretResponse>, Status> {
    Err(Status::unimplemented("not yet implemented"))
}

pub async fn delete_secret(
    state: &AppState,
    request: Request<DeleteSecretRequest>,
) -> Result<Response<DeleteSecretResponse>, Status> {
    let req = request.into_inner();
    let store = state.kube_store.secrets(req.namespace.clone());

    let result = store
        .delete(&req.name, Some(DeleteOption::timeout_millis(1500)))
        .await
        .map_err(|e| match e {
            AppError::NotFound(msg) => Status::not_found(msg),
            _ => Status::internal(format!("Failed to delete secret: {}", e)),
        })?;

    let (success, message) = match result {
        DeleteResult::Deleted => (true, "Secret deleted successfully".to_string()),
        DeleteResult::Deleting => (false, "Secret is being deleted".to_string()),
    };

    Ok(Response::new(DeleteSecretResponse { success, message }))
}
