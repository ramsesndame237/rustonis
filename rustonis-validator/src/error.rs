use std::collections::HashMap;

/// Erreurs de validation accumulées par champ.
///
/// ```rust
/// use rustonis_validator::ValidationErrors;
///
/// let mut errors = ValidationErrors::new();
/// errors.add("email", "must be a valid email address");
/// errors.add("email", "must not be empty");
/// errors.add("password", "must be at least 8 characters");
///
/// assert!(errors.has_errors());
/// assert_eq!(errors.fields()["email"].len(), 2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    fields: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un message d'erreur pour un champ donné.
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.fields
            .entry(field.into())
            .or_default()
            .push(message.into());
    }

    /// Fusionne les erreurs d'un autre `ValidationErrors`.
    pub fn merge(&mut self, other: ValidationErrors) {
        for (field, messages) in other.fields {
            self.fields.entry(field).or_default().extend(messages);
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.fields.is_empty()
    }

    pub fn fields(&self) -> &HashMap<String, Vec<String>> {
        &self.fields
    }

    /// Consomme `self` et retourne `Ok(())` s'il n'y a pas d'erreurs,
    /// sinon `Err(self)`.
    pub fn into_result(self) -> Result<(), Self> {
        if self.has_errors() {
            Err(self)
        } else {
            Ok(())
        }
    }

    /// Consomme `self` pour retourner la map de champs.
    pub fn into_fields(self) -> HashMap<String, Vec<String>> {
        self.fields
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation failed: {:?}", self.fields)
    }
}

impl std::error::Error for ValidationErrors {}
