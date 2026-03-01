use async_trait::async_trait;

use crate::{context::WsContext, error::WsError, message::WsMessage};

/// Implement this trait to define the behaviour of a WebSocket endpoint.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use rustonis_ws::{WsContext, WsError, WsHandler, WsMessage};
///
/// pub struct ChatHandler;
///
/// #[async_trait]
/// impl WsHandler for ChatHandler {
///     async fn on_connect(&self, ctx: &WsContext) -> Result<(), WsError> {
///         ctx.join("lobby").await;
///         ctx.send_text("Welcome to the chat!")?;
///         Ok(())
///     }
///
///     async fn on_message(&self, ctx: &WsContext, msg: WsMessage) -> Result<(), WsError> {
///         ctx.broadcast_to_others("lobby", msg).await;
///         Ok(())
///     }
///
///     async fn on_disconnect(&self, ctx: &WsContext) {
///         ctx.leave("lobby").await;
///     }
/// }
/// ```
#[async_trait]
pub trait WsHandler: Send + Sync + 'static {
    /// Called once when a new WebSocket connection is established.
    async fn on_connect(&self, ctx: &WsContext) -> Result<(), WsError>;

    /// Called each time the client sends a message.
    async fn on_message(&self, ctx: &WsContext, msg: WsMessage) -> Result<(), WsError>;

    /// Called when the connection is closed (gracefully or not).
    async fn on_disconnect(&self, ctx: &WsContext);
}
