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
        },
    };

    if let Err(e) = result {
        eprintln!("❌ Erreur : {}", e);
        std::process::exit(1);
    }
}
