use crate::api::{client::grpc_web_call, APICaller};
use crate::models::namespace::Namespace;
use proto_web::{
    CreateNamespaceRequest, GetNamespaceRequest, ListNamespacesRequest, ListNamespacesResponse,
    NamespaceResponse,
};

impl APICaller {
    pub async fn list_namespaces(&self) -> Result<Vec<Namespace>, String> {
        let request = ListNamespacesRequest {
            label: None,
            first: None,
            after: None,
        };

        let response: ListNamespacesResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(response.data.into_iter().map(Namespace::from).collect())
    }

    pub async fn get_namespace(&self, name: &str) -> Result<Namespace, String> {
        let request = GetNamespaceRequest {
            name: name.to_string(),
        };

        let response: NamespaceResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/GetNamespace",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Namespace::from(response))
    }

    pub async fn create_namespace(
        &self,
        request: CreateNamespaceRequest,
    ) -> Result<Namespace, String> {
        let response: NamespaceResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/CreateNamespace",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Namespace::from(response))
    }
}
