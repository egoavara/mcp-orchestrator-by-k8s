use axum::{
    Json,
    body::Body,
    handler::Handler,
    response::{IntoResponse, Response},
};
use http::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthError, manager::AuthManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub redirect_uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub client_id: String,
    pub client_secret: String,
    pub client_id_issued_at: i64,
    pub client_secret_expires_at: i64,
    pub redirect_uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
}

impl AuthManager {
    pub async fn register(
        &self,
        Json(body): Json<RegisterRequest>,
    ) -> Result<Response<Body>, AuthError> {
        let client_id = format!("dynamic-{}", Uuid::new_v4());
        let client_secret = Uuid::new_v4().to_string();
        let issued_at = chrono::Utc::now().timestamp();
        let expires_at = issued_at + 5 * 60; // 5 minutes

        let registered_client = RegisterResponse {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            client_id_issued_at: issued_at,
            client_secret_expires_at: expires_at,
            redirect_uris: body.redirect_uris.clone(),
            token_endpoint_auth_method: body.token_endpoint_auth_method.clone(),
            grant_types: body.grant_types.clone(),
            response_types: body.response_types.clone(),
            client_name: body.client_name.clone(),
        };

        self.store_client_in_k8s(&registered_client).await?;

        Ok((StatusCode::CREATED, Json(registered_client)).into_response())
    }
}
