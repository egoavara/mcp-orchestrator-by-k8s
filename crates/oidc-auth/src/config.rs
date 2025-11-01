use std::path::PathBuf;
use std::str;

use crate::AuthError;
use crate::jwks::{
    LocalAlgorithmParameters, LocalEllipticCurveKeyParameters, LocalJwk, LocalJwkSet,
    LocalOctetKeyPairParameters,
};
use argon2::Argon2;
use base64::Engine;
use chrono::Local;
use ed25519_dalek::SigningKey;
use http::Uri;
use jsonwebtoken::jwk::{
    CommonParameters, Jwk, JwkSet, KeyAlgorithm, OctetKeyPairType, OctetKeyParameters, OctetKeyType,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use openidconnect::core::{
    CoreClient, CoreJwsSigningAlgorithm, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::{
    AuthUrl, ClientId, ClientSecret, IssuerUrl, JsonWebKeySetUrl, RedirectUrl, ResponseTypes,
    Scope, TokenUrl, core::CoreProviderMetadata,
};
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet};
use rand_chacha::{
    ChaCha20Rng,
    rand_core::{CryptoRng, SeedableRng},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::{StringWithSeparator, serde_as};

const SALT: &[u8] = b"0ec6beb6-82a7-46e1-bf3b-a0822a1031af";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIdConfig {
    pub discovery: DiscoveryConfig,
    pub client: DefaultClientConfig,
    pub resource_metadata: Option<ResourceServerConfig>,

    #[serde(default = "default_runtime_config")]
    pub runtime: RuntimeConfig,

    pub jwks: JwksConfig,
}

fn default_runtime_config() -> RuntimeConfig {
    tracing::info!("default_runtime_config called");
    RuntimeConfig::default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_iss_remove_postfix_slash")]
    pub iss_remove_postfix_slash: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            iss_remove_postfix_slash: default_iss_remove_postfix_slash(),
        }
    }
}

fn default_iss_remove_postfix_slash() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceServerConfig {
    pub url: Option<String>,
    pub authorization_servers: Option<Vec<String>>,
    pub scopes_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiscoveryConfig {
    Discovery {
        discovery_url: String,
    },
    Static {
        issuer_url: String,
        authorization_endpoint: String,
        token_endpoint: String,
        jwks_uri: String,
    },
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultClientConfig {
    pub id: String,
    pub secret: Option<String>,
    pub redirect: String,
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksConfig {
    pub url: Option<Url>,
    pub path: Option<PathBuf>,
    pub json: Option<serde_json::Value>,
    pub secret: Option<String>,
}

impl OpenIdConfig {
    pub async fn load_provider_metadata(&self) -> Result<CoreProviderMetadata, AuthError> {
        match &self.discovery {
            DiscoveryConfig::Discovery { discovery_url, .. } => {
                let discovery_url = IssuerUrl::new(discovery_url.clone()).map_err(|e| {
                    AuthError::DiscoveryError(format!("Invalid discovery URL: {}", e))
                })?;
                let metadata =
                    CoreProviderMetadata::discover_async(discovery_url, &reqwest::Client::new())
                        .await
                        .map_err(|e| {
                            AuthError::DiscoveryError(format!("OIDC discovery failed: {}", e))
                        })?;
                Ok(metadata)
            }
            DiscoveryConfig::Static {
                issuer_url,
                authorization_endpoint,
                token_endpoint,
                jwks_uri,
                ..
            } => {
                let issuer = IssuerUrl::new(issuer_url.clone())
                    .map_err(|e| AuthError::DiscoveryError(format!("Invalid issuer URL: {}", e)))?;

                let auth_url = AuthUrl::new(authorization_endpoint.clone()).map_err(|e| {
                    AuthError::DiscoveryError(format!("Invalid authorization endpoint: {}", e))
                })?;

                let token_url = TokenUrl::new(token_endpoint.clone()).map_err(|e| {
                    AuthError::DiscoveryError(format!("Invalid token endpoint: {}", e))
                })?;

                let jwks_url = JsonWebKeySetUrl::new(jwks_uri.clone())
                    .map_err(|e| AuthError::DiscoveryError(format!("Invalid JWKS URI: {}", e)))?;

                let metadata = CoreProviderMetadata::new(
                    issuer,
                    auth_url,
                    jwks_url,
                    vec![ResponseTypes::new(vec![CoreResponseType::Code])],
                    vec![CoreSubjectIdentifierType::Public],
                    vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256],
                    Default::default(),
                )
                .set_token_endpoint(Some(token_url));

                Ok(metadata)
            }
        }
    }

    pub(crate) async fn create_http_client(&self) -> Result<reqwest::Client, AuthError> {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AuthError::DiscoveryError(format!("Failed to build HTTP client: {}", e)))
    }

    pub(crate) async fn create_jwt_validation(
        &self,
        client: &reqwest::Client,
    ) -> Result<LocalJwkSet, AuthError> {
        if let Some(jwts_url) = &self.jwks.url {
            let jwks_response =
                client.get(jwts_url.as_str()).send().await.map_err(|e| {
                    AuthError::DiscoveryError(format!("Failed to fetch JWKS: {}", e))
                })?;
            let jwks: LocalJwkSet = jwks_response.json().await.map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to parse JWKS response: {}", e))
            })?;
            return Ok(jwks);
        }

        if let Some(jwks_path) = &self.jwks.path {
            let jwks_data = tokio::fs::read_to_string(jwks_path).await.map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to read JWKS file: {}", e))
            })?;
            let jwks: LocalJwkSet = serde_json::from_str(&jwks_data).map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to parse JWKS file: {}", e))
            })?;
            return Ok(jwks);
        }

        if let Some(jwks_json) = &self.jwks.json {
            let jwks: LocalJwkSet = serde_json::from_value(jwks_json.clone()).map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to parse JWKS JSON: {}", e))
            })?;
            return Ok(jwks);
        }
        if let Some(jwks_secret) = &self.jwks.secret {
            match jwks_secret.len() {
                keylen @ ..10 => {
                    return Err(AuthError::DiscoveryError(format!(
                        "JWKS secret is too short ({} characters); must be at least 10 characters long",
                        keylen
                    )));
                }
                keylen @ 10..24 => {
                    tracing::error!(
                        length = keylen,
                        "JWKS secret is less than 24 characters long; this is not recommended for production use. Use only for testing purposes."
                    );
                }
                keylen @ 24..48 => {
                    tracing::warn!(
                        length = keylen,
                        "JWKS secret is less than 48 characters long; consider using a longer secret for better security.",
                    );
                }
                48.. => {}
            }
            let mut seed = [0u8; 32];
            Argon2::default()
                .hash_password_into(jwks_secret.as_bytes(), SALT, &mut seed)
                .unwrap();

            let mut rng = ChaCha20Rng::from_seed(seed);
            let key = ed25519_dalek::SigningKey::generate(&mut rng);
            let verifying_key = key.verifying_key();
            let jwk = LocalJwk {
                common: CommonParameters {
                    key_algorithm: Some(KeyAlgorithm::EdDSA),
                    key_id: Some("jwks_generated".to_string()),
                    ..Default::default()
                },
                algorithm: LocalAlgorithmParameters::OctetKeyPair(LocalOctetKeyPairParameters {
                    key_type: OctetKeyPairType::OctetKeyPair,
                    curve: jsonwebtoken::jwk::EllipticCurve::Ed25519,
                    x: base64::engine::general_purpose::URL_SAFE_NO_PAD
                        .encode(verifying_key.to_bytes()),
                    d: Some(
                        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key.to_bytes()),
                    ),
                }),
            };
            let jwks = LocalJwkSet { keys: vec![jwk] };
            return Ok(jwks);
        }
        Err(AuthError::DiscoveryError(
            "No JWKS source configured".to_string(),
        ))
    }
}
