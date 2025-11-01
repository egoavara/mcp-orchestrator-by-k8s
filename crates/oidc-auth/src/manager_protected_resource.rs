use serde::{Deserialize, Serialize};

use crate::manager::AuthManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedResourceResponse {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_methods_supported: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,
}

impl AuthManager {
    pub fn protected_resource(&self) -> ProtectedResourceResponse {
        ProtectedResourceResponse {
            resource: self
                .config
                .resource_metadata
                .as_ref()
                .and_then(|r| r.url.clone())
                .unwrap_or_else(|| self.base_url.clone()),
            authorization_servers: self
                .config
                .resource_metadata
                .as_ref()
                .and_then(|r| r.authorization_servers.clone())
                .unwrap_or_else(|| vec![self.base_url.clone()]),
            bearer_methods_supported: Some(vec!["header".to_string()]),
            scopes_supported: Some(
                self.config
                    .resource_metadata
                    .as_ref()
                    .map(|r| r.scopes_supported.clone())
                    .unwrap_or_else(|| vec!["openid".to_string(), "profile".to_string()]),
            ),
        }
    }
}
