use crate::api::{client::grpc_web_call, APICaller};
use crate::models::template::Template;
use proto_web::{
    CreateMcpTemplateRequest, DeleteMcpTemplateRequest, DeleteMcpTemplateResponse,
    GetMcpTemplateRequest, ListMcpTemplatesRequest, ListMcpTemplatesResponse, McpTemplateResponse,
};

impl APICaller {
    pub async fn list_templates(&self, namespace: &str) -> Result<Vec<Template>, String> {
        let request = ListMcpTemplatesRequest {
            namespace: Some(namespace.to_string()),
            label: None,
            first: None,
            after: None,
        };

        let response: ListMcpTemplatesResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/ListMcpTemplates",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(response.data.into_iter().map(Template::from).collect())
    }

    pub async fn get_template(&self, namespace: &str, name: &str) -> Result<Template, String> {
        let request = GetMcpTemplateRequest {
            namespace: Some(namespace.to_string()),
            name: name.to_string(),
        };

        let response: McpTemplateResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/GetMcpTemplate",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Template::from(response))
    }

    pub async fn create_template(
        &self,
        request: CreateMcpTemplateRequest,
    ) -> Result<Template, String> {
        let response: McpTemplateResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(Template::from(response))
    }

    pub async fn delete_template(&self, namespace: &str, name: &str) -> Result<(), String> {
        let request = DeleteMcpTemplateRequest {
            namespace: Some(namespace.to_string()),
            name: name.to_string(),
        };

        let _response: DeleteMcpTemplateResponse = grpc_web_call(
            "/mcp.orchestrator.v1.McpOrchestratorService/DeleteMcpTemplate",
            request,
            self.access_token.as_deref(),
        )
        .await?;

        Ok(())
    }
}
