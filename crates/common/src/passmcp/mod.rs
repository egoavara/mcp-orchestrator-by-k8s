use std::{borrow::Cow, sync::Arc};

use k8s_openapi::api::core::v1::Pod;
use kube::api::{Attach, AttachParams, AttachedProcess};
use rmcp::{
    model::{ClientRequest, ErrorCode, Implementation, InitializeResult, ServerInfo, ServerResult},
    serve_client,
    service::{RequestContext, RunningService},
    transport::{async_rw::AsyncRwTransport, Transport},
    RoleClient, ServiceExt,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::info;

use crate::{consts::MCP_NAMESPACE, failure::Failure};

pub struct PassthroughMcpService {
    pub session_id: String,
    pub kube_client: kube::Client,
    pub attach: Arc<AttachedProcess>,
    pub mcp_client: RunningService<RoleClient, ()>,
}

impl rmcp::Service<rmcp::RoleServer> for PassthroughMcpService {
    async fn handle_request(
        &self,
        request: ClientRequest,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ServerResult, rmcp::ErrorData> {
        self.mcp_client
            .send_request(request)
            .await
            .map_err(|e| rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!(
                    "Failed to forward request to MCP Pod with session_id '{}': {}",
                    self.session_id, e
                )),
                data: None,
            })
    }

    async fn handle_notification(
        &self,
        notification: <rmcp::RoleServer as rmcp::service::ServiceRole>::PeerNot,
        context: rmcp::service::NotificationContext<rmcp::RoleServer>,
    ) -> Result<(), rmcp::ErrorData> {
        self.mcp_client
            .send_notification(notification)
            .await
            .map_err(|e| rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!(
                    "Failed to forward notification to MCP Pod with session_id '{}': {}",
                    self.session_id, e
                )),
                data: None,
            })
    }

    fn get_info(&self) -> <rmcp::RoleServer as rmcp::service::ServiceRole>::Info {
        ServerInfo {
            server_info: Implementation {
                name: "Passthrough MCP Service".to_string(),
                version: "0.1.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl PassthroughMcpService {
    pub async fn new(session_id: String, kube_client: kube::Client) -> Result<Self, Failure> {
        let api: kube::Api<Pod> = kube::Api::namespaced(kube_client.clone(), MCP_NAMESPACE);
        let param = AttachParams {
            stdin: true,
            stdout: true,
            ..Default::default()
        };
        let mut attach = api.attach(&session_id, &param).await.map_err(|e| {
            Failure::internal_error(&format!(
                "Failed to attach to Pod with session_id '{}': {}",
                session_id, e
            ))
        })?;
        let transport =
            AsyncRwTransport::new_client(attach.stdout().unwrap(), attach.stdin().unwrap());
        let mcp_client = serve_client((), transport).await.map_err(|e| {
            Failure::internal_error(&format!(
                "Failed to create MCP client for Pod with session_id '{}': {}",
                session_id, e
            ))
        })?;
        Ok(Self {
            session_id,
            kube_client,
            attach: Arc::new(attach),
            mcp_client,
        })
    }
}
