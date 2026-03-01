use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailError {
    #[error("SMTP transport error: {0}")]
    Transport(String),

    #[error("Message build error: {0}")]
    Message(String),

    #[error("Invalid address: {0}")]
    Address(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
