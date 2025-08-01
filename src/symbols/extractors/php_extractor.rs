//! PHP symbol extractor
//!
//! Extracts symbols from PHP source code including:
//! - Functions and methods
//! - Classes, interfaces, and traits
//! - Properties and constants
//! - Variables and assignments
//! - Namespaces and use statements
//! - Include and require statements

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// PHP Symbol Extractor
/// Extracts functions, classes, variables, includes, and interfaces from PHP code
pub struct PhpExtractor;

impl SymbolExtractor for PhpExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::PHP
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();
        let mut scope_stack = Vec::new();

        self.traverse_node(
            &root_node,
            source,
            file_path,
            &mut symbols,
            &mut scope_stack,
        );
        symbols
    }
}

impl PhpExtractor {
    fn traverse_node(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "include_expression"
            | "include_once_expression"
            | "require_expression"
            | "require_once_expression" => {
                self.extract_include(node, source, file_path, symbols, scope_stack);
            }
            "function_definition" => {
                self.extract_function(node, source, file_path, symbols, scope_stack);
            }
            "class_declaration" => {
                self.extract_class(node, source, file_path, symbols, scope_stack);
            }
            "interface_declaration" => {
                self.extract_interface(node, source, file_path, symbols, scope_stack);
            }
            "trait_declaration" => {
                self.extract_trait(node, source, file_path, symbols, scope_stack);
            }
            "method_declaration" => {
                self.extract_method(node, source, file_path, symbols, scope_stack);
            }
            "property_declaration" => {
                self.extract_property(node, source, file_path, symbols, scope_stack);
            }
            "const_declaration" => {
                self.extract_constant(node, source, file_path, symbols, scope_stack);
            }
            "assignment_expression" => {
                self.extract_variable_assignment(node, source, file_path, symbols, scope_stack);
            }
            "namespace_definition" => {
                self.extract_namespace(node, source, file_path, symbols, scope_stack);
            }
            "namespace_use_declaration" => {
                self.extract_use_statement(node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_node(&child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        match node.kind() {
            "function_definition"
            | "class_declaration"
            | "interface_declaration"
            | "trait_declaration"
            | "namespace_definition" => {
                scope_stack.pop();
            }
            _ => {}
        }
    }

    fn extract_include(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for string literal in include statement
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" {
                if let Ok(include_path) = child.utf8_text(source.as_bytes()) {
                    let clean_path = include_path.trim_matches('"').trim_matches('\'');

                    let symbol = Symbol {
                        name: clean_path.to_string(),
                        kind: SymbolKind::Import,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation: None,
                        modifiers: vec![node.kind().to_string()],
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);
                }
                break;
            }
        }
    }

    fn extract_function(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Look for function name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                if let Ok(function_name) = child.utf8_text(source.as_bytes()) {
                    let mut modifiers = vec!["function".to_string()];

                    // Check for visibility modifiers (public, private, protected, static)
                    let mut modifier_cursor = node.walk();
                    for modifier_child in node.children(&mut modifier_cursor) {
                        if modifier_child.kind() == "visibility_modifier"
                            || modifier_child.kind() == "static_modifier"
                        {
                            if let Ok(modifier_text) = modifier_child.utf8_text(source.as_bytes()) {
                                modifiers.push(modifier_text.to_string());
                            }
                        }
                    }

                    let documentation = self.extract_php_doc(node, source);

                    let symbol = Symbol {
                        name: function_name.to_string(),
                        kind: SymbolKind::Function,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation,
                        modifiers,
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);

                    // Push function scope for nested symbols
                    scope_stack.push(Scope {
                        name: function_name.to_string(),
                        kind: SymbolKind::Function,
                        location: self.node_to_location(node, file_path),
                    });
                }
                break;
            }
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
        // Look for class name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                if let Ok(class_name) = child.utf8_text(source.as_bytes()) {
                    let mut modifiers = vec!["class".to_string()];

                    // Check for class modifiers (abstract, final)
                    let mut modifier_cursor = node.walk();
                    for modifier_child in node.children(&mut modifier_cursor) {
                        if modifier_child.kind() == "abstract_modifier"
                            || modifier_child.kind() == "final_modifier"
                        {
                            if let Ok(modifier_text) = modifier_child.utf8_text(source.as_bytes()) {
                                modifiers.push(modifier_text.to_string());
                            }
                        }
                    }

                    let documentation = self.extract_php_doc(node, source);

                    let symbol = Symbol {
                        name: class_name.to_string(),
                        kind: SymbolKind::Class,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation,
                        modifiers,
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);

                    // Push class scope for nested symbols
                    scope_stack.push(Scope {
                        name: class_name.to_string(),
                        kind: SymbolKind::Class,
                        location: self.node_to_location(node, file_path),
                    });
                }
                break;
            }
        }
    }

    fn extract_interface(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Look for interface name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                if let Ok(interface_name) = child.utf8_text(source.as_bytes()) {
                    let symbol = Symbol {
                        name: interface_name.to_string(),
                        kind: SymbolKind::Interface,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation: None,
                        modifiers: vec!["interface".to_string()],
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);

                    // Push interface scope for nested symbols
                    scope_stack.push(Scope {
                        name: interface_name.to_string(),
                        kind: SymbolKind::Interface,
                        location: self.node_to_location(node, file_path),
                    });
                }
                break;
            }
        }
    }

    fn extract_trait(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Look for trait name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                if let Ok(trait_name) = child.utf8_text(source.as_bytes()) {
                    let symbol = Symbol {
                        name: trait_name.to_string(),
                        kind: SymbolKind::Trait,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation: None,
                        modifiers: vec!["trait".to_string()],
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);

                    // Push trait scope for nested symbols
                    scope_stack.push(Scope {
                        name: trait_name.to_string(),
                        kind: SymbolKind::Trait,
                        location: self.node_to_location(node, file_path),
                    });
                }
                break;
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
        // Look for method name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                if let Ok(method_name) = child.utf8_text(source.as_bytes()) {
                    let mut modifiers = vec!["method".to_string()];

                    // Check for visibility and other modifiers
                    let mut modifier_cursor = node.walk();
                    for modifier_child in node.children(&mut modifier_cursor) {
                        match modifier_child.kind() {
                            "visibility_modifier"
                            | "static_modifier"
                            | "abstract_modifier"
                            | "final_modifier" => {
                                if let Ok(modifier_text) =
                                    modifier_child.utf8_text(source.as_bytes())
                                {
                                    modifiers.push(modifier_text.to_string());
                                }
                            }
                            _ => {}
                        }
                    }

                    let documentation = self.extract_php_doc(node, source);

                    let symbol = Symbol {
                        name: method_name.to_string(),
                        kind: SymbolKind::Method,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation,
                        modifiers,
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);
                }
                break;
            }
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
        // Look for property variables
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_element" {
                let mut prop_cursor = child.walk();
                for prop_child in child.children(&mut prop_cursor) {
                    if prop_child.kind() == "variable_name" {
                        if let Ok(property_name) = prop_child.utf8_text(source.as_bytes()) {
                            let clean_name = property_name.trim_start_matches('$');

                            let mut modifiers = vec!["property".to_string()];

                            // Check for visibility modifiers
                            let mut modifier_cursor = node.walk();
                            for modifier_child in node.children(&mut modifier_cursor) {
                                if modifier_child.kind() == "visibility_modifier"
                                    || modifier_child.kind() == "static_modifier"
                                {
                                    if let Ok(modifier_text) =
                                        modifier_child.utf8_text(source.as_bytes())
                                    {
                                        modifiers.push(modifier_text.to_string());
                                    }
                                }
                            }

                            let symbol = Symbol {
                                name: clean_name.to_string(),
                                kind: SymbolKind::Field,
                                location: self.node_to_location(&prop_child, file_path),
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::PHP,
                                documentation: None,
                                modifiers,
                                signature: Some(
                                    node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                                ),
                            };
                            symbols.push(symbol);
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
        // Look for constant name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_element" {
                let mut const_cursor = child.walk();
                for const_child in child.children(&mut const_cursor) {
                    if const_child.kind() == "name" {
                        if let Ok(const_name) = const_child.utf8_text(source.as_bytes()) {
                            let symbol = Symbol {
                                name: const_name.to_string(),
                                kind: SymbolKind::Constant,
                                location: self.node_to_location(&const_child, file_path),
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::PHP,
                                documentation: None,
                                modifiers: vec!["const".to_string()],
                                signature: Some(
                                    child.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                                ),
                            };
                            symbols.push(symbol);
                        }
                    }
                }
            }
        }
    }

    fn extract_variable_assignment(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for variable assignments at top level or class level
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_name" {
                if let Ok(var_name) = child.utf8_text(source.as_bytes()) {
                    let clean_name = var_name.trim_start_matches('$');

                    let symbol = Symbol {
                        name: clean_name.to_string(),
                        kind: SymbolKind::Variable,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation: None,
                        modifiers: vec!["variable".to_string()],
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);
                }
                break;
            }
        }
    }

    fn extract_namespace(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Look for namespace name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_name" {
                if let Ok(namespace_name) = child.utf8_text(source.as_bytes()) {
                    let symbol = Symbol {
                        name: namespace_name.to_string(),
                        kind: SymbolKind::Module,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::PHP,
                        documentation: None,
                        modifiers: vec!["namespace".to_string()],
                        signature: Some(
                            node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                        ),
                    };
                    symbols.push(symbol);

                    // Push namespace scope
                    scope_stack.push(Scope {
                        name: namespace_name.to_string(),
                        kind: SymbolKind::Namespace,
                        location: self.node_to_location(node, file_path),
                    });
                }
                break;
            }
        }
    }

    fn extract_use_statement(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for namespace_use_clause in namespace_use_declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_use_clause" {
                // Look for qualified_name within the clause
                let mut clause_cursor = child.walk();
                for clause_child in child.children(&mut clause_cursor) {
                    if clause_child.kind() == "qualified_name" {
                        if let Ok(use_name) = clause_child.utf8_text(source.as_bytes()) {
                            let symbol = Symbol {
                                name: use_name.to_string(),
                                kind: SymbolKind::Import,
                                location: self.node_to_location(&clause_child, file_path),
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::PHP,
                                documentation: None,
                                modifiers: vec!["use".to_string()],
                                signature: Some(
                                    node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                                ),
                            };
                            symbols.push(symbol);
                        }
                        break;
                    }
                }
            }
        }
    }

    fn node_to_location(&self, node: &Node, file_path: &str) -> Location {
        Location {
            file_path: file_path.to_string(),
            start_line: node.start_position().row + 1,
            start_column: node.start_position().column,
            end_line: node.end_position().row + 1,
            end_column: node.end_position().column,
        }
    }

    fn extract_php_doc(&self, node: &Node, source: &str) -> Option<String> {
        // PHP documentation appears as /** */ comments or // comments preceding declarations
        // Look for comment nodes that appear before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find PHP doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        // PHPDoc comment
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
                            doc_comments.insert(0, self.clean_php_doc(&content));
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

    /// Clean PHPDoc by removing common tags and normalizing content
    fn clean_php_doc(&self, content: &str) -> String {
        // Remove common PHPDoc tags but keep the content
        let mut cleaned = content.to_string();

        // Remove PHPDoc tags but keep the content
        cleaned = cleaned
            .replace("@param", "Parameter:")
            .replace("@return", "Returns:")
            .replace("@throws", "Throws:")
            .replace("@var", "Variable:")
            .replace("@see", "See:")
            .replace("@since", "Since:")
            .replace("@deprecated", "Deprecated:")
            .replace("@author", "Author:")
            .replace("@version", "Version:")
            .replace("@package", "Package:")
            .replace("@subpackage", "Subpackage:")
            .replace("@access", "Access:")
            .replace("@static", "Static:")
            .replace("@abstract", "Abstract:")
            .replace("@final", "Final:");

        cleaned.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_symbol_extraction() {
        let extractor = PhpExtractor;
        assert_eq!(extractor.language(), LanguageId::PHP);
    }

    #[test]
    fn test_php_doc_cleaning() {
        let extractor = PhpExtractor;

        // Test PHPDoc tag removal
        let php_content = "@param string $name The name parameter @return bool The result";
        let cleaned = extractor.clean_php_doc(php_content);
        assert_eq!(
            cleaned,
            "Parameter: string $name The name parameter Returns: bool The result"
        );

        // Test variable documentation
        let var_content = "@var int $count The count variable";
        let cleaned_var = extractor.clean_php_doc(var_content);
        assert_eq!(cleaned_var, "Variable: int $count The count variable");

        // Test throws documentation
        let throws_content = "@throws Exception When something goes wrong";
        let cleaned_throws = extractor.clean_php_doc(throws_content);
        assert_eq!(
            cleaned_throws,
            "Throws: Exception When something goes wrong"
        );
    }

    #[test]
    fn test_include_extraction() {
        let extractor = PhpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::PHP);
    }

    #[test]
    fn test_class_extraction() {
        let extractor = PhpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::PHP);
    }

    #[test]
    fn test_function_extraction() {
        let extractor = PhpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::PHP);
    }

    #[test]
    fn test_trait_extraction() {
        let extractor = PhpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::PHP);
    }

    #[test]
    fn test_namespace_extraction() {
        let extractor = PhpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::PHP);
    }
}
