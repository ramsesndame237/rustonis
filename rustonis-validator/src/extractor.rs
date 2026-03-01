use axum::{
    extract::{FromRequest, Request},
    Json,
};
use http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::Validate;

/// Extracteur Axum qui désérialise le corps JSON **et** valide les données.
///
/// Si la désérialisation échoue, retourne `400 Bad Request`.
/// Si la validation échoue, retourne `422 Unprocessable Entity` avec le
/// détail des erreurs par champ (compatible avec `AppError::validation`).
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_validator::{Valid, Validate};
/// use rustonis_macros::Validate;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Validate)]
/// pub struct CreateUserInput {
///     #[validate(email)]
///     pub email: String,
///     #[validate(min_length = 8)]
///     pub password: String,
/// }
///
/// pub async fn create(Valid(input): Valid<CreateUserInput>) -> impl IntoResponse {
///     // input est garanti valide ici
/// }
/// ```
pub struct Valid<T>(pub T);

impl<T, S> FromRequest<S> for Valid<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Étape 1 : désérialisation JSON
        let Json(value): Json<T> = Json::from_request(req, state)
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Bad Request",
                        "message": e.to_string()
                    })),
                )
            })?;

        // Étape 2 : validation
        value.validate().map_err(|errors| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "Unprocessable Entity",
                    "message": "Validation failed",
                    "errors": errors.into_fields()
                })),
            )
        })?;

        Ok(Valid(value))
    }
}
