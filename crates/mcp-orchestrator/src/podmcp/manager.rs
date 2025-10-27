use std::{collections::HashMap, sync::Arc};

use futures::Stream;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api,
    api::{DeleteParams, PostParams},
};
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
    storage::{McpTemplateData, store::KubeStore},
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

impl PodMcpSessionManager {
    pub async fn create_session(&self) -> Result<SessionId, McpPodError> {
        let id = session_id();
        let data = self.0.template.to_pod(&id, &self.1.client).await?;
        self.0.api.create(&PostParams::default(), &data).await?;
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

    pub async fn close_session(&self, id: &SessionId) -> Result<(), McpPodError> {
        tracing::info!("Deleting pod for session {}", id);
        self.1.transports.write().await.remove(id);

        self.0
            .api
            .delete(id.to_string().as_str(), &DeleteParams::default())
            .await?;

        Ok(())
    }

    pub async fn has_session(&self, id: &SessionId) -> Result<bool, McpPodError> {
        tracing::debug!("Checking existence of session {}", id);
        let pod = self.0.api.get_opt(id.to_string().as_str()).await?;
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
}
