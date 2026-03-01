use std::{fs, path::PathBuf};

use super::make::find_project_root;
use anyhow::{Context, Result};

/// Execute `rustonis make view <name>`.
pub fn execute_view(name: &str) -> Result<()> {
    let file_stem = parse_name(name);
    let root      = find_project_root()?;
    let dir       = root.join("resources").join("views");

    fs::create_dir_all(&dir)
        .with_context(|| format!("Cannot create directory {}", dir.display()))?;

    let file_path = dir.join(format!("{}.html", file_stem));

    if file_path.exists() {
        anyhow::bail!("File already exists: {}", file_path.display());
    }

    let content = generate_view(&file_stem);
    fs::write(&file_path, &content)
        .with_context(|| format!("Cannot write {}", file_path.display()))?;

    println!("✅ Created: {}", relative(&root, &file_path));
    Ok(())
}

fn generate_view(stem: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{{{ title | default(value="{stem}") }}}}</title>
</head>
<body>
    {{%- block content -%}}
    <h1>{{{{ title | default(value="{stem}") }}}}</h1>
    {{%- endblock content -%}}
</body>
</html>
"#
    )
}

fn relative(root: &PathBuf, path: &PathBuf) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

/// Parse the user-supplied name into a `file_stem`.
///
/// * `Home`      → `home`
/// * `blog-post` → `blog_post`
/// * `UserIndex` → `user_index`  (PascalCase split on uppercase)
pub fn parse_name(raw: &str) -> String {
    // Handle PascalCase: insert _ before each uppercase letter (except the first)
    let with_underscores = insert_underscores(raw);

    with_underscores
        .replace('-', "_")
        .to_lowercase()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn insert_underscores(s: &str) -> String {
    let mut out = String::new();
    let mut prev_upper = false;
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 && !prev_upper {
            out.push('_');
        }
        out.push(ch);
        prev_upper = ch.is_uppercase();
    }
    out
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_lowercase() {
        assert_eq!(parse_name("home"), "home");
    }

    #[test]
    fn test_parse_name_kebab() {
        assert_eq!(parse_name("blog-post"), "blog_post");
    }

    #[test]
    fn test_parse_name_pascal() {
        assert_eq!(parse_name("UserIndex"), "user_index");
    }

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate_view("home");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("home"));
        assert!(html.contains("block content"));
    }
}
