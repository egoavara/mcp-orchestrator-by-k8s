use std::{collections::HashMap, sync::Arc};

use futures::Stream;
use k8s_openapi::api::core::v1::{Pod, PodTemplate};
use kube::{Api, Client, api::PostParams};
use rmcp::{
    model::{
        ClientJsonRpcMessage, ClientNotification, Extensions, InitializedNotification,
        JsonRpcNotification, JsonRpcVersion2_0, ServerJsonRpcMessage,
    },
    serve_server,
    transport::{
        WorkerTransport,
        common::server_side_http::{ServerSseMessage, session_id},
        streamable_http_server::{
            SessionId, SessionManager,
            session::local::{
                LocalSessionHandle, LocalSessionManagerError, LocalSessionWorker, SessionConfig,
                create_local_session,
            },
        },
    },
};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    podmcp::{McpPodError, PodMcpProxy, PodMcpTransport},
    storage::{McpTemplateData, McpTemplateStore},
};

#[derive(Clone)]
pub struct PodMcp(Arc<PodMcpInner>);

pub struct PodMcpInner {
    client: Client,
    config: SessionConfig,
    sessions: RwLock<HashMap<SessionId, LocalSessionHandle>>,
    transports: RwLock<HashMap<SessionId, PodMcpTransport>>,
}

impl PodMcp {
    pub fn new(client: Client) -> Self {
        Self(Arc::new(PodMcpInner {
            client,
            config: SessionConfig {
                channel_capacity: 16,
                keep_alive: Some(std::time::Duration::from_secs(30)),
            },
            sessions: RwLock::new(HashMap::new()),
            transports: RwLock::new(HashMap::new()),
        }))
    }

    pub fn factory(
        &self,
    ) -> impl Fn() -> Result<PodMcpProxy, std::io::Error> + Send + Sync + 'static {
        let this = self.clone();
        move || Ok(PodMcpProxy::new(this.clone()))
    }

    pub async fn session_manager(&self, template: McpTemplateData) -> PodMcpSessionManager {
        PodMcpSessionManager(
            Arc::new(PodMcpSessionManagerInner {
                api: Api::namespaced(self.0.client.clone(), &template.namespace),
                template,
            }),
            self.0.clone(),
        )
    }

    pub async fn get_transport(&self, session_id: &SessionId) -> Option<PodMcpTransport> {
        let transports = self.0.transports.read().await;
        transports.get(session_id).cloned()
    }
}

#[derive(Clone)]
pub struct PodMcpSessionManager(Arc<PodMcpSessionManagerInner>, Arc<PodMcpInner>);

pub struct PodMcpSessionManagerInner {
    api: Api<Pod>,
    template: McpTemplateData,
}

impl PodMcpSessionManager {
    async fn get_handle(&self, id: &SessionId) -> Result<LocalSessionHandle, McpPodError> {
        self.get_opt_handle(id)
            .await?
            .ok_or_else(|| McpPodError::SessionNotFound {
                session_id: id.to_string(),
            })
    }

    async fn get_opt_handle(
        &self,
        id: &SessionId,
    ) -> Result<Option<LocalSessionHandle>, McpPodError> {
        let sessions = self.1.sessions.write().await;
        if let Some(local) = sessions.get(id).cloned() {
            return Ok(Some(local));
        }
        let Some(mcp_server) = self.0.api.get_opt(&id.to_string()).await? else {
            return Ok(None);
        };
        let (handle, worker) = create_local_session(id.clone(), self.1.config.clone());
        //
        self.1
            .sessions
            .write()
            .await
            .insert(id.clone(), handle.clone());
        self.1.transports.write().await.insert(
            id.clone(),
            PodMcpTransport::connect(
                self.1.client.clone(),
                &self.0.template.namespace,
                &id.to_string(),
            )
            .await?,
        );
        let service = PodMcpProxy::new_with_id(PodMcp(self.1.clone()), id.clone());
        // spawn a task to serve the session
        tokio::spawn({
            let session_manager = self.clone();
            let session_id = id.clone();
            async move {
                let service = serve_server(service, worker).await;
                match service {
                    Ok(service) => {
                        // on service created
                        let _ = service.waiting().await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to create service: {e}");
                    }
                }
                let _ = session_manager
                    .close_session(&session_id)
                    .await
                    .inspect_err(|e| {
                        tracing::error!("Failed to close session {session_id}: {e}");
                    });
            }
        });
        //
        Ok(Some(handle))
    }
}

impl SessionManager for PodMcpSessionManager {
    type Error = McpPodError;

    type Transport = WorkerTransport<LocalSessionWorker>;
    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        let id = session_id();
        let data = self.0.template.to_pod(&id);
        self.0.api.create(&PostParams::default(), &data).await?;
        let (handle, worker) = create_local_session(id.clone(), self.1.config.clone());
        self.1.sessions.write().await.insert(id.clone(), handle);
        self.1.transports.write().await.insert(
            id.clone(),
            PodMcpTransport::connect(
                self.1.client.clone(),
                &self.0.template.namespace,
                &id.to_string(),
            )
            .await?,
        );
        Ok((id, WorkerTransport::spawn(worker)))
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        mut message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, Self::Error> {
        let handle = self.get_handle(id).await?;
        message.insert_extension(id.clone());
        let response = handle.initialize(message).await?;
        Ok(response)
    }

    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        let Some(_pod) = self.0.api.get_opt(&id.to_string()).await? else {
            return Ok(());
        };
        tracing::info!("Deleting pod for session {}", id);
        self.0
            .api
            .delete(&id.to_string(), &Default::default())
            .await?;
        self.1.sessions.write().await.remove(id);
        self.1.transports.write().await.remove(id);
        Ok(())
    }

    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        let pod = self.0.api.get_opt(id.to_string().as_str()).await?;
        Ok(pod.is_some())
    }
    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        let handle = self.get_handle(id).await?;
        let receiver = handle.establish_request_wise_channel().await?;
        handle
            .push_message(message, receiver.http_request_id)
            .await?;
        Ok(ReceiverStream::new(receiver.inner))
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        let handle = self.get_handle(id).await?;
        let receiver = handle.establish_common_channel().await?;
        Ok(ReceiverStream::new(receiver.inner))
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        let handle = self.get_handle(id).await?;
        let receiver = handle.resume(last_event_id.parse()?).await?;
        Ok(ReceiverStream::new(receiver.inner))
    }

    async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        let handle = self.get_handle(id).await?;
        handle.push_message(message, None).await?;
        Ok(())
    }
}
