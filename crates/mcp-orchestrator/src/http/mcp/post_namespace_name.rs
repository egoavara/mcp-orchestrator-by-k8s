use axum::{
    body::{Body, Bytes},
    extract::{Path, Request, State},
    http::{self, Response},
};
use http_body_util::{BodyExt, Full};
use rmcp::{
    model::{ClientJsonRpcMessage, ClientRequest, GetExtensions},
    transport::common::{
        http_header::{EVENT_STREAM_MIME_TYPE, HEADER_SESSION_ID, JSON_MIME_TYPE},
        server_side_http::ServerSseMessage,
    },
};

use crate::http::mcp::utils::{
    accepted_response, expect_json, internal_error_response, sse_stream_response,
    unexpected_message_response,
};
use crate::{
    http::mcp::utils::{BoxResponse, get_session_manager},
    state::AppState,
};

pub async fn handler(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<BoxResponse, BoxResponse> {
    let session_manager = get_session_manager(&state, &namespace, &name).await?;
    // check accept header
    if !request
        .headers()
        .get(http::header::ACCEPT)
        .and_then(|header| header.to_str().ok())
        .is_some_and(|header| {
            header.contains(JSON_MIME_TYPE) && header.contains(EVENT_STREAM_MIME_TYPE)
        })
    {
        return Ok(Response::builder()
                .status(http::StatusCode::NOT_ACCEPTABLE)
                .body(Full::new(Bytes::from("Not Acceptable: Client must accept both application/json and text/event-stream")).boxed())
                .expect("valid response"));
    }

    // check content type
    if !request
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|header| header.to_str().ok())
        .is_some_and(|header| header.starts_with(JSON_MIME_TYPE))
    {
        return Ok(Response::builder()
            .status(http::StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .body(
                Full::new(Bytes::from(
                    "Unsupported Media Type: Content-Type must be application/json",
                ))
                .boxed(),
            )
            .expect("valid response"));
    }

    // json deserialize request body
    let (part, body) = request.into_parts();
    let mut message = match expect_json(body).await {
        Ok(message) => message,
        Err(response) => return Ok(response),
    };

    // do we have a session id?
    let session_id = part
        .headers
        .get(HEADER_SESSION_ID)
        .and_then(|v| v.to_str().ok());
    if let Some(session_id) = session_id {
        let session_id = session_id.to_owned().into();
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

        // inject request part to extensions
        match &mut message {
            ClientJsonRpcMessage::Request(req) => {
                req.request.extensions_mut().insert(part);
            }
            ClientJsonRpcMessage::Notification(not) => {
                not.notification.extensions_mut().insert(part);
            }
            _ => {
                // skip
            }
        }

        match message {
            ClientJsonRpcMessage::Request(_) => {
                let stream = session_manager
                    .create_stream(&session_id, message)
                    .await
                    .map_err(internal_error_response("get session"))?;
                Ok(sse_stream_response(stream, state.config.mcp.keep_alive))
            }
            ClientJsonRpcMessage::Notification(_)
            | ClientJsonRpcMessage::Response(_)
            | ClientJsonRpcMessage::Error(_) => {
                // handle notification
                session_manager
                    .accept_message(&session_id, message)
                    .await
                    .map_err(internal_error_response("accept message"))?;
                Ok(accepted_response())
            }
        }
    } else {
        let session_id = session_manager
            .create_session()
            .await
            .map_err(internal_error_response("create session"))?;
        if let ClientJsonRpcMessage::Request(req) = &mut message {
            if !matches!(req.request, ClientRequest::InitializeRequest(_)) {
                return Err(unexpected_message_response("initialize request"));
            }
            // inject request part to extensions
            req.request.extensions_mut().insert(part);
        } else {
            return Err(unexpected_message_response("initialize request"));
        }
        // get initialize response
        let response = session_manager
            .initialize_session(&session_id, message)
            .await
            .map_err(internal_error_response("create stream"))?;
        let mut response = sse_stream_response(
            futures::stream::once({
                async move {
                    ServerSseMessage {
                        event_id: None,
                        message: response.into(),
                    }
                }
            }),
            state.config.mcp.keep_alive,
        );

        response.headers_mut().insert(
            HEADER_SESSION_ID,
            session_id
                .parse()
                .map_err(internal_error_response("create session id header"))?,
        );
        Ok(response)
    }
}
