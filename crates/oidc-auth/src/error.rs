
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    FailedPassthroughQueryParam(#[from] serde_qs::Error),

    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthHeaderFormat,

    #[error("Unknown key ID")]
    UnknownKeyId,

    #[error("Invalid token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    #[error("Token expired")]
    TokenExpired,

    #[error("OIDC discovery error: {0}")]
    DiscoveryError(String),

    #[error("Jwk not private key: kid = {0:?}")]
    InvalidPrivateKey(Option<String>),

    #[error("Invalid Base64 encoding for JWK: kid = {0:?}")]
    InvalidPrivateKeyBase64(Option<String>),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::FailedPassthroughQueryParam(err) => (
                StatusCode::BAD_REQUEST,
                format!("Failed to passthrough query parameters: {}", err),
            ),
            AuthError::UnknownKeyId => (StatusCode::UNAUTHORIZED, "Unknown key ID".to_string()),
            AuthError::MissingAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".to_string(),
            ),
            AuthError::InvalidAuthHeaderFormat => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format".to_string(),
            ),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AuthError::DiscoveryError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AuthError::InvalidPrivateKey(kid) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid private key in JWK: kid = {:?}", kid),
            ),
            AuthError::InvalidPrivateKeyBase64(kid) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid Base64 encoding for JWK private key: kid = {:?}", kid),
            ),
        };

        (status, message).into_response()
    }
}
