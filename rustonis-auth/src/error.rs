use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use thiserror::Error;

/// Errors produced by `rustonis-auth`.
#[derive(Debug, Error)]
pub enum AuthError {
    /// `Authorization` header is absent.
    #[error("Missing authorization token")]
    MissingToken,

    /// `Authorization` header is present but not a valid `Bearer <token>`.
    #[error("Malformed authorization token")]
    MalformedToken,

    /// Token signature is invalid, expired, or otherwise rejected by `jsonwebtoken`.
    #[error("Invalid token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    /// `JWT_SECRET` is not set (or auth configuration is incomplete).
    #[error("Auth configuration error: {0}")]
    Config(String),

    /// Bcrypt hashing or verification failed.
    #[error("Password error: {0}")]
    Password(String),

    /// Caller does not have the required role/permission.
    #[error("Forbidden: insufficient permissions")]
    Forbidden,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::MissingToken   => (StatusCode::UNAUTHORIZED,            self.to_string()),
            AuthError::MalformedToken => (StatusCode::UNAUTHORIZED,            self.to_string()),
            AuthError::InvalidToken(_)=> (StatusCode::UNAUTHORIZED,            "Invalid or expired token".to_string()),
            AuthError::Config(_)      => (StatusCode::INTERNAL_SERVER_ERROR,   self.to_string()),
            AuthError::Password(_)    => (StatusCode::INTERNAL_SERVER_ERROR,   self.to_string()),
            AuthError::Forbidden      => (StatusCode::FORBIDDEN,               self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
