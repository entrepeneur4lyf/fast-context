//! Input validation utilities for the fast-context library
//!
//! Provides comprehensive validation for:
//! - File paths and directory access
//! - Configuration parameters
//! - String and buffer bounds checking
//! - User input sanitization

use std::path::{Path, PathBuf};
use std::env;

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    
    #[error("Path does not exist: {0}")]
    PathNotFound(String),
    
    #[error("Path is not a directory: {0}")]
    NotADirectory(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    
    #[error("Invalid file extension: {0}")]
    InvalidExtension(String),
    
    #[error("Configuration value out of range: {0}")]
   OutOfRange(String),
    
    #[error("Empty or whitespace-only string")]
    EmptyString,
    
    #[error("String exceeds maximum length: {0}")]
    StringTooLong(String),
    
    #[error("Invalid character encoding")]
    InvalidEncoding,
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validate a file path is safe and accessible
pub fn validate_file_path(path: &str) -> ValidationResult<PathBuf> {
    // Check for empty string
    if path.trim().is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    // Check string length (prevent excessively long paths)
    if path.len() > 4096 {
        return Err(ValidationError::StringTooLong(format!("Path length: {}", path.len())));
    }
    
    // Check for path traversal attempts
    if path.contains("..") || path.contains("~/") {
        return Err(ValidationError::PathTraversal(path.to_string()));
    }
    
    let path_buf = PathBuf::from(path);
    
    // Check if path exists
    if !path_buf.exists() {
        return Err(ValidationError::PathNotFound(path.to_string()));
    }
    
    // For directory operations, ensure it's actually a directory
    if path_buf.is_dir() {
        // Check if we can read the directory
        if let Err(e) = std::fs::read_dir(&path_buf) {
            return Err(ValidationError::PermissionDenied(
                format!("Permission denied when reading directory '{}': {}", path, e)
            ));
        }
    } else if !path_buf.is_file() {
        return Err(ValidationError::InvalidPath(format!("Path is neither file nor directory: {}", path)));
    }
    
    Ok(path_buf)
}

/// Validate a directory path specifically
pub fn validate_directory_path(path: &str) -> ValidationResult<PathBuf> {
    let path_buf = validate_file_path(path)?;
    
    if !path_buf.is_dir() {
        return Err(ValidationError::NotADirectory(path.to_string()));
    }
    
    Ok(path_buf)
}

/// Validate file extension is supported
pub fn validate_file_extension(extension: &str) -> ValidationResult<()> {
    if extension.trim().is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    // Check for obviously malicious extensions
    let malicious_extensions = ["exe", "bat", "cmd", "scr", "pif", "com", "jar", "app"];
    if malicious_extensions.contains(&extension.to_lowercase().as_str()) {
        return Err(ValidationError::InvalidExtension(extension.to_string()));
    }
    
    Ok(())
}

/// Validate a numeric range
pub fn validate_range<T: std::cmp::PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    param_name: &str,
) -> ValidationResult<T> {
    if value < min || value > max {
        return Err(ValidationError::OutOfRange(
            format!("{} must be between {} and {}, got: {}", param_name, min, max, value)
        ));
    }
    
    Ok(value)
}

/// Validate string is not empty and within reasonable length
pub fn validate_string(input: &str, max_length: usize, field_name: &str) -> ValidationResult<String> {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    if trimmed.len() > max_length {
        return Err(ValidationError::StringTooLong(
            format!("{} exceeds maximum length of {}: {}", field_name, max_length, trimmed.len())
        ));
    }
    
    // Basic sanitization - remove control characters except newline and tab
    let sanitized: String = trimmed
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect();
    
    if sanitized.is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    Ok(sanitized)
}

/// Validate language list contains only supported languages
pub fn validate_languages(languages: &[String]) -> ValidationResult<Vec<String>> {
    let supported_languages = [
        "rust", "python", "javascript", "typescript", "java", "go", "csharp", 
        "cpp", "swift", "objectivec", "php", "ruby", "scala", "zig", "dart",
        "lua", "bash", "css", "html", "xml", "json", "yaml", "markdown", "jsdoc", "regex"
    ];
    
    let mut validated = Vec::new();
    
    for lang in languages {
        let validated_lang = validate_string(lang, 50, "language")?;
        
        if !supported_languages.contains(&validated_lang.to_lowercase().as_str()) {
            return Err(ValidationError::InvalidExtension(
                format!("Unsupported language: {}", validated_lang)
            ));
        }
        
        validated.push(validated_lang);
    }
    
    Ok(validated)
}

/// Validate ignore patterns are safe and reasonable
pub fn validate_ignore_patterns(patterns: &[String]) -> ValidationResult<Vec<String>> {
    let mut validated = Vec::new();
    
    for pattern in patterns {
        let validated_pattern = validate_string(pattern, 200, "ignore_pattern")?;
        
        // Check for potentially dangerous patterns
        let dangerous_patterns = ["**/*", "*", "/**", "../", "..\\"];
        if dangerous_patterns.contains(&validated_pattern.as_str()) {
            return Err(ValidationError::InvalidPath(
                format!("Potentially dangerous ignore pattern: {}", validated_pattern)
            ));
        }
        
        validated.push(validated_pattern);
    }
    
    Ok(validated)
}

/// Safe path resolution that prevents traversal attacks
pub fn safe_resolve_path(base: &Path, path: &str) -> ValidationResult<PathBuf> {
    let path_buf = PathBuf::from(path);
    
    // Check for absolute paths - they might be dangerous
    if path_buf.is_absolute() {
        return Err(ValidationError::PathTraversal(
            format!("Absolute path not allowed: {}", path)
        ));
    }
    
    // Resolve the path relative to base
    let resolved = base.join(path_buf);
    
    // Normalize the path and check for traversal
    let normalized = match resolved.canonicalize() {
        Ok(path) => path,
        Err(_) if !resolved.exists() => {
            // If path doesn't exist, normalize parent directory and append
            if let Some(parent) = resolved.parent() {
                match parent.canonicalize() {
                    Ok(normalized_parent) => normalized_parent.join(resolved.file_name().unwrap_or_default()),
                    Err(_) => return Err(ValidationError::InvalidPath(
                        format!("Cannot normalize path: {}", path)
                    )),
                }
            } else {
                return Err(ValidationError::InvalidPath(
                    format!("Cannot normalize path: {}", path)
                ));
            }
        }
        Err(e) => return Err(ValidationError::InvalidPath(
            format!("Cannot access path {}: {}", path, e)
        )),
    };
    
    // Ensure the resolved path is still within the base directory
    if let Ok(base_normalized) = base.canonicalize() {
        if !normalized.starts_with(&base_normalized) {
            return Err(ValidationError::PathTraversal(
                format!("Path traversal attempt: {} resolves outside base directory", path)
            ));
        }
    }
    
    Ok(normalized)
}

/// Environment variable validation
pub fn validate_env_var(name: &str) -> ValidationResult<String> {
    if name.trim().is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    // Check for potentially dangerous env var names
    let dangerous_vars = ["PATH", "LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH", "HOME"];
    if dangerous_vars.contains(&name.to_uppercase().as_str()) {
        return Err(ValidationError::InvalidPath(
            format!("Access to sensitive environment variable blocked: {}", name)
        ));
    }
    
    env::var(name).map_err(|_| ValidationError::InvalidPath(
        format!("Environment variable not found: {}", name)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path() {
        // Valid path
        assert!(validate_file_path(".").is_ok());
        
        // Empty path
        assert!(matches!(validate_file_path(""), Err(ValidationError::EmptyString)));
        
        // Path traversal
        assert!(matches!(
            validate_file_path("../etc/passwd"), 
            Err(ValidationError::PathTraversal(_))
        ));
    }

    #[test]
    fn test_validate_string() {
        // Valid string
        assert_eq!(validate_string("test", 10, "field").unwrap(), "test");
        
        // Empty string
        assert!(matches!(validate_string("   ", 10, "field"), Err(ValidationError::EmptyString)));
        
        // Too long
        assert!(matches!(
            validate_string("a".repeat(101).as_str(), 100, "field"),
            Err(ValidationError::StringTooLong(_))
        ));
    }

    #[test]
    fn test_validate_range() {
        // Valid range
        assert_eq!(validate_range(5, 1, 10, "test").unwrap(), 5);
        
        // Out of range
        assert!(matches!(
            validate_range(15, 1, 10, "test"),
            Err(ValidationError::OutOfRange(_))
        ));
    }
}