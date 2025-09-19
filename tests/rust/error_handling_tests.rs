//! Comprehensive error handling tests
//! Tests error conditions, edge cases, and recovery scenarios

use fast_context::LanguageId;

#[test]
fn test_language_id_from_string() {
    // Test valid language detection
    assert_eq!(LanguageId::from_string("rust"), Some(LanguageId::Rust));
    assert_eq!(LanguageId::from_string("javascript"), Some(LanguageId::JavaScript));
    assert_eq!(LanguageId::from_string("python"), Some(LanguageId::Python));
    assert_eq!(LanguageId::from_string("typescript"), Some(LanguageId::TypeScript));

    // Test case insensitive
    assert_eq!(LanguageId::from_string("RUST"), Some(LanguageId::Rust));
    assert_eq!(LanguageId::from_string("JavaScript"), Some(LanguageId::JavaScript));

    // Test unknown language
    assert_eq!(LanguageId::from_string("unknown"), None);
    assert_eq!(LanguageId::from_string(""), None);
}

#[test]
fn test_language_id_display() {
    // Test that LanguageId can be displayed as strings
    assert_eq!(format!("{:?}", LanguageId::Rust), "Rust");
    assert_eq!(format!("{:?}", LanguageId::JavaScript), "JavaScript");
    assert_eq!(format!("{:?}", LanguageId::Python), "Python");
    assert_eq!(format!("{:?}", LanguageId::CSS), "CSS");
}

#[test]
fn test_language_id_equality() {
    // Test equality comparisons
    assert_eq!(LanguageId::Rust, LanguageId::Rust);
    assert_ne!(LanguageId::Rust, LanguageId::JavaScript);
    assert_ne!(LanguageId::Python, LanguageId::CSS);
}
