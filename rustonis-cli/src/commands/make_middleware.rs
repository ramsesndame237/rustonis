use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

// ─── Point d'entrée ───────────────────────────────────────────────────────────

/// Génère un fichier de middleware Axum dans `src/app/middleware/{stem}.rs`.
pub fn execute_middleware(name: &str) -> Result<()> {
    let (fn_name, file_stem) = parse_name(name);
    let project_root = find_project_root()?;

    generate_middleware_file(&project_root, &fn_name, &file_stem)?;

    println!();
    println!("👉 Enregistre le middleware dans start/kernel.rs :");
    println!();
    println!("   use axum::middleware;");
    println!("   use crate::app::middleware::{file_stem}::{fn_name};");
    println!();
    println!("   router.layer(middleware::from_fn({fn_name}))");

    Ok(())
}

// ─── Génération ───────────────────────────────────────────────────────────────

fn generate_middleware_file(root: &PathBuf, fn_name: &str, file_stem: &str) -> Result<()> {
    let dir       = root.join("src").join("app").join("middleware");
    let file_path = dir.join(format!("{file_stem}.rs"));

    if file_path.exists() {
        bail!(
            "Le middleware {} existe déjà : {}",
            fn_name,
            file_path.display()
        );
    }

    fs::create_dir_all(&dir)
        .with_context(|| format!("Impossible de créer {}", dir.display()))?;

    let content = format!(
        r#"use axum::{{extract::Request, middleware::Next, response::Response}};

/// {fn_name} middleware.
///
/// Enregistrement dans `start/kernel.rs` :
/// ```rust,ignore
/// router.layer(axum::middleware::from_fn({fn_name}))
/// ```
pub async fn {fn_name}(request: Request, next: Next) -> Response {{
    // TODO: implement middleware logic
    next.run(request).await
}}
"#
    );

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/middleware/{file_stem}.rs");
    Ok(())
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

/// Normalise le nom en `(fn_name, file_stem)`.
///
/// Exemples :
/// - `"Auth"` → `("auth", "auth")`
/// - `"AuthMiddleware"` → `("auth", "auth")`
/// - `"rate-limit"` → `("rate_limit", "rate_limit")`
fn parse_name(name: &str) -> (String, String) {
    let pascal = name.to_pascal_case();
    let base   = pascal
        .strip_suffix("Middleware")
        .unwrap_or(&pascal)
        .to_string();

    let fn_name   = base.to_snake_case();
    let file_stem = fn_name.clone();
    (fn_name, file_stem)
}

fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let has_cargo = current.join("Cargo.toml").exists();
        let has_env   = current.join(".env").exists()
            || current.join(".env.example").exists();
        if has_cargo && has_env {
            return Ok(current);
        }
        match current.parent() {
            Some(p) => current = p.to_path_buf(),
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
        let (f, s) = parse_name("auth");
        assert_eq!(f, "auth");
        assert_eq!(s, "auth");
    }

    #[test]
    fn test_parse_name_pascal() {
        let (f, s) = parse_name("RateLimit");
        assert_eq!(f, "rate_limit");
        assert_eq!(s, "rate_limit");
    }

    #[test]
    fn test_parse_name_kebab() {
        let (f, s) = parse_name("verify-token");
        assert_eq!(f, "verify_token");
        assert_eq!(s, "verify_token");
    }

    #[test]
    fn test_parse_name_strips_suffix() {
        let (f, s) = parse_name("AuthMiddleware");
        assert_eq!(f, "auth");
        assert_eq!(s, "auth");
    }
}
