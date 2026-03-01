use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::{AuthError, Claims};

/// JWT configuration loaded from environment variables.
///
/// | Variable        | Default  | Description                          |
/// |-----------------|----------|--------------------------------------|
/// | `JWT_SECRET`    | —        | **Required.** HMAC signing secret.   |
/// | `JWT_EXPIRES_IN`| `86400`  | Token lifetime in seconds (1 day).   |
/// | `JWT_ALGORITHM` | `HS256`  | `HS256`, `HS384` or `HS512`.         |
///
/// ```rust,no_run
/// use rustonis_auth::JwtConfig;
///
/// // std::env::set_var("JWT_SECRET", "super-secret");
/// let cfg = JwtConfig::from_env().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    /// Token lifetime in seconds used by [`JwtConfig::sign`].
    pub expires_in: u64,
    pub algorithm: Algorithm,
}

impl JwtConfig {
    /// Create a `JwtConfig` from well-known environment variables.
    pub fn from_env() -> Result<Self, AuthError> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| AuthError::Config("JWT_SECRET is not set".to_string()))?;

        let expires_in = std::env::var("JWT_EXPIRES_IN")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(86_400);

        let algorithm = match std::env::var("JWT_ALGORITHM").as_deref() {
            Ok("HS384") => Algorithm::HS384,
            Ok("HS512") => Algorithm::HS512,
            _           => Algorithm::HS256,
        };

        Ok(Self { secret, expires_in, algorithm })
    }

    /// Build a `JwtConfig` directly (useful in tests / providers).
    pub fn new(secret: impl Into<String>, expires_in: u64, algorithm: Algorithm) -> Self {
        Self {
            secret: secret.into(),
            expires_in,
            algorithm,
        }
    }

    /// Sign a [`Claims`] and return the compact JWT string.
    pub fn sign(&self, claims: &Claims) -> Result<String, AuthError> {
        let header = Header::new(self.algorithm);
        let key    = EncodingKey::from_secret(self.secret.as_bytes());
        Ok(encode(&header, claims, &key)?)
    }

    /// Verify and decode a JWT string, returning the inner [`Claims`].
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let key        = DecodingKey::from_secret(self.secret.as_bytes());
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = true;
        let data = decode::<Claims>(token, &key, &validation)?;
        Ok(data.claims)
    }

    /// Convenience: build `Claims` and sign them in one call.
    pub fn issue(&self, sub: impl Into<String>, role: Option<impl Into<String>>) -> Result<String, AuthError> {
        let claims = Claims::new(sub, role, self.expires_in);
        self.sign(&claims)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> JwtConfig {
        JwtConfig::new("test-secret-key-that-is-long-enough", 3600, Algorithm::HS256)
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let config = cfg();
        let claims = Claims::new("user-1", Some("admin"), 3600);
        let token  = config.sign(&claims).unwrap();
        let decoded = config.verify(&token).unwrap();
        assert_eq!(decoded.sub,  claims.sub);
        assert_eq!(decoded.role, claims.role);
    }

    #[test]
    fn test_verify_rejects_tampered_token() {
        let config = cfg();
        let claims = Claims::new("user-2", Option::<String>::None, 3600);
        let mut token = config.sign(&claims).unwrap();
        // Flip the last character to tamper the signature
        token.push('X');
        assert!(config.verify(&token).is_err());
    }

    #[test]
    fn test_verify_rejects_expired_token() {
        let config = cfg();
        // exp in the past → jsonwebtoken rejects it
        let claims = Claims {
            sub:  "user-3".into(),
            exp:  1,
            iat:  0,
            role: None,
        };
        let token = config.sign(&claims).unwrap();
        assert!(config.verify(&token).is_err());
    }

    #[test]
    fn test_verify_rejects_wrong_secret() {
        let config1 = cfg();
        let config2 = JwtConfig::new("completely-different-secret-long", 3600, Algorithm::HS256);
        let claims  = Claims::new("user-4", Option::<String>::None, 3600);
        let token   = config1.sign(&claims).unwrap();
        assert!(config2.verify(&token).is_err());
    }

    #[test]
    fn test_issue_convenience() {
        let config = cfg();
        let token  = config.issue("user-5", Some("editor")).unwrap();
        let claims = config.verify(&token).unwrap();
        assert_eq!(claims.sub, "user-5");
        assert_eq!(claims.role.as_deref(), Some("editor"));
    }
}
