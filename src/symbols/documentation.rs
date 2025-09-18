//! # Enhanced Documentation Analysis
//!
//! Provides semantic analysis of code documentation including parameter extraction,
//! return type analysis, example detection, and cross-reference resolution.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced documentation information with semantic analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentationInfo {
    /// Raw documentation text
    pub raw_text: String,
    /// Cleaned and formatted description
    pub description: String,
    /// Extracted parameters with descriptions
    pub parameters: Vec<ParameterDoc>,
    /// Return value description
    pub returns: Option<String>,
    /// Extracted examples
    pub examples: Vec<CodeExample>,
    /// Cross-references to other symbols
    pub references: Vec<String>,
    /// Extracted tags and annotations
    pub tags: HashMap<String, String>,
    /// Detected issues or warnings in documentation
    pub issues: Vec<DocIssue>,
    /// Confidence score for documentation quality (0 to 100)
    pub quality_score: u8,
}

/// Parameter documentation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterDoc {
    pub name: String,
    pub type_info: Option<String>,
    pub description: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Code example extracted from documentation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeExample {
    pub language: Option<String>,
    pub code: String,
    pub description: Option<String>,
}

/// Documentation issue or warning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocIssue {
    pub severity: DocIssueSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Severity levels for documentation issues
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocIssueSeverity {
    Info,
    Warning,
    Error,
}

/// Enhanced documentation analyzer
pub struct DocumentationAnalyzer {
    // Regex patterns for different documentation formats
    param_patterns: HashMap<String, Regex>,
    return_patterns: HashMap<String, Regex>,
    example_patterns: HashMap<String, Regex>,
    reference_patterns: HashMap<String, Regex>,
}

impl DocumentationAnalyzer {
    /// Create a new documentation analyzer
    pub fn new() -> Self {
        let mut param_patterns = HashMap::new();
        let mut return_patterns = HashMap::new();
        let mut example_patterns = HashMap::new();
        let mut reference_patterns = HashMap::new();

        // JSDoc patterns
        param_patterns.insert(
            "jsdoc".to_string(),
            Regex::new(r"@param\s+\{([^}]+)\}\s+(\w+)\s+(.+)").unwrap(),
        );
        return_patterns.insert(
            "jsdoc".to_string(),
            Regex::new(r"@returns?\s+\{([^}]+)\}\s+(.+)").unwrap(),
        );
        example_patterns.insert(
            "jsdoc".to_string(),
            Regex::new(r"@example\s*\n((?:[^@]+\n?)*)").unwrap(),
        );

        // Javadoc patterns
        param_patterns.insert(
            "javadoc".to_string(),
            Regex::new(r"@param\s+(\w+)\s+(.+)").unwrap(),
        );
        return_patterns.insert(
            "javadoc".to_string(),
            Regex::new(r"@return\s+(.+)").unwrap(),
        );

        // Python docstring patterns (Google style)
        param_patterns.insert(
            "python".to_string(),
            Regex::new(r"(\w+)\s*\(([^)]+)\):\s*(.+)").unwrap(),
        );
        return_patterns.insert(
            "python".to_string(),
            Regex::new(r"Returns:\s*(.+)").unwrap(),
        );

        // Rust doc patterns
        param_patterns.insert(
            "rust".to_string(),
            Regex::new(r"#\s*(\w+)\s*-\s*(.+)").unwrap(),
        );
        return_patterns.insert(
            "rust".to_string(),
            Regex::new(r"Returns?\s*:?\s*(.+)").unwrap(),
        );

        // C# XML doc patterns
        param_patterns.insert(
            "csharp".to_string(),
            Regex::new(r#"<param name="(\w+)">(.+?)</param>"#).unwrap(),
        );
        return_patterns.insert(
            "csharp".to_string(),
            Regex::new(r"<returns>(.+?)</returns>").unwrap(),
        );

        // Reference patterns (common across languages)
        reference_patterns.insert("see".to_string(), Regex::new(r"@see\s+([^\s]+)").unwrap());
        reference_patterns.insert(
            "link".to_string(),
            Regex::new(r"\{@link\s+([^}]+)\}").unwrap(),
        );
        reference_patterns.insert(
            "ref".to_string(),
            Regex::new(r#"<see cref="([^"]+)"/>"#).unwrap(),
        );

        Self {
            param_patterns,
            return_patterns,
            example_patterns,
            reference_patterns,
        }
    }

    /// Analyze documentation text and extract semantic information
    pub fn analyze(&self, raw_text: &str, doc_format: &str) -> DocumentationInfo {
        let cleaned_text = self.clean_documentation(raw_text, doc_format);
        let description = self.extract_description(&cleaned_text);
        let parameters = self.extract_parameters(&cleaned_text, doc_format);
        let returns = self.extract_return_info(&cleaned_text, doc_format);
        let examples = self.extract_examples(&cleaned_text, doc_format);
        let references = self.extract_references(&cleaned_text);
        let tags = self.extract_tags(&cleaned_text, doc_format);
        let issues = self.detect_issues(&cleaned_text, &parameters, &returns);
        let quality_score =
            self.calculate_quality_score(&description, &parameters, &returns, &examples);

        DocumentationInfo {
            raw_text: raw_text.to_string(),
            description,
            parameters,
            returns,
            examples,
            references,
            tags,
            issues,
            quality_score,
        }
    }

    /// Clean and normalize documentation text
    fn clean_documentation(&self, text: &str, format: &str) -> String {
        let mut cleaned = text.to_string();

        match format {
            "jsdoc" | "javadoc" => {
                // Remove /** */ wrapper
                cleaned = cleaned
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .to_string();
                // Remove leading * from each line
                cleaned = cleaned
                    .lines()
                    .map(|line| line.trim().trim_start_matches('*').trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            "python" => {
                // Remove triple quotes
                cleaned = cleaned
                    .trim_start_matches("\"\"\"")
                    .trim_end_matches("\"\"\"")
                    .to_string();
                cleaned = cleaned
                    .trim_start_matches("'''")
                    .trim_end_matches("'''")
                    .to_string();
            }
            "rust" => {
                // Remove /// prefix from each line
                cleaned = cleaned
                    .lines()
                    .map(|line| line.trim().trim_start_matches("///").trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            "csharp" => {
                // Remove /// prefix and clean XML
                cleaned = cleaned
                    .lines()
                    .map(|line| line.trim().trim_start_matches("///").trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            _ => {
                // Generic cleaning
                cleaned = cleaned
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }

        cleaned
    }

    /// Extract the main description from documentation
    fn extract_description(&self, text: &str) -> String {
        // Take everything before the first @ tag or special section
        let lines: Vec<&str> = text.lines().collect();
        let mut description_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('@')
                || trimmed.starts_with("Parameters:")
                || trimmed.starts_with("Returns:")
                || trimmed.starts_with("Args:")
                || trimmed.starts_with("Example")
            {
                break;
            }
            description_lines.push(trimmed);
        }

        description_lines.join(" ").trim().to_string()
    }

    /// Extract parameter information
    fn extract_parameters(&self, text: &str, format: &str) -> Vec<ParameterDoc> {
        let mut parameters = Vec::new();

        if let Some(pattern) = self.param_patterns.get(format) {
            for captures in pattern.captures_iter(text) {
                let param = match format {
                    "jsdoc" => ParameterDoc {
                        name: captures
                            .get(2)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        type_info: captures.get(1).map(|m| m.as_str().to_string()),
                        description: captures
                            .get(3)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        is_optional: captures.get(1).is_some_and(|m| m.as_str().contains("?")),
                        default_value: None,
                    },
                    "javadoc" | "rust" => ParameterDoc {
                        name: captures
                            .get(1)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        type_info: None,
                        description: captures
                            .get(2)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        is_optional: false,
                        default_value: None,
                    },
                    "python" => ParameterDoc {
                        name: captures
                            .get(1)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        type_info: captures.get(2).map(|m| m.as_str().to_string()),
                        description: captures
                            .get(3)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        is_optional: captures
                            .get(2)
                            .is_some_and(|m| m.as_str().contains("optional")),
                        default_value: None,
                    },
                    "csharp" => ParameterDoc {
                        name: captures
                            .get(1)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        type_info: None,
                        description: captures
                            .get(2)
                            .map_or("".to_string(), |m| m.as_str().to_string()),
                        is_optional: false,
                        default_value: None,
                    },
                    _ => continue,
                };
                parameters.push(param);
            }
        }

        parameters
    }

    /// Extract return information
    fn extract_return_info(&self, text: &str, format: &str) -> Option<String> {
        if let Some(pattern) = self.return_patterns.get(format) {
            if let Some(captures) = pattern.captures(text) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    /// Extract code examples
    fn extract_examples(&self, text: &str, format: &str) -> Vec<CodeExample> {
        let mut examples = Vec::new();

        if let Some(pattern) = self.example_patterns.get(format) {
            for captures in pattern.captures_iter(text) {
                if let Some(code_match) = captures.get(1) {
                    examples.push(CodeExample {
                        language: None,
                        code: code_match.as_str().trim().to_string(),
                        description: None,
                    });
                }
            }
        }

        // Also look for code blocks
        let code_block_pattern = Regex::new(r"```(\w+)?\n(.*?)\n```").unwrap();
        for captures in code_block_pattern.captures_iter(text) {
            examples.push(CodeExample {
                language: captures.get(1).map(|m| m.as_str().to_string()),
                code: captures
                    .get(2)
                    .map_or("".to_string(), |m| m.as_str().to_string()),
                description: None,
            });
        }

        examples
    }

    /// Extract cross-references
    fn extract_references(&self, text: &str) -> Vec<String> {
        let mut references = Vec::new();

        for pattern in self.reference_patterns.values() {
            for captures in pattern.captures_iter(text) {
                if let Some(reference) = captures.get(1) {
                    references.push(reference.as_str().to_string());
                }
            }
        }

        references
    }

    /// Extract tags and annotations
    fn extract_tags(&self, text: &str, _format: &str) -> HashMap<String, String> {
        let mut tags = HashMap::new();

        // Extract common tags
        let tag_patterns = [
            ("author", r"@author\s+(.+)"),
            ("version", r"@version\s+(.+)"),
            ("since", r"@since\s+(.+)"),
            ("deprecated", r"@deprecated\s*(.*)"),
            ("todo", r"@todo\s+(.+)"),
            ("note", r"@note\s+(.+)"),
            ("warning", r"@warning\s+(.+)"),
        ];

        for (tag_name, pattern_str) in &tag_patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if let Some(captures) = pattern.captures(text) {
                    let value = captures
                        .get(1)
                        .map_or("".to_string(), |m| m.as_str().to_string());
                    tags.insert(tag_name.to_string(), value);
                }
            }
        }

        tags
    }

    /// Detect issues in documentation
    fn detect_issues(
        &self,
        text: &str,
        parameters: &[ParameterDoc],
        returns: &Option<String>,
    ) -> Vec<DocIssue> {
        let mut issues = Vec::new();

        // Check for empty documentation
        if text.trim().is_empty() {
            issues.push(DocIssue {
                severity: DocIssueSeverity::Warning,
                message: "Documentation is empty".to_string(),
                suggestion: Some("Add a description of what this symbol does".to_string()),
            });
        }

        // Check for very short descriptions
        if text.len() < 20 {
            issues.push(DocIssue {
                severity: DocIssueSeverity::Info,
                message: "Documentation is very brief".to_string(),
                suggestion: Some("Consider adding more detailed description".to_string()),
            });
        }

        // Check for missing parameter descriptions
        for param in parameters {
            if param.description.trim().is_empty() {
                issues.push(DocIssue {
                    severity: DocIssueSeverity::Warning,
                    message: format!("Parameter '{}' lacks description", param.name),
                    suggestion: Some("Add description for this parameter".to_string()),
                });
            }
        }

        // Check for missing return documentation
        if returns.is_none() && text.contains("return") {
            issues.push(DocIssue {
                severity: DocIssueSeverity::Info,
                message: "Return value not documented".to_string(),
                suggestion: Some(
                    "Add @returns or @return tag to document return value".to_string(),
                ),
            });
        }

        issues
    }

    /// Calculate documentation quality score
    fn calculate_quality_score(
        &self,
        description: &str,
        parameters: &[ParameterDoc],
        returns: &Option<String>,
        examples: &[CodeExample],
    ) -> u8 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Description quality (40% of total score)
        max_score += 40.0;
        if !description.is_empty() {
            score += 20.0; // Base score for having description
            if description.len() > 50 {
                score += 10.0; // Bonus for detailed description
            }
            if description.contains('.') {
                score += 5.0; // Bonus for proper sentences
            }
            if description.split_whitespace().count() > 10 {
                score += 5.0; // Bonus for comprehensive description
            }
        }

        // Parameter documentation (30% of total score)
        max_score += 30.0;
        if !parameters.is_empty() {
            let documented_params = parameters
                .iter()
                .filter(|p| !p.description.trim().is_empty())
                .count();
            score += (documented_params as f32 / parameters.len() as f32) * 30.0;
        } else {
            score += 30.0; // Full score if no parameters to document
        }

        // Return documentation (20% of total score)
        max_score += 20.0;
        if returns.is_some() {
            score += 20.0;
        }

        // Examples (10% of total score)
        max_score += 10.0;
        if !examples.is_empty() {
            score += 10.0;
        }

        ((score / max_score) * 100.0).min(100.0) as u8
    }
}

impl Default for DocumentationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
