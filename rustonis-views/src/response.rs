//! Axum response types for rendered HTML.

use axum::response::{IntoResponse, Response};
use http::StatusCode;

// ─── HtmlResponse ─────────────────────────────────────────────────────────────

/// An Axum [`IntoResponse`] that renders an HTML body with the correct
/// `Content-Type: text/html; charset=utf-8` header.
///
/// ```rust
/// use rustonis_views::HtmlResponse;
/// use http::StatusCode;
///
/// let res = HtmlResponse::ok("<h1>Hello</h1>");
/// // Use in an Axum handler: async fn handler() -> HtmlResponse { … }
/// ```
pub struct HtmlResponse {
    html:   String,
    status: StatusCode,
}

impl HtmlResponse {
    /// 200 OK response.
    pub fn ok(html: impl Into<String>) -> Self {
        Self { html: html.into(), status: StatusCode::OK }
    }

    /// Response with a custom status code.
    pub fn with_status(html: impl Into<String>, status: StatusCode) -> Self {
        Self { html: html.into(), status }
    }

    /// Return the inner HTML string (useful for assertions in tests).
    pub fn html(&self) -> &str {
        &self.html
    }

    /// Return the HTTP status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for HtmlResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            [("content-type", "text/html; charset=utf-8")],
            self.html,
        )
            .into_response()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_sets_200() {
        let res = HtmlResponse::ok("<p>Hello</p>");
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[test]
    fn test_with_status() {
        let res = HtmlResponse::with_status("<p>Created</p>", StatusCode::CREATED);
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[test]
    fn test_html_accessor() {
        let res = HtmlResponse::ok("<b>test</b>");
        assert_eq!(res.html(), "<b>test</b>");
    }
}
