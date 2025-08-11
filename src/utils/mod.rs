//! # Utility Functions Module
//!
//! This module contains utility functions extracted from the monolithic lib.rs

use napi_derive::napi;
use crate::analyzer::AnalyzerConfig;

/// Get the version of the fast-context package
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get list of supported programming languages
#[napi]
pub fn get_supported_languages() -> Vec<String> {
    vec![
        "rust".to_string(),
        "javascript".to_string(),
        "typescript".to_string(),
        "python".to_string(),
        "java".to_string(),
        "go".to_string(),
        "cpp".to_string(),
        "c".to_string(),
        "csharp".to_string(),
        "php".to_string(),
        "ruby".to_string(),
        "swift".to_string(),
        "kotlin".to_string(),
        "scala".to_string(),
        "lua".to_string(),
        "bash".to_string(),
        "css".to_string(),
        "html".to_string(),
        "xml".to_string(),
        "json".to_string(),
        "yaml".to_string(),
        "toml".to_string(),
        "markdown".to_string(),
    ]
}

/// Detect the programming language of a file based on its extension
#[napi]
pub fn detect_language(file_path: String) -> Option<String> {
    let path = std::path::Path::new(&file_path);

    // Handle special files without extensions
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        match file_name {
            "Dockerfile" => return Some("Dockerfile".to_string()),
            "Makefile" => return Some("Makefile".to_string()),
            _ => {}
        }
    }

    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())?;

    let language = match extension.as_str() {
        "rs" => "Rust",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" => "TypeScript",
        "py" | "pyw" | "pyi" => "Python",
        "java" => "Java",
        "go" => "Go",
        "cpp" | "cc" | "cxx" | "c++" => "C++",
        "c" | "h" => "C",
        "cs" => "CSharp",
        "php" => "PHP",
        "rb" => "Ruby",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "scala" | "sc" => "Scala",
        "lua" => "Lua",
        "sh" | "bash" | "zsh" | "fish" => "Bash",
        "css" => "CSS",
        "html" | "htm" => "HTML",
        "xml" => "XML",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "md" | "markdown" => "Markdown",
        _ => return None,
    };

    Some(language.to_string())
}

/// Check if the analyzer configuration is valid
#[napi]
pub fn check_configuration(config: Option<AnalyzerConfig>) -> napi::Result<String> {
    let config = match config {
        Some(c) => c,
        None => return Ok("Configuration check: No configuration provided. Using defaults.".to_string()),
    };
    // Validate project root
    if config.project_root.trim().is_empty() {
        return Err(napi::Error::from_reason("Project root cannot be empty"));
    }

    // Check if project root exists
    if !std::path::Path::new(&config.project_root).exists() {
        return Err(napi::Error::from_reason(format!(
            "Project root does not exist: {}",
            config.project_root
        )));
    }

    // Validate languages if specified
    if let Some(languages) = &config.languages {
        let supported = get_supported_languages();
        for lang in languages {
            if !supported.contains(lang) {
                return Err(napi::Error::from_reason(format!(
                    "Unsupported language: {lang}"
                )));
            }
        }
    }

    // Validate cache policy if specified
    if let Some(cache_policy) = &config.cache_policy {
        let valid_policies = ["auto", "minimal", "balanced", "adaptive", "persistent"];
        if !valid_policies.contains(&cache_policy.as_str()) {
            return Err(napi::Error::from_reason(format!(
                "Invalid cache policy: {}. Valid options: {}",
                cache_policy,
                valid_policies.join(", ")
            )));
        }
    }

    // Validate max_files if specified
    if let Some(max_files) = config.max_files {
        if max_files > 100000 {
            return Err(napi::Error::from_reason(
                "max_files cannot exceed 100,000 for performance reasons"
            ));
        }
    }

    Ok("Configuration is valid".to_string())
}

/// Get system information
#[napi]
pub fn get_system_info() -> String {
    format!(
        r#"Fast-Context System Information:
Version: {}
Supported Languages: {}
Platform: {}
Architecture: {}
Rust Version: {}
"#,
        get_version(),
        get_supported_languages().len(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        "1.70.0" // Static version for compatibility
    )
}

/// Validate file path for analysis
pub fn validate_file_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("File path cannot be empty".to_string());
    }

    let path_obj = std::path::Path::new(path);
    
    if !path_obj.exists() {
        return Err(format!("File does not exist: {path}"));
    }

    if !path_obj.is_file() {
        return Err(format!("Path is not a file: {path}"));
    }

    // Check file size (limit to 10MB for performance)
    if let Ok(metadata) = path_obj.metadata() {
        if metadata.len() > 10 * 1024 * 1024 {
            return Err(format!("File too large (>10MB): {path}"));
        }
    }

    Ok(())
}

/// Validate directory path for analysis
pub fn validate_directory_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Directory path cannot be empty".to_string());
    }

    let path_obj = std::path::Path::new(path);
    
    if !path_obj.exists() {
        return Err(format!("Directory does not exist: {path}"));
    }

    if !path_obj.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }

    Ok(())
}

/// Get file extension from path
pub fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if file should be ignored based on patterns
pub fn should_ignore_file(path: &str, ignore_patterns: &[String]) -> bool {
    for pattern in ignore_patterns {
        if path.contains(pattern) {
            return true;
        }
    }
    false
}

/// Format file size in human readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
