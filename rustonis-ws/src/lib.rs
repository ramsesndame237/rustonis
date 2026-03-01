//! `rustonis-ws` — WebSocket support for the Rustonis framework.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use axum::{Router, routing::get};
//! use rustonis_ws::{Hub, WsContext, WsError, WsHandler, WsHub, WsMessage, make_ws_handler};
//!
//! // 1. Define a handler
//! pub struct ChatHandler;
//!
//! #[async_trait]
//! impl WsHandler for ChatHandler {
//!     async fn on_connect(&self, ctx: &WsContext) -> Result<(), WsError> {
//!         ctx.join("lobby").await;
//!         ctx.send_text("Welcome!")?;
//!         Ok(())
//!     }
//!
//!     async fn on_message(&self, ctx: &WsContext, msg: WsMessage) -> Result<(), WsError> {
//!         ctx.broadcast_to_others("lobby", msg).await;
//!         Ok(())
//!     }
//!
//!     async fn on_disconnect(&self, ctx: &WsContext) {
//!         ctx.leave("lobby").await;
//!     }
//! }
//!
//! # async fn run() {
//! // 2. Boot the hub
//! Hub::init(WsHub::new());
//! let hub = Hub::get().unwrap();
//!
//! // 3. Mount the handler
//! let handler = Arc::new(ChatHandler);
//! let router: Router = Router::new()
//!     .route("/ws/chat", get(make_ws_handler(handler, hub)));
//! # }
//! ```

pub mod context;
pub mod error;
pub mod handler;
pub mod hub;
pub mod message;
pub mod router;

pub use context::{ConnId, WsContext, WsSender};
pub use error::WsError;
pub use handler::WsHandler;
pub use hub::{Hub, WsHub};
pub use message::WsMessage;
pub use router::{make_ws_handler, upgrade, ws_route_handler, WsRouteState};

/// Convenience re-export for `use rustonis_ws::prelude::*`.
pub mod prelude {
    pub use crate::{
        ConnId, Hub, WsContext, WsError, WsHandler, WsHub, WsMessage, WsSender,
        make_ws_handler, ws_route_handler, WsRouteState,
    };
}
