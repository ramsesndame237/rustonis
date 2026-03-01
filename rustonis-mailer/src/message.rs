/// A mail message built with a fluent builder API.
///
/// ```rust
/// use rustonis_mailer::MailMessage;
///
/// let msg = MailMessage::new()
///     .to("bob@example.com")
///     .subject("Welcome!")
///     .html("<h1>Hello Bob!</h1>")
///     .text("Hello Bob!");
///
/// assert_eq!(msg.subject, "Welcome!");
/// assert_eq!(msg.to[0], "bob@example.com");
/// ```
#[derive(Debug, Default, Clone)]
pub struct MailMessage {
    /// Primary recipients.
    pub to:        Vec<String>,
    /// Carbon-copy recipients.
    pub cc:        Vec<String>,
    /// Blind carbon-copy recipients.
    pub bcc:       Vec<String>,
    /// Email subject line.
    pub subject:   String,
    /// HTML body (rendered in email clients that support HTML).
    pub html_body: Option<String>,
    /// Plain-text body (fallback for email clients without HTML support).
    pub text_body: Option<String>,
    /// Optional Reply-To address.
    pub reply_to:  Option<String>,
}

impl MailMessage {
    /// Create an empty message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a primary recipient.
    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to.push(address.into());
        self
    }

    /// Add a CC recipient.
    pub fn cc(mut self, address: impl Into<String>) -> Self {
        self.cc.push(address.into());
        self
    }

    /// Add a BCC recipient.
    pub fn bcc(mut self, address: impl Into<String>) -> Self {
        self.bcc.push(address.into());
        self
    }

    /// Set the subject line.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    /// Set the HTML body.
    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.html_body = Some(body.into());
        self
    }

    /// Set the plain-text body.
    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.text_body = Some(body.into());
        self
    }

    /// Set the Reply-To address.
    pub fn reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to = Some(address.into());
        self
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_sets_fields() {
        let msg = MailMessage::new()
            .to("bob@example.com")
            .subject("Hello")
            .html("<h1>Hi</h1>")
            .text("Hi");

        assert_eq!(msg.to, vec!["bob@example.com"]);
        assert_eq!(msg.subject, "Hello");
        assert_eq!(msg.html_body.as_deref(), Some("<h1>Hi</h1>"));
        assert_eq!(msg.text_body.as_deref(), Some("Hi"));
    }

    #[test]
    fn test_multiple_recipients() {
        let msg = MailMessage::new()
            .to("a@example.com")
            .to("b@example.com")
            .cc("c@example.com")
            .bcc("d@example.com");

        assert_eq!(msg.to.len(), 2);
        assert_eq!(msg.cc.len(), 1);
        assert_eq!(msg.bcc.len(), 1);
    }

    #[test]
    fn test_html_only_no_text() {
        let msg = MailMessage::new().html("<p>Hello</p>");
        assert!(msg.html_body.is_some());
        assert!(msg.text_body.is_none());
    }

    #[test]
    fn test_reply_to() {
        let msg = MailMessage::new().reply_to("noreply@example.com");
        assert_eq!(msg.reply_to.as_deref(), Some("noreply@example.com"));
    }

    #[test]
    fn test_default_is_empty() {
        let msg = MailMessage::default();
        assert!(msg.to.is_empty());
        assert!(msg.subject.is_empty());
        assert!(msg.html_body.is_none());
    }
}
