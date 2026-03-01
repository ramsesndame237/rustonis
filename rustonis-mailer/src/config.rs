use crate::MailError;

/// SMTP mail configuration, loaded from environment variables.
///
/// | Variable             | Default       | Required |
/// |----------------------|---------------|----------|
/// | `MAIL_HOST`          | `localhost`   | –        |
/// | `MAIL_PORT`          | `587`         | –        |
/// | `MAIL_USERNAME`      | *(empty)*     | –        |
/// | `MAIL_PASSWORD`      | *(empty)*     | –        |
/// | `MAIL_FROM_ADDRESS`  | –             | **yes**  |
/// | `MAIL_FROM_NAME`     | `Rustonis`    | –        |
#[derive(Debug, Clone)]
pub struct MailConfig {
    pub host:         String,
    pub port:         u16,
    pub username:     String,
    pub password:     String,
    pub from_address: String,
    pub from_name:    String,
}

impl MailConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, MailError> {
        let from_address = std::env::var("MAIL_FROM_ADDRESS")
            .map_err(|_| MailError::Config("MAIL_FROM_ADDRESS is required".to_string()))?;

        let port = std::env::var("MAIL_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| MailError::Config("MAIL_PORT must be a valid port number".to_string()))?;

        Ok(Self {
            host:         std::env::var("MAIL_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port,
            username:     std::env::var("MAIL_USERNAME").unwrap_or_default(),
            password:     std::env::var("MAIL_PASSWORD").unwrap_or_default(),
            from_address,
            from_name:    std::env::var("MAIL_FROM_NAME").unwrap_or_else(|_| "Rustonis".to_string()),
        })
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_with_required_var() {
        std::env::set_var("MAIL_FROM_ADDRESS", "app@example.com");
        std::env::remove_var("MAIL_HOST");
        std::env::remove_var("MAIL_PORT");
        std::env::remove_var("MAIL_FROM_NAME");

        let config = MailConfig::from_env().unwrap();

        assert_eq!(config.from_address, "app@example.com");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 587);
        assert_eq!(config.from_name, "Rustonis");
    }

    #[test]
    fn test_from_env_missing_required_returns_error() {
        std::env::remove_var("MAIL_FROM_ADDRESS");
        let result = MailConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MAIL_FROM_ADDRESS"));
    }

    #[test]
    fn test_from_env_custom_port() {
        std::env::set_var("MAIL_FROM_ADDRESS", "app@example.com");
        std::env::set_var("MAIL_PORT", "465");
        let config = MailConfig::from_env().unwrap();
        assert_eq!(config.port, 465);
        std::env::remove_var("MAIL_PORT");
    }

    #[test]
    fn test_from_env_invalid_port_returns_error() {
        std::env::set_var("MAIL_FROM_ADDRESS", "app@example.com");
        std::env::set_var("MAIL_PORT", "notanumber");
        let result = MailConfig::from_env();
        assert!(result.is_err());
        std::env::remove_var("MAIL_PORT");
    }
}
