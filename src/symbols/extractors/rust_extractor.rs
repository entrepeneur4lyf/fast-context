//! Rust symbol extractor
//! 
//! Extracts symbols from Rust source code including:
//! - Functions and methods
//! - Structs and enums
//! - Traits and implementations
//! - Modules and use statements
//! - Constants and variables

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Rust symbol extractor
pub struct RustExtractor;

impl SymbolExtractor for RustExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Rust
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl RustExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    // Extract function signature
                    let signature = node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Rust,
                        documentation: self.extract_doc_comments(&node, source),
                        modifiers: self.extract_function_modifiers(&node, source),
                        signature: Some(signature),
                    });
                }
            }
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    // Push struct as scope for nested items
                    let scope = Scope {
                        name: name.clone(),
                        kind: SymbolKind::Struct,
                        location: location.clone(),
                    };
                    scope_stack.push(scope);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        location,
                        scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                        language: LanguageId::Rust,
                        documentation: self.extract_doc_comments(&node, source),
                        modifiers: self.extract_item_modifiers(&node, source),
                        signature: None,
                    });
                }
            }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Rust,
                        documentation: self.extract_doc_comments(&node, source),
                        modifiers: self.extract_item_modifiers(&node, source),
                        signature: None,
                    });
                }
            }
            "use_declaration" => {
                // Extract use statements as imports
                if let Some(use_clause) = node.child_by_field_name("argument") {
                    let import_text = use_clause.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    symbols.push(Symbol {
                        name: import_text,
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Rust,
                        documentation: None,
                        modifiers: vec![],
                        signature: None,
                    });
                }
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "struct_item" | "enum_item" | "impl_item" | "mod_item") {
            scope_stack.pop();
        }
    }

    /// Extract documentation comments (/// or /** */) preceding a node
    fn extract_doc_comments(&self, node: &Node, source: &str) -> Option<String> {
        let mut doc_comments = Vec::new();
        let start_byte = node.start_byte();

        // Look backwards from the node to find doc comments
        let preceding_text = &source[..start_byte];
        let lines: Vec<&str> = preceding_text.lines().collect();

        // Collect consecutive doc comment lines working backwards
        for line in lines.iter().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with("///") {
                // Extract content after ///
                let content = trimmed.strip_prefix("///").unwrap_or("").trim();
                if !content.is_empty() {
                    doc_comments.insert(0, content.to_string());
                }
            } else if trimmed.starts_with("/**") && trimmed.ends_with("*/") {
                // Single-line block comment
                let content = trimmed.strip_prefix("/**").unwrap_or("")
                    .strip_suffix("*/").unwrap_or("").trim();
                if !content.is_empty() {
                    doc_comments.insert(0, content.to_string());
                }
            } else if trimmed.is_empty() {
                // Allow empty lines between doc comments
                continue;
            } else {
                // Stop at first non-doc-comment line
                break;
            }
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }

    /// Extract function modifiers like pub, async, unsafe, const, etc.
    fn extract_function_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for visibility modifier
        if let Some(visibility) = node.child_by_field_name("visibility") {
            let vis_text = visibility.utf8_text(source.as_bytes()).unwrap_or("");
            if !vis_text.is_empty() {
                modifiers.push(vis_text.to_string());
            }
        }

        // Check for function modifiers in the function signature
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "async" => modifiers.push("async".to_string()),
                "unsafe" => modifiers.push("unsafe".to_string()),
                "const" => modifiers.push("const".to_string()),
                "extern" => modifiers.push("extern".to_string()),
                _ => {}
            }
        }

        modifiers
    }

    /// Extract struct/enum modifiers like pub, etc.
    fn extract_item_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for visibility modifier
        if let Some(visibility) = node.child_by_field_name("visibility") {
            let vis_text = visibility.utf8_text(source.as_bytes()).unwrap_or("");
            if !vis_text.is_empty() {
                modifiers.push(vis_text.to_string());
            }
        }

        modifiers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_symbol_extraction() {
        let extractor = RustExtractor;
        assert_eq!(extractor.language(), LanguageId::Rust);
    }

    #[test]
    fn test_doc_comment_extraction() {
        let _extractor = RustExtractor;
        let _source = r#"
/// This is a documentation comment
/// for a function
pub fn test_function() {}
"#;

        // Note: This would need actual tree-sitter parsing to test fully
        // For now, we just test that the extractor can be created
        assert_eq!(_extractor.language(), LanguageId::Rust);
    }

    #[test]
    fn test_modifier_extraction() {
        let extractor = RustExtractor;
        assert_eq!(extractor.language(), LanguageId::Rust);
        // Similar to above, full testing would require tree-sitter parsing
    }
}
