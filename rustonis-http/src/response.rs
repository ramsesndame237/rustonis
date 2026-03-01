use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Réponse JSON typée.
///
/// Wrapping d'un payload `T: Serialize` avec un status HTTP.
/// Implémente `IntoResponse` pour être retournée directement
/// depuis un handler Axum.
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_http::JsonResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User { id: u64, name: String }
///
/// pub async fn index() -> JsonResponse<Vec<User>> {
///     JsonResponse::ok(vec![
///         User { id: 1, name: "Alice".to_string() },
///     ])
/// }
///
/// pub async fn create(body: Json<CreateUser>) -> JsonResponse<User> {
///     JsonResponse::created(User { id: 2, name: body.name.clone() })
/// }
/// ```
pub struct JsonResponse<T: Serialize> {
    status: StatusCode,
    data: T,
}

impl<T: Serialize> JsonResponse<T> {
    /// `200 OK` avec des données.
    pub fn ok(data: T) -> Self {
        Self {
            status: StatusCode::OK,
            data,
        }
    }

    /// `201 Created` avec des données.
    pub fn created(data: T) -> Self {
        Self {
            status: StatusCode::CREATED,
            data,
        }
    }

    /// Status HTTP personnalisé.
    pub fn with_status(status: StatusCode, data: T) -> Self {
        Self { status, data }
    }
}

impl<T: Serialize> IntoResponse for JsonResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.data)).into_response()
    }
}

/// Réponse `204 No Content`.
///
/// À utiliser pour les DELETE, ou toute action sans corps de réponse.
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_http::{AppError, NoContent};
///
/// pub async fn destroy(id: u64) -> Result<NoContent, AppError> {
///     db.delete(id)?;
///     Ok(NoContent)
/// }
/// ```
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
