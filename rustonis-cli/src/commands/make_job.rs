use std::fs;

use anyhow::{bail, Context, Result};
use heck::{ToPascalCase, ToSnakeCase};

use crate::commands::make::find_project_root;

// ─── Point d'entrée ───────────────────────────────────────────────────────────

/// Génère un job Rustonis.
///
/// `name` : ex. "SendEmail", "send-welcome-email", "ProcessOrderJob"
pub fn execute_job(name: &str) -> Result<()> {
    let (struct_name, file_stem) = parse_name(name);
    let project_root = find_project_root()?;

    let jobs_dir = project_root.join("src").join("app").join("jobs");
    let file_path = jobs_dir.join(format!("{file_stem}.rs"));

    if file_path.exists() {
        bail!(
            "Le job {} existe déjà : {}",
            struct_name,
            file_path.display()
        );
    }

    fs::create_dir_all(&jobs_dir)
        .with_context(|| format!("Impossible de créer {}", jobs_dir.display()))?;

    let content = generate_job(&struct_name);

    fs::write(&file_path, content)
        .with_context(|| format!("Impossible d'écrire {}", file_path.display()))?;

    println!("  ✅ CREATED  src/app/jobs/{file_stem}.rs");
    println!();
    println!("👉 Dispatch ce job dans ton code :");
    println!();
    println!("   use crate::app::jobs::{file_stem}::{struct_name};");
    println!();
    println!("   // Immédiat");
    println!("   Dispatcher::dispatch({struct_name} {{ /* champs */ }}).await?;");
    println!();
    println!("   // Avec délai");
    println!("   Dispatcher::dispatch_later({struct_name} {{ /* champs */ }}, Duration::from_secs(60)).await?;");

    Ok(())
}

// ─── Génération du template ───────────────────────────────────────────────────

fn generate_job(struct_name: &str) -> String {
    format!(
        r#"use rustonis_queue::{{Job, JobError}};

pub struct {struct_name} {{
    // Ajoute ici les données nécessaires au job
}}

#[async_trait::async_trait]
impl Job for {struct_name} {{
    async fn handle(&self) -> Result<(), JobError> {{
        // TODO: implémenter la logique du job
        println!("Processing {struct_name}...");
        Ok(())
    }}

    fn max_attempts(&self) -> u32 {{
        3
    }}

    fn queue_name(&self) -> &'static str {{
        "default"
    }}
}}
"#,
        struct_name = struct_name,
    )
}

// ─── Utilitaires ──────────────────────────────────────────────────────────────

/// Transforme l'input utilisateur en (StructName, file_stem).
///
/// Exemples :
/// - "SendEmail"         → ("SendEmailJob", "send_email")
/// - "SendEmailJob"      → ("SendEmailJob", "send_email")
/// - "send-welcome-email" → ("SendWelcomeEmailJob", "send_welcome_email")
fn parse_name(name: &str) -> (String, String) {
    let pascal = name.to_pascal_case();
    let base = pascal
        .strip_suffix("Job")
        .unwrap_or(&pascal)
        .to_string();

    let struct_name = format!("{base}Job");
    let file_stem   = base.to_snake_case();

    (struct_name, file_stem)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_simple() {
        let (s, f) = parse_name("SendEmail");
        assert_eq!(s, "SendEmailJob");
        assert_eq!(f, "send_email");
    }

    #[test]
    fn test_parse_name_already_has_job_suffix() {
        let (s, f) = parse_name("SendEmailJob");
        assert_eq!(s, "SendEmailJob");
        assert_eq!(f, "send_email");
    }

    #[test]
    fn test_parse_name_kebab_case() {
        let (s, f) = parse_name("send-welcome-email");
        assert_eq!(s, "SendWelcomeEmailJob");
        assert_eq!(f, "send_welcome_email");
    }

    #[test]
    fn test_parse_name_snake_case() {
        let (s, f) = parse_name("process_order");
        assert_eq!(s, "ProcessOrderJob");
        assert_eq!(f, "process_order");
    }

    #[test]
    fn test_generate_job_contains_struct_and_impl() {
        // generate_job receives the full struct name (with "Job" suffix)
        let code = generate_job("ProcessOrderJob");
        assert!(code.contains("pub struct ProcessOrderJob"));
        assert!(code.contains("impl Job for ProcessOrderJob"));
        assert!(code.contains("async fn handle("));
        assert!(code.contains("fn max_attempts("));
        assert!(code.contains("fn queue_name("));
    }
}
