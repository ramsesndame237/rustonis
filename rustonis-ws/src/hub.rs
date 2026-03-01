use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use tokio::sync::RwLock;
use tracing::warn;

use crate::{context::ConnId, error::WsError, message::WsMessage};
use crate::context::WsSender;

// ─── WsHub ────────────────────────────────────────────────────────────────────

/// Central registry that tracks all live connections and room memberships.
///
/// Connections are stored as [`WsSender`] handles; rooms map a name to a set of
/// [`ConnId`]s.
#[derive(Debug, Default)]
pub struct WsHub {
    /// All live connections: `ConnId → WsSender`
    connections: RwLock<HashMap<ConnId, WsSender>>,

    /// Room → set of ConnId
    rooms: RwLock<HashMap<String, HashSet<ConnId>>>,
}

impl WsHub {
    /// Create a new, empty hub.
    pub fn new() -> Self {
        WsHub::default()
    }

    // ── Connection lifecycle ───────────────────────────────────────────────────

    /// Register a new connection.
    pub async fn add(&self, id: ConnId, sender: WsSender) {
        self.connections.write().await.insert(id, sender);
    }

    /// Remove a connection and clean it up from all rooms.
    pub async fn remove(&self, id: ConnId) {
        self.connections.write().await.remove(&id);

        let mut rooms = self.rooms.write().await;
        for members in rooms.values_mut() {
            members.remove(&id);
        }
        rooms.retain(|_, members| !members.is_empty());
    }

    /// Number of live connections.
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    // ── Rooms ─────────────────────────────────────────────────────────────────

    /// Add `id` to `room`.
    pub async fn join(&self, room: String, id: ConnId) {
        self.rooms
            .write()
            .await
            .entry(room)
            .or_default()
            .insert(id);
    }

    /// Remove `id` from `room`.
    pub async fn leave(&self, room: &str, id: ConnId) {
        let mut rooms = self.rooms.write().await;
        if let Some(members) = rooms.get_mut(room) {
            members.remove(&id);
            if members.is_empty() {
                rooms.remove(room);
            }
        }
    }

    /// Number of connections currently in `room`.
    pub async fn room_size(&self, room: &str) -> usize {
        self.rooms
            .read()
            .await
            .get(room)
            .map_or(0, |m| m.len())
    }

    // ── Broadcast helpers ─────────────────────────────────────────────────────

    /// Send `msg` to every connection in `room`.
    pub async fn broadcast_room(&self, room: &str, msg: WsMessage) {
        let ids: Vec<ConnId> = {
            let rooms = self.rooms.read().await;
            rooms
                .get(room)
                .map(|m| m.iter().copied().collect())
                .unwrap_or_default()
        };
        self.send_to_ids(&ids, msg).await;
    }

    /// Send `msg` to every connection in `room` except `exclude`.
    pub async fn broadcast_room_except(&self, room: &str, exclude: ConnId, msg: WsMessage) {
        let ids: Vec<ConnId> = {
            let rooms = self.rooms.read().await;
            rooms
                .get(room)
                .map(|m| m.iter().copied().filter(|&c| c != exclude).collect())
                .unwrap_or_default()
        };
        self.send_to_ids(&ids, msg).await;
    }

    /// Send `msg` to **all** live connections.
    pub async fn broadcast_all(&self, msg: WsMessage) {
        let ids: Vec<ConnId> = {
            self.connections
                .read()
                .await
                .keys()
                .copied()
                .collect()
        };
        self.send_to_ids(&ids, msg).await;
    }

    /// Send `msg` to a specific connection.
    pub async fn send_to(&self, id: ConnId, msg: WsMessage) -> Result<(), WsError> {
        let conns = self.connections.read().await;
        match conns.get(&id) {
            Some(sender) => sender.send(msg),
            None => Err(WsError::ConnectionNotFound(id.0)),
        }
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    async fn send_to_ids(&self, ids: &[ConnId], msg: WsMessage) {
        let conns = self.connections.read().await;
        for &id in ids {
            if let Some(sender) = conns.get(&id) {
                if let Err(e) = sender.send(msg.clone()) {
                    warn!("Failed to send to {}: {}", id, e);
                }
            }
        }
    }
}

// ─── Hub (global facade) ──────────────────────────────────────────────────────

static HUB: OnceLock<Arc<WsHub>> = OnceLock::new();

/// Global WebSocket hub facade.
///
/// Initialise once at application startup, then access from anywhere.
///
/// ```rust,no_run
/// use rustonis_ws::{Hub, WsHub};
///
/// Hub::init(WsHub::new());
/// let hub = Hub::get().unwrap();
/// ```
pub struct Hub;

impl Hub {
    /// Initialise the global hub.  Subsequent calls are silently ignored.
    pub fn init(hub: WsHub) {
        HUB.set(Arc::new(hub)).ok();
    }

    /// Obtain a reference to the global hub.
    pub fn get() -> Result<Arc<WsHub>, WsError> {
        HUB.get().cloned().ok_or(WsError::NotInitialized)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use crate::context::WsSender;

    fn make_sender() -> (WsSender, tokio::sync::mpsc::UnboundedReceiver<WsMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (WsSender(tx), rx)
    }

    #[tokio::test]
    async fn test_add_and_count() {
        let hub = WsHub::new();
        let (sender, _rx) = make_sender();
        let id = ConnId::new();
        hub.add(id, sender).await;
        assert_eq!(hub.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_remove_cleans_rooms() {
        let hub = WsHub::new();
        let (sender, _rx) = make_sender();
        let id = ConnId::new();
        hub.add(id, sender).await;
        hub.join("room1".into(), id).await;
        assert_eq!(hub.room_size("room1").await, 1);

        hub.remove(id).await;
        assert_eq!(hub.connection_count().await, 0);
        assert_eq!(hub.room_size("room1").await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_room() {
        let hub = WsHub::new();
        let (s1, mut rx1) = make_sender();
        let (s2, mut rx2) = make_sender();
        let id1 = ConnId::new();
        let id2 = ConnId::new();

        hub.add(id1, s1).await;
        hub.add(id2, s2).await;
        hub.join("chat".into(), id1).await;
        hub.join("chat".into(), id2).await;

        hub.broadcast_room("chat", WsMessage::Text("hi".into())).await;

        assert_eq!(rx1.recv().await.unwrap(), WsMessage::Text("hi".into()));
        assert_eq!(rx2.recv().await.unwrap(), WsMessage::Text("hi".into()));
    }

    #[tokio::test]
    async fn test_broadcast_room_except() {
        let hub = WsHub::new();
        let (s1, mut rx1) = make_sender();
        let (s2, mut rx2) = make_sender();
        let id1 = ConnId::new();
        let id2 = ConnId::new();

        hub.add(id1, s1).await;
        hub.add(id2, s2).await;
        hub.join("chat".into(), id1).await;
        hub.join("chat".into(), id2).await;

        hub.broadcast_room_except("chat", id1, WsMessage::Text("hello".into()))
            .await;

        // id1 should NOT receive
        assert!(rx1.try_recv().is_err());
        // id2 should receive
        assert_eq!(rx2.recv().await.unwrap(), WsMessage::Text("hello".into()));
    }

    #[tokio::test]
    async fn test_send_to_specific() {
        let hub = WsHub::new();
        let (sender, mut rx) = make_sender();
        let id = ConnId::new();
        hub.add(id, sender).await;

        hub.send_to(id, WsMessage::Text("direct".into()))
            .await
            .unwrap();
        assert_eq!(rx.recv().await.unwrap(), WsMessage::Text("direct".into()));
    }

    #[tokio::test]
    async fn test_send_to_missing_returns_error() {
        let hub = WsHub::new();
        let fake_id = ConnId(99999);
        let result = hub.send_to(fake_id, WsMessage::Text("x".into())).await;
        assert!(result.is_err());
    }
}
