use rmcp::{
    service::ClientInitializeError,
    transport::streamable_http_server::session::local::{EventIdParseError, SessionError},
};

use crate::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum McpPodError {
    #[error(transparent)]
    KubeError(#[from] kube::Error),

    #[error(transparent)]
    StdIoError(#[from] std::io::Error),

    #[error(transparent)]
    SessionError(#[from] SessionError),

    #[error(transparent)]
    AppError(#[from] AppError),

    #[error(transparent)]
    EventIdParseError(#[from] EventIdParseError),

    #[error(transparent)]
    ClientInitializeError(#[from] ClientInitializeError),

    #[error("Pod not found: {session_id}")]
    PodNotFound { session_id: String },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Pod {session_id} has no stdin attached")]
    NoStdin { session_id: String },

    #[error("Pod {session_id} has no stdout attached")]
    NoStdout { session_id: String },

    #[error("Pod {session_id} is not ready yet")]
    NoConnection { session_id: String },

    #[error("Failed to send message to pod")]
    SendTransportError,

    #[error("Authorization failed: {reason}")]
    AuthorizationFailed { reason: String },
}
