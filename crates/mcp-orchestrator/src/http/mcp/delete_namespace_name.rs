use axum::{
    body::{Body, Bytes},
    extract::{Path, Request, State},
    http::{self, Response},
};
use http_body_util::{BodyExt, Full};
use rmcp::transport::common::http_header::HEADER_SESSION_ID;

use crate::http::mcp::utils::{BoxResponse, get_session_manager};
use crate::{
    http::mcp::utils::{accepted_response, internal_error_response},
    state::AppState,
};

pub async fn handler(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<BoxResponse, BoxResponse> {
    let session_manager = get_session_manager(&state, &namespace, &name).await?;

    // check session id
    let session_id = request
        .headers()
        .get(HEADER_SESSION_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned().into());
    let Some(session_id) = session_id else {
        // unauthorized
        return Ok(Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from("Unauthorized: Session ID is required")).boxed())
            .expect("valid response"));
    };
    // close session
    session_manager
        .close_session(&session_id)
        .await
        .map_err(internal_error_response("close session"))?;
    Ok(accepted_response())
}
