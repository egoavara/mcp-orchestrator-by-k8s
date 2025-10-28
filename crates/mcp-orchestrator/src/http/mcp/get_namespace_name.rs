use std::fmt::Display;

use axum::{
    body::Bytes,
    extract::{Path, Request, State},
    http,
    http::Response,
};
use http_body::Body;
use http_body_util::{BodyExt, Full};
use rmcp::transport::common::http_header::{
    EVENT_STREAM_MIME_TYPE, HEADER_LAST_EVENT_ID, HEADER_SESSION_ID,
};

use crate::http::mcp::utils::{BoxResponse, sse_stream_response};
use crate::{
    http::mcp::utils::{get_session_manager, internal_error_response},
    state::AppState,
};

pub async fn handler<B>(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    request: Request<B>,
) -> Result<BoxResponse, BoxResponse>
where
    B: Body + Send + 'static,
    B::Error: Display,
{
    let session_manager = get_session_manager(&state, &namespace, &name).await?;

    // check accept header
    if !request
        .headers()
        .get(http::header::ACCEPT)
        .and_then(|header| header.to_str().ok())
        .is_some_and(|header| header.contains(EVENT_STREAM_MIME_TYPE))
    {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_ACCEPTABLE)
            .body(
                Full::new(Bytes::from(
                    "Not Acceptable: Client must accept text/event-stream",
                ))
                .boxed(),
            )
            .expect("valid response"));
    }
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
    // check if session exists
    let has_session = session_manager
        .has_session(&session_id)
        .await
        .map_err(internal_error_response("check session"))?;
    if !has_session {
        // unauthorized
        return Ok(Response::builder()
            .status(http::StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from("Unauthorized: Session not found")).boxed())
            .expect("valid response"));
    }
    // check if last event id is provided
    let last_event_id = request
        .headers()
        .get(HEADER_LAST_EVENT_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());
    if let Some(last_event_id) = last_event_id {
        // check if session has this event id
        let stream = session_manager
            .resume(&session_id, last_event_id)
            .await
            .map_err(internal_error_response("resume session"))?;
        Ok(sse_stream_response(stream, state.config.mcp.keep_alive))
    } else {
        // create standalone stream
        let stream = session_manager
            .create_standalone_stream(&session_id)
            .await
            .map_err(internal_error_response("create standalone stream"))?;
        Ok(sse_stream_response(stream, state.config.mcp.keep_alive))
    }
}
