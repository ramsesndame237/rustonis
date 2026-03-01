use crate::AuthError;

/// Abstraction over password hashing algorithms.
///
/// The default implementation is [`BcryptHasher`].
pub trait PasswordHasher: Send + Sync {
    /// Hash `password` and return the stored hash string.
    fn hash(&self, password: &str) -> Result<String, AuthError>;

    /// Return `true` if `password` matches `hash`.
    fn verify(&self, password: &str, hash: &str) -> Result<bool, AuthError>;
}

// ─── Bcrypt ───────────────────────────────────────────────────────────────────

/// Bcrypt-based password hasher.
///
/// ```rust
/// use rustonis_auth::{BcryptHasher, PasswordHasher};
///
/// let hasher = BcryptHasher::default(); // cost = 12
/// let hash   = hasher.hash("s3cr3t").unwrap();
/// assert!(hasher.verify("s3cr3t", &hash).unwrap());
/// assert!(!hasher.verify("wrong", &hash).unwrap());
/// ```
pub struct BcryptHasher {
    /// Work factor (4–31). **Default: 12** — safe for modern hardware.
    pub cost: u32,
}

impl Default for BcryptHasher {
    fn default() -> Self {
        Self { cost: 12 }
    }
}

impl BcryptHasher {
    pub fn new(cost: u32) -> Self {
        Self { cost }
    }
}

impl PasswordHasher for BcryptHasher {
    fn hash(&self, password: &str) -> Result<String, AuthError> {
        bcrypt::hash(password, self.cost)
            .map_err(|e| AuthError::Password(e.to_string()))
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        bcrypt::verify(password, hash)
            .map_err(|e| AuthError::Password(e.to_string()))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Use cost 4 in tests for speed.
    fn hasher() -> BcryptHasher {
        BcryptHasher::new(4)
    }

    #[test]
    fn test_hash_and_verify_correct_password() {
        let h    = hasher();
        let hash = h.hash("correct-horse-battery-staple").unwrap();
        assert!(h.verify("correct-horse-battery-staple", &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password_returns_false() {
        let h    = hasher();
        let hash = h.hash("my-password").unwrap();
        assert!(!h.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_two_hashes_of_same_password_differ() {
        let h  = hasher();
        let h1 = h.hash("same").unwrap();
        let h2 = h.hash("same").unwrap();
        // Bcrypt generates a fresh salt each time
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_verify_invalid_hash_returns_error() {
        let h = hasher();
        // bcrypt::verify returns an error on a malformed hash
        assert!(h.verify("any", "not-a-bcrypt-hash").is_err());
    }
}
