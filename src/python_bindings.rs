//! # Python Bindings for Fast-Context - Phase 1: Simple Functions
//!
//! This module provides Python bindings using PyO3 with a simple, stateless API
//! that works immediately without complex thread safety requirements.

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use std::collections::HashMap;

#[cfg(feature = "python")]
use std::fs;

#[cfg(feature = "python")]
use walkdir::WalkDir;

/// Simple analysis result for Python
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct AnalysisResult {
    #[pyo3(get)]
    pub file_count: u32,

    #[pyo3(get)]
    pub symbol_count: u32,

    #[pyo3(get)]
    pub languages: Vec<String>,

    #[pyo3(get)]
    pub duration_ms: u32,
}

/// Phase 1: Simple stateless analysis function
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (project_root, languages=None, ignore_patterns=None))]
pub fn analyze_project(
    project_root: String,
    languages: Option<Vec<String>>,
    ignore_patterns: Option<Vec<String>>,
) -> PyResult<AnalysisResult> {
    let start_time = std::time::Instant::now();

    let supported_languages = languages.unwrap_or_else(|| vec![
        "rust".to_string(),
        "javascript".to_string(),
        "typescript".to_string(),
        "python".to_string(),
    ]);

    let ignore_patterns = ignore_patterns.unwrap_or_else(|| vec![
        "node_modules/**".to_string(),
        "target/**".to_string(),
        ".git/**".to_string(),
    ]);

    let mut file_count = 0;
    let mut symbol_count = 0;
    let mut detected_languages = std::collections::HashSet::new();

    // Walk through project files
    for entry in WalkDir::new(&project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                // Skip ignored patterns
                if should_ignore_file(path_str, &ignore_patterns) {
                    continue;
                }

                if let Some(language) = crate::utils::detect_language(path_str.to_string()) {
                    if supported_languages.iter().any(|l| language.to_lowercase().contains(&l.to_lowercase())) {
                        file_count += 1;
                        detected_languages.insert(language.clone());

                        // Count symbols by reading file content
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            symbol_count += count_symbols_in_content(&content, &language);
                        }
                    }
                }
            }
        }
    }

    let duration = start_time.elapsed();

    Ok(AnalysisResult {
        file_count,
        symbol_count,
        languages: detected_languages.into_iter().collect(),
        duration_ms: duration.as_millis() as u32,
    })
}

/// Find symbols by kind in a project
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (project_root, symbol_kind, languages=None))]
pub fn find_symbols_by_kind(
    project_root: String,
    symbol_kind: String,
    languages: Option<Vec<String>>,
) -> PyResult<Vec<String>> {
    let supported_languages = languages.unwrap_or_else(|| vec![
        "rust".to_string(),
        "javascript".to_string(),
        "python".to_string(),
    ]);

    let mut symbols = Vec::new();

    for entry in WalkDir::new(&project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                if let Some(language) = crate::utils::detect_language(path_str.to_string()) {
                    if supported_languages.iter().any(|l| language.to_lowercase().contains(&l.to_lowercase())) {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            let file_symbols = extract_symbols_by_kind(&content, &language, &symbol_kind);
                            symbols.extend(file_symbols);
                        }
                    }
                }
            }
        }
    }

    Ok(symbols)
}

/// Find symbols in a specific file
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_symbols_in_file(file_path: String) -> PyResult<Vec<String>> {
    if !std::path::Path::new(&file_path).exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("File not found: {file_path}")
        ));
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
            format!("Failed to read file: {e}")
        ))?;

    if let Some(language) = crate::utils::detect_language(file_path) {
        Ok(extract_all_symbols(&content, &language))
    } else {
        Ok(vec!["Unknown file type".to_string()])
    }
}

/// Find dependencies of a symbol in a project
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_dependencies(project_root: String, symbol_name: String) -> PyResult<Vec<String>> {
    let mut dependencies = Vec::new();

    for entry in WalkDir::new(&project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                if let Some(_language) = crate::utils::detect_language(path_str.to_string()) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        // Look for imports, includes, or references
                        if content.contains(&symbol_name) {
                            let file_name = entry.path().file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            dependencies.push(format!("{file_name}:{symbol_name}"));
                        }
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

/// Find complex symbols in a project
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_complex_symbols(project_root: String, threshold: u32) -> PyResult<Vec<String>> {
    let mut complex_symbols = Vec::new();

    for entry in WalkDir::new(&project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                if let Some(language) = crate::utils::detect_language(path_str.to_string()) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let symbols = find_complex_symbols_in_content(&content, &language, threshold);
                        complex_symbols.extend(symbols);
                    }
                }
            }
        }
    }

    Ok(complex_symbols)
}

/// Utility functions for Python
#[cfg(feature = "python")]
#[pyfunction]
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
    ]
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn detect_language(file_path: String) -> Option<String> {
    crate::utils::detect_language(file_path)
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn get_version() -> String {
    crate::utils::get_version()
}

/// Python module definition
#[cfg(feature = "python")]
#[pymodule]
fn fast_context(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AnalysisResult>()?;
    m.add_function(wrap_pyfunction!(analyze_project, m)?)?;
    m.add_function(wrap_pyfunction!(find_symbols_by_kind, m)?)?;
    m.add_function(wrap_pyfunction!(find_symbols_in_file, m)?)?;
    m.add_function(wrap_pyfunction!(find_dependencies, m)?)?;
    m.add_function(wrap_pyfunction!(find_complex_symbols, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_languages, m)?)?;
    m.add_function(wrap_pyfunction!(detect_language, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    Ok(())
}

#[cfg(feature = "python")]
fn should_ignore_file(path: &str, ignore_patterns: &[String]) -> bool {
    for pattern in ignore_patterns {
        if pattern.ends_with('/') {
            if path.contains(pattern) {
                return true;
            }
        } else if let Some(ext) = pattern.strip_prefix("*.") {
            if path.ends_with(ext) {
                return true;
            }
        } else if path.contains(pattern) {
            return true;
        }
    }
    false
}

#[cfg(feature = "python")]
fn count_symbols_in_content(content: &str, language: &str) -> u32 {
    let mut count = 0;
    match language {
        "Rust" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ")
                {
                    count += 1;
                }
            }
        },
        "JavaScript" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("function ") || trimmed.starts_with("class ") {
                    count += 1;
                }
            }
        },
        "Python" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                    count += 1;
                }
            }
        },
        _ => {}
    }
    count
}

#[cfg(feature = "python")]
fn extract_symbols_by_kind(content: &str, language: &str, kind: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let kind = kind.to_lowercase();

    match (language, kind.as_str()) {
        ("Rust", "function") => {
            for line in content.lines() {
                if line.trim().starts_with("fn ") || line.trim().starts_with("pub fn ") {
                    if let Some(name) = extract_function_name(line, "fn") {
                        symbols.push(name);
                    }
                }
            }
        },
        ("Rust", "struct") => {
            for line in content.lines() {
                if line.trim().starts_with("struct ") || line.trim().starts_with("pub struct ") {
                    if let Some(name) = extract_type_name(line, "struct") {
                        symbols.push(name);
                    }
                }
            }
        },
        ("JavaScript", "function") => {
            for line in content.lines() {
                if line.contains("function ") {
                    if let Some(name) = extract_function_name(line, "function") {
                        symbols.push(name);
                    }
                }
            }
        },
        ("JavaScript", "class") => {
            for line in content.lines() {
                if line.trim().starts_with("class ") {
                    if let Some(name) = extract_type_name(line, "class") {
                        symbols.push(name);
                    }
                }
            }
        },
        ("Python", "function") => {
            for line in content.lines() {
                if line.trim().starts_with("def ") {
                    if let Some(name) = extract_function_name(line, "def") {
                        symbols.push(name);
                    }
                }
            }
        },
        ("Python", "class") => {
            for line in content.lines() {
                if line.trim().starts_with("class ") {
                    if let Some(name) = extract_type_name(line, "class") {
                        symbols.push(name);
                    }
                }
            }
        },
        _ => {}
    }

    symbols
}

#[cfg(feature = "python")]
fn extract_all_symbols(content: &str, language: &str) -> Vec<String> {
    let mut symbols = Vec::new();

    match language {
        "Rust" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                    if let Some(name) = extract_function_name(line, "fn") {
                        symbols.push(format!("function: {}", name));
                    }
                } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                    if let Some(name) = extract_type_name(line, "struct") {
                        symbols.push(format!("struct: {}", name));
                    }
                } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
                    if let Some(name) = extract_type_name(line, "enum") {
                        symbols.push(format!("enum: {}", name));
                    }
                }
            }
        },
        "JavaScript" => {
            for line in content.lines() {
                if line.contains("function ") {
                    if let Some(name) = extract_function_name(line, "function") {
                        symbols.push(format!("function: {}", name));
                    }
                } else if line.trim().starts_with("class ") {
                    if let Some(name) = extract_type_name(line, "class") {
                        symbols.push(format!("class: {}", name));
                    }
                }
            }
        },
        "Python" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("def ") {
                    if let Some(name) = extract_function_name(line, "def") {
                        symbols.push(format!("function: {}", name));
                    }
                } else if trimmed.starts_with("class ") {
                    if let Some(name) = extract_type_name(line, "class") {
                        symbols.push(format!("class: {}", name));
                    }
                }
            }
        },
        _ => {}
    }

    symbols
}

#[cfg(feature = "python")]
fn find_complex_symbols_in_content(content: &str, language: &str, threshold: u32) -> Vec<String> {
    let mut complex_symbols = Vec::new();

    match language {
        "Rust" | "JavaScript" | "Python" => {
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;

            while i < lines.len() {
                let line = lines[i].trim();

                // Look for function definitions
                if line.contains("fn ") || line.contains("function ") || line.contains("def ") {
                    if let Some(name) = extract_function_name(lines[i], "") {
                        // Calculate complexity by counting control flow statements
                        let complexity = calculate_function_complexity(&lines, i);
                        if complexity >= threshold {
                            complex_symbols.push(format!("{} (complexity: {})", name, complexity));
                        }
                    }
                }
                i += 1;
            }
        },
        _ => {}
    }

    complex_symbols
}

#[cfg(feature = "python")]
fn extract_function_name(line: &str, keyword: &str) -> Option<String> {
    let line = line.trim();

    if keyword.is_empty() {
        if line.contains("fn ") {
            return extract_function_name(line, "fn");
        } else if line.contains("function ") {
            return extract_function_name(line, "function");
        } else if line.contains("def ") {
            return extract_function_name(line, "def");
        }
    }

    if let Some(start) = line.find(keyword) {
        let after_keyword = &line[start + keyword.len()..].trim();
        if let Some(paren_pos) = after_keyword.find('(') {
            let name = after_keyword[..paren_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        } else if let Some(space_pos) = after_keyword.find(' ') {
            let name = after_keyword[..space_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

#[cfg(feature = "python")]
fn extract_type_name(line: &str, keyword: &str) -> Option<String> {
    let line = line.trim();

    if let Some(start) = line.find(keyword) {
        let after_keyword = &line[start + keyword.len()..].trim();
        if let Some(space_pos) = after_keyword.find(' ') {
            let name = after_keyword[..space_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        } else if let Some(brace_pos) = after_keyword.find('{') {
            let name = after_keyword[..brace_pos].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

#[cfg(feature = "python")]
fn calculate_function_complexity(lines: &[&str], start_index: usize) -> u32 {
    let mut complexity = 1; // Base complexity
    let mut brace_count = 0;
    let mut in_function = false;

    for i in start_index..lines.len() {
        let line = lines[i].trim();

        if line.contains('{') {
            brace_count += line.matches('{').count() as i32;
            in_function = true;
        }
        if line.contains('}') {
            brace_count -= line.matches('}').count() as i32;
            if brace_count <= 0 && in_function {
                break; // End of function
            }
        }

        if in_function {
            if line.contains("if ") || line.contains("else if ") {
                complexity += 1;
            }
            if line.contains("for ") || line.contains("while ") || line.contains("loop ") {
                complexity += 1;
            }
            if line.contains("match ") || line.contains("switch ") {
                complexity += 1;
            }
            if line.contains("catch ") || line.contains("except ") {
                complexity += 1;
            }
            complexity += line.matches("case ").count() as u32;
            complexity += line.matches("=>").count() as u32; // Rust match arms
        }
    }

    complexity
}
