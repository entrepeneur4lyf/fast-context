//! Input validation utilities for the fast-context library
//!
//! Provides comprehensive validation for:
//! - File paths and directory access
//! - Configuration parameters
//! - String and buffer bounds checking
//! - User input sanitization

use std::path::{Path, PathBuf};
use std::env;
use std::io::Read;

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

/// Secure file reading with path validation
pub fn secure_read_file(path: &Path) -> ValidationResult<String> {
    // Validate the path first
    let validated_path = validate_file_path(&path.to_string_lossy())?;
    
    // Additional check: ensure we're not reading sensitive system files
    let path_str = validated_path.to_string_lossy().to_lowercase();
    let sensitive_patterns = [
        "/etc/passwd", "/etc/shadow", "/etc/hosts", "/etc/hostname",
        "/proc/", "/sys/", "/dev/", "/boot/", "/usr/bin/", "/bin/",
        "password", "secret", "key", "token", "credential"
    ];
    
    for pattern in &sensitive_patterns {
        if path_str.contains(pattern) {
            return Err(ValidationError::PathTraversal(
                format!("Access to sensitive file blocked: {}", path_str)
            ));
        }
    }
    
    // Check file size to prevent memory exhaustion (max 10MB)
    match validated_path.metadata() {
        Ok(metadata) => {
            if metadata.len() > 10 * 1024 * 1024 {
                return Err(ValidationError::StringTooLong(
                    format!("File too large: {} bytes", metadata.len())
                ));
            }
        }
        Err(_) => {
            // If we can't get metadata, proceed with caution
        }
    }
    
    std::fs::read_to_string(&validated_path)
        .map_err(|e| ValidationError::InvalidPath(
            format!("Failed to read file {}: {}", validated_path.display(), e)
        ))
}

/// Safe file path resolution within project boundaries
pub fn resolve_project_path(project_root: &Path, relative_path: &str) -> ValidationResult<PathBuf> {
    safe_resolve_path(project_root, relative_path)
}

/// Comprehensive input validation for configuration parameters
pub fn validate_config_params<T: std::fmt::Debug>(
    params: &std::collections::HashMap<String, T>,
    required_keys: &[&str],
    max_config_size: usize,
) -> ValidationResult<()> {
    // Check overall configuration size
    if params.len() > max_config_size {
        return Err(ValidationError::OutOfRange(
            format!("Configuration too large: {} parameters (max: {})", params.len(), max_config_size)
        ));
    }
    
    // Validate all required keys are present
    for key in required_keys {
        if !params.contains_key(*key) {
            return Err(ValidationError::InvalidPath(
                format!("Missing required configuration parameter: {}", key)
            ));
        }
    }
    
    // Validate parameter names don't contain suspicious patterns
    for key in params.keys() {
        validate_config_key(key)?;
    }
    
    Ok(())
}

/// Validate configuration parameter keys for security
pub fn validate_config_key(key: &str) -> ValidationResult<()> {
    // Check for empty or whitespace-only keys
    if key.trim().is_empty() {
        return Err(ValidationError::EmptyString);
    }
    
    // Check key length (prevent excessive memory usage)
    if key.len() > 100 {
        return Err(ValidationError::StringTooLong(
            format!("Configuration key too long: {} characters", key.len())
        ));
    }
    
    // Check for suspicious patterns that might indicate injection attempts
    let suspicious_patterns = [
        "..", "~/", "$", "`", "\\", "<", ">", "|", "&", ";", "\n", "\r",
        "javascript:", "data:", "file:", "http:", "https:", "ftp:",
        "eval(", "exec(", "system(", "shell_exec(", "passthru("
    ];
    
    for pattern in &suspicious_patterns {
        if key.to_lowercase().contains(pattern) {
            return Err(ValidationError::InvalidPath(
                format!("Suspicious pattern in configuration key '{}': {}", key, pattern)
            ));
        }
    }
    
    // Only allow alphanumeric, underscore, hyphen, and dot in keys
    if !key.chars().all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.')) {
        return Err(ValidationError::InvalidPath(
            format!("Invalid characters in configuration key: {}", key)
        ));
    }
    
    Ok(())
}

/// Validate numeric input with range and type checking
pub fn validate_numeric_input<T: std::str::FromStr + std::cmp::PartialOrd + std::fmt::Display>(
    input: &str,
    field_name: &str,
    min: Option<T>,
    max: Option<T>,
) -> ValidationResult<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    // Parse the input
    let value = input.trim().parse::<T>()
        .map_err(|e| ValidationError::InvalidPath(
            format!("Invalid numeric value for '{}': '{}', error: {}", field_name, input, e)
        ))?;
    
    // Check minimum bound if provided
    if let Some(min_val) = min {
        if value < min_val {
            return Err(ValidationError::OutOfRange(
                format!("Value for '{}' must be at least {}, got: {}", field_name, min_val, value)
            ));
        }
    }
    
    // Check maximum bound if provided
    if let Some(max_val) = max {
        if value > max_val {
            return Err(ValidationError::OutOfRange(
                format!("Value for '{}' must be at most {}, got: {}", field_name, max_val, value)
            ));
        }
    }
    
    Ok(value)
}

/// Validate boolean input with flexible parsing
pub fn validate_boolean_input(input: &str, field_name: &str) -> ValidationResult<bool> {
    let normalized = input.trim().to_lowercase();
    
    match normalized.as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => Ok(true),
        "false" | "0" | "no" | "off" | "disabled" => Ok(false),
        _ => Err(ValidationError::InvalidPath(
            format!("Invalid boolean value for '{}': '{}'. Expected: true/false, 1/0, yes/no, on/off, enabled/disabled", field_name, input)
        )),
    }
}

/// Validate URL/URI input for security
pub fn validate_url(input: &str, field_name: &str, allow_local: bool) -> ValidationResult<()> {
    let trimmed = validate_string(input, 2048, field_name)?;
    
    // Check for common URI schemes
    let allowed_schemes = if allow_local {
        vec!["http:", "https:", "ftp:", "file:", "data:"]
    } else {
        vec!["http:", "https:", "ftp:"]
    };
    
    // Basic URL validation - check if any allowed scheme matches the URL
    let lower_trimmed = trimmed.to_lowercase();
    let has_valid_scheme = allowed_schemes.iter().any(|scheme| {
        let scheme_base = scheme.trim_end_matches(':');
        // Check if URL starts with scheme or scheme://
        lower_trimmed.starts_with(&format!("{}://", scheme_base)) || 
        lower_trimmed.starts_with(scheme)
    });
    
    if !has_valid_scheme {
        return Err(ValidationError::InvalidPath(
            format!("Invalid URL scheme for '{}': {}. Allowed: {}", field_name, input, allowed_schemes.join(", "))
        ));
    }
    
    // Check for potentially dangerous characters
    let dangerous_chars = ['<', '>', '"', '\'', '`', '\\', '|', '&', ';'];
    for char in dangerous_chars.iter() {
        if trimmed.contains(*char) {
            return Err(ValidationError::InvalidPath(
                format!("URL contains dangerous character '{}' for '{}'", char, field_name)
            ));
        }
    }
    
    // Check for path traversal patterns (be more careful with URLs)
    if trimmed.contains("..") && !trimmed.starts_with("http") {
        return Err(ValidationError::PathTraversal(
            format!("URL contains suspicious patterns for '{}': {}", field_name, input)
        ));
    }
    
    Ok(())
}

/// Validate email address format
pub fn validate_email(input: &str, field_name: &str) -> ValidationResult<String> {
    let email = validate_string(input, 254, field_name)?; // RFC 5321 max length
    
    // Basic email validation using regex
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| ValidationError::InvalidPath("Invalid email regex pattern".to_string()))?;
    
    if !email_regex.is_match(&email) {
        return Err(ValidationError::InvalidPath(
            format!("Invalid email format for '{}': {}", field_name, email)
        ));
    }
    
    // Check for potentially dangerous patterns
    let dangerous_patterns = ["javascript:", "data:", "<script", "onclick", "onload"];
    for pattern in &dangerous_patterns {
        if email.to_lowercase().contains(pattern) {
            return Err(ValidationError::InvalidPath(
                format!("Email contains suspicious pattern for '{}': {}", field_name, pattern)
            ));
        }
    }
    
    Ok(email)
}

/// Validate JSON string structure and content
pub fn validate_json_string(input: &str, field_name: &str, max_size: usize) -> ValidationResult<serde_json::Value> {
    // Validate string input first
    let trimmed = validate_string(input, max_size, field_name)?;
    
    // Parse JSON
    let json_value: serde_json::Value = serde_json::from_str(&trimmed)
        .map_err(|e| ValidationError::InvalidPath(
            format!("Invalid JSON for '{}': {}", field_name, e)
        ))?;
    
    // Validate JSON structure for security
    validate_json_security(&json_value, field_name)?;
    
    Ok(json_value)
}

/// Recursive validation of JSON structure for security
fn validate_json_security(value: &serde_json::Value, field_name: &str) -> ValidationResult<()> {
    use serde_json::Value;
    
    match value {
        Value::String(s) => {
            // Check for potentially dangerous strings
            let dangerous_patterns = [
                "<script", "javascript:", "data:", "eval(", "exec(",
                "system(", "shell_exec(", "passthru(", "document.",
                "window.", "alert(", "prompt(", "confirm("
            ];
            
            for pattern in &dangerous_patterns {
                if s.to_lowercase().contains(pattern) {
                    return Err(ValidationError::InvalidPath(
                        format!("JSON string contains suspicious pattern in '{}': {}", field_name, pattern)
                    ));
                }
            }
            
            // Check string length
            if s.len() > 10000 {
                return Err(ValidationError::StringTooLong(
                    format!("JSON string too long in '{}': {} characters", field_name, s.len())
                ));
            }
        },
        Value::Array(arr) => {
            // Check array length to prevent memory exhaustion
            if arr.len() > 1000 {
                return Err(ValidationError::OutOfRange(
                    format!("JSON array too large in '{}': {} items", field_name, arr.len())
                ));
            }
            
            // Recursively validate array elements
            for (i, item) in arr.iter().enumerate() {
                validate_json_security(item, &format!("{}[{}]", field_name, i))?;
            }
        },
        Value::Object(obj) => {
            // Check object size to prevent memory exhaustion
            if obj.len() > 100 {
                return Err(ValidationError::OutOfRange(
                    format!("JSON object too large in '{}': {} properties", field_name, obj.len())
                ));
            }
            
            // Validate key names
            for (key, value) in obj.iter() {
                validate_config_key(key)?;
                validate_json_security(value, &format!("{}.{}", field_name, key))?;
            }
        },
        Value::Number(n) => {
            // Check numeric ranges
            if n.is_u64() && n.as_u64().unwrap_or(0) > 1_000_000_000 {
                return Err(ValidationError::OutOfRange(
                    format!("JSON number too large in '{}': {}", field_name, n)
                ));
            }
            
            if n.is_i64() && n.as_i64().unwrap_or(0).abs() > 1_000_000_000 {
                return Err(ValidationError::OutOfRange(
                    format!("JSON number magnitude too large in '{}': {}", field_name, n)
                ));
            }
        },
        _ => {} // Null and boolean values are safe
    }
    
    Ok(())
}

/// Validate and sanitize command-line arguments
pub fn validate_command_args(args: &[String], program_name: &str) -> ValidationResult<Vec<String>> {
    let mut validated = Vec::new();
    
    // Check argument count to prevent command injection
    if args.len() > 50 {
        return Err(ValidationError::OutOfRange(
            format!("Too many command-line arguments for {}: {} (max: 50)", program_name, args.len())
        ));
    }
    
    for (i, arg) in args.iter().enumerate() {
        let sanitized = validate_string(arg, 1000, &format!("arg_{}", i))?;
        
        // Check for shell injection patterns
        let shell_patterns = ["|", "&", ";", "`", "$(", ")", "<", ">", "\n", "\r"];
        for pattern in &shell_patterns {
            if sanitized.contains(pattern) {
                return Err(ValidationError::InvalidPath(
                    format!("Command argument {} contains shell injection pattern: '{}'", i, pattern)
                ));
            }
        }
        
        validated.push(sanitized);
    }
    
    Ok(validated)
}

/// Validate buffer/byte input for size and content
pub fn validate_buffer_input<'a>(buffer: &'a [u8], field_name: &str, max_size: usize) -> ValidationResult<&'a [u8]> {
    // Check buffer size
    if buffer.len() > max_size {
        return Err(ValidationError::StringTooLong(
            format!("Buffer '{}' too large: {} bytes (max: {})", field_name, buffer.len(), max_size)
        ));
    }
    
    // Check for null bytes in strings (potential security issue)
    if buffer.contains(&0) {
        return Err(ValidationError::InvalidEncoding);
    }
    
    // Basic content analysis for binary safety
    if let Some(first_null) = buffer.iter().position(|&b| b == 0) {
        // Found null byte, check if it's in a suspicious position
        if first_null < buffer.len() - 1 {
            return Err(ValidationError::InvalidEncoding);
        }
    }
    
    Ok(buffer)
}

/// Validate and parse duration strings (e.g., "5s", "100ms", "1h")
pub fn validate_duration(input: &str, field_name: &str, max_duration_ms: u64) -> ValidationResult<std::time::Duration> {
    let trimmed = validate_string(input, 50, field_name)?;
    let lower = trimmed.to_lowercase();
    
    // Parse duration with units
    let duration = if lower.ends_with("ms") {
        let ms = validate_numeric_input(&lower[..lower.len()-2], field_name, Some(0u64), Some(max_duration_ms))?;
        std::time::Duration::from_millis(ms)
    } else if lower.ends_with('s') {
        let s = validate_numeric_input(&lower[..lower.len()-1], field_name, Some(0u64), Some(max_duration_ms / 1000))?;
        std::time::Duration::from_secs(s)
    } else if lower.ends_with('m') {
        let m = validate_numeric_input(&lower[..lower.len()-1], field_name, Some(0u64), Some(max_duration_ms / (1000 * 60)))?;
        std::time::Duration::from_secs(m * 60)
    } else if lower.ends_with('h') {
        let h = validate_numeric_input(&lower[..lower.len()-1], field_name, Some(0u64), Some(max_duration_ms / (1000 * 60 * 60)))?;
        std::time::Duration::from_secs(h * 60 * 60)
    } else {
        return Err(ValidationError::InvalidPath(
            format!("Invalid duration format for '{}': '{}'. Use units: ms, s, m, h", field_name, input)
        ));
    };
    
    // Final check against maximum
    if duration.as_millis() as u64 > max_duration_ms {
        return Err(ValidationError::OutOfRange(
            format!("Duration '{}' exceeds maximum: {}ms", input, max_duration_ms)
        ));
    }
    
    Ok(duration)
}

/// Streaming file reader with chunked processing
pub struct StreamingFileReader {
    file: std::fs::File,
    buffer: Vec<u8>,
    chunk_size: usize,
    total_read: usize,
    max_size: usize,
}

impl StreamingFileReader {
    /// Create a new streaming file reader with validation
    pub fn new(path: &Path, chunk_size: Option<usize>, max_size: Option<usize>) -> ValidationResult<Self> {
        let validated_path = validate_file_path(&path.to_string_lossy())?;
        
        // Check file size to prevent memory exhaustion
        let metadata = validated_path.metadata()
            .map_err(|e| ValidationError::InvalidPath(
                format!("Failed to get metadata for {}: {}", path.display(), e)
            ))?;
        
        let max_size = max_size.unwrap_or(50 * 1024 * 1024); // Default 50MB
        if metadata.len() > max_size as u64 {
            return Err(ValidationError::StringTooLong(
                format!("File too large: {} bytes (max: {})", metadata.len(), max_size)
            ));
        }
        
        let file = std::fs::File::open(&validated_path)
            .map_err(|e| ValidationError::InvalidPath(
                format!("Failed to open file {}: {}", path.display(), e)
            ))?;
        
        let chunk_size = chunk_size.unwrap_or(64 * 1024); // Default 64KB chunks
        
        Ok(Self {
            file,
            buffer: Vec::with_capacity(chunk_size),
            chunk_size,
            total_read: 0,
            max_size,
        })
    }
    
    /// Read the next chunk from the file
    pub fn read_next_chunk(&mut self) -> ValidationResult<Option<Vec<u8>>> {
        if self.total_read >= self.max_size {
            return Err(ValidationError::StringTooLong(
                format!("Read limit exceeded: {} bytes", self.max_size)
            ));
        }
        
        self.buffer.clear();
        self.buffer.resize(self.chunk_size, 0);
        
        match self.file.read(&mut self.buffer) {
            Ok(0) => Ok(None), // EOF
            Ok(bytes_read) => {
                self.total_read += bytes_read;
                self.buffer.truncate(bytes_read);
                Ok(Some(self.buffer.clone()))
            }
            Err(e) => Err(ValidationError::InvalidPath(
                format!("Failed to read from file: {}", e)
            ))
        }
    }
    
    /// Process file in chunks with a callback function
    pub fn process_chunks<F, R>(&mut self, mut processor: F) -> ValidationResult<R>
    where
        F: FnMut(&[u8]) -> ValidationResult<R>,
        R: Default,
    {
        let mut result = R::default();
        let mut processed_any = false;
        
        while let Some(chunk) = self.read_next_chunk()? {
            result = processor(&chunk)?;
            processed_any = true;
        }
        
        if !processed_any {
            // File was empty, return default result
            return Ok(result);
        }
        
        Ok(result)
    }
    
    /// Get total bytes read so far
    pub fn bytes_read(&self) -> usize {
        self.total_read
    }
    
    /// Check if we've reached the end of file
    pub fn is_eof(&self) -> bool {
        self.total_read >= self.max_size || self.buffer.is_empty()
    }
}

/// Streaming text file reader that handles encoding and line breaks
pub struct StreamingTextReader {
    reader: StreamingFileReader,
    encoding: &'static encoding_rs::Encoding,
    incomplete_bytes: Vec<u8>,
}

impl StreamingTextReader {
    /// Create a new streaming text reader
    pub fn new(path: &Path, chunk_size: Option<usize>, max_size: Option<usize>) -> ValidationResult<Self> {
        let reader = StreamingFileReader::new(path, chunk_size, max_size)?;
        Ok(Self {
            reader,
            encoding: encoding_rs::UTF_8,
            incomplete_bytes: Vec::new(),
        })
    }
    
    /// Read next line from the file
    pub fn read_next_line(&mut self) -> ValidationResult<Option<String>> {
        let mut line_buffer = Vec::new();
        
        // Start with any incomplete bytes from previous reads
        if !self.incomplete_bytes.is_empty() {
            line_buffer.extend_from_slice(&self.incomplete_bytes);
            self.incomplete_bytes.clear();
        }
        
        while let Some(chunk) = self.reader.read_next_chunk()? {
            // Find the next newline character
            if let Some(newline_pos) = chunk.iter().position(|&b| b == b'\n') {
                // Split at newline
                line_buffer.extend_from_slice(&chunk[..newline_pos]);
                
                // Save the rest for next time (including the \n character for line counting)
                let remaining_start = newline_pos + 1;
                if remaining_start < chunk.len() {
                    self.incomplete_bytes = chunk[remaining_start..].to_vec();
                }
                
                // Decode the line
                let (text, _, _) = self.encoding.decode(&line_buffer);
                return Ok(Some(text.to_string()));
            } else {
                // No newline found, add to buffer and continue
                line_buffer.extend_from_slice(&chunk);
            }
        }
        
        if line_buffer.is_empty() {
            Ok(None) // EOF
        } else {
            // Last line without newline
            let (text, _, _) = self.encoding.decode(&line_buffer);
            Ok(Some(text.to_string()))
        }
    }
    
    /// Process file line by line with a callback
    pub fn process_lines<F, R>(&mut self, mut processor: F) -> ValidationResult<R>
    where
        F: FnMut(&str, usize) -> ValidationResult<R>,
        R: Default,
    {
        let mut result = R::default();
        let mut line_number = 1;
        
        while let Some(line) = self.read_next_line()? {
            result = processor(&line, line_number)?;
            line_number += 1;
        }
        
        Ok(result)
    }
    
    /// Get total bytes read
    pub fn bytes_read(&self) -> usize {
        self.reader.bytes_read()
    }
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

    #[test]
    fn test_validate_config_params() {
        use std::collections::HashMap;
        
        // Valid configuration
        let mut config = HashMap::new();
        config.insert("timeout".to_string(), 30u32);
        config.insert("max_files".to_string(), 1000u32);
        let required_keys = ["timeout", "max_files"];
        
        assert!(validate_config_params(&config, &required_keys, 10).is_ok());
        
        // Missing required key
        let config_missing = HashMap::from([("timeout".to_string(), 30u32)]);
        assert!(matches!(
            validate_config_params(&config_missing, &required_keys, 10),
            Err(ValidationError::InvalidPath(_))
        ));
        
        // Configuration too large
        let mut large_config = HashMap::new();
        for i in 0..11 {
            large_config.insert(format!("key_{}", i), i);
        }
        assert!(matches!(
            validate_config_params(&large_config, &required_keys, 10),
            Err(ValidationError::OutOfRange(_))
        ));
    }

    #[test]
    fn test_validate_config_key() {
        // Valid keys
        assert!(validate_config_key("timeout").is_ok());
        assert!(validate_config_key("max-files").is_ok());
        assert!(validate_config_key("cache.size").is_ok());
        
        // Invalid keys
        assert!(matches!(validate_config_key(""), Err(ValidationError::EmptyString)));
        assert!(matches!(validate_config_key("   "), Err(ValidationError::EmptyString)));
        assert!(matches!(
            validate_config_key("a".repeat(101).as_str()),
            Err(ValidationError::StringTooLong(_))
        ));
        assert!(matches!(
            validate_config_key("key_with_dollar$"),
            Err(ValidationError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_config_key("javascript:alert(1)"),
            Err(ValidationError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_numeric_input() {
        // Valid numeric inputs
        assert_eq!(validate_numeric_input::<u32>("42", "test", None, None).unwrap(), 42);
        assert_eq!(validate_numeric_input::<i32>("-10", "test", Some(-20), Some(0)).unwrap(), -10);
        
        // Invalid inputs
        assert!(matches!(
            validate_numeric_input::<u32>("not_a_number", "test", None, None),
            Err(ValidationError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_numeric_input::<u32>("150", "test", Some(0), Some(100)),
            Err(ValidationError::OutOfRange(_))
        ));
    }

    #[test]
    fn test_validate_boolean_input() {
        // Valid boolean inputs
        assert!(validate_boolean_input("true", "test").unwrap());
        assert!(!validate_boolean_input("false", "test").unwrap());
        assert!(validate_boolean_input("1", "test").unwrap());
        assert!(!validate_boolean_input("0", "test").unwrap());
        assert!(validate_boolean_input("yes", "test").unwrap());
        
        // Invalid inputs
        assert!(matches!(
            validate_boolean_input("maybe", "test"),
            Err(ValidationError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_url() {
        // Valid URLs
        assert!(validate_url("https://example.com", "test", false).is_ok());
        assert!(validate_url("http://localhost:8080", "test", false).is_ok());
        assert!(validate_url("file:///path/to/file", "test", true).is_ok());
        
        // Invalid URLs
        assert!(matches!(
            validate_url("javascript:alert('xss')", "test", false),
            Err(ValidationError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_url("ftp://example.com/../../../etc/passwd", "test", false),
            Err(ValidationError::PathTraversal(_))
        ));
    }

    #[test]
    fn test_validate_email() {
        // Valid emails
        assert_eq!(validate_email("user@example.com", "test").unwrap(), "user@example.com");
        assert_eq!(validate_email("test.email+tag@domain.co.uk", "test").unwrap(), "test.email+tag@domain.co.uk");
        
        // Invalid emails
        assert!(matches!(
            validate_email("invalid-email", "test"),
            Err(ValidationError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_email("user@javascript:alert(1).com", "test"),
            Err(ValidationError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_json_string() {
        // Valid JSON
        let json = r#"{"name": "test", "value": 42, "enabled": true}"#;
        assert!(validate_json_string(json, "test", 1000).is_ok());
        
        // Invalid JSON
        assert!(matches!(
            validate_json_string("invalid json", "test", 1000),
            Err(ValidationError::InvalidPath(_))
        ));
        
        // JSON with dangerous content
        let dangerous_json = r#"{"script": "<script>alert('xss')</script>"}"#;
        assert!(matches!(
            validate_json_string(dangerous_json, "test", 1000),
            Err(ValidationError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_command_args() {
        // Valid arguments
        let args = vec!["program".to_string(), "--config".to_string(), "file.conf".to_string()];
        assert!(validate_command_args(&args, "program").is_ok());
        
        // Too many arguments
        let mut many_args = vec!["program".to_string()];
        for i in 0..50 {
            many_args.push(format!("arg_{}", i));
        }
        assert!(matches!(
            validate_command_args(&many_args, "program"),
            Err(ValidationError::OutOfRange(_))
        ));
        
        // Arguments with shell injection
        let dangerous_args = vec!["program".to_string(), "arg; rm -rf /".to_string()];
        assert!(matches!(
            validate_command_args(&dangerous_args, "program"),
            Err(ValidationError::InvalidPath(_))
        ));
    }

    #[test]
    fn test_validate_buffer_input() {
        // Valid buffer
        let valid_buffer = b"valid data";
        assert!(validate_buffer_input(valid_buffer, "test", 100).is_ok());
        
        // Buffer too large
        let large_buffer = vec![0u8; 101];
        assert!(matches!(
            validate_buffer_input(&large_buffer, "test", 100),
            Err(ValidationError::StringTooLong(_))
        ));
        
        // Buffer with null bytes
        let null_buffer = b"data\x00with\x00nulls";
        assert!(matches!(
            validate_buffer_input(null_buffer, "test", 100),
            Err(ValidationError::InvalidEncoding)
        ));
    }

    #[test]
    fn test_validate_duration() {
        // Valid durations
        assert!(validate_duration("5s", "test", 10000).is_ok());
        assert!(validate_duration("100ms", "test", 10000).is_ok());
        assert!(validate_duration("1h", "test", 3600000).is_ok());
        
        // Invalid durations
        assert!(matches!(
            validate_duration("5x", "test", 10000),
            Err(ValidationError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_duration("120s", "test", 10000), // 120 seconds > 10 seconds max
            Err(ValidationError::OutOfRange(_))
        ));
    }
}