use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "rustonis")]
#[command(version = "0.1.0")]
#[command(about = "Rustonis — The AdonisJS of Rust 🦀")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée un nouveau projet Rustonis
    New {
        /// Nom du projet
        name: String,

        /// Template de projet : api (défaut) ou fullstack
        #[arg(long, default_value = "api")]
        template: String,
    },

    /// Démarre le serveur de développement
    Serve {
        /// Active le hot reload avec cargo-watch
        #[arg(long, short)]
        watch: bool,

        /// Port d'écoute (surcharge APP_PORT dans .env)
        #[arg(long, short)]
        port: Option<u16>,
    },

    /// Génère des fichiers Rustonis (controllers, models, …)
    Make {
        #[command(subcommand)]
        generator: Generator,
    },

    /// Commandes de base de données (migrate, rollback, fresh, seed)
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}

#[derive(Subcommand)]
enum Generator {
    /// Génère un controller
    Controller {
        /// Nom du controller : ex. User, blog-post, UserController
        name: String,

        /// Génère les 5 méthodes CRUD complètes (index, show, create, update, destroy)
        #[arg(long, short)]
        resource: bool,
    },

    /// Génère un validator
    Validator {
        /// Nom du validator : ex. CreateUser, login, blog-post
        name: String,
    },

    /// Génère un model et optionnellement sa migration SQL
    Model {
        /// Nom du model : ex. User, blog-post, UserProfile
        name: String,

        /// Génère aussi la migration SQL associée
        #[arg(long, short)]
        migration: bool,
    },

    /// Génère un middleware Axum
    Middleware {
        /// Nom du middleware : ex. auth, rate-limit, VerifyToken
        name: String,
    },

    /// Génère un mailer (classe d'email)
    Mailer {
        /// Nom du mailer : ex. Welcome, order-confirmation
        name: String,
    },

    /// Génère un template de vue Tera
    View {
        /// Nom du template : ex. home, blog-post, UserIndex
        name: String,
    },

    /// Génère un job de queue
    Job {
        /// Nom du job : ex. SendEmail, process-order, CleanupExpired
        name: String,
    },
}

#[derive(Subcommand)]
enum DbAction {
    /// Applique les migrations en attente
    Migrate,

    /// Annule le dernier batch de migrations
    Rollback,

    /// Rollback tout puis re-applique toutes les migrations
    Fresh,

    /// Exécute les fichiers dans database/seeders/
    Seed,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name, template } => commands::new::execute(&name, &template),
        Commands::Serve { watch, port } => commands::serve::execute(watch, port),
        Commands::Make { generator } => match generator {
            Generator::Controller { name, resource } => {
                commands::make::execute_controller(&name, resource)
            }
            Generator::Validator { name } => {
                commands::make_validator::execute_validator(&name)
            }
            Generator::Model { name, migration } => {
                commands::make_model::execute_model(&name, migration)
            }
            Generator::Middleware { name } => {
                commands::make_middleware::execute_middleware(&name)
            }
            Generator::Mailer { name } => {
                commands::make_mailer::execute_mailer(&name)
            }
            Generator::View { name } => {
                commands::make_view::execute_view(&name)
            }
            Generator::Job { name } => {
                commands::make_job::execute_job(&name)
            }
        },
        Commands::Db { action } => match action {
            DbAction::Migrate  => commands::db::execute_migrate(),
            DbAction::Rollback => commands::db::execute_rollback(),
            DbAction::Fresh    => commands::db::execute_fresh(),
            DbAction::Seed     => commands::db::execute_seed(),
        },
    };

    if let Err(e) = result {
        eprintln!("❌ Erreur : {}", e);
        std::process::exit(1);
    }
}
