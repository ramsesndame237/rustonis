//! Fonctions de validation built-in.
//!
//! Chaque fonction retourne `true` si la valeur est valide.
//! Elles sont utilisées à la fois par le derive macro et peuvent
//! être appelées directement dans les implémentations manuelles de `Validate`.

use regex::Regex;
use std::sync::OnceLock;

// ─── Chaînes ─────────────────────────────────────────────────────────────────

/// La chaîne n'est pas vide (après trim).
pub fn required(value: &str) -> bool {
    !value.trim().is_empty()
}

/// La chaîne est une adresse email valide (format RFC 5322 simplifié).
pub fn email(value: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$").unwrap()
    });
    re.is_match(value)
}

/// La chaîne contient au moins `min` caractères Unicode.
pub fn min_length(value: &str, min: usize) -> bool {
    value.chars().count() >= min
}

/// La chaîne contient au plus `max` caractères Unicode.
pub fn max_length(value: &str, max: usize) -> bool {
    value.chars().count() <= max
}

/// La longueur de la chaîne est comprise entre `min` et `max` inclus.
pub fn length_between(value: &str, min: usize, max: usize) -> bool {
    let len = value.chars().count();
    len >= min && len <= max
}

/// La chaîne correspond à l'expression régulière donnée.
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    Regex::new(pattern).map(|re| re.is_match(value)).unwrap_or(false)
}

/// La chaîne est entièrement alphanumérique.
pub fn alphanumeric(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_alphanumeric())
}

/// La chaîne est une URL valide (http ou https).
pub fn url(value: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
    });
    re.is_match(value)
}

// ─── Numériques ──────────────────────────────────────────────────────────────

/// La valeur entière est supérieure ou égale à `min`.
pub fn min_val(value: i64, min: i64) -> bool {
    value >= min
}

/// La valeur entière est inférieure ou égale à `max`.
pub fn max_val(value: i64, max: i64) -> bool {
    value <= max
}

/// La valeur float est supérieure ou égale à `min`.
pub fn min_float(value: f64, min: f64) -> bool {
    value >= min
}

/// La valeur float est inférieure ou égale à `max`.
pub fn max_float(value: f64, max: f64) -> bool {
    value <= max
}

// ─── Confirmation ─────────────────────────────────────────────────────────────

/// Les deux valeurs sont identiques (ex: password confirmation).
pub fn confirmed(value: &str, confirmation: &str) -> bool {
    value == confirmation
}

// ─── Enums ───────────────────────────────────────────────────────────────────

/// La valeur fait partie de la liste autorisée.
pub fn one_of<'a>(value: &str, allowed: &[&'a str]) -> bool {
    allowed.contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        assert!(required("hello"));
        assert!(!required(""));
        assert!(!required("   "));
    }

    #[test]
    fn test_email() {
        assert!(email("user@example.com"));
        assert!(email("user+tag@sub.domain.io"));
        assert!(!email("not-an-email"));
        assert!(!email("@example.com"));
        assert!(!email("user@"));
    }

    #[test]
    fn test_min_length() {
        assert!(min_length("hello", 3));
        assert!(min_length("hello", 5));
        assert!(!min_length("hi", 5));
        // Unicode characters count as 1
        assert!(min_length("héllo", 5));
    }

    #[test]
    fn test_max_length() {
        assert!(max_length("hi", 5));
        assert!(max_length("hello", 5));
        assert!(!max_length("toolong", 5));
    }

    #[test]
    fn test_min_val() {
        assert!(min_val(10, 5));
        assert!(min_val(5, 5));
        assert!(!min_val(3, 5));
    }

    #[test]
    fn test_max_val() {
        assert!(max_val(3, 5));
        assert!(max_val(5, 5));
        assert!(!max_val(10, 5));
    }

    #[test]
    fn test_confirmed() {
        assert!(confirmed("secret", "secret"));
        assert!(!confirmed("secret", "different"));
    }

    #[test]
    fn test_one_of() {
        assert!(one_of("admin", &["admin", "user", "moderator"]));
        assert!(!one_of("superuser", &["admin", "user", "moderator"]));
    }

    #[test]
    fn test_url() {
        assert!(url("https://example.com"));
        assert!(url("http://example.com/path?q=1"));
        assert!(!url("not-a-url"));
        assert!(!url("ftp://example.com"));
    }

    #[test]
    fn test_alphanumeric() {
        assert!(alphanumeric("hello123"));
        assert!(!alphanumeric("hello!"));
        assert!(!alphanumeric(""));
    }
}
