use std::fmt::format;

use serde::{Deserialize, Serialize};

use crate::manager::AuthManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthorizationServerMetadata {
    pub issuer: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    pub response_types_supported: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modes_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_documentation: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_locales_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_policy_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_tos_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint_auth_methods_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint_auth_methods_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint_auth_signing_alg_values_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_metadata: Option<String>,
}

impl AuthManager {
    pub fn authorization_server(&self) -> AuthorizationServerMetadata {
        AuthorizationServerMetadata {
            issuer: self.metadata.issuer().to_string(),
            authorization_endpoint: Some(format!("{}/oauth/authorize", &self.base_url)),
            token_endpoint: self.metadata.token_endpoint().map(|u| u.to_string()),
            jwks_uri: Some(self.metadata.jwks_uri().to_string()),
            registration_endpoint: Some(format!("{}/oauth/register", &self.base_url)),
            scopes_supported: self
                .metadata
                .scopes_supported()
                .map(|scopes| scopes.iter().map(|s| s.to_string()).collect()),
            response_types_supported: self
                .metadata
                .response_types_supported()
                .iter()
                .map(|rt| {
                    rt.iter()
                        .map(|t| t.as_ref().to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect(),
            response_modes_supported: self
                .metadata
                .response_modes_supported()
                .map(|modes| modes.iter().map(|m| m.as_ref().to_string()).collect()),
            grant_types_supported: self
                .metadata
                .grant_types_supported()
                .map(|grants| grants.iter().map(|g| g.as_ref().to_string()).collect()),
            token_endpoint_auth_methods_supported: self
                .metadata
                .token_endpoint_auth_methods_supported()
                .map(|methods| methods.iter().map(|m| m.as_ref().to_string()).collect()),
            token_endpoint_auth_signing_alg_values_supported: self
                .metadata
                .token_endpoint_auth_signing_alg_values_supported()
                .map(|algs| {
                    algs.iter()
                        .map(|a| {
                            serde_json::to_string(a)
                                .unwrap()
                                .strip_prefix('"')
                                .unwrap()
                                .strip_suffix('"')
                                .unwrap()
                                .to_string()
                        })
                        .collect()
                }),
            service_documentation: self.metadata.service_documentation().map(|u| u.to_string()),
            ui_locales_supported: self
                .metadata
                .ui_locales_supported()
                .map(|locales| locales.iter().map(|l| l.to_string()).collect()),
            op_policy_uri: self.metadata.op_policy_uri().map(|u| u.to_string()),
            op_tos_uri: self.metadata.op_tos_uri().map(|u| u.to_string()),
            revocation_endpoint: None,
            revocation_endpoint_auth_methods_supported: None,
            revocation_endpoint_auth_signing_alg_values_supported: None,
            introspection_endpoint: None,
            introspection_endpoint_auth_methods_supported: None,
            introspection_endpoint_auth_signing_alg_values_supported: None,
            code_challenge_methods_supported: None,
            signed_metadata: None,
        }
    }
}
