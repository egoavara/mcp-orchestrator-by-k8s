use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;

use crate::extractor::extract_bearer_token;
use crate::manager::AuthManager;
use crate::{AuthError, OptionalAuthenticatedUser};

#[derive(Clone)]
pub struct RequiredAuthMiddleware<S> {
    inner: S,
}

impl<S> RequiredAuthMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<Request> for RequiredAuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let Some(manager) = req.extensions().get::<Arc<AuthManager>>().cloned() else {
                tracing::error!("AuthManager not configured in extensions (middleware)");
                let response = (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "manager not configured. Make sure AuthManager is added as layer to the stack.",
                )
                    .into_response();
                return Ok(response);
            };

            let Some(authorization) = req.headers().get(http::header::AUTHORIZATION) else {
                tracing::warn!("missing authorization header in required auth middleware");
                let mut response = AuthError::MissingAuthHeader.into_response();
                manager.www_authenticate(response.headers_mut());
                return Ok(response);
            };
            let Ok(authorization) = authorization.to_str() else {
                tracing::warn!("invalid authorization header format: not valid UTF-8 (middleware)");
                let mut response = AuthError::InvalidAuthHeaderFormat.into_response();
                manager.www_authenticate(response.headers_mut());
                return Ok(response);
            };

            let token = match extract_bearer_token(authorization) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("failed to extract bearer token (middleware): {:?}", e);
                    let mut response = e.into_response();
                    manager.www_authenticate(response.headers_mut());
                    return Ok(response);
                }
            };

            let token_header = match jsonwebtoken::decode_header(token.as_bytes()) {
                Ok(token_header) => token_header,
                Err(err) => {
                    tracing::warn!("failed to decode token header (middleware): {}", err);
                    let mut response = AuthError::InvalidToken(err).into_response();
                    manager.www_authenticate(response.headers_mut());
                    return Ok(response);
                }
            };

            tracing::debug!(
                "token header decoded (middleware), kid: {:?}, alg: {:?}",
                token_header.kid,
                token_header.alg
            );

            let Some(decoding_key) = manager.get_decoding_key(&token_header.kid) else {
                tracing::warn!("unknown key id (middleware): {:?}", token_header.kid);
                let mut response = AuthError::UnknownKeyId.into_response();
                manager.www_authenticate(response.headers_mut());
                return Ok(response);
            };

            let validation = manager.validator(token_header);

            let token = match jsonwebtoken::decode::<crate::claim::Claims>(
                &token,
                decoding_key,
                &validation,
            ) {
                Ok(t) => t,
                Err(err) => {
                    tracing::warn!(error=%err, "token validation failed (middleware)");
                    let mut response = AuthError::InvalidToken(jsonwebtoken::errors::Error::from(
                        jsonwebtoken::errors::ErrorKind::InvalidToken,
                    ))
                    .into_response();
                    manager.www_authenticate(response.headers_mut());
                    return Ok(response);
                }
            };

            tracing::debug!(
                "user authenticated successfully (middleware): sub={}",
                token.claims.sub
            );
            req.extensions_mut()
                .insert(OptionalAuthenticatedUser(Some(token.claims)));

            inner.call(req).await
        })
    }
}
