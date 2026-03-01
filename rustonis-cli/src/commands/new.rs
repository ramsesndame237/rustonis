use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(project_name: &str, template: &str) -> Result<()> {
    println!("🦀 Création du projet Rustonis : {}", project_name);
    println!();

    if Path::new(project_name).exists() {
        anyhow::bail!("Le répertoire '{}' existe déjà", project_name);
    }

    create_directories(project_name)?;
    create_files(project_name, template)?;

    println!("✨ Projet '{}' créé avec succès !", project_name);
    println!();
    println!("   Prochaines étapes :");
    println!("     cd {}", project_name);
    println!("     rustonis serve");
    println!();
    println!("   Documentation : https://rustonis.dev/docs");
    println!();

    Ok(())
}

fn create_directories(name: &str) -> Result<()> {
    let dirs = [
        "src/app/controllers",
        "src/app/models",
        "src/app/middleware",
        "src/app/validators",
        "src/app/services",
        "src/app/mailers",
        "src/app/jobs",
        "src/config",
        "src/providers",
        "src/start",
        "database/migrations",
        "database/seeders",
        "resources/views",
        "tests",
    ];

    for dir in &dirs {
        fs::create_dir_all(format!("{}/{}", name, dir))?;
        println!("  create  {}/{}", name, dir);
    }

    println!();
    Ok(())
}

fn create_files(name: &str, _template: &str) -> Result<()> {
    let files: Vec<(&str, String)> = vec![
        ("Cargo.toml", tpl_cargo_toml(name)),
        (".env", tpl_dot_env(name)),
        (".env.example", tpl_dot_env_example()),
        (".gitignore", tpl_gitignore()),
        ("src/main.rs", tpl_main_rs()),
        ("src/app/mod.rs", tpl_app_mod()),
        ("src/app/controllers/mod.rs", tpl_controllers_mod()),
        ("src/app/controllers/home_controller.rs", tpl_home_controller()),
        ("src/app/models/mod.rs", tpl_empty_mod("models")),
        ("src/app/middleware/mod.rs", tpl_empty_mod("middleware")),
        ("src/app/validators/mod.rs", tpl_empty_mod("validators")),
        ("src/app/services/mod.rs", tpl_empty_mod("services")),
        ("src/app/mailers/mod.rs", tpl_empty_mod("mailers")),
        ("src/app/jobs/mod.rs", tpl_empty_mod("jobs")),
        ("src/config/mod.rs", tpl_config_mod()),
        ("src/config/app.rs", tpl_config_app(name)),
        ("src/config/database.rs", tpl_config_database()),
        ("src/providers/mod.rs", tpl_providers_mod()),
        ("src/start/mod.rs", tpl_start_mod()),
        ("src/start/routes.rs", tpl_start_routes()),
        ("src/start/kernel.rs", tpl_start_kernel()),
    ];

    for (path, content) in files {
        let full_path = format!("{}/{}", name, path);
        fs::write(&full_path, content)?;
        println!("  create  {}", full_path);
    }

    Ok(())
}

// ─── Templates ───────────────────────────────────────────────────────────────

fn tpl_cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
axum = "0.8"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"#
    )
}

fn tpl_dot_env(name: &str) -> String {
    format!(
        r#"APP_NAME={name}
APP_ENV=development
APP_PORT=3333
APP_KEY=

DB_CONNECTION=sqlite
DB_DATABASE=database/database.sqlite3

RUST_LOG=info
"#
    )
}

fn tpl_dot_env_example() -> String {
    r#"APP_NAME=MyApp
APP_ENV=development
APP_PORT=3333
APP_KEY=

DB_CONNECTION=sqlite
DB_DATABASE=database/database.sqlite3

RUST_LOG=info
"#
    .to_string()
}

fn tpl_gitignore() -> String {
    r#"/target
.env
*.sqlite3
"#
    .to_string()
}

fn tpl_main_rs() -> String {
    r#"mod app;
mod config;
mod providers;
mod start;

#[tokio::main]
async fn main() {
    // Charge les variables d'environnement depuis .env
    dotenvy::dotenv().ok();

    // Initialise le système de logs
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Construit l'application (routes + middleware)
    let app = start::kernel::build();

    let port = config::AppConfig::from_env().port;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("🦀 Rustonis server running on http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#
    .to_string()
}

fn tpl_app_mod() -> String {
    r#"pub mod controllers;
pub mod jobs;
pub mod mailers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod validators;
"#
    .to_string()
}

fn tpl_controllers_mod() -> String {
    r#"pub mod home_controller;
"#
    .to_string()
}

fn tpl_home_controller() -> String {
    r#"use axum::Json;
use serde_json::{json, Value};

/// GET /
///
/// Équivalent du HomeController dans AdonisJS.
/// Remplacez ce handler par votre logique métier.
pub async fn index() -> Json<Value> {
    Json(json!({
        "message": "Hello from Rustonis! 🦀",
        "docs": "https://rustonis.dev/docs"
    }))
}
"#
    .to_string()
}

fn tpl_empty_mod(module: &str) -> String {
    format!("// Module {module}\n// Ajoutez vos {module} ici.\n")
}

fn tpl_config_mod() -> String {
    r#"pub mod app;
pub mod database;

pub use app::AppConfig;
pub use database::DatabaseConfig;
"#
    .to_string()
}

fn tpl_config_app(name: &str) -> String {
    format!(
        r#"/// Configuration principale chargée depuis les variables d'environnement.
///
/// Équivalent de `config/app.ts` dans AdonisJS.
/// Ajoutez vos propres champs et chargez-les depuis .env.
pub struct AppConfig {{
    pub name: String,
    pub env: String,
    pub port: u16,
    pub app_key: String,
}}

impl AppConfig {{
    pub fn from_env() -> Self {{
        Self {{
            name: std::env::var("APP_NAME")
                .unwrap_or_else(|_| "{name}".to_string()),
            env: std::env::var("APP_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "3333".to_string())
                .parse()
                .unwrap_or(3333),
            app_key: std::env::var("APP_KEY").unwrap_or_default(),
        }}
    }}

    pub fn is_production(&self) -> bool {{
        self.env == "production"
    }}

    pub fn is_development(&self) -> bool {{
        self.env == "development"
    }}
}}
"#
    )
}

fn tpl_config_database() -> String {
    r#"/// Configuration de la base de données.
///
/// Équivalent de `config/database.ts` dans AdonisJS.
pub struct DatabaseConfig {
    pub connection: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            connection: std::env::var("DB_CONNECTION")
                .unwrap_or_else(|_| "sqlite".to_string()),
            database: std::env::var("DB_DATABASE")
                .unwrap_or_else(|_| "database/database.sqlite3".to_string()),
        }
    }
}
"#
    .to_string()
}

fn tpl_providers_mod() -> String {
    r#"// Providers — bootstrappent les services de l'application.
//
// Équivalent des Service Providers dans AdonisJS.
// Chaque provider a deux phases :
//   - register() : lie les services au container IoC
//   - boot()     : initialise les services après enregistrement
//
// API Rustonis future (v0.2 — IoC Container) :
//
// use rustonis::prelude::*;
//
// #[provider]
// pub struct AppProvider;
//
// impl ServiceProvider for AppProvider {
//     async fn register(&self, container: &mut Container) {
//         container.bind_singleton::<Database>(|| async {
//             Database::connect(&DatabaseConfig::from_env().url).await
//         });
//     }
//
//     async fn boot(&self, container: &Container) {
//         let db = container.make::<Database>().await;
//         db.run_migrations().await.unwrap();
//     }
// }
"#
    .to_string()
}

fn tpl_start_mod() -> String {
    r#"pub mod kernel;
pub mod routes;
"#
    .to_string()
}

fn tpl_start_routes() -> String {
    r#"use axum::{routing::get, Router};
use crate::app::controllers::home_controller;

/// Enregistre les routes de l'application.
///
/// Équivalent de `start/routes.ts` dans AdonisJS.
/// Toutes les routes HTTP sont définies ici.
pub fn register() -> Router {
    Router::new()
        .route("/", get(home_controller::index))
    // Ajoutez vos routes ci-dessous :
    //
    // .route("/users", get(user_controller::index).post(user_controller::store))
    // .route("/users/:id", get(user_controller::show)
    //     .put(user_controller::update)
    //     .delete(user_controller::destroy))
    //
    // API Route Groups (v0.3) :
    // router.group()
    //     .prefix("/api/v1")
    //     .middleware(AuthMiddleware::guard("api"))
    //     .routes(|r| { ... })
}
"#
    .to_string()
}

fn tpl_start_kernel() -> String {
    r#"use axum::Router;
use super::routes;

/// Construit l'application avec tous les middleware et les routes.
///
/// Équivalent de `start/kernel.ts` dans AdonisJS.
/// Enregistrez ici les middleware globaux (CORS, tracing, auth, etc.).
pub fn build() -> Router {
    Router::new()
        .merge(routes::register())
    // Middleware globaux (décommentez selon vos besoins) :
    //
    // use tower_http::trace::TraceLayer;
    // use tower_http::cors::CorsLayer;
    //
    // .layer(TraceLayer::new_for_http())
    // .layer(CorsLayer::permissive())
}
"#
    .to_string()
}
