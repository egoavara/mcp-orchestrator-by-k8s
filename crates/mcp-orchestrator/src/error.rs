use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Namespace not managed by MCP Orchestrator: {0}")]
    NotDefaultNamespaceManaged(String),

    #[error("Invalid label key: {0}")]
    InvalidLabelKey(String),

    #[error("JSON Patch error: {0}")]
    Patch(#[from] json_patch::PatchError),

    #[error("Protected namespace: {0}")]
    ProtectedNamespace(String),
}
