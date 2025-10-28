use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid label key: {0}")]
    InvalidLabelKey(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("JSON Patch error: {0}")]
    Patch(#[from] json_patch::PatchError),

    #[error("Protected namespace: {0}")]
    ProtectedNamespace(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg.clone()),
            AppError::InvalidLabelKey(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InvalidInput(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ProtectedNamespace(msg) => (axum::http::StatusCode::FORBIDDEN, msg.clone()),
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
        };
        (status, message).into_response()
    }
}
