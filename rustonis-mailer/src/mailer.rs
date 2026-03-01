use lettre::{
    message::{header::ContentType, Mailbox, MultiPart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::{MailConfig, MailError, MailMessage};

// ─── Transport trait ──────────────────────────────────────────────────────────

/// Low-level mail transport interface.
///
/// Implement this trait to add alternative transports (log-to-console, file,
/// test double, etc.).  The [`SmtpMailer`] uses this trait internally.
#[async_trait::async_trait]
pub trait MailTransport: Send + Sync {
    async fn send_message(&self, message: MailMessage) -> Result<(), MailError>;
}

// ─── SmtpMailer ───────────────────────────────────────────────────────────────

/// High-level mailer backed by an SMTP connection.
///
/// ```rust,no_run
/// use rustonis_mailer::{MailConfig, MailMessage, SmtpMailer};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MailConfig::from_env()?;
/// let mailer = SmtpMailer::new(config)?;
///
/// mailer.send(
///     MailMessage::new()
///         .to("user@example.com")
///         .subject("Welcome to Rustonis!")
///         .html("<h1>Hello!</h1>")
///         .text("Hello!")
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct SmtpMailer {
    transport:    AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
    from_name:    String,
}

impl SmtpMailer {
    /// Create a new `SmtpMailer` from a [`MailConfig`].
    pub fn new(config: MailConfig) -> Result<Self, MailError> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        // `builder_dangerous` connects without TLS; add a TLS feature to lettre
        // (e.g. `tokio1-rustls-tls`) and use `relay()` for production SMTP.
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            .port(config.port)
            .credentials(creds)
            .build();

        Ok(Self {
            transport,
            from_address: config.from_address,
            from_name:    config.from_name,
        })
    }

    /// Send a [`MailMessage`].
    pub async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        let from_str  = format!("{} <{}>", self.from_name, self.from_address);
        let from_box: Mailbox = from_str.parse()
            .map_err(|e: lettre::address::AddressError| MailError::Address(e.to_string()))?;

        let mut builder = Message::builder()
            .from(from_box)
            .subject(message.subject.clone());

        for to in &message.to {
            let mailbox: Mailbox = to.parse()
                .map_err(|e: lettre::address::AddressError| MailError::Address(e.to_string()))?;
            builder = builder.to(mailbox);
        }
        for cc in &message.cc {
            let mailbox: Mailbox = cc.parse()
                .map_err(|e: lettre::address::AddressError| MailError::Address(e.to_string()))?;
            builder = builder.cc(mailbox);
        }
        if let Some(reply_to) = &message.reply_to {
            let mailbox: Mailbox = reply_to.parse()
                .map_err(|e: lettre::address::AddressError| MailError::Address(e.to_string()))?;
            builder = builder.reply_to(mailbox);
        }

        let lettre_message = match (&message.html_body, &message.text_body) {
            (Some(html), Some(text)) => {
                builder
                    .multipart(MultiPart::alternative_plain_html(text.clone(), html.clone()))
                    .map_err(|e| MailError::Message(e.to_string()))?
            }
            (Some(html), None) => builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())
                .map_err(|e| MailError::Message(e.to_string()))?,
            (None, Some(text)) => builder
                .header(ContentType::TEXT_PLAIN)
                .body(text.clone())
                .map_err(|e| MailError::Message(e.to_string()))?,
            (None, None) => {
                return Err(MailError::Config("MailMessage has no body (call .html() or .text())".to_string()));
            }
        };

        self.transport
            .send(lettre_message)
            .await
            .map_err(|e| MailError::Transport(e.to_string()))?;

        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_mailer_construction_with_valid_config() {
        let config = MailConfig {
            host:         "localhost".to_string(),
            port:         1025,
            username:     "user".to_string(),
            password:     "pass".to_string(),
            from_address: "app@example.com".to_string(),
            from_name:    "Test App".to_string(),
        };
        // Construction should succeed (no network call at build time)
        assert!(SmtpMailer::new(config).is_ok());
    }
}
