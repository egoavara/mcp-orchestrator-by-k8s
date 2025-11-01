use axum::response::{IntoResponse, Response};

pub struct Qs<T>(pub T);

impl<S, T> axum::extract::FromRequestParts<S> for Qs<T>
where
    S: Send + Sync,
    T: for<'de> serde::Deserialize<'de>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        let value = serde_qs::from_str::<T>(query).map_err(|e| {
            tracing::error!("Failed to parse query parameters: {}", e);
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Failed to parse query parameters: {}", e),
            )
                .into_response()
        })?;
        Ok(Qs(value))
    }
}
