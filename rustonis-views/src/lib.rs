//! `rustonis-views` — Tera template engine for Rustonis.
//!
//! ## Quick Start
//!
//! ```rust
//! use rustonis_views::ViewEngine;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Ctx { name: String }
//!
//! let mut engine = ViewEngine::default();
//! engine.add_raw_template("welcome.html", "<h1>Hello {{ name }}!</h1>").unwrap();
//!
//! let ctx = Ctx { name: "Alice".to_string() };
//! let html = engine.render("welcome.html", &ctx).unwrap();
//! assert_eq!(html, "<h1>Hello Alice!</h1>");
//! ```
//!
//! ## Global Façade
//!
//! ```rust,no_run
//! use rustonis_views::{View, ViewEngine};
//!
//! // At boot time:
//! View::init(ViewEngine::from_glob("resources/views/**/*.html").unwrap());
//!
//! // Anywhere in handlers:
//! # use serde::Serialize;
//! # #[derive(Serialize)] struct C {}
//! let html = View::render("home.html", &C {}).unwrap();
//! ```

mod engine;
mod error;
mod response;

pub use engine::{View, ViewEngine};
pub use error::ViewError;
pub use response::HtmlResponse;

pub mod prelude {
    pub use super::{HtmlResponse, View, ViewEngine, ViewError};
}
