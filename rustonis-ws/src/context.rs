use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tokio::sync::mpsc;

use crate::{error::WsError, hub::WsHub, message::WsMessage};

// ─── ConnId ───────────────────────────────────────────────────────────────────

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnId(pub(crate) u64);

impl ConnId {
    /// Generate a new unique connection ID.
    pub fn new() -> Self {
        ConnId(NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Return the raw numeric value.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl Default for ConnId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "conn-{}", self.0)
    }
}

// ─── WsSender ─────────────────────────────────────────────────────────────────

/// A cheap-to-clone handle for sending messages to a single WebSocket connection.
#[derive(Debug, Clone)]
pub struct WsSender(pub(crate) mpsc::UnboundedSender<WsMessage>);

impl WsSender {
    /// Send a message to this connection.
    ///
    /// Returns an error if the connection's receive-side has been closed.
    pub fn send(&self, msg: WsMessage) -> Result<(), WsError> {
        self.0
            .send(msg)
            .map_err(|e| WsError::SendFailed(0, e.to_string()))
    }
}

// ─── WsContext ────────────────────────────────────────────────────────────────

/// Per-connection context passed to every [`WsHandler`](crate::WsHandler) callback.
///
/// Provides helpers for sending messages, joining / leaving rooms, and broadcasting.
#[derive(Debug, Clone)]
pub struct WsContext {
    pub id: ConnId,
    sender: WsSender,
    hub: Arc<WsHub>,
}

impl WsContext {
    pub(crate) fn new(id: ConnId, sender: WsSender, hub: Arc<WsHub>) -> Self {
        WsContext { id, sender, hub }
    }

    // ── Unicast ───────────────────────────────────────────────────────────────

    /// Send a message to this connection only.
    pub fn send(&self, msg: WsMessage) -> Result<(), WsError> {
        self.sender
            .0
            .send(msg)
            .map_err(|e| WsError::SendFailed(self.id.0, e.to_string()))
    }

    /// Send a text message to this connection.
    pub fn send_text(&self, text: impl Into<String>) -> Result<(), WsError> {
        self.send(WsMessage::Text(text.into()))
    }

    // ── Rooms ─────────────────────────────────────────────────────────────────

    /// Join a named room.
    pub async fn join(&self, room: impl Into<String>) {
        self.hub.join(room.into(), self.id).await;
    }

    /// Leave a named room.
    pub async fn leave(&self, room: impl Into<String>) {
        self.hub.leave(&room.into(), self.id).await;
    }

    // ── Broadcast ─────────────────────────────────────────────────────────────

    /// Broadcast a message to all connections in `room` (including self).
    pub async fn broadcast_to(&self, room: &str, msg: WsMessage) {
        self.hub.broadcast_room(room, msg).await;
    }

    /// Broadcast a message to all connections in `room` except self.
    pub async fn broadcast_to_others(&self, room: &str, msg: WsMessage) {
        self.hub.broadcast_room_except(room, self.id, msg).await;
    }

    /// Broadcast a message to **all** connected clients (global broadcast).
    pub async fn broadcast_all(&self, msg: WsMessage) {
        self.hub.broadcast_all(msg).await;
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conn_id_is_unique() {
        let a = ConnId::new();
        let b = ConnId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_conn_id_display() {
        let id = ConnId(42);
        assert_eq!(id.to_string(), "conn-42");
    }

    #[tokio::test]
    async fn test_ws_sender_send() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender = WsSender(tx);

        sender.send(WsMessage::Text("hello".to_string())).unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, WsMessage::Text("hello".to_string()));
    }
}
