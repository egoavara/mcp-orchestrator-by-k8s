use crate::api::client::grpc_web_call;
use crate::models::namespace::Namespace;
use proto_web::{
    ListNamespacesRequest, ListNamespacesResponse,
    GetNamespaceRequest, NamespaceResponse,
    CreateNamespaceRequest,
};

pub async fn list_namespaces() -> Result<Vec<Namespace>, String> {
    let request = ListNamespacesRequest {
        label: None,
        first: None,
        after: None,
    };

    let response: ListNamespacesResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces",
        request,
    )
    .await?;

    Ok(response
        .data
        .into_iter()
        .map(Namespace::from)
        .collect())
}

pub async fn get_namespace(name: &str) -> Result<Namespace, String> {
    let request = GetNamespaceRequest {
        name: name.to_string(),
    };

    let response: NamespaceResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/GetNamespace",
        request,
    )
    .await?;

    Ok(Namespace::from(response))
}

pub async fn create_namespace(request: CreateNamespaceRequest) -> Result<Namespace, String> {
    let response: NamespaceResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/CreateNamespace",
        request,
    )
    .await?;

    Ok(Namespace::from(response))
}
