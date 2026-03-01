use std::fs;

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

use crate::commands::make::find_project_root;

// ─── Entry point ──────────────────────────────────────────────────────────────

/// Generate a Rustonis WebSocket handler.
///
/// `name` : e.g. "Chat", "live-feed", "NotificationHandler"
pub fn execute_ws(name: &str) -> Result<()> {
    let (struct_name, file_stem) = parse_name(name);
    let project_root = find_project_root()?;

    let ws_dir = project_root.join("src").join("app").join("ws");
    let file_path = ws_dir.join(format!("{file_stem}.rs"));

    if file_path.exists() {
        bail!(
            "Le handler WebSocket {} existe déjà : {}",
            struct_name,
            file_path.display()
        );
    }

    fs::create_dir_all(&ws_dir)
        .with_context(|| format!("Impossible de créer {}", ws_dir.display()))?;

    let content = generate_ws_handler(&struct_name);

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/ws/{file_stem}.rs");
    println!();
    println!("👉 Monte ce handler dans ton routeur :");
    println!();
    println!("   use std::sync::Arc;");
    println!("   use axum::{{Router, routing::get}};");
    println!("   use rustonis_ws::{{Hub, WsHub, make_ws_handler}};");
    println!("   use crate::app::ws::{file_stem}::{struct_name};");
    println!();
    println!("   Hub::init(WsHub::new());");
    println!("   let hub     = Hub::get()?;");
    println!("   let handler = Arc::new({struct_name});");
    println!("   let router  = Router::new()");
    println!("       .route(\"/ws/{file_stem}\", get(make_ws_handler(handler, hub)));");

    Ok(())
}

// ─── Template ─────────────────────────────────────────────────────────────────

fn generate_ws_handler(struct_name: &str) -> String {
    format!(
        r#"use async_trait::async_trait;
use rustonis_ws::{{WsContext, WsError, WsHandler, WsMessage}};

pub struct {struct_name};

#[async_trait]
impl WsHandler for {struct_name} {{
    async fn on_connect(&self, ctx: &WsContext) -> Result<(), WsError> {{
        // TODO: handle new connection (e.g. join a room)
        ctx.send_text(format!("Connected as {{}}", ctx.id))?;
        Ok(())
    }}

    async fn on_message(&self, ctx: &WsContext, msg: WsMessage) -> Result<(), WsError> {{
        // TODO: handle incoming messages
        match msg {{
            WsMessage::Text(text) => {{
                println!("[{struct_name}] received: {{text}}");
                ctx.send_text(format!("Echo: {{text}}"))?;
            }}
            _ => {{}}
        }}
        Ok(())
    }}

    async fn on_disconnect(&self, ctx: &WsContext) {{
        // TODO: clean up on disconnect
        println!("[{struct_name}] {{}} disconnected", ctx.id);
    }}
}}
"#,
        struct_name = struct_name,
    )
}

// ─── Utilities ────────────────────────────────────────────────────────────────

/// Transform user input into (StructName, file_stem).
///
/// Examples:
/// - "Chat"            → ("ChatHandler", "chat")
/// - "ChatHandler"     → ("ChatHandler", "chat")
/// - "live-feed"       → ("LiveFeedHandler", "live_feed")
fn parse_name(name: &str) -> (String, String) {
    let pascal = name.to_pascal_case();
    let base = pascal
        .strip_suffix("Handler")
        .unwrap_or(&pascal)
        .to_string();

    let struct_name = format!("{base}Handler");
    let file_stem = base.to_snake_case();

    (struct_name, file_stem)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_simple() {
        let (s, f) = parse_name("Chat");
        assert_eq!(s, "ChatHandler");
        assert_eq!(f, "chat");
    }

    #[test]
    fn test_parse_name_already_has_handler_suffix() {
        let (s, f) = parse_name("ChatHandler");
        assert_eq!(s, "ChatHandler");
        assert_eq!(f, "chat");
    }

    #[test]
    fn test_parse_name_kebab_case() {
        let (s, f) = parse_name("live-feed");
        assert_eq!(s, "LiveFeedHandler");
        assert_eq!(f, "live_feed");
    }

    #[test]
    fn test_parse_name_snake_case() {
        let (s, f) = parse_name("notification_center");
        assert_eq!(s, "NotificationCenterHandler");
        assert_eq!(f, "notification_center");
    }

    #[test]
    fn test_generate_ws_handler_contains_struct_and_impl() {
        let code = generate_ws_handler("ChatHandler");
        assert!(code.contains("pub struct ChatHandler"));
        assert!(code.contains("impl WsHandler for ChatHandler"));
        assert!(code.contains("async fn on_connect("));
        assert!(code.contains("async fn on_message("));
        assert!(code.contains("async fn on_disconnect("));
    }
}
