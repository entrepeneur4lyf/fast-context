//! # Utility Functions Module
//!
//! This module contains utility functions extracted from the monolithic lib.rs

#[cfg(feature = "nodejs")]
use crate::analyzer::AnalyzerConfig;
#[cfg(feature = "nodejs")]
use napi_derive::napi;

/// Get the version of the fast-context package
#[cfg(feature = "nodejs")]
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get list of supported programming languages
#[cfg(feature = "nodejs")]
#[napi]
pub fn get_supported_languages() -> Vec<String> {
    // Use the actual LanguageId enum to ensure consistency
    vec![
        LanguageId::Rust.to_string(),
        LanguageId::JavaScript.to_string(),
        LanguageId::TypeScript.to_string(),
        LanguageId::Python.to_string(),
        LanguageId::Java.to_string(),
        LanguageId::Go.to_string(),
        LanguageId::Cpp.to_string(),
        LanguageId::CSharp.to_string(),
        LanguageId::Swift.to_string(),
        LanguageId::ObjectiveC.to_string(),
        LanguageId::PHP.to_string(),
        LanguageId::Ruby.to_string(),
        LanguageId::Scala.to_string(),
        LanguageId::Zig.to_string(),
        LanguageId::Dart.to_string(),
        LanguageId::Lua.to_string(),
        LanguageId::Bash.to_string(),
        LanguageId::CSS.to_string(),
        LanguageId::HTML.to_string(),
        LanguageId::XML.to_string(),
        LanguageId::JSON.to_string(),
        LanguageId::YAML.to_string(),
        LanguageId::Markdown.to_string(),
        LanguageId::JSDoc.to_string(),
        LanguageId::Regex.to_string(),
    ]
}
use crate::errors::{FastContextError, FastContextResult};
use crate::parsers::LanguageId;

/// Internal: Detect language as enum for internal use
pub fn detect_language_id(file_path: &str) -> Option<LanguageId> {
    let path = std::path::Path::new(file_path);
    // Handle special files without extensions
    if let Some("Dockerfile" | "Makefile") = path.file_name().and_then(|n| n.to_str()) {
        return None; // These aren't in LanguageId enum
    }
    let ext = path.extension()?.to_str()?;
    LanguageId::from_extension(ext)
}

/// Detect the programming language of a file based on its extension
#[cfg(feature = "nodejs")]
#[napi]
pub fn detect_language(file_path: String) -> Option<String> {
    // Use the internal LanguageId detection and convert to string
    detect_language_id(&file_path).map(|lang_id| lang_id.to_string())
}

/// Check if the analyzer configuration is valid
#[cfg(feature = "nodejs")]
#[napi]
pub fn check_configuration(config: Option<AnalyzerConfig>) -> napi::Result<String> {
    let config = match config {
        Some(c) => c,
        None => {
            return Ok(
                "Configuration check: No configuration provided. Using defaults.".to_string(),
            )
        }
    };
    // Validate project root
    if config.project_root.trim().is_empty() {
        return Err(napi::Error::from_reason(
            "Configuration validation failed: project root cannot be empty",
        ));
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
        for lang in languages {
            // Use the actual LanguageId::from_string to validate
            if LanguageId::from_string(lang).is_none() {
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
                "max_files cannot exceed 100,000 for performance reasons",
            ));
        }
    }

    Ok("Configuration is valid".to_string())
}

/// Get system information
#[cfg(feature = "nodejs")]
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
pub fn validate_file_path(path: &str) -> FastContextResult<()> {
    if path.is_empty() {
        return Err(FastContextError::Validation {
            field: "file_path".to_string(),
            message: "File path cannot be empty".to_string(),
            value: None,
        });
    }

    let path_obj = std::path::Path::new(path);

    if !path_obj.exists() {
        return Err(FastContextError::FileNotFound {
            path: path_obj.to_path_buf(),
        });
    }

    if !path_obj.is_file() {
        return Err(FastContextError::Validation {
            field: "file_path".to_string(),
            message: "Path is not a file".to_string(),
            value: Some(path.to_string()),
        });
    }

    // Check file size (limit to 10MB for performance)
    if let Ok(metadata) = path_obj.metadata() {
        if metadata.len() > 10 * 1024 * 1024 {
            return Err(FastContextError::ResourceLimit {
                resource: "file_size".to_string(),
                message: "File too large (>10MB)".to_string(),
                current: Some(metadata.len() as usize),
                limit: Some(10 * 1024 * 1024),
            });
        }
    }

    Ok(())
}

/// Validate directory path for analysis
pub fn validate_directory_path(path: &str) -> FastContextResult<()> {
    if path.is_empty() {
        return Err(FastContextError::Validation {
            field: "directory_path".to_string(),
            message: "Directory path cannot be empty".to_string(),
            value: None,
        });
    }

    let path_obj = std::path::Path::new(path);

    if !path_obj.exists() {
        return Err(FastContextError::FileNotFound {
            path: path_obj.to_path_buf(),
        });
    }

    if !path_obj.is_dir() {
        return Err(FastContextError::Validation {
            field: "directory_path".to_string(),
            message: "Path is not a directory".to_string(),
            value: Some(path.to_string()),
        });
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

/// Check if file should be ignored based on glob patterns (e.g., "node_modules/**", ".git/**")
pub fn should_ignore_file(path: &str, ignore_patterns: &[String]) -> bool {
    if ignore_patterns.is_empty() {
        return false;
    }

    let normalized_path = path.replace('\\', "/");

    // Build a GlobSet for the provided patterns. On invalid patterns, fall back to non-ignore.
    let mut builder = globset::GlobSetBuilder::new();
    let mut added = false;
    for pat in ignore_patterns {
        let normalized_pattern = pat.replace('\\', "/");
        let candidate_patterns = if normalized_pattern.starts_with("**/")
            || normalized_pattern.starts_with('/')
            || normalized_pattern.contains(':')
        {
            vec![normalized_pattern]
        } else {
            vec![
                normalized_pattern.clone(),
                format!("**/{}", normalized_pattern),
            ]
        };

        for candidate in candidate_patterns {
            if let Ok(glob) = globset::Glob::new(&candidate) {
                builder.add(glob);
                added = true;
            }
        }
    }
    if !added {
        return false;
    }

    match builder.build() {
        Ok(set) => set.is_match(&normalized_path),
        Err(_) => false,
    }
}

/// Check if file should be ignored using default patterns (for testing)
pub fn should_ignore_file_default(path: &str) -> bool {
    let default_patterns = vec![
        "node_modules/**".to_string(),
        ".git/**".to_string(),
        "target/**".to_string(),
        "dist/**".to_string(),
        "coverage/**".to_string(),
        ".nyc_output/**".to_string(),
    ];
    should_ignore_file(path, &default_patterns)
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "nodejs")]
    use super::detect_language;
    use super::{detect_language_id, should_ignore_file};

    #[test]
    fn test_should_ignore_file_basic_globs() {
        let patterns = vec![
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/target/**".to_string(),
        ];
        assert!(should_ignore_file(
            "project/node_modules/lodash/index.js",
            &patterns
        ));
        assert!(should_ignore_file("project/.git/objects/abc", &patterns));
        assert!(should_ignore_file(
            "project/target/debug/fast-context",
            &patterns
        ));
        assert!(!should_ignore_file("src/main.rs", &patterns));
        assert!(!should_ignore_file("README.md", &patterns));
    }

    #[test]
    fn test_should_ignore_file_invalid_patterns() {
        let patterns = vec!["[invalid[".to_string()];
        assert!(!should_ignore_file("src/lib.rs", &patterns));
    }

    #[test]
    fn test_language_detection_consistency() {
        // Test that language detection is now consistent between string and ID detection
        let test_files = vec![
            ("test.rs", Some("rust")),
            ("test.js", Some("javascript")),
            ("test.py", Some("python")),
            ("test.cs", Some("csharp")),
            ("test.cpp", Some("cpp")),
            ("test.html", Some("html")),
            ("test.css", Some("css")),
            ("test.xml", Some("xml")),
        ];

        for (file_path, expected) in test_files {
            #[cfg(feature = "nodejs")]
            let detected_string = detect_language(file_path.to_string());
            let detected_id = detect_language_id(file_path);

            match expected {
                Some(expected_lang) => {
                    let detected_id = detected_id
                        .unwrap_or_else(|| panic!("ID detection failed for {}", file_path));
                    #[cfg(feature = "nodejs")]
                    assert_eq!(
                        detected_string,
                        Some(detected_id.to_string()),
                        "String detection failed for {}",
                        file_path
                    );
                    assert_eq!(
                        detected_id.to_lowercase_string(),
                        expected_lang,
                        "ID->string conversion failed for {}",
                        file_path
                    );
                }
                None => {
                    #[cfg(feature = "nodejs")]
                    assert!(
                        detected_string.is_none(),
                        "Should not detect language for {}",
                        file_path
                    );
                    assert!(
                        detected_id.is_none(),
                        "Should not detect language ID for {}",
                        file_path
                    );
                }
            }
        }
    }

    #[test]
    fn test_special_files() {
        // Test special files that don't have extensions
        // Note: These return None for LanguageId since they're not in the enum
        assert!(detect_language_id("Dockerfile").is_none());
        assert!(detect_language_id("Makefile").is_none());
    }

    #[test]
    fn test_unknown_extensions() {
        // Test unknown file extensions
        #[cfg(feature = "nodejs")]
        assert!(detect_language("test.unknown".to_string()).is_none());
        assert!(detect_language_id("test.unknown").is_none());
        #[cfg(feature = "nodejs")]
        assert!(detect_language("file_without_extension".to_string()).is_none());
        assert!(detect_language_id("file_without_extension").is_none());
    }
}
