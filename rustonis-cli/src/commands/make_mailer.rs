use std::{fs, path::PathBuf};

use super::make::find_project_root;
use anyhow::{Context, Result};

/// Execute `rustonis make mailer <name>`.
pub fn execute_mailer(name: &str) -> Result<()> {
    let (struct_name, file_stem) = parse_name(name);
    let root = find_project_root()?;
    let dir  = root.join("src").join("app").join("mailers");

    fs::create_dir_all(&dir)
        .with_context(|| format!("Cannot create directory {}", dir.display()))?;

    let file_path = dir.join(format!("{}.rs", file_stem));

    if file_path.exists() {
        anyhow::bail!("File already exists: {}", file_path.display());
    }

    let content = generate_mailer(&struct_name, &file_stem);
    fs::write(&file_path, &content)
        .with_context(|| format!("Cannot write {}", file_path.display()))?;

    println!("✅ Created: {}", relative(&root, &file_path));
    Ok(())
}

fn generate_mailer(struct_name: &str, _file_stem: &str) -> String {
    format!(
        r#"use rustonis_mailer::{{MailMessage, MailError}};

pub struct {struct_name};

impl {struct_name} {{
    /// Build the mail message. Called by the mailer infrastructure.
    pub fn build(&self) -> Result<MailMessage, MailError> {{
        Ok(MailMessage::new()
            .to("recipient@example.com")
            .subject("Subject here")
            .html("<h1>Hello!</h1>")
            .text("Hello!"))
    }}
}}
"#
    )
}

fn relative(root: &PathBuf, path: &PathBuf) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

/// Parse the user-supplied name into `(StructName, file_stem)`.
///
/// * `WelcomeMailer`  → `("WelcomeMailer",  "welcome")`
/// * `welcome`        → `("WelcomeMailer",  "welcome")`
/// * `welcome-email`  → `("WelcomeEmailMailer", "welcome_email")`
pub fn parse_name(raw: &str) -> (String, String) {
    // Remove Mailer suffix if present (case-insensitive)
    let stripped = if raw.to_lowercase().ends_with("mailer") {
        &raw[..raw.len() - "mailer".len()]
    } else {
        raw
    };

    // Normalise to words
    let words: Vec<String> = stripped
        .replace('-', "_")
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();

    let pascal  = format!("{}Mailer", words.join(""));
    let snake   = words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("_");

    (pascal, snake)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_simple() {
        let (s, f) = parse_name("welcome");
        assert_eq!(s, "WelcomeMailer");
        assert_eq!(f, "welcome");
    }

    #[test]
    fn test_parse_name_strips_suffix() {
        let (s, f) = parse_name("WelcomeMailer");
        assert_eq!(s, "WelcomeMailer");
        assert_eq!(f, "welcome");
    }

    #[test]
    fn test_parse_name_kebab() {
        let (s, f) = parse_name("order-confirmation");
        assert_eq!(s, "OrderConfirmationMailer");
        assert_eq!(f, "order_confirmation");
    }

    #[test]
    fn test_generate_contains_build_method() {
        let content = generate_mailer("WelcomeMailer", "welcome");
        assert!(content.contains("pub struct WelcomeMailer"));
        assert!(content.contains("pub fn build"));
        assert!(content.contains("MailMessage::new()"));
    }
}
