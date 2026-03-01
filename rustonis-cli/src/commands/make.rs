use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

// ─── Point d'entrée ───────────────────────────────────────────────────────────

/// Génère un controller Rustonis.
///
/// `name`     : ex. "User", "UserController", "blog-post"
/// `resource` : si true, génère les 5 méthodes CRUD complètes
pub fn execute_controller(name: &str, resource: bool) -> Result<()> {
    let (struct_name, file_stem) = parse_name(name);
    let project_root = find_project_root()?;

    let controllers_dir = project_root.join("src").join("app").join("controllers");
    let file_path = controllers_dir.join(format!("{file_stem}_controller.rs"));

    if file_path.exists() {
        bail!(
            "Le controller {} existe déjà : {}",
            struct_name,
            file_path.display()
        );
    }

    fs::create_dir_all(&controllers_dir)
        .with_context(|| format!("Impossible de créer {}", controllers_dir.display()))?;

    // Les générateurs attendent le nom de base (sans suffixe Controller)
    let base = struct_name
        .strip_suffix("Controller")
        .unwrap_or(&struct_name);

    let content = if resource {
        generate_resource_controller(base)
    } else {
        generate_basic_controller(base)
    };

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/controllers/{file_stem}_controller.rs");
    println!();

    // Afficher le snippet à ajouter dans start/routes.rs
    // struct_name contient déjà le suffixe Controller (ex: "UserController")
    println!("👉 Ajoute dans src/start/routes.rs :");
    println!();
    if resource {
        println!("   use crate::app::controllers::{file_stem}_controller::{struct_name};");
        println!();
        println!("   router");
        println!("       .get(\"/{file_stem}s\", {struct_name}::index)");
        println!("       .post(\"/{file_stem}s\", {struct_name}::create)");
        println!("       .get(\"/{file_stem}s/:id\", {struct_name}::show)");
        println!("       .put(\"/{file_stem}s/:id\", {struct_name}::update)");
        println!("       .delete(\"/{file_stem}s/:id\", {struct_name}::destroy)");
    } else {
        println!("   use crate::app::controllers::{file_stem}_controller::{struct_name};");
        println!();
        println!("   router.get(\"/{file_stem}s\", {struct_name}::index)");
    }

    Ok(())
}

// ─── Génération des templates ─────────────────────────────────────────────────

fn generate_basic_controller(struct_name: &str) -> String {
    format!(
        r#"use axum::Json;
use serde_json::{{json, Value}};

pub struct {struct_name}Controller;

impl {struct_name}Controller {{
    /// GET /{snake}s
    pub async fn index() -> Json<Value> {{
        Json(json!({{ "data": [] }}))
    }}
}}
"#,
        struct_name = struct_name,
        snake = struct_name.to_snake_case(),
    )
}

fn generate_resource_controller(struct_name: &str) -> String {
    let snake = struct_name.to_snake_case();
    format!(
        r#"use axum::{{
    extract::{{Json, Path}},
    http::StatusCode,
    response::IntoResponse,
}};
use serde_json::{{json, Value}};

pub struct {struct_name}Controller;

impl {struct_name}Controller {{
    /// GET /{snake}s
    pub async fn index() -> impl IntoResponse {{
        Json(json!({{ "data": [] }}))
    }}

    /// GET /{snake}s/:id
    pub async fn show(Path(id): Path<String>) -> impl IntoResponse {{
        Json(json!({{ "data": {{ "id": id }} }}))
    }}

    /// POST /{snake}s
    pub async fn create(Json(body): Json<Value>) -> impl IntoResponse {{
        (StatusCode::CREATED, Json(json!({{ "data": body }})))
    }}

    /// PUT /{snake}s/:id
    pub async fn update(Path(id): Path<String>, Json(body): Json<Value>) -> impl IntoResponse {{
        Json(json!({{ "data": {{ "id": id, "updated": body }} }}))
    }}

    /// DELETE /{snake}s/:id
    pub async fn destroy(Path(id): Path<String>) -> impl IntoResponse {{
        StatusCode::NO_CONTENT
    }}
}}
"#,
        struct_name = struct_name,
        snake = snake,
    )
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

/// Transforme l'input utilisateur en (StructName, file_stem).
///
/// Exemples :
/// - "User"           → ("UserController", "user")
/// - "UserController" → ("UserController", "user")
/// - "blog-post"      → ("BlogPostController", "blog_post")
/// - "BlogPostAdmin"  → ("BlogPostAdminController", "blog_post_admin")
fn parse_name(name: &str) -> (String, String) {
    // Normaliser en PascalCase d'abord
    let pascal = name.to_pascal_case();

    // Retirer le suffixe "Controller" s'il est déjà là
    let base = pascal
        .strip_suffix("Controller")
        .unwrap_or(&pascal)
        .to_string();

    let struct_name = format!("{base}Controller");
    let file_stem = base.to_snake_case();

    (struct_name, file_stem)
}

/// Cherche le répertoire racine du projet Rustonis.
///
/// Un projet Rustonis est identifié par la présence de :
/// - `Cargo.toml`
/// - `.env` ou `.env.example`
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let has_cargo = current.join("Cargo.toml").exists();
        let has_env = current.join(".env").exists() || current.join(".env.example").exists();

        if has_cargo && has_env {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => bail!(
                "Aucun projet Rustonis trouvé.\n\
                 Lance cette commande depuis la racine d'un projet créé avec `rustonis new`."
            ),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_simple() {
        let (struct_name, file_stem) = parse_name("User");
        assert_eq!(struct_name, "UserController");
        assert_eq!(file_stem, "user");
    }

    #[test]
    fn test_parse_name_already_has_controller_suffix() {
        let (struct_name, file_stem) = parse_name("UserController");
        assert_eq!(struct_name, "UserController");
        assert_eq!(file_stem, "user");
    }

    #[test]
    fn test_parse_name_kebab_case() {
        let (struct_name, file_stem) = parse_name("blog-post");
        assert_eq!(struct_name, "BlogPostController");
        assert_eq!(file_stem, "blog_post");
    }

    #[test]
    fn test_parse_name_pascal_case() {
        let (struct_name, file_stem) = parse_name("BlogPostAdmin");
        assert_eq!(struct_name, "BlogPostAdminController");
        assert_eq!(file_stem, "blog_post_admin");
    }

    #[test]
    fn test_parse_name_snake_case() {
        let (struct_name, file_stem) = parse_name("blog_post");
        assert_eq!(struct_name, "BlogPostController");
        assert_eq!(file_stem, "blog_post");
    }

    #[test]
    fn test_generate_basic_controller_contains_struct() {
        let code = generate_basic_controller("User");
        assert!(code.contains("pub struct UserController"));
        assert!(code.contains("pub async fn index()"));
    }

    #[test]
    fn test_generate_resource_controller_has_all_methods() {
        let code = generate_resource_controller("Post");
        assert!(code.contains("pub struct PostController"));
        assert!(code.contains("pub async fn index()"));
        assert!(code.contains("pub async fn show("));
        assert!(code.contains("pub async fn create("));
        assert!(code.contains("pub async fn update("));
        assert!(code.contains("pub async fn destroy("));
    }
}
