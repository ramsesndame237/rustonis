use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// JWT payload stored inside every signed token.
///
/// ```rust
/// use rustonis_auth::Claims;
///
/// let claims = Claims::new("user-42", Some("admin"), 3600);
/// assert_eq!(claims.sub, "user-42");
/// assert_eq!(claims.role.as_deref(), Some("admin"));
/// assert!(!claims.is_expired());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — typically the user ID.
    pub sub: String,
    /// Expiration time (Unix timestamp, seconds).
    pub exp: u64,
    /// Issued-at time (Unix timestamp, seconds).
    pub iat: u64,
    /// Optional role attached to this token (e.g. `"admin"`, `"editor"`).
    pub role: Option<String>,
}

impl Claims {
    /// Build a new `Claims` that expires `expires_in` seconds from now.
    pub fn new(
        sub: impl Into<String>,
        role: Option<impl Into<String>>,
        expires_in: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            sub: sub.into(),
            exp: now + expires_in,
            iat: now,
            role: role.map(Into::into),
        }
    }

    /// Returns `true` if the token has already expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.exp < now
    }

    /// Returns `true` if `role` matches the one stored in the claims.
    pub fn has_role(&self, role: &str) -> bool {
        self.role.as_deref() == Some(role)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_fields_are_set_correctly() {
        let c = Claims::new("u1", Some("admin"), 3600);
        assert_eq!(c.sub, "u1");
        assert_eq!(c.role.as_deref(), Some("admin"));
        assert!(!c.is_expired());
        assert!(c.exp > c.iat);
    }

    #[test]
    fn test_claims_no_role() {
        let c = Claims::new("u2", Option::<String>::None, 60);
        assert!(c.role.is_none());
    }

    #[test]
    fn test_claims_expired() {
        let c = Claims {
            sub: "u3".into(),
            exp: 1, // far in the past
            iat: 0,
            role: None,
        };
        assert!(c.is_expired());
    }

    #[test]
    fn test_has_role() {
        let c = Claims::new("u4", Some("editor"), 60);
        assert!(c.has_role("editor"));
        assert!(!c.has_role("admin"));
    }
}
