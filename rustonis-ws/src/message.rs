use axum::extract::ws::Message as AxumMessage;

/// A WebSocket message, normalised away from the Axum type.
#[derive(Debug, Clone, PartialEq)]
pub enum WsMessage {
    /// UTF-8 text frame.
    Text(String),
    /// Binary frame.
    Binary(Vec<u8>),
    /// Ping frame with optional payload.
    Ping(Vec<u8>),
    /// Pong frame with optional payload.
    Pong(Vec<u8>),
    /// Close frame.
    Close,
}

impl WsMessage {
    /// Convert an Axum [`Message`](AxumMessage) into a [`WsMessage`].
    ///
    /// Returns `None` for message types that Rustonis treats as internal
    /// (e.g. Axum's internal close representation handled by the socket itself).
    pub fn from_axum(msg: AxumMessage) -> Option<Self> {
        match msg {
            AxumMessage::Text(t)   => Some(WsMessage::Text(t.to_string())),
            AxumMessage::Binary(b) => Some(WsMessage::Binary(b.to_vec())),
            AxumMessage::Ping(p)   => Some(WsMessage::Ping(p.to_vec())),
            AxumMessage::Pong(p)   => Some(WsMessage::Pong(p.to_vec())),
            AxumMessage::Close(_)  => Some(WsMessage::Close),
        }
    }

    /// Convert a [`WsMessage`] into an Axum [`Message`](AxumMessage).
    pub fn into_axum(self) -> AxumMessage {
        match self {
            WsMessage::Text(t)   => AxumMessage::Text(t.into()),
            WsMessage::Binary(b) => AxumMessage::Binary(b.into()),
            WsMessage::Ping(p)   => AxumMessage::Ping(p.into()),
            WsMessage::Pong(p)   => AxumMessage::Pong(p.into()),
            WsMessage::Close     => AxumMessage::Close(None),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_round_trip() {
        let msg = WsMessage::Text("hello".to_string());
        let axum_msg = msg.clone().into_axum();
        let recovered = WsMessage::from_axum(axum_msg).unwrap();
        assert_eq!(msg, recovered);
    }

    #[test]
    fn test_binary_round_trip() {
        let msg = WsMessage::Binary(vec![1, 2, 3]);
        let axum_msg = msg.clone().into_axum();
        let recovered = WsMessage::from_axum(axum_msg).unwrap();
        assert_eq!(msg, recovered);
    }

    #[test]
    fn test_close_round_trip() {
        let msg = WsMessage::Close;
        let axum_msg = msg.into_axum();
        let recovered = WsMessage::from_axum(axum_msg).unwrap();
        assert_eq!(recovered, WsMessage::Close);
    }
}
