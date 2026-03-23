//! Common utilities shared by all symbol extractors
//!
//! This module contains shared functionality to eliminate code duplication
//! across language-specific extractors.

use tree_sitter::Node;

/// Safe text extraction from tree-sitter nodes with bounds checking
///
/// This function safely extracts text from a tree-sitter node, ensuring that
/// the byte range is valid within the source string bounds. This prevents
/// panics that could occur from invalid byte access.
pub fn safe_node_text(node: &Node, source: &str) -> String {
    let start = node.start_byte();
    let end = node.end_byte();

    // Ensure byte range is within source bounds
    if start <= end && end <= source.len() {
        // Use direct slice access with bounds checking
        if let Some(slice) = source.get(start..end) {
            return slice.to_string();
        }
    }

    // Return empty string if bounds are invalid
    String::new()
}

/// Clean string literals by removing surrounding quotes
///
/// Removes single or double quotes from the beginning and end of string literals.
/// If the string doesn't have matching quotes, returns the original string unchanged.
pub fn clean_string_literal(text: &str) -> String {
    // Remove quotes from string literals
    if (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
    {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

/// Extract documentation from a previous sibling node
///
/// This is a common pattern for extracting documentation comments that
/// appear immediately before the symbol being documented.
pub fn extract_documentation_from_prev_sibling(node: &Node, source: &str) -> Option<String> {
    let mut prev = node.prev_sibling();

    while let Some(prev_node) = prev {
        if prev_node.kind().contains("comment") {
            if let Ok(comment_text) = prev_node.utf8_text(source.as_bytes()) {
                return Some(comment_text.trim().to_string());
            }
        }
        prev = prev_node.prev_sibling();
    }

    None
}

/// Check if a node represents a documentation comment
///
/// Determines if a node is a comment that should be treated as documentation
/// based on common comment patterns across languages.
pub fn is_documentation_comment(node: &Node) -> bool {
    let kind = node.kind();
    kind.contains("doc") || kind == "comment" || kind == "line_comment" || kind == "block_comment"
}

/// Extract text using tree-sitter's utf8_text method with error handling
///
/// A safer alternative to direct byte range access that handles potential
/// UTF-8 conversion errors gracefully.
pub fn utf8_text_with_fallback(node: &Node, source: &str) -> String {
    node.utf8_text(source.as_bytes())
        .map(|s| s.to_string())
        .unwrap_or_else(|_| safe_node_text(node, source))
}

/// Create a default symbol location from a node with file path
///
/// Generates a standard location object with file path, line and column information
/// extracted from the node's position.
pub fn create_location_from_node(node: &Node, file_path: &str) -> crate::symbols::Location {
    crate::symbols::Location::from_node(node, file_path)
}

/// Trait providing common utility methods for all extractors
///
/// This trait can be implemented by individual extractors to gain access
/// to shared utility functions while maintaining language-specific logic.
pub trait ExtractorUtils {
    /// Get text from a node using the safe extraction method
    fn get_node_text(&self, node: &Node, source: &str) -> String {
        safe_node_text(node, source)
    }

    /// Clean a string literal by removing quotes
    fn clean_string(&self, text: &str) -> String {
        clean_string_literal(text)
    }

    /// Extract documentation for a node
    fn extract_documentation(&self, node: &Node, source: &str) -> Option<String> {
        extract_documentation_from_prev_sibling(node, source)
    }

    /// Create a symbol location from a node
    fn create_location(&self, node: &Node, file_path: &str) -> crate::symbols::Location {
        create_location_from_node(node, file_path)
    }

    /// Check if a node is a documentation comment
    fn is_doc_comment(&self, node: &Node) -> bool {
        is_documentation_comment(node)
    }
}

/// Default implementation of ExtractorUtils for any type
impl<T> ExtractorUtils for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_literal() {
        // Double quotes
        assert_eq!(clean_string_literal("\"hello\""), "hello");

        // Single quotes
        assert_eq!(clean_string_literal("'hello'"), "hello");

        // No quotes
        assert_eq!(clean_string_literal("hello"), "hello");

        // Mismatched quotes
        assert_eq!(clean_string_literal("\"hello'"), "\"hello'");

        // Empty string
        assert_eq!(clean_string_literal("\"\""), "");
    }
}
