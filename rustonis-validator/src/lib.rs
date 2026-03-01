pub mod error;
pub mod extractor;
pub mod rules;

pub use error::ValidationErrors;
pub use extractor::Valid;
// Re-export du derive macro pour usage en one-liner : use rustonis_validator::Validate;
pub use rustonis_macros::Validate;

/// Trait à implémenter pour valider une struct.
///
/// Généré automatiquement par `#[derive(Validate)]` ou implémenté manuellement
/// pour des règles inter-champs (ex: confirmation de mot de passe).
///
/// # Exemple manuel
///
/// ```rust
/// use rustonis_validator::{Validate, ValidationErrors};
///
/// struct CreateUserInput {
///     email: String,
///     password: String,
///     password_confirmation: String,
/// }
///
/// impl Validate for CreateUserInput {
///     fn validate(&self) -> Result<(), ValidationErrors> {
///         let mut errors = ValidationErrors::new();
///
///         if !rustonis_validator::rules::email(&self.email) {
///             errors.add("email", "must be a valid email address");
///         }
///         if !rustonis_validator::rules::min_length(&self.password, 8) {
///             errors.add("password", "must be at least 8 characters");
///         }
///         if !rustonis_validator::rules::confirmed(&self.password, &self.password_confirmation) {
///             errors.add("password_confirmation", "does not match password");
///         }
///
///         errors.into_result()
///     }
/// }
/// ```
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// Module de réexports pratiques pour un usage groupé.
pub mod prelude {
    pub use crate::error::ValidationErrors;
    pub use crate::extractor::Valid;
    pub use crate::rules;
    // Importe le trait (namespace type) et le derive macro (namespace macro)
    // via la crate root qui les expose tous les deux sous le même nom.
    pub use crate::Validate;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Tests du trait Validate (implémentation manuelle) ───────────────────

    struct LoginInput {
        email: String,
        password: String,
    }

    impl Validate for LoginInput {
        fn validate(&self) -> Result<(), ValidationErrors> {
            let mut errors = ValidationErrors::new();

            if !rules::email(&self.email) {
                errors.add("email", "must be a valid email address");
            }
            if !rules::min_length(&self.password, 8) {
                errors.add("password", "must be at least 8 characters");
            }

            errors.into_result()
        }
    }

    #[test]
    fn test_valid_login_input() {
        let input = LoginInput {
            email: "user@example.com".into(),
            password: "supersecret".into(),
        };
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let input = LoginInput {
            email: "not-an-email".into(),
            password: "supersecret".into(),
        };
        let err = input.validate().unwrap_err();
        assert!(err.has_errors());
        assert!(err.fields().contains_key("email"));
        assert!(!err.fields().contains_key("password"));
    }

    #[test]
    fn test_multiple_field_errors() {
        let input = LoginInput {
            email: "bad".into(),
            password: "short".into(),
        };
        let err = input.validate().unwrap_err();
        assert!(err.fields().contains_key("email"));
        assert!(err.fields().contains_key("password"));
    }

    // ─── Tests de ValidationErrors ───────────────────────────────────────────

    #[test]
    fn test_validation_errors_accumulates() {
        let mut errors = ValidationErrors::new();
        errors.add("email", "invalid format");
        errors.add("email", "already taken");
        errors.add("name", "required");

        assert_eq!(errors.fields()["email"].len(), 2);
        assert_eq!(errors.fields()["name"].len(), 1);
    }

    #[test]
    fn test_validation_errors_merge() {
        let mut a = ValidationErrors::new();
        a.add("email", "invalid");

        let mut b = ValidationErrors::new();
        b.add("email", "taken");
        b.add("name", "required");

        a.merge(b);
        assert_eq!(a.fields()["email"].len(), 2);
        assert_eq!(a.fields()["name"].len(), 1);
    }

    #[test]
    fn test_no_errors_returns_ok() {
        let errors = ValidationErrors::new();
        assert!(!errors.has_errors());
        assert!(errors.into_result().is_ok());
    }
}
