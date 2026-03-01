use std::sync::Arc;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::{
    context::{ConnId, WsContext, WsSender},
    handler::WsHandler,
    hub::WsHub,
    message::WsMessage,
};

// ─── upgrade ──────────────────────────────────────────────────────────────────

/// Handle a raw [`WebSocket`] connection for a given `handler` and `hub`.
///
/// This is the low-level function called by [`make_ws_handler`]. You rarely need
/// to call it directly.
pub async fn upgrade<H>(ws: WebSocket, handler: Arc<H>, hub: Arc<WsHub>)
where
    H: WsHandler,
{
    let id = ConnId::new();
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<WsMessage>();
    let sender = WsSender(msg_tx);

    // Register in hub
    hub.add(id, sender.clone()).await;

    let ctx = WsContext::new(id, sender, Arc::clone(&hub));

    // Split the socket
    let (mut ws_sink, mut ws_stream) = ws.split();

    // on_connect
    if let Err(e) = handler.on_connect(&ctx).await {
        warn!("{id}: on_connect error: {e}");
    }

    // Forward outbound messages (from WsSender channel) to the real socket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            if ws_sink.send(msg.into_axum()).await.is_err() {
                break;
            }
        }
        // Flush any remaining data
        let _ = ws_sink.close().await;
    });

    // Receive inbound frames from the socket
    while let Some(Ok(raw)) = ws_stream.next().await {
        match WsMessage::from_axum(raw) {
            Some(WsMessage::Close) | None => break,
            Some(msg) => {
                debug!("{id}: received {msg:?}");
                if let Err(e) = handler.on_message(&ctx, msg).await {
                    warn!("{id}: on_message error: {e}");
                }
            }
        }
    }

    // Cleanup
    handler.on_disconnect(&ctx).await;
    hub.remove(id).await;
    send_task.abort();
    debug!("{id}: disconnected");
}

// ─── make_ws_handler ──────────────────────────────────────────────────────────

/// Create an Axum route handler for a given [`WsHandler`] and [`WsHub`].
///
/// Returns a closure that can be used directly with `.route("/ws", get(…))`.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use axum::{Router, routing::get};
/// use rustonis_ws::{make_ws_handler, WsHub};
/// # use async_trait::async_trait;
/// # use rustonis_ws::{WsContext, WsError, WsHandler, WsMessage};
/// # struct MyHandler;
/// # #[async_trait]
/// # impl WsHandler for MyHandler {
/// #   async fn on_connect(&self, _: &WsContext) -> Result<(), WsError> { Ok(()) }
/// #   async fn on_message(&self, _: &WsContext, _: WsMessage) -> Result<(), WsError> { Ok(()) }
/// #   async fn on_disconnect(&self, _: &WsContext) {}
/// # }
///
/// let hub     = Arc::new(WsHub::new());
/// let handler = Arc::new(MyHandler);
/// let router: Router = Router::new()
///     .route("/ws", get(make_ws_handler(handler, hub)));
/// ```
pub fn make_ws_handler<H>(
    handler: Arc<H>,
    hub: Arc<WsHub>,
) -> impl Fn(WebSocketUpgrade) -> std::pin::Pin<Box<dyn std::future::Future<Output = axum::response::Response> + Send>>
       + Clone
       + Send
       + 'static
where
    H: WsHandler,
{
    move |ws: WebSocketUpgrade| {
        let h = Arc::clone(&handler);
        let hub = Arc::clone(&hub);
        Box::pin(async move {
            ws.on_upgrade(move |socket| upgrade(socket, h, hub))
                .into_response()
        })
    }
}

// ─── Axum State-based handler ─────────────────────────────────────────────────

/// State wrapper for Axum's `State` extractor pattern.
///
/// Useful when you prefer to use `.with_state(…)` instead of closures.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use axum::{Router, routing::get};
/// use rustonis_ws::{WsHub, WsRouteState, ws_route_handler};
/// # use async_trait::async_trait;
/// # use rustonis_ws::{WsContext, WsError, WsHandler, WsMessage};
/// # #[derive(Clone)]
/// # struct MyHandler;
/// # #[async_trait]
/// # impl WsHandler for MyHandler {
/// #   async fn on_connect(&self, _: &WsContext) -> Result<(), WsError> { Ok(()) }
/// #   async fn on_message(&self, _: &WsContext, _: WsMessage) -> Result<(), WsError> { Ok(()) }
/// #   async fn on_disconnect(&self, _: &WsContext) {}
/// # }
///
/// let state = WsRouteState::new(Arc::new(MyHandler), Arc::new(WsHub::new()));
/// // Mount using axum's with_state pattern
/// let router: Router<WsRouteState<MyHandler>> = Router::new()
///     .route("/ws", get(ws_route_handler::<MyHandler>));
/// let _router: Router = router.with_state(state);
/// ```
#[derive(Clone)]
pub struct WsRouteState<H: WsHandler> {
    pub handler: Arc<H>,
    pub hub: Arc<WsHub>,
}

impl<H: WsHandler> WsRouteState<H> {
    pub fn new(handler: Arc<H>, hub: Arc<WsHub>) -> Self {
        WsRouteState { handler, hub }
    }
}

/// Axum handler that reads its dependencies from [`WsRouteState`].
pub async fn ws_route_handler<H: WsHandler>(
    ws: WebSocketUpgrade,
    State(state): State<WsRouteState<H>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| upgrade(socket, state.handler, state.hub))
}
