use chrono::Utc;
use futures::{FutureExt, TryFutureExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client, ResourceExt,
    api::{AttachParams, AttachedProcess, ListParams, Patch, PatchParams},
};
use rmcp::{
    RoleClient, RoleServer,
    model::{ClientRequest, RequestNoParam, ServerInfo},
    serve_client,
    service::{
        NotificationContext, RequestContext, RunningService, RxJsonRpcMessage, TxJsonRpcMessage,
    },
    transport::{
        IntoTransport, Transport, async_rw::AsyncRwTransport, streamable_http_server::session,
    },
};
use serde_json::json;
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    podmcp::{LABEL_KEY_SESSION_ID, McpPodError, PodMcpSessionManager},
    state,
    storage::annotations::ANNOTATION_LAST_ACCESS_AT,
};

#[derive(Clone)]
pub struct PodMcpTransport {
    pub(crate) session_id: String,
    api: Api<Pod>,
    attach: Arc<AttachedProcess>,
    pub(crate) remote: Arc<RunningService<RoleClient, ()>>,
}

impl PodMcpTransport {
    pub async fn connect(
        client: Client,
        namespace: &str,
        session_id: &str,
    ) -> Result<Self, McpPodError> {
        let api = Api::<Pod>::namespaced(client.clone(), namespace);
        let label_selector = format!("{}={}", LABEL_KEY_SESSION_ID, session_id);
        let param = ListParams::default().labels(&label_selector).limit(2);
        let mut attach = Option::<AttachedProcess>::None;

        for _ in 0..16 {
            let mut pods = api
                .list(&param)
                .await
                .map_err(|_| McpPodError::PodNotFound {
                    session_id: session_id.to_string(),
                })?;
            if pods.items.len() != 1 {
                return Err(McpPodError::SessionDuplicate {
                    session_id: session_id.to_string(),
                });
            }

            let pod = pods.items.remove(0);
            if let Some(state) = &pod.status {
                if state.phase.as_deref() != Some("Running") {
                    tracing::debug!("Pod {} is not ready yet", pod.name_any());
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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
                .stderr(true)
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
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }

        let Some(mut attach) = attach else {
            return Err(McpPodError::NoConnection {
                session_id: session_id.to_string(),
            });
        };

        let stdin = attach.stdin().ok_or_else(|| McpPodError::NoStdin {
            session_id: session_id.to_string(),
        })?;
        let stdout = attach.stdout().ok_or_else(|| McpPodError::NoStdout {
            session_id: session_id.to_string(),
        })?;
        let transport = AsyncRwTransport::new_client(stdout, stdin);
        let remote = serve_client((), transport).await?;
        let ping_handler = remote.clone();
        let ping_session_id = session_id.to_string();
        tokio::spawn(async move {
            let remote = ping_handler;
            let session_id = ping_session_id;
            while !remote.is_transport_closed() {
                let ping_fut = remote
                    .send_request(ClientRequest::PingRequest(RequestNoParam {
                        method: rmcp::model::PingRequestMethod,
                        extensions: Default::default(),
                    }))
                    .await;
                match ping_fut {
                    Ok(_) => {
                        tracing::debug!("Ping to session {} successful", session_id);
                    }
                    Err(err) => {
                        tracing::warn!("Ping to session {} failed: {}", session_id, err);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            }
        });
        Ok(Self {
            session_id: session_id.to_string(),
            api,
            attach: Arc::new(attach),
            remote: Arc::new(remote),
        })
    }

    pub async fn update_last_access_time(&self) -> Result<(), McpPodError> {
        tracing::debug!("Updating last access time for session {}", self.session_id);
        let now = Utc::now().to_rfc3339();
        let patch = Patch::Strategic(json!({
            "metadata": {
                "annotations": {
                    ANNOTATION_LAST_ACCESS_AT: &now,
                }
            }
        }));
        self.api
            .patch(&self.session_id, &PatchParams::default(), &patch)
            .await?;
        Ok(())
    }
}
