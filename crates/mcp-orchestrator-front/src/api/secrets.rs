use crate::api::{client::grpc_web_call, APICaller};
use crate::models::secret::Secret;
use proto_web::{
    CreateSecretRequest, DeleteSecretRequest, DeleteSecretResponse, GetSecretRequest,
    ListSecretsRequest, ListSecretsResponse, SecretResponse, UpdateSecretRequest,
};

impl APICaller {
    pub async fn list_secrets(&self, namespace: &str) -> Result<Vec<Secret>, String> {
        let request = ListSecretsRequest {
            namespace: Some(namespace.to_string()),
            label: None,
            first: None,
            after: None,
        };

        let response: ListSecretsResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/ListSecrets",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(response.data.into_iter().map(Secret::from).collect())
    }

    pub async fn get_secret(&self, namespace: &str, name: &str) -> Result<Secret, String> {
        let request = GetSecretRequest {
            namespace: Some(namespace.to_string()),
            name: name.to_string(),
        };

        let response: SecretResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/GetSecret",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Secret::from(response))
    }

    pub async fn create_secret(&self, request: CreateSecretRequest) -> Result<Secret, String> {
        let response: SecretResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/CreateSecret",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Secret::from(response))
    }

    pub async fn update_secret(&self, request: UpdateSecretRequest) -> Result<Secret, String> {
        let response: SecretResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/UpdateSecret",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Secret::from(response))
    }

    pub async fn delete_secret(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<DeleteSecretResponse, String> {
        let request = DeleteSecretRequest {
            namespace: Some(namespace.to_string()),
            name: name.to_string(),
        };

        grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret",
            request,
            self.access_token.as_deref(),
        )
        .await
    }
}
