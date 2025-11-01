use crate::middleware::RequiredAuthMiddleware;
use crate::{AuthError, OpenIdConfig};
use jsonwebtoken::{DecodingKey, Validation};
use openidconnect::RedirectUrl;
use tower_layer::Layer;

#[derive(Clone)]
pub struct RequiredAuthLayer;

impl RequiredAuthLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequiredAuthLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RequiredAuthLayer {
    type Service = RequiredAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequiredAuthMiddleware::new(inner)
    }
}
