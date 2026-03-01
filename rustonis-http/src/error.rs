use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Erreurs HTTP applicatives.
///
/// Chaque variante se convertit automatiquement en réponse HTTP
/// avec le status code approprié et un corps JSON standardisé.
///
/// # Format de réponse JSON
///
/// ```json
/// {
///   "error": "Not Found",
///   "message": "User 42 not found"
/// }
/// ```
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_http::{AppError, JsonResponse};
///
/// pub async fn show(id: u64) -> Result<JsonResponse<User>, AppError> {
///     let user = db.find(id).ok_or_else(|| AppError::not_found("User not found"))?;
///     Ok(JsonResponse::ok(user))
/// }
/// ```
#[derive(Debug)]
pub enum AppError {
    /// 404 Not Found
    NotFound(String),
    /// 401 Unauthorized
    Unauthorized(String),
    /// 403 Forbidden
    Forbidden(String),
    /// 400 Bad Request
    BadRequest(String),
    /// 422 Unprocessable Entity — validation errors
    UnprocessableEntity {
        message: String,
        errors: HashMap<String, Vec<String>>,
    },
    /// 500 Internal Server Error
    InternalServerError(String),
}

impl AppError {
    /// Crée une erreur 404 avec un message.
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    /// Crée une erreur 401 avec un message.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized(msg.into())
    }

    /// Crée une erreur 403 avec un message.
    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }

    /// Crée une erreur 400 avec un message.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    /// Crée une erreur 422 avec des erreurs de validation.
    pub fn validation(
        message: impl Into<String>,
        errors: HashMap<String, Vec<String>>,
    ) -> Self {
        AppError::UnprocessableEntity {
            message: message.into(),
            errors,
        }
    }

    /// Crée une erreur 500 avec un message.
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::InternalServerError(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                json!({ "error": "Not Found", "message": msg }),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                json!({ "error": "Unauthorized", "message": msg }),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                json!({ "error": "Forbidden", "message": msg }),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "Bad Request", "message": msg }),
            ),
            AppError::UnprocessableEntity { message, errors } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "error": "Unprocessable Entity",
                    "message": message,
                    "errors": errors
                }),
            ),
            AppError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "Internal Server Error", "message": msg }),
            ),
        };

        (status, Json(body)).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(m) => write!(f, "Not Found: {m}"),
            AppError::Unauthorized(m) => write!(f, "Unauthorized: {m}"),
            AppError::Forbidden(m) => write!(f, "Forbidden: {m}"),
            AppError::BadRequest(m) => write!(f, "Bad Request: {m}"),
            AppError::UnprocessableEntity { message, .. } => {
                write!(f, "Unprocessable Entity: {message}")
            }
            AppError::InternalServerError(m) => write!(f, "Internal Server Error: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

/// Permet de convertir n'importe quelle erreur standard en 500.
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
