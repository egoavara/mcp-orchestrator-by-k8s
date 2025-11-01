mod mcp_authorization;
mod mcp_generate_token;
mod mcp_server;
mod mcp_template;
mod namespace;
mod resource_limit;
mod secret;
pub mod utils;

use proto::mcp::orchestrator::v1::{mcp_orchestrator_service_server::McpOrchestratorService, *};
use tonic::{Request, Response, Status};

use crate::state::AppState;

pub struct GrpcService {
    state: AppState,
}

impl GrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl McpOrchestratorService for GrpcService {
    async fn create_mcp_template(
        &self,
        request: Request<CreateMcpTemplateRequest>,
    ) -> Result<Response<McpTemplateResponse>, Status> {
        mcp_template::create_mcp_template(&self.state, request).await
    }

    async fn get_mcp_template(
        &self,
        request: Request<GetMcpTemplateRequest>,
    ) -> Result<Response<McpTemplateResponse>, Status> {
        mcp_template::get_mcp_template(&self.state, request).await
    }

    async fn list_mcp_templates(
        &self,
        request: Request<ListMcpTemplatesRequest>,
    ) -> Result<Response<ListMcpTemplatesResponse>, Status> {
        mcp_template::list_mcp_templates(&self.state, request).await
    }

    async fn delete_mcp_template(
        &self,
        request: Request<DeleteMcpTemplateRequest>,
    ) -> Result<Response<DeleteMcpTemplateResponse>, Status> {
        mcp_template::delete_mcp_template(&self.state, request).await
    }

    async fn get_mcp(
        &self,
        _request: Request<McpRequest>,
    ) -> Result<Response<McpResponse>, Status> {
        // mcp_server::get_mcp_server(&self.state, request).await
        todo!()
    }

    async fn list_mcp_servers(
        &self,
        _request: Request<ListMcpServersRequest>,
    ) -> Result<Response<ListMcpServersResponse>, Status> {
        // mcp_server::list_mcp_servers(&self.state, request).await
        todo!()
    }

    async fn create_namespace(
        &self,
        request: Request<CreateNamespaceRequest>,
    ) -> Result<Response<NamespaceResponse>, Status> {
        namespace::create_namespace(&self.state, request).await
    }

    async fn get_namespace(
        &self,
        request: Request<GetNamespaceRequest>,
    ) -> Result<Response<NamespaceResponse>, Status> {
        namespace::get_namespace(&self.state, request).await
    }

    async fn list_namespaces(
        &self,
        request: Request<ListNamespacesRequest>,
    ) -> Result<Response<ListNamespacesResponse>, Status> {
        namespace::list_namespaces(&self.state, request).await
    }

    async fn delete_namespace(
        &self,
        request: Request<DeleteNamespaceRequest>,
    ) -> Result<Response<DeleteNamespaceResponse>, Status> {
        namespace::delete_namespace(&self.state, request).await
    }

    async fn create_secret(
        &self,
        request: Request<CreateSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        secret::create_secret(&self.state, request).await
    }

    async fn get_secret(
        &self,
        request: Request<GetSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        secret::get_secret(&self.state, request).await
    }

    async fn list_secrets(
        &self,
        request: Request<ListSecretsRequest>,
    ) -> Result<Response<ListSecretsResponse>, Status> {
        secret::list_secrets(&self.state, request).await
    }

    async fn update_secret(
        &self,
        request: Request<UpdateSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        secret::update_secret(&self.state, request).await
    }

    async fn delete_secret(
        &self,
        request: Request<DeleteSecretRequest>,
    ) -> Result<Response<DeleteSecretResponse>, Status> {
        secret::delete_secret(&self.state, request).await
    }

    async fn create_resource_limit(
        &self,
        request: Request<CreateResourceLimitRequest>,
    ) -> Result<Response<ResourceLimitResponse>, Status> {
        resource_limit::create_resource_limit(&self.state, request).await
    }

    async fn get_resource_limit(
        &self,
        request: Request<GetResourceLimitRequest>,
    ) -> Result<Response<ResourceLimitResponse>, Status> {
        resource_limit::get_resource_limit(&self.state, request).await
    }

    async fn list_resource_limits(
        &self,
        request: Request<ListResourceLimitsRequest>,
    ) -> Result<Response<ListResourceLimitsResponse>, Status> {
        resource_limit::list_resource_limits(&self.state, request).await
    }

    async fn delete_resource_limit(
        &self,
        request: Request<DeleteResourceLimitRequest>,
    ) -> Result<Response<DeleteResourceLimitResponse>, Status> {
        resource_limit::delete_resource_limit(&self.state, request).await
    }

    async fn create_authorization(
        &self,
        request: Request<CreateAuthorizationRequest>,
    ) -> Result<Response<AuthorizationResponse>, Status> {
        mcp_authorization::create_authorization(&self.state, request).await
    }

    async fn list_authorizations(
        &self,
        request: Request<ListAuthorizationsRequest>,
    ) -> Result<Response<ListAuthorizationsResponse>, Status> {
        mcp_authorization::list_authorizations(&self.state, request).await
    }

    async fn get_authorization(
        &self,
        request: Request<GetAuthorizationRequest>,
    ) -> Result<Response<AuthorizationResponse>, Status> {
        mcp_authorization::get_authorization(&self.state, request).await
    }

    async fn delete_authorization(
        &self,
        request: Request<DeleteAuthorizationRequest>,
    ) -> Result<Response<DeleteAuthorizationResponse>, Status> {
        mcp_authorization::delete_authorization(&self.state, request).await
    }

    async fn generate_token(
        &self,
        request: Request<GenerateTokenRequest>,
    ) -> Result<Response<GenerateTokenResponse>, Status> {
        mcp_generate_token::generate_token(&self.state, request).await
    }
}
