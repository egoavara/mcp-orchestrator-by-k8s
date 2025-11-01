use std::{
    collections::{BTreeMap, HashMap},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{extract::Request, response::Response};
use http::{HeaderMap, HeaderValue};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, jwk::JwkSet};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, api::ObjectMeta};
use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet,
};
use rand::{Rng, distributions::Alphanumeric};
use tower::{Layer, Service};

use crate::{
    AuthError, OpenIdConfig, jwk_to_decoding_key, jwk_to_encoding_key,
    manager_register::RegisterResponse,
};

#[derive(Clone)]
pub struct AuthManager {
    pub(crate) config: Arc<OpenIdConfig>,
    pub(crate) client: reqwest::Client,
    pub(crate) kube: kube::Client,
    pub(crate) metadata: openidconnect::core::CoreProviderMetadata,
    pub(crate) oidc: openidconnect::core::CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointMaybeSet,
        EndpointMaybeSet,
    >,
    pub(crate) default_namespace: String,
    pub(crate) remote_decoding_key: HashMap<String, DecodingKey>,
    #[allow(dead_code)]
    pub(crate) local_decoding_key: HashMap<String, DecodingKey>,
    #[allow(dead_code)]
    pub(crate) local_encoding_key: HashMap<String, EncodingKey>,
    pub(crate) base_url: String,
}

#[derive(Clone)]
pub struct AuthManagerMiddleware<S> {
    pub inner: S,
    pub manager: Arc<AuthManager>,
}

impl<S> AuthManagerMiddleware<S> {
    pub fn new(inner: S, manager: Arc<AuthManager>) -> Self {
        Self { inner, manager }
    }
}

impl AuthManager {
    pub async fn new(
        config: OpenIdConfig,
        kube: kube::Client,
        default_namespace: &str,
        base_url: &str,
    ) -> Result<Self, AuthError> {
        let default_namespace = default_namespace.to_string();
        let base_url = base_url.to_string();
        let client = config.create_http_client().await?;
        let metadata = config.load_provider_metadata().await?;
        let oidc = openidconnect::core::CoreClient::from_provider_metadata(
            metadata.clone(),
            ClientId::new(config.client.id.clone()),
            config
                .client
                .secret
                .as_ref()
                .map(|s| ClientSecret::new(s.clone())),
        );

        let jwks_uri = metadata.jwks_uri();

        let jwks_response = client
            .get(jwks_uri.url().as_str())
            .send()
            .await
            .map_err(|e| AuthError::DiscoveryError(format!("Failed to fetch JWKS: {}", e)))?;

        if !jwks_response.status().is_success() {
            return Err(AuthError::DiscoveryError(format!(
                "JWKS endpoint returned status: {}",
                jwks_response.status()
            )));
        }

        let jwks = jwks_response
            .json::<JwkSet>()
            .await
            .map_err(|e| AuthError::DiscoveryError(format!("Failed to parse JWKS: {}", e)))?;

        if jwks.keys.is_empty() {
            return Err(AuthError::DiscoveryError(
                "JWKS keys array is empty".to_string(),
            ));
        }

        let mut remote_decoding_key = HashMap::new();
        for jwk in &jwks.keys {
            let kid = jwk.common.key_id.clone().ok_or_else(|| {
                AuthError::DiscoveryError("JWK is missing 'kid' field".to_string())
            })?;
            let key = DecodingKey::from_jwk(jwk).map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to create DecodingKey: {}", e))
            })?;
            remote_decoding_key.insert(kid, key);
        }

        let local_jwks = config.create_jwt_validation(&client).await?;
        let mut local_decoding_key = HashMap::new();
        let mut local_encoding_key = HashMap::new();
        let mut rng = rand::thread_rng();
        for jwk in &local_jwks.keys {
            let kid = jwk.common.key_id.clone().unwrap_or_else(|| {
                rng.sample_iter(&Alphanumeric)
                    .take(10).collect::<String>()
            });
            let deckey = jwk_to_decoding_key(jwk).map_err(AuthError::DiscoveryError)?;
            let enckey = jwk_to_encoding_key(jwk).map_err(AuthError::DiscoveryError)?;
            local_decoding_key.insert(kid.clone(), deckey);
            local_encoding_key.insert(kid, enckey);
        }

        Ok(Self {
            config: Arc::new(config),
            remote_decoding_key,
            local_decoding_key,
            local_encoding_key,
            kube,
            default_namespace,
            metadata,
            oidc,
            client,
            base_url,
        })
    }

    pub(crate) async fn store_client_in_k8s(
        &self,
        client: &RegisterResponse,
    ) -> Result<(), AuthError> {
        let secret_name = self.client_id_to_secret_name(&client.client_id);

        let client_json = serde_json::to_string(client)
            .map_err(|e| AuthError::DiscoveryError(format!("Failed to serialize client: {}", e)))?;

        let mut data = BTreeMap::new();
        data.insert("client_data".to_string(), client_json);

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), "oidc-broker".to_string());
        labels.insert("type".to_string(), "dynamic-client".to_string());

        let mut annotations = BTreeMap::new();
        annotations.insert("client-id".to_string(), client.client_id.clone());
        annotations.insert(
            "expires-at".to_string(),
            chrono::DateTime::from_timestamp(client.client_secret_expires_at, 0)
                .unwrap()
                .to_rfc3339(),
        );

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(secret_name),
                namespace: Some(self.default_namespace.clone()),
                labels: Some(labels),
                annotations: Some(annotations),
                ..Default::default()
            },
            string_data: Some(data),
            ..Default::default()
        };

        let secrets: Api<Secret> = Api::namespaced(self.kube.clone(), &self.default_namespace);
        secrets
            .create(&Default::default(), &secret)
            .await
            .map_err(|e| {
                AuthError::DiscoveryError(format!("Failed to create k8s secret: {}", e))
            })?;

        Ok(())
    }

    fn client_id_to_secret_name(&self, client_id: &str) -> String {
        let sanitized = client_id.replace("dynamic-", "").replace("_", "-");
        format!("oauth-client-{}", sanitized)
    }

    pub fn www_authenticate(&self, headers: &mut HeaderMap<HeaderValue>) {
        headers.insert(
            http::header::WWW_AUTHENTICATE,
            HeaderValue::from_str(&format!("Bearer resource_metadata=\"{}\"", &self.base_url))
                .unwrap(),
        );
    }

    pub fn get_decoding_key(&self, kid: &Option<String>) -> Option<&DecodingKey> {
        match kid {
            Some(kid) => self.remote_decoding_key.get(kid),
            None => self.remote_decoding_key.values().next(),
        }
    }
    pub fn validator(&self, header: jsonwebtoken::Header) -> Validation {
        let mut validation = Validation::new(header.alg);
        let mut iss = self.metadata.issuer().url().to_string();
        let aud = self.config.client.id.clone();
        if self.config.runtime.iss_remove_postfix_slash && iss.ends_with('/') {
            iss = iss.trim_end_matches('/').to_string();
        }
        validation.set_audience(&[aud]);
        validation.set_issuer(&[iss]);
        tracing::debug!(
            aud=?validation.aud,
            iss=?validation.iss,
            "validator created"
        );
        validation
    }
}

impl<S> Layer<S> for AuthManager {
    type Service = AuthManagerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthManagerMiddleware::new(inner, Arc::new(self.clone()))
    }
}

impl<S> Service<Request> for AuthManagerMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        let manager = self.manager.clone();

        Box::pin(async move {
            req.extensions_mut().insert(manager);

            inner.call(req).await
        })
    }
}
