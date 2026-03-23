//! TypeScript-specific symbol extractor
//!
//! Extends JavaScript extraction with TypeScript-specific features:
//! - Interfaces and type aliases
//! - Generics and type parameters
//! - Decorators and metadata
//! - Namespace declarations
//! - Enum declarations
//! - Abstract classes and methods

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Safe text extraction from tree-sitter nodes with bounds checking
fn safe_node_text(node: &Node, source: &str) -> String {
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

/// TypeScript Symbol Extractor
/// Specialized for TypeScript-specific language features
pub struct TypeScriptExtractor;

impl SymbolExtractor for TypeScriptExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::TypeScript
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

impl TypeScriptExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            // TypeScript-specific nodes
            "interface_declaration" => {
                self.extract_interface(node, source, file_path, symbols, scope_stack);
            }
            "type_alias_declaration" => {
                self.extract_type_alias(node, source, file_path, symbols, scope_stack);
            }
            "enum_declaration" => {
                self.extract_enum(node, source, file_path, symbols, scope_stack);
            }
            "namespace_declaration" | "module_declaration" => {
                self.extract_namespace(node, source, file_path, symbols, scope_stack);
            }
            "abstract_class_declaration" => {
                self.extract_abstract_class(node, source, file_path, symbols, scope_stack);
            }
            // Enhanced JavaScript nodes with TypeScript features
            "class_declaration" => {
                self.extract_class(node, source, file_path, symbols, scope_stack);
            }
            "function_declaration" => {
                self.extract_function(node, source, file_path, symbols, scope_stack);
            }
            "method_definition" => {
                self.extract_method(node, source, file_path, symbols, scope_stack);
            }
            "variable_declaration" => {
                self.extract_variable(node, source, file_path, symbols, scope_stack);
            }
            "property_signature" => {
                self.extract_property_signature(node, source, file_path, symbols, scope_stack);
            }
            "method_signature" => {
                self.extract_method_signature_node(node, source, file_path, symbols, scope_stack);
            }
            _ => {
                // Continue processing child nodes
            }
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }
    }

    fn extract_interface(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Interface,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_interface_signature(&node, source),
            });
        }
    }

    fn extract_type_alias(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Type,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: vec![],
                signature: self.extract_type_signature(&node, source),
            });
        }
    }

    fn extract_enum(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Enum,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_enum_signature(&node, source),
            });
        }
    }

    fn extract_namespace(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Namespace,
                location: location.clone(),
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: Some(format!("namespace {}", name)),
            });

            // Add namespace to scope stack for nested symbols
            scope_stack.push(Scope {
                name,
                kind: SymbolKind::Namespace,
                location: location.clone(),
            });
        }
    }

    fn extract_abstract_class(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            let mut modifiers = self.extract_modifiers(&node, source);
            modifiers.push("abstract".to_string());

            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers,
                signature: self.extract_class_signature(&node, source),
            });

            // Add class to scope stack for nested symbols
            scope_stack.push(Scope {
                name,
                kind: SymbolKind::Class,
                location: location.clone(),
            });
        }
    }

    fn extract_class(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_class_signature(&node, source),
            });

            // Add class to scope stack for nested symbols
            scope_stack.push(Scope {
                name,
                kind: SymbolKind::Class,
                location,
            });
        }
    }

    fn extract_function(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_function_signature(&node, source),
            });
        }
    }

    fn extract_method(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_method_signature(&node, source),
            });
        }
    }

    fn extract_variable(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Handle variable declarations with type annotations
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = safe_node_text(&name_node, source);
                    let location = Location::from_node(&child, file_path);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::TypeScript,
                        documentation: self.extract_documentation(&node, source),
                        modifiers: self.extract_modifiers(&node, source),
                        signature: self.extract_variable_signature(&child, source),
                    });
                }
            }
        }
    }

    fn extract_property_signature(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Field,  // Use Field instead of Property
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_property_signature_text(&node, source),
            });
        }
    }

    fn extract_method_signature_node(&self, node: Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(&node, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::TypeScript,
                documentation: self.extract_documentation(&node, source),
                modifiers: self.extract_modifiers(&node, source),
                signature: self.extract_method_signature_text(&node, source),
            });
        }
    }

    // Helper methods for extracting TypeScript-specific information
    fn extract_documentation(&self, node: &Node, source: &str) -> Option<String> {
        // Extract TSDoc comments from leading comments
        
        // Get the previous sibling to look for comments
        let mut current = *node;
        let mut comments = Vec::new();
        
        // Look for comment nodes before this node
        while let Some(prev_sibling) = current.prev_sibling() {
            if prev_sibling.kind() == "comment" {
                if let Ok(comment_text) = prev_sibling.utf8_text(source.as_bytes()) {
                    comments.push(comment_text.trim().to_string());
                }
            } else if !prev_sibling.kind().is_empty() {
                // Stop at non-comment, non-empty node
                break;
            }
            current = prev_sibling;
        }
        
        if comments.is_empty() {
            return None;
        }
        
        // Process TSDoc comments (remove /** */, *, etc.)
        let mut documentation = String::new();
        for comment in comments.iter().rev() { // Reverse to maintain order
            let cleaned = self.clean_tsdoc_comment(comment);
            if !cleaned.is_empty() {
                if !documentation.is_empty() {
                    documentation.push('\n');
                }
                documentation.push_str(&cleaned);
            }
        }
        
        if documentation.is_empty() {
            None
        } else {
            Some(documentation)
        }
    }

    fn extract_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        
        // Check if this node is within an export statement
        let mut parent = node.parent();
        while let Some(p) = parent {
            if p.kind() == "export_statement" {
                modifiers.push("export".to_string());
                break;
            }
            parent = p.parent();
        }
        
        // Look for TypeScript modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "public" | "private" | "protected" | "readonly" | "static" | "abstract" | "async" => {
                    if let Ok(modifier) = child.utf8_text(source.as_bytes()) {
                        modifiers.push(modifier.to_string());
                    }
                }
                _ => {}
            }
        }
        
        modifiers
    }

    fn extract_interface_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract interface signature with generics
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            // Find the interface declaration line
            let lines: Vec<&str> = text.lines().collect();
            if let Some(first_line) = lines.first() {
                return Some(first_line.trim().to_string());
            }
        }
        None
    }

    fn extract_type_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract type alias signature
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(first_line) = lines.first() {
                return Some(first_line.trim().to_string());
            }
        }
        None
    }

    fn extract_enum_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract enum signature
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                return Some(format!("enum {}", name));
            }
        }
        None
    }

    fn extract_class_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract class signature with generics and extends/implements
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(first_line) = lines.first() {
                return Some(first_line.trim().to_string());
            }
        }
        None
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract function signature with type annotations
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(first_line) = lines.first() {
                return Some(first_line.trim().to_string());
            }
        }
        None
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract method signature with type annotations
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(first_line) = lines.first() {
                return Some(first_line.trim().to_string());
            }
        }
        None
    }

    fn extract_variable_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract variable signature with type annotation
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return Some(text.trim().to_string());
        }
        None
    }

    fn extract_property_signature_text(&self, node: &Node, source: &str) -> Option<String> {
        // Extract property signature
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return Some(text.trim().to_string());
        }
        None
    }

    fn extract_method_signature_text(&self, node: &Node, source: &str) -> Option<String> {
        // Extract method signature
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            return Some(text.trim().to_string());
        }
        None
    }

    /// Clean TSDoc comment by removing comment markers and extra whitespace
    fn clean_tsdoc_comment(&self, comment: &str) -> String {
        let mut cleaned = comment.to_string();
        
        // Remove TSDoc comment markers
        if cleaned.starts_with("/**") {
            cleaned = cleaned[3..].to_string();
        } else if let Some(stripped) = cleaned
            .strip_prefix("/*")
            .or_else(|| cleaned.strip_prefix("//"))
        {
            cleaned = stripped.to_string();
        }
        
        if cleaned.ends_with("*/") {
            cleaned = cleaned[..cleaned.len() - 2].to_string();
        }
        
        // Remove leading * on each line
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut result_lines = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix('*') {
                result_lines.push(stripped.trim());
            } else if !trimmed.is_empty() {
                result_lines.push(trimmed);
            } else {
                // Preserve empty lines for paragraph breaks
                result_lines.push("");
            }
        }
        
        // Remove leading and trailing empty lines
        while result_lines.first().is_some_and(|s| s.is_empty()) {
            result_lines.remove(0);
        }
        while result_lines.last().is_some_and(|s| s.is_empty()) {
            result_lines.pop();
        }
        
        result_lines.join("\n")
    }
}
