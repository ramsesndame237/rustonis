use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

// ─── Point d'entrée ───────────────────────────────────────────────────────────

/// Génère un model et optionnellement sa migration.
pub fn execute_model(name: &str, with_migration: bool) -> Result<()> {
    let (struct_name, file_stem, table_name) = parse_name(name);
    let project_root = find_project_root()?;

    generate_model_file(&project_root, &struct_name, &file_stem, &table_name)?;

    if with_migration {
        generate_migration_file(&project_root, &table_name)?;
    }

    println!();
    println!("👉 Utilisation dans un controller :");
    println!();
    println!("   use crate::app::models::{file_stem}::{struct_name};");
    println!("   use rustonis_orm::prelude::*;");
    println!();
    println!("   let users = {struct_name}::all(&pool).await?;");
    println!("   let user  = {struct_name}::find(1, &pool).await?;");
    println!("   let list  = {struct_name}::query()");
    println!("       .where_eq(\"email\", \"alice@example.com\")");
    println!("       .order_by(\"created_at\", \"DESC\")");
    println!("       .limit(10)");
    println!("       .all(&pool).await?;");

    Ok(())
}

// ─── Génération du model ──────────────────────────────────────────────────────

fn generate_model_file(
    root: &PathBuf,
    struct_name: &str,
    file_stem: &str,
    table_name: &str,
) -> Result<()> {
    let models_dir = root.join("src").join("app").join("models");
    let file_path  = models_dir.join(format!("{file_stem}.rs"));

    if file_path.exists() {
        bail!("Le model {struct_name} existe déjà : {}", file_path.display());
    }

    fs::create_dir_all(&models_dir)
        .with_context(|| format!("Impossible de créer {}", models_dir.display()))?;

    let content = format!(
        r#"use serde::{{Deserialize, Serialize}};
use rustonis_orm::prelude::*;

#[derive(model, Debug, Clone, Serialize, Deserialize)]
#[model(table = "{table_name}")]
pub struct {struct_name} {{
    pub id:         i64,
    // Ajoute tes champs ici :
    // pub name:    String,
    // pub email:   String,
    // pub active:  bool,
}}
"#
    );

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/models/{file_stem}.rs");
    Ok(())
}

// ─── Génération de la migration ───────────────────────────────────────────────

fn generate_migration_file(root: &PathBuf, table_name: &str) -> Result<()> {
    let migrations_dir = root.join("database").join("migrations");
    fs::create_dir_all(&migrations_dir)
        .with_context(|| format!("Impossible de créer {}", migrations_dir.display()))?;

    let timestamp = chrono_now();
    let file_name = format!("{timestamp}_create_{table_name}_table.sql");
    let file_path = migrations_dir.join(&file_name);

    let content = format!(
        r#"CREATE TABLE {table_name} (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Down
DROP TABLE {table_name};
"#
    );

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!(
        "  ✅ CREATED  database/migrations/{file_name}"
    );
    Ok(())
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

fn parse_name(name: &str) -> (String, String, String) {
    let pascal = name.to_pascal_case();
    let base   = pascal
        .strip_suffix("Model")
        .unwrap_or(&pascal)
        .to_string();

    let struct_name = base.clone();
    let file_stem   = base.to_snake_case();
    let table_name  = format!("{}s", file_stem);

    (struct_name, file_stem, table_name)
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

/// Returns a compact UTC timestamp: YYYYMMDDHHMMSS
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple manual conversion (no chrono dep in CLI)
    let (mut s, epoch) = (secs, 0u64);
    let _ = epoch;
    // Use seconds-since-epoch formatted as a sortable timestamp approximation
    // For real timestamps, the project can add chrono — here we keep it simple.
    format!("{:014}", s)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_simple() {
        let (s, f, t) = parse_name("User");
        assert_eq!(s, "User");
        assert_eq!(f, "user");
        assert_eq!(t, "users");
    }

    #[test]
    fn test_parse_name_pascal() {
        let (s, f, t) = parse_name("BlogPost");
        assert_eq!(s, "BlogPost");
        assert_eq!(f, "blog_post");
        assert_eq!(t, "blog_posts");
    }

    #[test]
    fn test_parse_name_strips_suffix() {
        let (s, f, t) = parse_name("UserModel");
        assert_eq!(s, "User");
        assert_eq!(f, "user");
        assert_eq!(t, "users");
    }
}
