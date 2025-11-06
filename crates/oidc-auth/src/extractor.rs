use std::sync::Arc;

use crate::manager::AuthManager;
use crate::{AuthError, Claims};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
pub struct AuthenticatedUser(pub Claims);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        OptionalAuthenticatedUser::from_request_parts(parts, _state)
            .await?
            .0
            .map(AuthenticatedUser)
            .ok_or_else(|| {
                let Some(manager) = parts.extensions.get::<Arc<AuthManager>>().cloned()else{
                    let response = (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "manager not configured. Make sure AuthManager is added as layer to the stack.",
                    )
                        .into_response();
                    return response;
                };
                let mut response = AuthError::MissingAuthHeader.into_response();
                manager.www_authenticate(response.headers_mut());
                response
            })
    }
}

#[derive(Clone)]
pub struct OptionalAuthenticatedUser(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalAuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<OptionalAuthenticatedUser>() {
            tracing::debug!("user already authenticated from extension");
            return Ok(user.clone());
        }

        let Some(authorization) = parts.headers.get(http::header::AUTHORIZATION) else {
            tracing::debug!("no authorization header found");
            return Ok(OptionalAuthenticatedUser(None));
        };

        let Some(manager) = parts.extensions.get::<Arc<AuthManager>>().cloned() else {
            tracing::debug!("AuthManager not configured in extensions");
            return Ok(OptionalAuthenticatedUser(None));
        };

        let Ok(authorization) = authorization.to_str() else {
            tracing::warn!("invalid authorization header format: not valid UTF-8");
            let mut response = AuthError::InvalidAuthHeaderFormat.into_response();
            manager.www_authenticate(response.headers_mut());
            return Err(response);
        };

        let token = match extract_bearer_token(authorization) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("failed to extract bearer token: {:?}", e);
                let mut response = e.into_response();
                manager.www_authenticate(response.headers_mut());
                return Err(response);
            }
        };

        let token_header = match jsonwebtoken::decode_header(token.as_bytes()) {
            Ok(token_header) => token_header,
            Err(err) => {
                tracing::warn!("failed to decode token header: {}", err);
                let mut response = AuthError::InvalidToken(err).into_response();
                manager.www_authenticate(response.headers_mut());
                return Err(response);
            }
        };

        tracing::debug!(
            "token header decoded, kid: {:?}, alg: {:?}",
            token_header.kid,
            token_header.alg
        );

        let Some(decoding_key) = manager.get_decoding_key(&token_header.kid) else {
            tracing::warn!("unknown key id: {:?}", token_header.kid);
            let mut response = AuthError::UnknownKeyId.into_response();
            manager.www_authenticate(response.headers_mut());
            return Err(response);
        };

        let validation = manager.validator(token_header);

        let token =
            match jsonwebtoken::decode::<crate::claim::Claims>(&token, decoding_key, &validation) {
                Ok(t) => t,
                Err(err) => {
                    tracing::warn!(error=%err, "token validation failed (middleware)");
                    let mut response = AuthError::InvalidToken(jsonwebtoken::errors::Error::from(
                        jsonwebtoken::errors::ErrorKind::InvalidToken,
                    ))
                    .into_response();
                    manager.www_authenticate(response.headers_mut());
                    return Err(response);
                }
            };
        Ok(OptionalAuthenticatedUser(Some(token.claims)))
    }
}

pub(crate) fn extract_bearer_token(auth_header: &str) -> Result<String, AuthError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidAuthHeaderFormat);
    }
    Ok(auth_header[7..].to_string())
}
