use std::{collections::HashMap, sync::Arc};

use futures::Stream;
use k8s_openapi::api::{
    authentication::v1::{TokenReview, TokenReviewSpec},
    core::v1::Pod,
};
use kube::{
    Api,
    api::{DeleteParams, PostParams},
};
use proto::mcp::orchestrator::v1::AuthorizationType;
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::{
        common::server_side_http::{ServerSseMessage, session_id},
        streamable_http_server::SessionId,
    },
};
use tokio::sync::RwLock;

use crate::{
    podmcp::{McpPodError, PodMcpTransport},
    storage::{
        McpTemplateData, resource_type::RESOURCE_TYPE_PREFIX_AUTHORIZATION_SA, store::KubeStore,
        store_authorization::AuthorizationData, util_name::encode_k8sname,
    },
};

#[derive(Clone)]
pub struct PodMcp(Arc<PodMcpInner>);

pub struct PodMcpInner {
    client: KubeStore,
    transports: RwLock<HashMap<SessionId, PodMcpTransport>>,
}

impl PodMcp {
    pub fn new(client: KubeStore) -> Self {
        Self(Arc::new(PodMcpInner {
            client,
            transports: RwLock::new(HashMap::new()),
        }))
    }
    pub async fn session_manager(&self, template: McpTemplateData) -> PodMcpSessionManager {
        PodMcpSessionManager(
            Arc::new(PodMcpSessionManagerInner {
                api: Api::namespaced(self.0.client.to_client(), &template.namespace),
                template,
            }),
            self.0.clone(),
        )
    }

    pub(crate) async fn remove_transport(&self, session_id: &SessionId) {
        let mut transports = self.0.transports.write().await;
        transports.remove(session_id);
    }
}

#[derive(Clone)]
pub struct PodMcpSessionManager(Arc<PodMcpSessionManagerInner>, Arc<PodMcpInner>);

pub struct PodMcpSessionManagerInner {
    api: Api<Pod>,
    template: McpTemplateData,
}

impl PodMcpSessionManager {
    async fn get_handle(&self, id: &SessionId) -> Result<PodMcpTransport, McpPodError> {
        self.get_opt_handle(id)
            .await?
            .ok_or_else(|| McpPodError::SessionNotFound {
                session_id: id.to_string(),
            })
    }

    async fn get_opt_handle(&self, id: &SessionId) -> Result<Option<PodMcpTransport>, McpPodError> {
        if let Some(local) = self.1.transports.read().await.get(id).cloned() {
            return Ok(Some(local));
        }

        tracing::info!("Creating new session handle for session {}", id);
        let conn = PodMcpTransport::connect(
            self.1.client.clone(),
            &self.0.template.namespace,
            id,
            PodMcp(self.1.clone()),
        )
        .await?;
        //
        let mut transport_manager = self.1.transports.write().await;
        if transport_manager.contains_key(id) {
            tracing::info!(
                "Transport for session {} already exists, another request created it concurrently",
                id
            );
        }
        transport_manager.insert(id.clone(), conn.clone());
        //
        Ok(Some(conn))
    }
}

pub struct PodMcpRequest {
    pub audience: String,
    pub token: Option<String>,
}

impl PodMcpSessionManager {
    pub async fn create_session(
        &self,
        req: PodMcpRequest,
        args: HashMap<String, String>,
    ) -> Result<SessionId, McpPodError> {
        let id = session_id();
        let (pod, auth) = self.0.template.to_pod(&id, &self.1.client, args).await?;
        //
        self.assert_auth_check(&auth, &req).await?;
        //
        self.0.api.create(&PostParams::default(), &pod).await?;
        let transport = PodMcpTransport::connect(
            self.1.client.clone(),
            &self.0.template.namespace,
            &id,
            PodMcp(self.1.clone()),
        )
        .await?;
        self.1
            .transports
            .write()
            .await
            .insert(id.clone(), transport.clone());

        tracing::debug!("Created pod for session {}", id);
        Ok(id)
    }

    pub async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, McpPodError> {
        tracing::debug!("Initializing session {}", id);
        let transport = self.get_handle(id).await?;
        let response = transport.initialize_session(message).await?;
        Ok(response)
    }

    pub async fn close_session(
        &self,
        id: &SessionId,
        req: PodMcpRequest,
    ) -> Result<(), McpPodError> {
        tracing::info!("Deleting pod for session {}", id);
        //
        let auth = self.0.template.get_authorization(&self.1.client).await?;
        self.assert_auth_check(&auth, &req).await?;
        //
        self.1.transports.write().await.remove(id);

        self.0
            .api
            .delete(id.to_string().as_str(), &DeleteParams::default())
            .await?;

        Ok(())
    }

    pub async fn has_session(
        &self,
        id: &SessionId,
        req: PodMcpRequest,
    ) -> Result<bool, McpPodError> {
        tracing::debug!("Checking existence of session {}", id);
        let pod = self.0.api.get_opt(id.to_string().as_str()).await?;
        //
        let auth = self.0.template.get_authorization(&self.1.client).await?;
        self.assert_auth_check(&auth, &req).await?;
        //
        Ok(pod.is_some())
    }
    pub async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, McpPodError> {
        tracing::debug!(message = ?message, "Creating stream for session {}", id);
        let transport = self.get_handle(id).await?;
        transport.upstream_tx_send(message).await?;
        let stream = transport.downstream_rx_stream().await;
        Ok(stream)
    }

    pub async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, McpPodError> {
        tracing::debug!("Creating standalone stream for session {}", id);
        let transport = self.get_handle(id).await?;
        let stream = transport.downstream_rx_stream().await;
        Ok(stream)
    }

    pub async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, McpPodError> {
        tracing::debug!("Resuming stream for session {}", id);
        tracing::warn!(
            "Resuming from last_event_id {} is not support",
            last_event_id
        );
        let transport = self.get_handle(id).await?;
        let stream = transport.downstream_rx_stream().await;
        Ok(stream)
    }

    pub async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), McpPodError> {
        tracing::debug!(message = ?message, "Accepting message for session {}", id);
        let transport = self.get_handle(id).await?;
        transport.upstream_tx_send(message).await?;
        Ok(())
    }

    async fn assert_auth_check(
        &self,
        auth: &AuthorizationData,
        req: &PodMcpRequest,
    ) -> Result<(), McpPodError> {
        if auth.r#type == AuthorizationType::Anonymous {
            return Ok(());
        }
        let Some(token) = &req.token else {
            tracing::debug!("Authorizing pod session failed: token is missing");
            return Err(McpPodError::AuthorizationFailed {
                reason: "Authorization token is missing".to_string(),
            });
        };
        let review = Api::<TokenReview>::all(self.1.client.to_client())
            .create(
                &PostParams::default(),
                &TokenReview {
                    spec: TokenReviewSpec {
                        token: Some(token.clone()),
                        audiences: Some(vec![req.audience.clone()]),
                    },
                    ..Default::default()
                },
            )
            .await?;
        let Some(review_status) = review.status else {
            return Ok(());
        };
        if !review_status.authenticated.unwrap_or(false) {
            return Err(McpPodError::AuthorizationFailed {
                reason: "Token is not authenticated".to_string(),
            });
        }
        let username = review_status
            .user
            .map(|x| x.username.unwrap_or_default())
            .unwrap_or_default();
        let expected_username = format!(
            "system:serviceaccount:{}:{}",
            self.0.template.namespace,
            encode_k8sname(
                RESOURCE_TYPE_PREFIX_AUTHORIZATION_SA,
                &self.0.template.authorization_name
            )
        );
        if username != expected_username {
            tracing::info!(
                "Authorization failed: expected service account {}, got {}",
                format!(
                    "system:serviceaccount:{}:{}",
                    self.0.template.namespace, self.0.template.authorization_name
                ),
                username
            );
            return Err(McpPodError::AuthorizationFailed {
                reason: format!(
                    "Token username {} does not match expected service account",
                    username
                ),
            });
        }
        Ok(())
    }
}
