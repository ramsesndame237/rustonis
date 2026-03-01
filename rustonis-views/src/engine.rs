//! Tera-backed template engine.

use std::sync::OnceLock;

use serde::Serialize;
use tera::Tera;

use crate::ViewError;

// ─── ViewEngine ───────────────────────────────────────────────────────────────

/// Wrapper around the Tera template engine.
///
/// ## Usage
///
/// ```rust
/// use rustonis_views::ViewEngine;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Ctx { name: String }
///
/// let mut engine = ViewEngine::default();
/// engine.add_raw_template("hello.html", "Hello {{ name }}!").unwrap();
///
/// let ctx = Ctx { name: "World".to_string() };
/// let out = engine.render("hello.html", &ctx).unwrap();
/// assert_eq!(out, "Hello World!");
/// ```
pub struct ViewEngine {
    tera: Tera,
}

impl Default for ViewEngine {
    fn default() -> Self {
        Self { tera: Tera::default() }
    }
}

impl ViewEngine {
    /// Discover templates from a glob pattern.
    ///
    /// ```rust,no_run
    /// use rustonis_views::ViewEngine;
    /// let engine = ViewEngine::from_glob("resources/views/**/*.html").unwrap();
    /// ```
    pub fn from_glob(glob: &str) -> Result<Self, ViewError> {
        let tera = Tera::new(glob)?;
        Ok(Self { tera })
    }

    /// Add a raw template from a string — useful for tests.
    pub fn add_raw_template(&mut self, name: &str, source: &str) -> Result<(), ViewError> {
        self.tera.add_raw_template(name, source)?;
        Ok(())
    }

    /// Render `template_name` with the provided context.
    pub fn render<C: Serialize>(&self, template_name: &str, context: &C) -> Result<String, ViewError> {
        let ctx = tera::Context::from_serialize(context)?;
        let html = self.tera.render(template_name, &ctx)?;
        Ok(html)
    }
}

// ─── Global façade ────────────────────────────────────────────────────────────

static ENGINE: OnceLock<ViewEngine> = OnceLock::new();

/// Static façade for the application-wide view engine.
///
/// Call [`View::init`] once at boot time. The `render` method is then
/// available globally without passing the engine around.
pub struct View;

impl View {
    /// Register the global view engine.
    pub fn init(engine: ViewEngine) {
        ENGINE.set(engine).ok();
    }

    fn engine() -> Result<&'static ViewEngine, ViewError> {
        ENGINE.get().ok_or(ViewError::NotInitialized)
    }

    /// Render a template using the global engine.
    pub fn render<C: Serialize>(template_name: &str, context: &C) -> Result<String, ViewError> {
        Self::engine()?.render(template_name, context)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Ctx {
        name: String,
    }

    fn engine_with(name: &str, source: &str) -> ViewEngine {
        let mut e = ViewEngine::default();
        e.add_raw_template(name, source).unwrap();
        e
    }

    #[test]
    fn test_render_simple_variable() {
        let engine = engine_with("hello.html", "Hello {{ name }}!");
        let ctx = Ctx { name: "World".to_string() };
        let out = engine.render("hello.html", &ctx).unwrap();
        assert_eq!(out, "Hello World!");
    }

    #[test]
    fn test_render_unknown_template_returns_error() {
        let engine = ViewEngine::default();
        let ctx = Ctx { name: "x".to_string() };
        let result = engine.render("missing.html", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_conditional_block() {
        let source = "{% if show %}visible{% else %}hidden{% endif %}";
        let engine = engine_with("cond.html", source);

        #[derive(Serialize)]
        struct C { show: bool }

        let out = engine.render("cond.html", &C { show: true }).unwrap();
        assert_eq!(out, "visible");

        let out = engine.render("cond.html", &C { show: false }).unwrap();
        assert_eq!(out, "hidden");
    }

    #[test]
    fn test_render_loop() {
        let source = "{% for item in items %}{{ item }},{% endfor %}";
        let engine = engine_with("list.html", source);

        #[derive(Serialize)]
        struct C { items: Vec<&'static str> }

        let out = engine.render("list.html", &C { items: vec!["a", "b", "c"] }).unwrap();
        assert_eq!(out, "a,b,c,");
    }

    #[test]
    fn test_add_raw_template_overwrites_previous() {
        let mut engine = ViewEngine::default();
        engine.add_raw_template("t.html", "v1").unwrap();
        engine.add_raw_template("t.html", "v2").unwrap();

        #[derive(Serialize)]
        struct Empty {}

        let out = engine.render("t.html", &Empty {}).unwrap();
        assert_eq!(out, "v2");
    }
}
