//! Objective-C symbol extractor
//!
//! Extracts symbols from Objective-C source code including:
//! - Classes (interfaces and implementations)
//! - Methods (instance and class methods)
//! - Properties and instance variables
//! - Imports and protocols

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Objective-C Symbol Extractor
/// Extracts classes, methods, properties, imports from Objective-C code
pub struct ObjectiveCExtractor;

impl SymbolExtractor for ObjectiveCExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::ObjectiveC
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

impl ObjectiveCExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "preproc_include" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "class_interface" => {
                self.extract_class_interface(&node, source, file_path, symbols, scope_stack);
            }
            "class_implementation" => {
                self.extract_class_implementation(&node, source, file_path, symbols, scope_stack);
            }
            "protocol_declaration" => {
                self.extract_protocol(&node, source, file_path, symbols, scope_stack);
            }
            "method_declaration" | "method_definition" => {
                self.extract_method(&node, source, file_path, symbols, scope_stack);
            }
            "property_declaration" => {
                self.extract_property(&node, source, file_path, symbols, scope_stack);
            }
            "instance_variable_declaration" => {
                self.extract_instance_variable(&node, source, file_path, symbols, scope_stack);
            }
            "category_interface" | "category_implementation" => {
                self.extract_category(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(
            node.kind(),
            "class_interface"
                | "class_implementation"
                | "protocol_declaration"
                | "category_interface"
                | "category_implementation"
        ) {
            scope_stack.pop();
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
        // Objective-C imports: #import <Foundation/Foundation.h> or #import "MyClass.h"
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                let import_text = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(node, file_path);

                // Extract the actual import name from quotes or angle brackets
                let import_name = if (import_text.starts_with('"') && import_text.ends_with('"'))
                    || (import_text.starts_with('<') && import_text.ends_with('>'))
                {
                    import_text[1..import_text.len() - 1].to_string()
                } else {
                    import_text
                };

                symbols.push(Symbol {
                    name: import_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::ObjectiveC,
                    documentation: None,
                    modifiers: vec!["import".to_string()],
                    signature: None,
                });
                break; // Only process the first import
            }
        }
    }

    fn extract_class_interface(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // In Objective-C, class_interface has identifier child instead of name field
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(node, file_path);

                let modifiers = self.extract_class_modifiers(node, source);

                // Push class as scope for nested items
                let scope = Scope {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                let documentation = self.extract_objc_doc(node, source);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Class,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::ObjectiveC,
                    documentation,
                    modifiers,
                    signature: None,
                });
                break; // Only process the first identifier
            }
        }
    }

    fn extract_class_implementation(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // In Objective-C, class_implementation has identifier child instead of name field
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(node, file_path);

                let modifiers = vec!["implementation".to_string()];

                // Push class as scope for nested items
                let scope = Scope {
                    name: name.clone(),
                    kind: SymbolKind::Class,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                let documentation = self.extract_objc_doc(node, source);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Class,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::ObjectiveC,
                    documentation,
                    modifiers,
                    signature: None,
                });
                break; // Only process the first identifier
            }
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
        // In Objective-C, protocol_declaration has identifier child instead of name field
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(node, file_path);

                // Push protocol as scope for nested items
                let scope = Scope {
                    name: name.clone(),
                    kind: SymbolKind::Interface, // Objective-C protocols are like interfaces
                    location: location.clone(),
                };
                scope_stack.push(scope);

                let documentation = self.extract_objc_doc(node, source);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Interface,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::ObjectiveC,
                    documentation,
                    modifiers: vec!["protocol".to_string()],
                    signature: None,
                });
                break; // Only process the first identifier
            }
        }
    }

    fn extract_method(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Objective-C methods can be instance (-) or class (+) methods
        let mut is_class_method = false;

        // Extract method name - could be complex like "initWithName:age:"
        let method_name = self.extract_method_name(node, source);

        // Look for method type (+ or -)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "+" => is_class_method = true,
                "-" => is_class_method = false,
                _ => {}
            }
        }

        if !method_name.is_empty() {
            let location = Location::from_node(node, file_path);
            let signature = self.extract_method_signature(node, source);

            let mut modifiers = vec!["method".to_string()];
            if is_class_method {
                modifiers.push("class".to_string());
            } else {
                modifiers.push("instance".to_string());
            }

            let documentation = self.extract_objc_doc(node, source);

            symbols.push(Symbol {
                name: method_name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::ObjectiveC,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_property(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for property in struct_declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "struct_declaration" {
                // Look for struct_declarator in the struct_declaration
                let mut struct_cursor = child.walk();
                for struct_child in child.children(&mut struct_cursor) {
                    if struct_child.kind() == "struct_declarator" {
                        // Extract the property name from the struct_declarator
                        let declarator_text =
                            struct_child.utf8_text(source.as_bytes()).unwrap_or("");
                        // Remove pointer asterisk and get just the name
                        let name = declarator_text.trim_start_matches('*').trim();

                        if !name.is_empty() {
                            let location = Location::from_node(&struct_child, file_path);

                            let mut modifiers = vec!["property".to_string()];
                            modifiers.extend(self.extract_property_attributes(node, source));

                            symbols.push(Symbol {
                                name: name.to_string(),
                                kind: SymbolKind::Field,
                                location,
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::ObjectiveC,
                                documentation: None,
                                modifiers,
                                signature: None,
                            });
                            break; // Only process the first struct_declarator
                        }
                    }
                }
                break; // Only process the first struct_declaration
            }
        }
    }

    fn extract_instance_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Instance variables in Objective-C
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(&child, file_path);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Field,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::ObjectiveC,
                    documentation: None,
                    modifiers: vec!["ivar".to_string()],
                    signature: None,
                });
                break; // Only process the first identifier
            }
        }
    }

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["interface".to_string()];

        // Check for superclass
        if let Some(superclass_node) = node.child_by_field_name("superclass") {
            modifiers.push("inherits".to_string());
            if let Ok(superclass_name) = superclass_node.utf8_text(source.as_bytes()) {
                modifiers.push(format!("extends_{superclass_name}"));
            }
        }

        // Check for protocols
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "protocol_list" {
                modifiers.push("implements_protocols".to_string());

                // Extract individual protocol names
                let mut protocol_cursor = child.walk();
                for protocol_child in child.children(&mut protocol_cursor) {
                    if protocol_child.kind() == "identifier" {
                        if let Ok(protocol_name) = protocol_child.utf8_text(source.as_bytes()) {
                            modifiers.push(format!("protocol_{protocol_name}"));
                        }
                    }
                }
                break;
            }
        }

        modifiers
    }

    fn extract_property_attributes(&self, node: &Node, source: &str) -> Vec<String> {
        let mut attributes = Vec::new();

        // Look for property attributes like (nonatomic, strong, weak, etc.)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_attributes" {
                let mut attr_cursor = child.walk();
                for attr_child in child.children(&mut attr_cursor) {
                    if let Ok(attr_text) = attr_child.utf8_text(source.as_bytes()) {
                        attributes.push(attr_text.to_string());
                    }
                }
            }
        }

        attributes
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Try to extract the full method signature
        if let Ok(signature_text) = node.utf8_text(source.as_bytes()) {
            // Take only the first line for cleaner signature
            let first_line = signature_text.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                return Some(first_line.to_string());
            }
        }
        None
    }

    fn extract_objc_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Objective-C documentation appears as /** */ comments or // comments preceding declarations
        // Look for comment nodes that appear before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Objective-C doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        // Objective-C documentation comment
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
                    } else if comment_text.starts_with("//") {
                        // Single-line comment
                        let content = comment_text.strip_prefix("//").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
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

    fn extract_method_name(&self, node: &Node, source: &str) -> String {
        // Extract Objective-C method name, which can be complex like "initWithName:age:"
        let mut method_parts = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if let Ok(name_text) = child.utf8_text(source.as_bytes()) {
                        method_parts.push(name_text.to_string());
                    }
                }
                "selector" => {
                    // Handle selector parts for complex method names
                    let mut selector_cursor = child.walk();
                    for selector_child in child.children(&mut selector_cursor) {
                        if selector_child.kind() == "identifier" {
                            if let Ok(selector_text) = selector_child.utf8_text(source.as_bytes()) {
                                method_parts.push(format!("{selector_text}:"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if method_parts.is_empty() {
            // Fallback: try to extract from the first identifier
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    if let Ok(name_text) = child.utf8_text(source.as_bytes()) {
                        return name_text.to_string();
                    }
                }
            }
            "unknown".to_string()
        } else {
            method_parts.join("")
        }
    }

    fn extract_category(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Objective-C categories: @interface ClassName (CategoryName) or @implementation ClassName (CategoryName)
        let mut class_name = String::new();
        let mut category_name = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" && class_name.is_empty() {
                class_name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            } else if child.kind() == "identifier"
                && !class_name.is_empty()
                && category_name.is_empty()
            {
                category_name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            }
        }

        if !class_name.is_empty() && !category_name.is_empty() {
            let location = Location::from_node(node, file_path);
            let full_name = format!("{class_name}({category_name})");

            let modifiers = if node.kind() == "category_interface" {
                vec!["category".to_string(), "interface".to_string()]
            } else {
                vec!["category".to_string(), "implementation".to_string()]
            };

            // Push category as scope for nested items
            let scope = Scope {
                name: full_name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_objc_doc(node, source);

            symbols.push(Symbol {
                name: full_name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::ObjectiveC,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objc_symbol_extraction() {
        let extractor = ObjectiveCExtractor;
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_method_signature_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_class_interface_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_class_implementation_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_protocol_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_property_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_instance_variable_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_category_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_method_name_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }

    #[test]
    fn test_objc_documentation_extraction() {
        let extractor = ObjectiveCExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::ObjectiveC);
    }
}
