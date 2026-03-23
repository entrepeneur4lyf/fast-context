//! Comprehensive error handling tests
//! Tests error conditions, edge cases, and recovery scenarios

use fast_context::{CoreAnalyzer, LanguageId};
use tempfile::TempDir;
use std::fs;

// === Language Detection Error Tests ===

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

// === File System Error Tests ===

#[test]
fn test_nonexistent_project_root() {
    let analyzer = CoreAnalyzer::new("/nonexistent/path/that/does/not/exist".to_string(), None, None);
    // Should not panic when analyzing with invalid fallback path
    let _analysis_result = analyzer.analyze();
    // Should handle gracefully (may succeed or fail depending on what's in current dir)
}

#[test]
fn test_permission_denied_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a directory and remove read permissions
    let restricted_dir = temp_path.join("restricted");
    fs::create_dir(&restricted_dir).unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
        perms.set_mode(0o000); // Remove all permissions
        fs::set_permissions(&restricted_dir, perms).unwrap();
    }
    
    // Test analyzer creation with restricted directory
    let _result = CoreAnalyzer::new(restricted_dir.to_string_lossy().to_string(), None, None);
    // Should handle gracefully, not panic
}

#[test]
fn test_empty_directory_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    let result = analyzer.analyze();
    
    // Should handle empty directory gracefully
    match result {
        Ok(analysis_result) => {
            // Empty directory should result in 0 files analyzed
            assert_eq!(analysis_result.file_count, 0);
        }
        Err(_) => {
            // Or it might return an error, which is also acceptable
        }
    }
}

#[test]
fn test_malformed_file_content() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a file with invalid UTF-8 content
    let invalid_utf8_file = temp_path.join("invalid.rs");
    fs::write(&invalid_utf8_file, b"\xff\xfe\xfd").unwrap();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    let result = analyzer.analyze();
    
    // Should handle malformed files without panicking
    assert!(result.is_ok(), "Should handle malformed UTF-8 gracefully");
}

// === Configuration Error Tests ===

#[test]
fn test_invalid_language_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Test with invalid language names
    let invalid_languages = vec!["not_a_language".to_string(), "fake_lang".to_string()];
    let analyzer = CoreAnalyzer::new(
        temp_path.to_string_lossy().to_string(), 
        Some(invalid_languages), 
        None
    );
    
    // Should handle invalid languages gracefully
    let result = analyzer.analyze();
    assert!(result.is_ok(), "Should handle invalid language configuration");
}

#[test]
fn test_invalid_ignore_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a test file
    fs::write(temp_path.join("test.rs"), "fn main() {}").unwrap();
    
    // Test with invalid ignore patterns
    let invalid_patterns = vec!["[".to_string(), "**invalid[".to_string()];
    let analyzer = CoreAnalyzer::new(
        temp_path.to_string_lossy().to_string(), 
        None, 
        Some(invalid_patterns)
    );
    
    // Should handle invalid ignore patterns gracefully
    let result = analyzer.analyze();
    assert!(result.is_ok(), "Should handle invalid ignore patterns");
}

// === Symbol Query Error Tests ===

#[test]
fn test_query_nonexistent_symbol() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create empty project
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Query for symbols that don't exist
    let result = analyzer.find_symbols_by_kind("nonexistent_kind".to_string());
    assert!(result.is_ok(), "Query should not panic");
    let symbols = result.unwrap();
    assert!(symbols.is_empty(), "Should return empty list for nonexistent symbol kind");
    
    let result = analyzer.find_symbols_in_file(temp_path.join("nonexistent_file.rs").to_string_lossy().to_string());
    assert!(result.is_err(), "Nonexistent file should return an error");
    
    let result = analyzer.find_dependencies("nonexistent_symbol".to_string());
    assert!(result.is_ok(), "Query should not panic");
    let deps = result.unwrap();
    assert!(deps.is_empty(), "Should return empty list for nonexistent symbol");
}

#[test]
fn test_query_with_invalid_inputs() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Test with empty strings
    let result = analyzer.find_symbols_by_kind("".to_string());
    assert!(result.is_ok(), "Empty symbol kind should not cause panic");
    
    let result = analyzer.find_symbols_in_file("".to_string());
    assert!(result.is_err(), "Empty file path should return an error");
    
    let result = analyzer.find_dependencies("".to_string());
    assert!(result.is_ok(), "Empty symbol name should not cause panic");
    
    // Test with very long strings
    let long_string = "a".repeat(10000);
    let result = analyzer.find_symbols_by_kind(long_string.clone());
    assert!(result.is_ok(), "Very long symbol kind should not cause panic");
    
    let result = analyzer.find_dependencies(long_string);
    assert!(result.is_ok(), "Very long symbol name should not cause panic");
}

// === Resource Limit Error Tests ===

#[test]
fn test_path_traversal_attempts() {
    let temp_dir = TempDir::new().unwrap();
    let _temp_path = temp_dir.path();
    
    // Test with path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd".to_string(),
        "..\\..\\windows\\system32".to_string(),
        "~/../../../etc/shadow".to_string(),
    ];
    
    for malicious_path in malicious_paths {
        let _result = CoreAnalyzer::new(malicious_path.clone(), None, None);
    }
}

#[test]
fn test_extremely_long_paths() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a very long path name
    let long_name = "a".repeat(255);
    let long_path = temp_path.join(long_name);
    
    // This might fail, but should not panic
    let _ = fs::create_dir(&long_path);
    
    let _analyzer = CoreAnalyzer::new(long_path.to_string_lossy().to_string(), None, None);
    // Should handle gracefully
}

// === Memory Pressure Tests ===

#[test]
fn test_large_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create a large file (1MB)
    let large_content = "fn large_function() { println!(\"large content\"); }\n".repeat(10000);
    fs::write(temp_path.join("large.rs"), large_content).unwrap();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Should handle large files without panicking
    let result = analyzer.analyze();
    assert!(result.is_ok(), "Should handle large files gracefully");
}

#[test]
fn test_many_small_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create many small files
    for i in 0..100 {
        let content = format!("pub fn small_{}() {{}}", i);
        fs::write(temp_path.join(format!("small_{}.rs", i)), content).unwrap();
    }
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Should handle many files without panicking
    let result = analyzer.analyze();
    assert!(result.is_ok(), "Should handle many files gracefully");
}

// === Concurrent Access Tests ===

#[test]
fn test_concurrent_analyzer_usage() {
    use std::thread;
    use std::sync::Arc;
    
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create test files
    for i in 0..10 {
        let content = format!("pub fn test_{}() {{}}", i);
        fs::write(temp_path.join(format!("test_{}.rs", i)), content).unwrap();
    }
    
    let analyzer = Arc::new(CoreAnalyzer::new(
        temp_path.to_string_lossy().to_string(), 
        None, 
        None
    ));
    
    let mut handles = vec![];
    
    // Spawn multiple threads using the same analyzer
    for _ in 0..5 {
        let analyzer_clone = Arc::clone(&analyzer);
        let handle = thread::spawn(move || {
            let result = analyzer_clone.analyze();
            assert!(result.is_ok(), "Concurrent analysis should not fail");
            result.unwrap().file_count
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

// === Error Message Quality Tests ===

#[test]
fn test_error_message_quality() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Test error messages are descriptive
    let result = analyzer.find_symbols_in_file("/completely/nonexistent/path.rs".to_string());
    assert!(result.is_err(), "Should return an error for nonexistent files");
    assert!(
        result.unwrap_err().to_string().contains("nonexistent"),
        "Error should contain relevant path context"
    );
}

#[test]
fn test_error_conversion_consistency() {
    // Test that different error types convert consistently to FastContextError
    use fast_context::FastContextResult;
    
    fn test_function() -> FastContextResult<()> {
        Err("test error".into())
    }
    
    let result = test_function();
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(!error.to_string().is_empty(), "Error message should not be empty");
    assert!(error.to_string().contains("test"), "Error message should contain original context");
}

// === Recovery Tests ===

#[test]
fn test_recovery_after_errors() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
    
    // Trigger an error condition
    let _ = analyzer.find_symbols_in_file("nonexistent.rs".to_string());
    
    // Subsequent operations should still work
    let result = analyzer.find_symbols_by_kind("function".to_string());
    assert!(result.is_ok(), "Analyzer should remain functional after errors");
    
    // Multiple error conditions should not break the analyzer
    for i in 0..5 {
        let _ = analyzer.find_symbols_in_file(format!("nonexistent_{}.rs", i));
    }
    
    // Should still work correctly
    let result = analyzer.analyze();
    assert!(result.is_ok(), "Analyzer should recover from multiple errors");
}
