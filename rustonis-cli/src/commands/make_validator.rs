use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

// ─── Point d'entrée ───────────────────────────────────────────────────────────

/// Génère un validator Rustonis.
///
/// `name` : ex. "CreateUser", "login", "blog-post"
pub fn execute_validator(name: &str) -> Result<()> {
    let (struct_name, file_stem) = parse_name(name);
    let project_root = find_project_root()?;

    let validators_dir = project_root.join("src").join("app").join("validators");
    let file_path = validators_dir.join(format!("{file_stem}_validator.rs"));

    if file_path.exists() {
        bail!(
            "Le validator {} existe déjà : {}",
            struct_name,
            file_path.display()
        );
    }

    fs::create_dir_all(&validators_dir)
        .with_context(|| format!("Impossible de créer {}", validators_dir.display()))?;

    let content = generate_validator(&struct_name);

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/validators/{file_stem}_validator.rs");
    println!();
    println!("👉 Utilisation dans un controller :");
    println!();
    println!("   use crate::app::validators::{file_stem}_validator::{struct_name};");
    println!("   use rustonis_validator::Valid;");
    println!();
    println!("   pub async fn create(Valid(input): Valid<{struct_name}>) -> impl IntoResponse {{");
    println!("       // input est garanti valide");
    println!("   }}");

    Ok(())
}

// ─── Génération du template ───────────────────────────────────────────────────

fn generate_validator(struct_name: &str) -> String {
    format!(
        r#"use serde::Deserialize;
use rustonis_validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct {struct_name} {{
    // Exemple :
    //
    // #[validate(email)]
    // pub email: String,
    //
    // #[validate(min_length = 8, max_length = 100)]
    // pub password: String,
    //
    // #[validate(required)]
    // pub name: String,
    //
    // #[validate(min = 18, max = 120)]
    // pub age: u32,
}}
"#,
        struct_name = struct_name
    )
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

fn parse_name(name: &str) -> (String, String) {
    let pascal = name.to_pascal_case();

    // Retirer le suffixe "Validator" s'il est déjà là
    let base = pascal
        .strip_suffix("Validator")
        .unwrap_or(&pascal)
        .to_string();

    let struct_name = format!("{base}Validator");
    let file_stem = base.to_snake_case();

    (struct_name, file_stem)
}

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
        let (s, f) = parse_name("CreateUser");
        assert_eq!(s, "CreateUserValidator");
        assert_eq!(f, "create_user");
    }

    #[test]
    fn test_parse_name_already_has_suffix() {
        let (s, f) = parse_name("CreateUserValidator");
        assert_eq!(s, "CreateUserValidator");
        assert_eq!(f, "create_user");
    }

    #[test]
    fn test_parse_name_kebab() {
        let (s, f) = parse_name("blog-post");
        assert_eq!(s, "BlogPostValidator");
        assert_eq!(f, "blog_post");
    }

    #[test]
    fn test_generate_contains_struct() {
        let code = generate_validator("CreateUserValidator");
        assert!(code.contains("pub struct CreateUserValidator"));
        assert!(code.contains("#[derive(Debug, Deserialize, Validate)]"));
    }
}
