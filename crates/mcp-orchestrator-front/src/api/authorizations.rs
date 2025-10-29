use crate::api::client::grpc_web_call;
use crate::models::authorization::{Authorization, AuthorizationFormData};
use proto_web::mcp::orchestrator::v1::{
    AuthorizationResponse, CreateAuthorizationRequest, DeleteAuthorizationRequest,
    DeleteAuthorizationResponse, GenerateTokenRequest, GenerateTokenResponse,
    GetAuthorizationRequest, ListAuthorizationsRequest, ListAuthorizationsResponse,
};

pub async fn create_authorization(form: AuthorizationFormData) -> Result<Authorization, String> {
    let request = CreateAuthorizationRequest {
        namespace: form.namespace,
        name: form.name,
        labels: form.labels,
        r#type: form.auth_type,
        data: form.data,
    };

    let response: AuthorizationResponse =
        grpc_web_call("/mcp.orchestrator.v1.McpOrchestratorService/CreateAuthorization", request)
            .await?;

    Ok(from_proto_authorization(response))
}

pub async fn list_authorizations(
    namespace: Option<String>,
    auth_type: Option<i32>,
) -> Result<Vec<Authorization>, String> {
    let request = ListAuthorizationsRequest {
        namespace,
        r#type: auth_type,
        label: None,
        first: Some(100),
        after: None,
    };

    let response: ListAuthorizationsResponse =
        grpc_web_call("/mcp.orchestrator.v1.McpOrchestratorService/ListAuthorizations", request)
            .await?;

    Ok(response
        .data
        .into_iter()
        .map(from_proto_authorization)
        .collect())
}

pub async fn get_authorization(namespace: String, name: String) -> Result<Authorization, String> {
    let request = GetAuthorizationRequest {
        namespace: Some(namespace),
        name,
    };

    let response: AuthorizationResponse =
        grpc_web_call("/mcp.orchestrator.v1.McpOrchestratorService/GetAuthorization", request)
            .await?;

    Ok(from_proto_authorization(response))
}

pub async fn delete_authorization(namespace: String, name: String) -> Result<String, String> {
    let request = DeleteAuthorizationRequest {
        namespace: Some(namespace),
        name,
    };

    let response: DeleteAuthorizationResponse =
        grpc_web_call("/mcp.orchestrator.v1.McpOrchestratorService/DeleteAuthorization", request)
            .await?;

    if response.success {
        Ok(response.message)
    } else {
        Err(response.message)
    }
}

pub async fn generate_token(
    namespace: String,
    name: String,
    expire_days: Option<i64>,
) -> Result<(String, Option<String>), String> {
    let expire_duration = expire_days.map(|days| prost_wkt_types::Duration {
        seconds: days * 24 * 60 * 60,
        nanos: 0,
    });

    let request = GenerateTokenRequest {
        namespace: Some(namespace),
        name,
        expire_duration,
    };

    let response: GenerateTokenResponse =
        grpc_web_call("/mcp.orchestrator.v1.McpOrchestratorService/GenerateToken", request)
            .await?;

    let expire_at = response
        .expire_at
        .map(|t| format!("{}.{:09}Z", t.seconds, t.nanos));

    Ok((response.token, expire_at))
}

fn from_proto_authorization(proto: AuthorizationResponse) -> Authorization {
    Authorization {
        namespace: proto.namespace,
        name: proto.name,
        labels: proto.labels,
        auth_type: proto.r#type,
        data: proto.data,
        created_at: proto
            .created_at
            .map(|t| format!("{}.{:09}Z", t.seconds, t.nanos))
            .unwrap_or_default(),
        deleted_at: proto
            .deleted_at
            .map(|t| format!("{}.{:09}Z", t.seconds, t.nanos)),
    }
}
