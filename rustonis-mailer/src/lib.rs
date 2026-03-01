//! `rustonis-mailer` — Email for Rustonis.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rustonis_mailer::{MailConfig, MailMessage, SmtpMailer};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = MailConfig::from_env()?;
//! let mailer = SmtpMailer::new(config)?;
//!
//! mailer.send(
//!     MailMessage::new()
//!         .to("alice@example.com")
//!         .subject("Welcome to Rustonis!")
//!         .html("<h1>Hello Alice!</h1>")
//!         .text("Hello Alice!"),
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod config;
mod error;
mod mailer;
mod message;

pub use config::MailConfig;
pub use error::MailError;
pub use mailer::{MailTransport, SmtpMailer};
pub use message::MailMessage;

pub mod prelude {
    pub use super::{MailConfig, MailError, MailMessage, SmtpMailer};
}
