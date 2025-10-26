use axum::response::IntoResponse;
use rmcp::{
    service::ClientInitializeError,
    transport::streamable_http_server::session::local::{EventIdParseError, SessionError},
};

#[derive(Debug, thiserror::Error)]
pub enum McpPodError {
    #[error(transparent)]
    KubeError(#[from] kube::Error),

    #[error(transparent)]
    StdIoError(#[from] std::io::Error),

    #[error(transparent)]
    SessionError(#[from] SessionError),

    #[error(transparent)]
    EventIdParseError(#[from] EventIdParseError),

    #[error(transparent)]
    ClientInitializeError(#[from] ClientInitializeError),

    #[error("Failed to map pod")]
    MapFailed,

    #[error("Pod not found: {session_id}")]
    PodNotFound { session_id: String },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session duplicate: {session_id}")]
    SessionDuplicate { session_id: String },

    #[error("Pod {session_id} has no stdin attached")]
    NoStdin { session_id: String },

    #[error("Pod {session_id} has no stdout attached")]
    NoStdout { session_id: String },

    #[error("Pod {session_id} is not ready yet")]
    NoConnection { session_id: String },
}
