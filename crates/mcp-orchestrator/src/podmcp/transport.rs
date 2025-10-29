use chrono::{DateTime, Duration, Utc};
use futures::{Stream, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, ResourceExt,
    api::{AttachParams, AttachedProcess, Patch, PatchParams},
};
use rmcp::{
    RoleServer,
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    service::RxJsonRpcMessage,
    transport::{
        IntoTransport, Transport, async_rw::AsyncRwTransport,
        common::server_side_http::ServerSseMessage, streamable_http_server::SessionId,
    },
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    broadcast::{self, error::RecvError},
    mpsc,
};
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    podmcp::{McpPodError, PodMcp, PodMcpRequest},
    storage::{annotations::ANNOTATION_LAST_ACCESS_AT, store::KubeStore},
};

#[derive(Clone)]
pub struct PodMcpTransport {
    pub(crate) session_id: String,
    last_event_time: Arc<Mutex<DateTime<Utc>>>,
    api: Api<Pod>,
    upstream_tx: mpsc::Sender<ClientJsonRpcMessage>,
    downstream_tx: broadcast::Sender<ServerJsonRpcMessage>,
}

impl PodMcpTransport {
    pub async fn connect(
        client: KubeStore,
        namespace: &str,
        session_id: &SessionId,
        podmcp: PodMcp,
    ) -> Result<Self, McpPodError> {
        let api = Api::<Pod>::namespaced(client.to_client(), namespace);
        let (upstream_tx, mut upstream_rx) = mpsc::channel::<ClientJsonRpcMessage>(16);
        let (downstream_tx, _) = broadcast::channel::<ServerJsonRpcMessage>(16);
        tokio::spawn({
            let api = api.clone();
            let mut attach = Option::<AttachedProcess>::None;
            for _ in 0..50 {
                tokio::time::sleep(Duration::milliseconds(300).to_std().unwrap()).await;
                tracing::debug!("Looking for pod with session ID {}", session_id);
                let Some(pod) =
                    api.get_opt(session_id)
                        .await
                        .map_err(|_| McpPodError::PodNotFound {
                            session_id: session_id.to_string(),
                        })?
                else {
                    tracing::debug!("Pod with session ID {} not found yet", session_id);
                    continue;
                };

                if let Some(state) = &pod.status {
                    tracing::debug!(
                        "Pod {} status phase: {:?}",
                        pod.name_any(),
                        state.phase.as_deref()
                    );
                    if state.phase.as_deref() != Some("Running") {
                        continue;
                    }
                } else {
                    tracing::debug!("Pod {} has no status yet", pod.name_any());
                    continue;
                }

                let pod_name = pod.name_any();
                let param = AttachParams::default()
                    .stdin(true)
                    .stdout(true)
                    .stderr(false)
                    .container("main")
                    .tty(false);

                // TODO: Lease 활용해 단일 스트림만 허용하도록 제한
                match api.attach(&pod_name, &param).await {
                    Ok(conn) => {
                        attach.replace(conn);
                        break;
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Failed to attach to pod {} for session {}: {}. Retrying...",
                            pod_name,
                            session_id,
                            err
                        );
                        continue;
                    }
                }
            }
            tracing::debug!(
                "Finished trying to attach to pod for session {}",
                session_id
            );
            let Some(attach) = attach else {
                return Err(McpPodError::NoConnection {
                    session_id: session_id.to_string(),
                });
            };
            let session_id = session_id.clone();
            let mut attach = attach;
            let stdin = attach.stdin().ok_or_else(|| McpPodError::NoStdin {
                session_id: session_id.to_string(),
            })?;
            let stdout = attach.stdout().ok_or_else(|| McpPodError::NoStdout {
                session_id: session_id.to_string(),
            })?;
            let transport = AsyncRwTransport::new_client(stdout, stdin);
            let mut transport = transport.into_transport();
            let downstream_tx = downstream_tx.clone();

            async move {
                let mut last_activity_at = DateTime::<Utc>::MIN_UTC;
                async fn update(
                    api: &Api<Pod>,
                    session_id: &SessionId,
                    last_activity_at: &mut DateTime<Utc>,
                ) -> Result<(), McpPodError> {
                    let current = Utc::now();
                    let last_activity_dur = current.signed_duration_since(*last_activity_at);
                    if last_activity_dur.num_seconds() < 5 {
                        return Ok(());
                    }
                    *last_activity_at = current;
                    let patch = Patch::Strategic(json!({
                        "metadata": {
                            "annotations": {
                                ANNOTATION_LAST_ACCESS_AT: &current,
                            }
                        }
                    }));
                    tracing::debug!(
                        "Updated last activity for session {}, to {}",
                        session_id,
                        current
                    );
                    api.patch(session_id, &PatchParams::default(), &patch)
                        .await
                        .map(|_| ())
                        .map_err(McpPodError::from)
                }
                loop {
                    // Timeout after duration of inactivity
                    let timeout_dur = Duration::seconds(600).to_std().unwrap();
                    let timeout_fut = tokio::time::sleep(timeout_dur);
                    tokio::select! {
                        result = transport.receive() => {
                            tracing::trace!("Receiving message from pod for session {}: result {:?}", session_id, result);
                            match result {
                                Some(msg) => {
                                    tracing::trace!("Received message from pod for session {}: {:?}", session_id, msg);
                                    if let Err(err) = downstream_tx.send(msg) {
                                        tracing::warn!("no active receivers for session {}: {}", session_id, err);
                                        continue;
                                    }
                                }
                                None => {
                                    tracing::info!("Transport closed by pod for session {}", session_id);
                                    break;
                                }
                            }
                        }
                        Some(msg) = upstream_rx.recv() => {
                            tracing::trace!("Sending message to pod for session {}: {:?}", session_id, msg);
                            if let Err(err) = transport.send(msg).await {
                                tracing::error!("Failed to send message to pod for session {}: {}", session_id, err);
                                break;
                            }
                        }
                        _ = timeout_fut => {
                            tracing::info!("Transport timeout for session {}", session_id);
                            break
                        }
                    }

                    if let Err(err) = update(&api, &session_id, &mut last_activity_at).await {
                        tracing::error!(
                            "Failed to update last activity for session {}: {}",
                            session_id,
                            err
                        );
                    }
                }
                tracing::info!("Transport task ended for session {}", session_id);
                podmcp.remove_transport(&session_id).await;
                if let Err(err) = transport.close().await {
                    tracing::error!(
                        "Failed to close transport for session {}: {}",
                        session_id,
                        err
                    );
                }
            }
        });
        Ok(Self {
            session_id: session_id.to_string(),
            api,
            last_event_time: Arc::new(Mutex::new(Utc::now())),
            upstream_tx,
            downstream_tx,
        })
    }

    pub async fn initialize_session(
        &self,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, McpPodError> {
        self.upstream_tx
            .send(message)
            .await
            .map_err(|_| McpPodError::SendTransportError)?;
        let response = self.downstream_rx_ignore_lag().await?;
        Ok(response)
    }

    pub(crate) async fn downstream_rx_ignore_lag(
        &self,
    ) -> Result<ServerJsonRpcMessage, McpPodError> {
        let mut downstream_rx = self.downstream_tx.subscribe();
        for _ in 0..4 {
            tracing::debug!(
                "Waiting for message from pod for session {}",
                self.session_id
            );
            match downstream_rx.recv().await {
                Ok(msg) => return Ok(msg),
                Err(RecvError::Lagged(_)) => {
                    tracing::warn!(
                        "Lagged behind in receiving messages for session {}, ignoring",
                        self.session_id
                    );
                    continue;
                }
                Err(RecvError::Closed) => {
                    tracing::error!("Downstream channel closed for session {}", self.session_id);
                    return Err(McpPodError::SendTransportError);
                }
            }
        }
        Err(McpPodError::SendTransportError)
    }

    pub(crate) async fn upstream_tx_send(
        &self,
        message: RxJsonRpcMessage<RoleServer>,
    ) -> Result<(), McpPodError> {
        self.upstream_tx
            .send(message)
            .await
            .map_err(|_| McpPodError::SendTransportError)
    }

    pub(crate) async fn downstream_rx_stream(
        &self,
    ) -> impl Stream<Item = ServerSseMessage> + Send + use<> {
        let downstream_rx = self.downstream_tx.subscribe();
        BroadcastStream::new(downstream_rx)
            .filter_map(async |x| x.ok())
            .map(|msg| ServerSseMessage {
                event_id: None,
                message: Arc::new(msg),
            })
    }
}
