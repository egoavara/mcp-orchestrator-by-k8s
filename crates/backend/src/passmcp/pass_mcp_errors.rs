use thiserror::Error;

#[derive(Debug, Error)]
pub enum PassMcpError {
    // #[error("Invalid request id: {0}")]
    // DuplicatedRequestId(HttpRequestId),
    // #[error("Channel closed: {0:?}")]
    // ChannelClosed(Option<HttpRequestId>),
    // #[error("Cannot parse event id: {0}")]
    // EventIdParseError(#[from] EventIdParseError),
    // #[error("Session service terminated")]
    // SessionServiceTerminated,
    #[error("Invalid event id")]
    InvalidEventId,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<PassMcpError> for std::io::Error {
    fn from(value: PassMcpError) -> Self {
        match value {
            PassMcpError::Io(io) => io,
            _ => std::io::Error::other(format!("PassMcp error: {value}")),
        }
    }
}
