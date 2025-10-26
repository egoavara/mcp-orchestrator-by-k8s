use axum::http::request::Parts;
use chrono::{DateTime, Utc};
use futures::{FutureExt, TryFutureExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client, ResourceExt,
    api::{AttachParams, AttachedProcess, ListParams},
};
use rmcp::{
    RoleClient, RoleServer, ServiceError,
    model::{ErrorCode, Extensions, GetExtensions, ServerInfo},
    serve_client,
    service::{
        NotificationContext, RequestContext, RunningService, RxJsonRpcMessage, TxJsonRpcMessage,
    },
    transport::{
        IntoTransport, Transport, async_rw::AsyncRwTransport, streamable_http_server::SessionId,
    },
};
use std::{borrow::Cow, cell::Cell, future::Future, mem::swap, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;

use crate::podmcp::{
    LABEL_KEY_SESSION_ID, McpPodError, PodMcp, PodMcpSessionManager, PodMcpTransport,
};

#[derive(Clone)]
pub struct PodMcpProxy {
    manager: PodMcp,
    session_id: Arc<Mutex<Option<SessionId>>>,
    last_event_time: Arc<Mutex<DateTime<Utc>>>,
}

impl PodMcpProxy {
    pub fn new(manager: PodMcp) -> Self {
        Self {
            manager,
            session_id: Arc::new(Mutex::new(None)),
            last_event_time: Arc::new(Mutex::new(DateTime::<Utc>::MIN_UTC.clone())),
        }
    }
    pub fn new_with_id(manager: PodMcp, session_id: SessionId) -> Self {
        Self {
            manager,
            session_id: Arc::new(Mutex::new(Some(session_id))),
            last_event_time: Arc::new(Mutex::new(Utc::now())),
        }
    }
    async fn check_session(&self, ext: &Extensions) -> Result<SessionId, rmcp::ErrorData> {
        let mut lock = self.session_id.lock().await;
        let no_session_id = lock.is_none();
        if no_session_id {
            if let Some(session_id) = ext.get::<SessionId>() {
                lock.replace(session_id.clone());
                return Ok(session_id.clone());
            } else {
                return Err(rmcp::ErrorData {
                    code: ErrorCode::METHOD_NOT_FOUND,
                    message: Cow::Borrowed("Method not implemented in PodMcpProxy"),
                    data: None,
                });
            }
        }
        Ok(lock.clone().unwrap())
    }
    async fn update_last_event_time(
        &self,
        transport: &PodMcpTransport,
    ) -> Result<(), rmcp::ErrorData> {
        let now = Utc::now();
        let prev = {
            let mut old = now.clone();
            let mut lock = self.last_event_time.lock().await;
            swap(&mut *lock, &mut old);
            old
        };
        let dur = now.signed_duration_since(prev);
        if dur > chrono::Duration::seconds(60) {
            tracing::info!(
                "Session {} received event after {}",
                &transport.session_id,
                &dur
            );

            transport
                .update_last_access_time()
                .await
                .map_err(|err| rmcp::ErrorData {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: Cow::Owned(format!("Failed to update last access time: {}", err)),
                    data: None,
                })?;
        }
        Ok(())
    }
}

impl rmcp::Service<RoleServer> for PodMcpProxy {
    async fn handle_request(
        &self,
        request: <RoleServer as rmcp::service::ServiceRole>::PeerReq,
        _context: RequestContext<RoleServer>,
    ) -> Result<<RoleServer as rmcp::service::ServiceRole>::Resp, rmcp::ErrorData> {
        let session_id = self.check_session(request.extensions()).await?;
        let transport = self
            .manager
            .get_transport(&session_id)
            .await
            .ok_or_else(|| rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!("session = {} not found", session_id)),
                data: None,
            })?;
        self.update_last_event_time(&transport).await?;

        match transport.remote.send_request(request).await {
            Ok(response) => Ok(response),
            Err(ServiceError::McpError(err)) => Err(err),
            Err(err) => Err(rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!("Failed to send request: {}", err)),
                data: None,
            }),
        }
    }

    async fn handle_notification(
        &self,
        notification: <RoleServer as rmcp::service::ServiceRole>::PeerNot,
        _context: NotificationContext<RoleServer>,
    ) -> Result<(), rmcp::ErrorData> {
        let session_id = self.check_session(notification.extensions()).await?;
        let transport = self
            .manager
            .get_transport(&session_id)
            .await
            .ok_or_else(|| rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!("session = {} not found", session_id)),
                data: None,
            })?;
        self.update_last_event_time(&transport).await?;

        match transport.remote.send_notification(notification).await {
            Ok(_) => Ok(()),
            Err(ServiceError::McpError(err)) => Err(err),
            Err(err) => Err(rmcp::ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(format!("Failed to send request: {}", err)),
                data: None,
            }),
        }
    }

    fn get_info(&self) -> <RoleServer as rmcp::service::ServiceRole>::Info {
        ServerInfo {
            ..Default::default()
        }
    }
}
