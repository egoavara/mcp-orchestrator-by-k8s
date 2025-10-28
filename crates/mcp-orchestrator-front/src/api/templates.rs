use crate::api::client::grpc_web_call;
use crate::models::template::Template;
use proto_web::{
    CreateMcpTemplateRequest, DeleteMcpTemplateRequest, DeleteMcpTemplateResponse,
    GetMcpTemplateRequest, ListMcpTemplatesRequest, ListMcpTemplatesResponse, McpTemplateResponse,
};

pub async fn list_templates(namespace: &str) -> Result<Vec<Template>, String> {
    let request = ListMcpTemplatesRequest {
        namespace: Some(namespace.to_string()),
        label: None,
        first: None,
        after: None,
    };

    let response: ListMcpTemplatesResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/ListMcpTemplates",
        request,
    )
    .await?;

    Ok(response.data.into_iter().map(Template::from).collect())
}

pub async fn get_template(namespace: &str, name: &str) -> Result<Template, String> {
    let request = GetMcpTemplateRequest {
        namespace: Some(namespace.to_string()),
        name: name.to_string(),
    };

    let response: McpTemplateResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/GetMcpTemplate",
        request,
    )
    .await?;

    Ok(Template::from(response))
}

pub async fn create_template(request: CreateMcpTemplateRequest) -> Result<Template, String> {
    let response: McpTemplateResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate",
        request,
    )
    .await?;

    Ok(Template::from(response))
}

pub async fn delete_template(namespace: &str, name: &str) -> Result<(), String> {
    let request = DeleteMcpTemplateRequest {
        namespace: Some(namespace.to_string()),
        name: name.to_string(),
    };

    let _response: DeleteMcpTemplateResponse = grpc_web_call(
        "/mcp.orchestrator.v1.McpOrchestratorService/DeleteMcpTemplate",
        request,
    )
    .await?;

    Ok(())
}
