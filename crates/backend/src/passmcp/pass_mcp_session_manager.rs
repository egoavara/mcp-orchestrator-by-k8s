use std::{
    collections::{HashMap, HashSet, VecDeque},
    num::ParseIntError,
    sync::Arc,
    time::Duration,
};

use futures::Stream;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{
        mpsc::{Receiver, Sender},
        oneshot, RwLock,
    },
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;

use rmcp::{
    model::{
        CancelledNotificationParam, ClientJsonRpcMessage, ClientNotification, ClientRequest,
        JsonRpcNotification, JsonRpcRequest, Notification, ProgressNotificationParam,
        ProgressToken, RequestId, ServerJsonRpcMessage, ServerNotification,
    },
    transport::{
        async_rw::AsyncRwTransport,
        common::server_side_http::{session_id, ServerSseMessage, SessionId},
        streamable_http_server::SessionManager,
        worker::{Worker, WorkerContext, WorkerQuitReason, WorkerSendRequest},
        WorkerTransport,
    },
    RoleServer,
};

use crate::passmcp::PassMcpError;

#[derive(Debug, Default)]
pub struct PassMcpSessionManager {
    kube_client: kube::Client,
}

impl PassMcpSessionManager {
    pub fn new(kube_client: kube::Client) -> Self {
        Self { kube_client }
    }
}

impl SessionManager for PassMcpSessionManager {
    type Error = PassMcpError;

    type Transport = AsyncRwTransport<
        RoleServer,
        Box<dyn AsyncRead + Unpin + Send>,
        Box<dyn AsyncWrite + Unpin + Send>,
    >;

    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        todo!()
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, Self::Error> {
        todo!()
    }

    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        todo!()
    }

    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        todo!()
    }

    async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        todo!()
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error> {
        todo!()
    }
}
