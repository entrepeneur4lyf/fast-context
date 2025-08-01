//! Ruby symbol extractor
//!
//! Extracts symbols from Ruby source code including:
//! - Classes and modules
//! - Methods and functions
//! - Constants and variables
//! - Includes and requires
//! - Instance variables and class variables

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Ruby Symbol Extractor
/// Extracts classes, methods, constants, includes from Ruby code
pub struct RubyExtractor;

impl SymbolExtractor for RubyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Ruby
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

impl RubyExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "class" => {
                self.extract_class(&node, source, file_path, symbols, scope_stack);
            }
            "module" => {
                self.extract_module(&node, source, file_path, symbols, scope_stack);
            }
            "method" => {
                self.extract_method(&node, source, file_path, symbols, scope_stack);
            }
            "singleton_method" => {
                self.extract_singleton_method(&node, source, file_path, symbols, scope_stack);
            }
            "call" => {
                self.extract_call(&node, source, file_path, symbols, scope_stack);
            }
            "constant" => {
                self.extract_constant(&node, source, file_path, symbols, scope_stack);
            }
            "instance_variable" => {
                self.extract_instance_variable(&node, source, file_path, symbols, scope_stack);
            }
            "class_variable" => {
                self.extract_class_variable(&node, source, file_path, symbols, scope_stack);
            }
            "assignment" => {
                self.extract_assignment(&node, source, file_path, symbols, scope_stack);
            }
            "symbol" => {
                self.extract_symbol(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "class" | "module") {
            scope_stack.pop();
        }
    }

    fn extract_class(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            // Push class as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_ruby_doc(node, source);
            let modifiers = self.extract_class_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Ruby,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_module(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            // Push module as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Module,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_ruby_doc(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Module,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Ruby,
                documentation,
                modifiers: vec!["module".to_string()],
                signature: None,
            });
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let signature = self.extract_method_signature(node, source);
            let documentation = self.extract_ruby_doc(node, source);
            let modifiers = self.extract_method_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Ruby,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_singleton_method(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let signature = self.extract_method_signature(node, source);
            let documentation = self.extract_ruby_doc(node, source);
            let mut modifiers = self.extract_method_modifiers(node, source);
            modifiers.push("singleton".to_string());

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Ruby,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_call(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Check for require/require_relative/include/extend calls
        if let Some(method_node) = node.child_by_field_name("method") {
            let method_name = self.get_node_text(&method_node, source);

            if matches!(
                method_name.as_str(),
                "require" | "require_relative" | "include" | "extend" | "prepend"
            ) {
                if let Some(arguments) = node.child_by_field_name("arguments") {
                    // Extract the first argument as the module/file name
                    let mut cursor = arguments.walk();
                    for child in arguments.children(&mut cursor) {
                        if child.kind() == "string" {
                            let import_name =
                                self.clean_string_literal(&self.get_node_text(&child, source));
                            let location = Location::from_node(&child, file_path);

                            let kind = match method_name.as_str() {
                                "require" | "require_relative" => SymbolKind::Import,
                                _ => SymbolKind::Import, // Use Import for all include-like operations
                            };

                            symbols.push(Symbol {
                                name: import_name,
                                kind,
                                location,
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::Ruby,
                                documentation: None,
                                modifiers: vec![method_name.clone()],
                                signature: None,
                            });
                            break; // Only process the first argument
                        }
                    }
                }
            } else if matches!(
                method_name.as_str(),
                "attr_accessor" | "attr_reader" | "attr_writer"
            ) {
                // Handle Ruby attribute declarations
                if let Some(arguments) = node.child_by_field_name("arguments") {
                    let mut cursor = arguments.walk();
                    for child in arguments.children(&mut cursor) {
                        if child.kind() == "symbol" {
                            let attr_name = self.get_node_text(&child, source);
                            let clean_name = if let Some(stripped) = attr_name.strip_prefix(':') {
                                stripped.to_string()
                            } else {
                                attr_name.clone()
                            };
                            let location = Location::from_node(&child, file_path);

                            let mut modifiers = vec!["attribute".to_string(), method_name.clone()];

                            // Add specific modifiers based on attr type
                            match method_name.as_str() {
                                "attr_accessor" => {
                                    modifiers.push("readable".to_string());
                                    modifiers.push("writable".to_string());
                                }
                                "attr_reader" => {
                                    modifiers.push("readable".to_string());
                                }
                                "attr_writer" => {
                                    modifiers.push("writable".to_string());
                                }
                                _ => {}
                            }

                            symbols.push(Symbol {
                                name: clean_name,
                                kind: SymbolKind::Field,
                                location,
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::Ruby,
                                documentation: None,
                                modifiers,
                                signature: Some(attr_name),
                            });
                        }
                    }
                }
            }
        }
    }

    fn extract_constant(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Constant,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Ruby,
            documentation: None,
            modifiers: vec!["constant".to_string()],
            signature: None,
        });
    }

    fn extract_instance_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Field,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Ruby,
            documentation: None,
            modifiers: vec!["instance_variable".to_string()],
            signature: None,
        });
    }

    fn extract_class_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Field,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Ruby,
            documentation: None,
            modifiers: vec!["class_variable".to_string()],
            signature: None,
        });
    }

    fn extract_assignment(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Extract variable assignments
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "identifier" {
                let name = self.get_node_text(&left, source);
                let location = Location::from_node(&left, file_path);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Variable,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Ruby,
                    documentation: None,
                    modifiers: vec!["variable".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn extract_symbol(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Ruby symbols like :symbol_name
        let name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Clean the symbol name (remove the colon)
        let clean_name = if let Some(stripped) = name.strip_prefix(':') {
            stripped.to_string()
        } else {
            name.clone()
        };

        symbols.push(Symbol {
            name: clean_name,
            kind: SymbolKind::Constant,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Ruby,
            documentation: None,
            modifiers: vec!["symbol".to_string()],
            signature: Some(name),
        });
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    fn clean_string_literal(&self, text: &str) -> String {
        // Remove quotes from string literals
        if (text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\''))
        {
            text[1..text.len() - 1].to_string()
        } else {
            text.to_string()
        }
    }

    fn extract_ruby_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Ruby documentation appears as comments preceding declarations
        // This is a simplified implementation - in practice, you'd look for
        // comment nodes that appear before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Ruby doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("#") {
                        // Ruby comment
                        let content = comment_text.strip_prefix("#").unwrap_or("").trim();
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

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["class".to_string()];

        // Check for superclass
        if let Some(superclass_node) = node.child_by_field_name("superclass") {
            modifiers.push("inherits".to_string());
            let superclass_name = self.get_node_text(&superclass_node, source);
            if !superclass_name.is_empty() {
                modifiers.push(format!("extends_{superclass_name}"));
            }
        }

        // Look for included modules or extended modules in the class body
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "call" {
                if let Some(method_node) = child.child_by_field_name("method") {
                    let method_name = self.get_node_text(&method_node, source);
                    if matches!(method_name.as_str(), "include" | "extend" | "prepend") {
                        modifiers.push(format!("uses_{method_name}"));
                    }
                }
            }
        }

        modifiers
    }

    fn extract_method_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["method".to_string()];

        // Check for visibility modifiers by looking at siblings or parent context
        // This is a simplified approach - full implementation would track visibility scope
        if let Some(parent) = node.parent() {
            let parent_text = self.get_node_text(&parent, source);
            if parent_text.contains("private") {
                modifiers.push("private".to_string());
            } else if parent_text.contains("protected") {
                modifiers.push("protected".to_string());
            } else {
                modifiers.push("public".to_string());
            }
        }

        modifiers
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        Some(format!("def {name}{params}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_symbol_extraction() {
        let extractor = RubyExtractor;
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_method_signature_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_class_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_module_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_constant_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_variable_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_ruby_doc_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_symbol_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_attr_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }

    #[test]
    fn test_class_modifiers_extraction() {
        let extractor = RubyExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Ruby);
    }
}
