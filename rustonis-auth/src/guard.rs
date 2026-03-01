use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AuthError, Claims, JwtConfig};

/// Axum extractor that validates a `Bearer` JWT from the `Authorization` header.
///
/// Add it as a handler parameter to protect a route:
///
/// ```rust,no_run
/// use rustonis_auth::JwtGuard;
/// use axum::response::IntoResponse;
///
/// async fn me(guard: JwtGuard) -> impl IntoResponse {
///     format!("Hello, {}", guard.claims.sub)
/// }
/// ```
///
/// The handler is rejected with **401 Unauthorized** when:
/// - The `Authorization` header is absent.
/// - The header value does not start with `Bearer `.
/// - The token signature is invalid or expired.
/// - `JWT_SECRET` is not configured.
pub struct JwtGuard {
    /// The decoded claims extracted from the token.
    pub claims: Claims,
}

impl JwtGuard {
    /// Returns `true` when the authenticated user holds `role`.
    pub fn has_role(&self, role: &str) -> bool {
        self.claims.has_role(role)
    }

    /// Returns `Err(AuthError::Forbidden)` when the authenticated user does
    /// not hold `role` — convenient for early-return guards in handlers.
    pub fn require_role(&self, role: &str) -> Result<(), AuthError> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

impl<S> FromRequestParts<S> for JwtGuard
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Read the Authorization header
        let auth = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        // 2. Strip the "Bearer " prefix
        let token = auth
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MalformedToken)?;

        // 3. Load JWT config from env and verify
        let config = JwtConfig::from_env()?;
        let claims = config.verify(token)?;

        Ok(JwtGuard { claims })
    }
}
