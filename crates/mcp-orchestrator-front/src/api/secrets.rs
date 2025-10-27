use crate::api::client::grpc_web_call;
use crate::models::secret::Secret;
use proto_web::{
    ListSecretsRequest, ListSecretsResponse,
    GetSecretRequest, SecretResponse,
    CreateSecretRequest, UpdateSecretRequest, DeleteSecretRequest, DeleteSecretResponse,
};

pub async fn list_secrets(namespace: &str) -> Result<Vec<Secret>, String> {
    let request = ListSecretsRequest {
        namespace: Some(namespace.to_string()),
        label: None,
        first: None,
        after: None,
    };

    let response: ListSecretsResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/ListSecrets",
        request,
    )
    .await?;

    Ok(response
        .data
        .into_iter()
        .map(Secret::from)
        .collect())
}

pub async fn get_secret(namespace: &str, name: &str) -> Result<Secret, String> {
    let request = GetSecretRequest {
        namespace: Some(namespace.to_string()),
        name: name.to_string(),
    };

    let response: SecretResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/GetSecret",
        request,
    )
    .await?;

    Ok(Secret::from(response))
}

pub async fn create_secret(request: CreateSecretRequest) -> Result<Secret, String> {
    let response: SecretResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/CreateSecret",
        request,
    )
    .await?;

    Ok(Secret::from(response))
}

pub async fn update_secret(request: UpdateSecretRequest) -> Result<Secret, String> {
    let response: SecretResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/UpdateSecret",
        request,
    )
    .await?;

    Ok(Secret::from(response))
}

pub async fn delete_secret(namespace: &str, name: &str) -> Result<DeleteSecretResponse, String> {
    let request = DeleteSecretRequest {
        namespace: Some(namespace.to_string()),
        name: name.to_string(),
    };

    grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret",
        request,
    )
    .await
}
