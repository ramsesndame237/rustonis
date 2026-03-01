//! `rustonis-auth` — Authentication & password hashing for Rustonis.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rustonis_auth::{JwtConfig, Claims, BcryptHasher, PasswordHasher};
//!
//! // ── JWT ──────────────────────────────────────────────────────────────────
//! // JWT_SECRET must be set in the environment.
//! // std::env::set_var("JWT_SECRET", "my-super-secret");
//! let cfg   = JwtConfig::from_env().unwrap();
//! let token = cfg.issue("user-42", Some("admin")).unwrap();
//! let claims = cfg.verify(&token).unwrap();
//! assert_eq!(claims.sub, "user-42");
//!
//! // ── Passwords ────────────────────────────────────────────────────────────
//! let hasher = BcryptHasher::default();
//! let hash   = hasher.hash("my-password").unwrap();
//! assert!(hasher.verify("my-password", &hash).unwrap());
//! ```
//!
//! ## Route Protection
//!
//! Add [`JwtGuard`] as an extractor in any Axum handler:
//!
//! ```rust,no_run
//! use rustonis_auth::JwtGuard;
//!
//! async fn profile(guard: JwtGuard) -> String {
//!     format!("Hello, {}", guard.claims.sub)
//! }
//! ```

mod claims;
mod config;
mod error;
mod guard;
mod password;

pub use claims::Claims;
pub use config::JwtConfig;
pub use error::AuthError;
pub use guard::JwtGuard;
pub use password::{BcryptHasher, PasswordHasher};

pub mod prelude {
    pub use super::{AuthError, BcryptHasher, Claims, JwtConfig, JwtGuard, PasswordHasher};
}
