use std::{
    collections::{HashMap, HashSet, VecDeque},
    future::Future,
    num::ParseIntError,
    sync::Arc,
    time::Duration,
};

use futures::Stream;
use thiserror::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot, RwLock,
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
        worker::{Worker, WorkerContext, WorkerQuitReason, WorkerSendRequest},
        Transport, WorkerTransport,
    },
    RoleServer,
};

use crate::passmcp::PassMcpError;
