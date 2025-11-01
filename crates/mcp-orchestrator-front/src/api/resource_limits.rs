use crate::api::{client::grpc_web_call, APICaller};
use crate::models::resource_limit::ResourceLimit;
use proto_web::{
    CreateResourceLimitRequest, DeleteResourceLimitRequest, DeleteResourceLimitResponse,
    GetResourceLimitRequest, ListResourceLimitsRequest, ListResourceLimitsResponse,
    ResourceLimitResponse,
};

impl APICaller {
    pub async fn list_resource_limits(&self) -> Result<Vec<ResourceLimit>, String> {
        let request = ListResourceLimitsRequest {
            label: None,
            first: None,
            after: None,
        };

        let response: ListResourceLimitsResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/ListResourceLimits",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(response.data.into_iter().map(ResourceLimit::from).collect())
    }

    pub async fn get_resource_limit(&self, name: &str) -> Result<ResourceLimit, String> {
        let request = GetResourceLimitRequest {
            name: name.to_string(),
        };

        let response: ResourceLimitResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/GetResourceLimit",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(ResourceLimit::from(response))
    }

    pub async fn create_resource_limit(
        &self,
        request: CreateResourceLimitRequest,
    ) -> Result<ResourceLimit, String> {
        let response: ResourceLimitResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(ResourceLimit::from(response))
    }

    pub async fn delete_resource_limit(
        &self,
        name: &str,
    ) -> Result<DeleteResourceLimitResponse, String> {
        let request = DeleteResourceLimitRequest {
            name: name.to_string(),
            force: false,
        };

        grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/DeleteResourceLimit",
            request,
            self.access_token.as_deref(),
        )
        .await
    }
}
