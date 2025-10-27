use gloo_net::http::{Request, Response};
use prost::Message;

pub fn get_base_url() -> String {
    web_sys::window()
        .unwrap()
        .location()
        .origin()
        .unwrap()
}

pub async fn grpc_web_call<Req, Res>(
    service_method: &str,
    request: Req,
) -> Result<Res, String>
where
    Req: Message,
    Res: Message + Default,
{
    // Encode the protobuf message
    let mut message_buf = Vec::new();
    request.encode(&mut message_buf).map_err(|e| format!("Encode error: {}", e))?;

    // gRPC-Web frame format: 1 byte (compression flag) + 4 bytes (message length) + message
    let message_len = message_buf.len() as u32;
    let mut frame = Vec::with_capacity(5 + message_buf.len());
    frame.push(0); // No compression
    frame.extend_from_slice(&message_len.to_be_bytes());
    frame.extend_from_slice(&message_buf);

    let url = format!("{}{}", get_base_url(), service_method);
    
    let response: Response = Request::post(&url)
        .header("Content-Type", "application/grpc-web+proto")
        .header("X-Grpc-Web", "1")
        .body(frame)
        .map_err(|e| format!("Body error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    // Check for gRPC errors in headers
    let headers = &response.headers();
    if let Some(grpc_status) = headers.get("grpc-status") {
        if grpc_status != "0" {
            let grpc_message = headers
                .get("grpc-message")
                .unwrap_or("Unknown error".to_string());
            return Err(format!("gRPC error (status {}): {}", grpc_status, grpc_message));
        }
    }

    if !response.ok() {
        return Err(format!(
            "HTTP error: status {} - {}",
            response.status(),
            response.status_text()
        ));
    }

    let bytes = response
        .binary()
        .await
        .map_err(|e| format!("Response read error: {}", e))?;

    // Parse gRPC-Web response frame
    if bytes.len() < 5 {
        return Err("Response too short".to_string());
    }

    let compression_flag = bytes[0];
    let message_length = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;

    if compression_flag != 0 {
        return Err(format!("Unsupported compression flag: {}", compression_flag));
    }

    if bytes.len() < 5 + message_length {
        return Err(format!(
            "Response too short: expected {} bytes, got {}",
            5 + message_length,
            bytes.len()
        ));
    }

    // Extract message bytes (skip frame header)
    let message_bytes = &bytes[5..5 + message_length];

    // Decode the protobuf message
    let decoded = Res::decode(message_bytes)
        .map_err(|e| format!("Decode error: {}", e))?;

    // Note: Trailers (if present) start at bytes[5 + message_length]
    // They have format: [0x80][4 bytes length][trailer data]
    // We ignore trailers since grpc-status is already in HTTP headers

    Ok(decoded)
}
