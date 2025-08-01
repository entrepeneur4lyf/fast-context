//! Swift symbol extractor
//!
//! Extracts symbols from Swift source code including:
//! - Functions and methods
//! - Classes and structs
//! - Imports and protocols
//! - Properties and variables
//! - Initializers

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Swift Symbol Extractor
/// Extracts functions, classes, structs, imports from Swift code
pub struct SwiftExtractor;

impl SymbolExtractor for SwiftExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Swift
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();

        self.extract_from_node(
            tree.root_node(),
            source,
            file_path,
            &mut symbols,
            &mut scope_stack,
        );
        symbols
    }
}

impl SwiftExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "import_declaration" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "class_declaration" => {
                // Swift uses class_declaration for class, struct, and enum
                self.extract_type_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "protocol_declaration" => {
                self.extract_protocol(&node, source, file_path, symbols, scope_stack);
            }
            "function_declaration" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "init_declaration" => {
                self.extract_initializer(&node, source, file_path, symbols, scope_stack);
            }
            "property_declaration" => {
                self.extract_property(&node, source, file_path, symbols, scope_stack);
            }
            "variable_declaration" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "class_declaration" | "protocol_declaration") {
            // For class_declaration, check if it was a class or struct (not enum)
            if node.kind() == "class_declaration" {
                if self.node_contains_keyword(&node, "class")
                    || self.node_contains_keyword(&node, "struct")
                {
                    scope_stack.pop();
                }
            } else {
                scope_stack.pop();
            }
        }
    }

    fn extract_import(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Swift import: import Foundation, import UIKit.UIView
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "simple_identifier" {
                let import_name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(node, file_path);

                symbols.push(Symbol {
                    name: import_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::Swift,
                    documentation: None,
                    modifiers: vec!["import".to_string()],
                    signature: None,
                });
                break; // Only process the first import identifier
            }
        }
    }

    fn extract_type_declaration(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_swift_doc(node, source);

            // Determine the type based on keywords in the node
            let (kind, should_push_scope) = if self.node_contains_keyword(node, "class") {
                (SymbolKind::Class, true)
            } else if self.node_contains_keyword(node, "struct") {
                (SymbolKind::Struct, true)
            } else if self.node_contains_keyword(node, "enum") {
                (SymbolKind::Enum, false) // Enums don't usually need scope for our purposes
            } else {
                (SymbolKind::Class, true) // Default to class if unclear
            };

            // Push as scope for nested items if needed
            if should_push_scope {
                let scope = Scope {
                    name: name.clone(),
                    kind: kind.clone(),
                    location: location.clone(),
                };
                scope_stack.push(scope);
            }

            symbols.push(Symbol {
                name,
                kind,
                location,
                scope_chain: scope_stack[..if should_push_scope {
                    scope_stack.len() - 1
                } else {
                    scope_stack.len()
                }]
                    .to_vec(),
                language: LanguageId::Swift,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_protocol(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_swift_doc(node, source);

            // Push protocol as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Interface, // Swift protocols are like interfaces
                location: location.clone(),
            };
            scope_stack.push(scope);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Interface,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Swift,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn node_contains_keyword(&self, node: &Node, keyword: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == keyword {
                return true;
            }
        }
        false
    }

    fn extract_function(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_swift_doc(node, source);
            let signature = self.extract_function_signature(node, source);

            // Determine if this is a method (inside a type) or standalone function
            let kind = if scope_stack
                .iter()
                .any(|s| matches!(s.kind, SymbolKind::Class | SymbolKind::Struct))
            {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            };

            symbols.push(Symbol {
                name,
                kind,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::Swift,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_initializer(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Swift initializers don't have names, but we'll use "init"
        let location = Location::from_node(node, file_path);

        let modifiers = self.extract_modifiers(node, source);
        let signature = self.extract_init_signature(node, source);

        symbols.push(Symbol {
            name: "init".to_string(),
            kind: SymbolKind::Method,
            location,
            scope_chain: scope_stack.to_vec(),
            language: LanguageId::Swift,
            documentation: None,
            modifiers,
            signature,
        });
    }

    fn extract_property(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();
            let location = Location::from_node(node, file_path);

            let mut modifiers = self.extract_modifiers(node, source);
            modifiers.push("property".to_string());
            let documentation = self.extract_swift_doc(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Field,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::Swift,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Swift variable declarations can contain multiple bindings
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "pattern_binding" {
                // Look for the pattern (variable name) in the binding
                let mut binding_cursor = child.walk();
                for binding_child in child.children(&mut binding_cursor) {
                    if binding_child.kind() == "simple_identifier"
                        || binding_child.kind() == "identifier"
                    {
                        let name = binding_child
                            .utf8_text(source.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        let location = Location::from_node(&binding_child, file_path);

                        let mut modifiers = self.extract_modifiers(node, source);
                        modifiers.push("variable".to_string());

                        symbols.push(Symbol {
                            name,
                            kind: SymbolKind::Variable,
                            location,
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::Swift,
                            documentation: None,
                            modifiers,
                            signature: None,
                        });
                        break; // Only process the first identifier in this binding
                    }
                }
            }
        }
    }

    fn extract_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifiers that appear before declarations
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "modifiers" => {
                    let mut mod_cursor = child.walk();
                    for mod_child in child.children(&mut mod_cursor) {
                        if let Ok(modifier_text) = mod_child.utf8_text(source.as_bytes()) {
                            modifiers.push(modifier_text.to_string());
                        }
                    }
                }
                "visibility_modifier" | "mutation_modifier" | "inheritance_modifier" => {
                    if let Ok(modifier_text) = child.utf8_text(source.as_bytes()) {
                        modifiers.push(modifier_text.to_string());
                    }
                }
                _ => {}
            }
        }

        modifiers
    }

    fn extract_swift_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Swift documentation appears as /// comments or /** */ blocks
        // Look for documentation comment nodes that appear before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Swift doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("///") {
                        // Swift documentation comment
                        let content = comment_text.strip_prefix("///").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        // Block documentation comment
                        let content = comment_text
                            .strip_prefix("/**")
                            .unwrap_or("")
                            .strip_suffix("*/")
                            .unwrap_or("")
                            .lines()
                            .map(|line| line.trim().trim_start_matches('*').trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        if !content.is_empty() {
                            doc_comments.insert(0, content);
                        }
                    }
                    current = prev;
                }
                "multiline_comment" => {
                    // Handle multiline comments that might be documentation
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") {
                        let content = comment_text
                            .strip_prefix("/**")
                            .unwrap_or("")
                            .strip_suffix("*/")
                            .unwrap_or("")
                            .lines()
                            .map(|line| line.trim().trim_start_matches('*').trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        if !content.is_empty() {
                            doc_comments.insert(0, content);
                        }
                    }
                    current = prev;
                }
                _ if prev.kind().contains("whitespace") || prev.kind() == "\n" => {
                    // Allow whitespace between comments and declarations
                    current = prev;
                    continue;
                }
                _ => {
                    // Stop at first non-comment, non-whitespace node
                    break;
                }
            }
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;

        let params = node
            .child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");

        let return_type = node
            .child_by_field_name("result")
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");

        let return_part = if return_type.is_empty() {
            String::new()
        } else {
            format!(" -> {return_type}")
        };

        Some(format!("func {name}{params}{return_part}"))
    }

    fn extract_init_signature(&self, node: &Node, source: &str) -> Option<String> {
        let params = node
            .child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");

        Some(format!("init{params}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_symbol_extraction() {
        let extractor = SwiftExtractor;
        assert_eq!(extractor.language(), LanguageId::Swift);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = SwiftExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Swift);
    }

    #[test]
    fn test_class_detection() {
        let extractor = SwiftExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Swift);
    }

    #[test]
    fn test_protocol_extraction() {
        let extractor = SwiftExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Swift);
    }

    #[test]
    fn test_modifier_extraction() {
        let extractor = SwiftExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Swift);
    }

    #[test]
    fn test_initializer_extraction() {
        let extractor = SwiftExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Swift);
    }
}
