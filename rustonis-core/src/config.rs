/// Trait implémenté par toutes les structs de configuration Rustonis.
///
/// Permet de charger la configuration depuis les variables d'environnement.
/// En Phase 2, la macro `#[config]` générera automatiquement cette implémentation.
///
/// # Exemple
///
/// ```rust
/// use rustonis_core::config::{FromEnv, AppConfig};
///
/// dotenvy::dotenv().ok();
/// let config = AppConfig::from_env();
/// println!("Serveur : {}", config.name);
/// ```
pub trait FromEnv: Sized {
    fn from_env() -> Self;
}

/// Environnement d'exécution de l'application.
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl std::str::FromStr for Environment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "production" => Ok(Environment::Production),
            "testing" | "test" => Ok(Environment::Testing),
            _ => Ok(Environment::Development),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Testing => write!(f, "testing"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// Configuration principale de l'application.
///
/// Chargée depuis les variables d'environnement définies dans `.env`.
/// Équivalent du fichier `config/app.ts` dans AdonisJS.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Nom de l'application (APP_NAME)
    pub name: String,
    /// Environnement d'exécution (APP_ENV)
    pub env: Environment,
    /// Port d'écoute du serveur HTTP (APP_PORT)
    pub port: u16,
    /// Clé secrète de l'application (APP_KEY) — utilisée pour le chiffrement
    pub app_key: String,
}

impl FromEnv for AppConfig {
    fn from_env() -> Self {
        Self {
            name: std::env::var("APP_NAME")
                .unwrap_or_else(|_| "Rustonis".to_string()),
            env: std::env::var("APP_ENV")
                .unwrap_or_else(|_| "development".to_string())
                .parse()
                .unwrap_or(Environment::Development),
            port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "3333".to_string())
                .parse()
                .unwrap_or(3333),
            app_key: std::env::var("APP_KEY").unwrap_or_default(),
        }
    }
}

impl AppConfig {
    pub fn is_production(&self) -> bool {
        self.env == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.env == Environment::Development
    }

    pub fn is_testing(&self) -> bool {
        self.env == Environment::Testing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_parses_production() {
        let env: Environment = "production".parse().unwrap();
        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn environment_parses_testing() {
        let env: Environment = "testing".parse().unwrap();
        assert_eq!(env, Environment::Testing);
    }

    #[test]
    fn environment_defaults_to_development() {
        let env: Environment = "unknown".parse().unwrap();
        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn app_config_is_development_by_default() {
        // Sans variables d'env définies, on est en développement
        let config = AppConfig {
            name: "Test".to_string(),
            env: Environment::Development,
            port: 3333,
            app_key: String::new(),
        };
        assert!(config.is_development());
        assert!(!config.is_production());
    }
}
