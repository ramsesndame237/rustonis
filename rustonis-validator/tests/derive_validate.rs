use rustonis_validator::Validate;
use serde::Deserialize;

// ─── Structs de test ──────────────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
struct CreateUserInput {
    #[validate(email)]
    email: String,

    #[validate(min_length = 8, max_length = 100)]
    password: String,

    #[validate(required)]
    name: String,
}

#[derive(Deserialize, Validate)]
struct AgeInput {
    #[validate(min = 18, max = 120)]
    age: u32,
}

#[derive(Deserialize, Validate)]
struct CustomMessageInput {
    #[validate(email, message = "Adresse email invalide")]
    email: String,

    #[validate(min_length = 3, message = "Au moins 3 caractères requis")]
    username: String,
}

#[derive(Deserialize, Validate)]
struct NoRulesInput {
    #[allow(dead_code)]
    name: String,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_derive_valid_input() {
    let input = CreateUserInput {
        email: "user@example.com".into(),
        password: "supersecret".into(),
        name: "Alice".into(),
    };
    assert!(input.validate().is_ok());
}

#[test]
fn test_derive_invalid_email() {
    let input = CreateUserInput {
        email: "not-an-email".into(),
        password: "supersecret".into(),
        name: "Alice".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("email"));
    assert_eq!(
        err.fields()["email"][0],
        "must be a valid email address"
    );
}

#[test]
fn test_derive_password_too_short() {
    let input = CreateUserInput {
        email: "user@example.com".into(),
        password: "short".into(),
        name: "Alice".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("password"));
    assert_eq!(err.fields()["password"][0], "must be at least 8 characters");
}

#[test]
fn test_derive_password_too_long() {
    let input = CreateUserInput {
        email: "user@example.com".into(),
        password: "a".repeat(101),
        name: "Alice".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("password"));
    assert_eq!(err.fields()["password"][0], "must be at most 100 characters");
}

#[test]
fn test_derive_required_empty() {
    let input = CreateUserInput {
        email: "user@example.com".into(),
        password: "supersecret".into(),
        name: "".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("name"));
    assert_eq!(err.fields()["name"][0], "is required");
}

#[test]
fn test_derive_required_whitespace_only() {
    let input = CreateUserInput {
        email: "user@example.com".into(),
        password: "supersecret".into(),
        name: "   ".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("name"));
}

#[test]
fn test_derive_multiple_errors() {
    let input = CreateUserInput {
        email: "bad".into(),
        password: "sh".into(),
        name: "".into(),
    };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("email"));
    assert!(err.fields().contains_key("password"));
    assert!(err.fields().contains_key("name"));
}

#[test]
fn test_derive_numeric_min() {
    let input = AgeInput { age: 15 };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("age"));
    assert_eq!(err.fields()["age"][0], "must be at least 18");
}

#[test]
fn test_derive_numeric_max() {
    let input = AgeInput { age: 200 };
    let err = input.validate().unwrap_err();
    assert!(err.fields().contains_key("age"));
    assert_eq!(err.fields()["age"][0], "must be at most 120");
}

#[test]
fn test_derive_numeric_valid() {
    let input = AgeInput { age: 25 };
    assert!(input.validate().is_ok());
}

#[test]
fn test_derive_custom_message_email() {
    let input = CustomMessageInput {
        email: "bad".into(),
        username: "alice".into(),
    };
    let err = input.validate().unwrap_err();
    assert_eq!(err.fields()["email"][0], "Adresse email invalide");
}

#[test]
fn test_derive_custom_message_min_length() {
    let input = CustomMessageInput {
        email: "user@example.com".into(),
        username: "ab".into(),
    };
    let err = input.validate().unwrap_err();
    assert_eq!(err.fields()["username"][0], "Au moins 3 caractères requis");
}

#[test]
fn test_derive_no_rules_always_valid() {
    let input = NoRulesInput { name: "".into() };
    assert!(input.validate().is_ok());
}
