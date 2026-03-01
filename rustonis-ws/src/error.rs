use thiserror::Error;

/// Errors that can occur in the WebSocket subsystem.
#[derive(Debug, Error)]
pub enum WsError {
    /// The hub has not been initialised via [`Hub::init`](crate::Hub::init).
    #[error("WebSocket hub not initialised — call Hub::init() at application startup")]
    NotInitialized,

    /// Sending a message to a connection failed (connection likely closed).
    #[error("Failed to send message to connection {0}: {1}")]
    SendFailed(u64, String),

    /// The requested connection was not found in the hub.
    #[error("Connection {0} not found")]
    ConnectionNotFound(u64),

    /// Generic WebSocket protocol error.
    #[error("WebSocket error: {0}")]
    Protocol(String),
}
