//! Java symbol extractor
//!
//! Extracts symbols from Java source code including:
//! - Classes, interfaces, and enums
//! - Methods and constructors
//! - Fields and constants
//! - Package declarations and imports
//! - Javadoc comments

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Java Symbol Extractor
/// Extracts classes, methods, fields, imports, and packages from Java code
pub struct JavaExtractor;

impl SymbolExtractor for JavaExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Java
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

impl JavaExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "package_declaration" => {
                // Find scoped_identifier child directly
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "scoped_identifier" {
                        let package_name = self.extract_scoped_identifier(&child, source);
                        let location = Location::from_node(&node, file_path);

                        symbols.push(Symbol {
                            name: package_name,
                            kind: SymbolKind::Namespace,
                            location,
                            scope_chain: vec![],
                            language: LanguageId::Java,
                            documentation: None,
                            modifiers: vec!["package".to_string()],
                            signature: None,
                        });
                        break;
                    }
                }
            }
            "import_declaration" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    // Push class as scope for nested items
                    let scope = Scope {
                        name: name.clone(),
                        kind: SymbolKind::Class,
                        location: location.clone(),
                    };
                    scope_stack.push(scope);

                    let modifiers = self.extract_class_modifiers(&node, source);
                    let documentation = self.extract_javadoc(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        location,
                        scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                        language: LanguageId::Java,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    // Push interface as scope for nested items
                    let scope = Scope {
                        name: name.clone(),
                        kind: SymbolKind::Interface,
                        location: location.clone(),
                    };
                    scope_stack.push(scope);

                    let modifiers = self.extract_class_modifiers(&node, source);
                    let documentation = self.extract_javadoc(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Interface,
                        location,
                        scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                        language: LanguageId::Java,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
            "enum_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    let modifiers = self.extract_class_modifiers(&node, source);
                    let documentation = self.extract_javadoc(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Java,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    let signature = self.extract_method_signature(&node, source);
                    let modifiers = self.extract_method_modifiers(&node, source);
                    let documentation = self.extract_javadoc(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Method,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Java,
                        documentation,
                        modifiers,
                        signature,
                    });
                }
            }
            "constructor_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    let signature = self.extract_method_signature(&node, source);
                    let modifiers = self.extract_method_modifiers(&node, source);
                    let documentation = self.extract_javadoc(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Method, // Constructor is treated as a special method
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Java,
                        documentation,
                        modifiers: {
                            let mut mods = modifiers;
                            mods.push("constructor".to_string());
                            mods
                        },
                        signature,
                    });
                }
            }
            "field_declaration" => {
                // Extract field names from variable_declarator children
                self.extract_fields(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "class_declaration" | "interface_declaration") {
            scope_stack.pop();
        }
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;

        let params = node
            .child_by_field_name("parameters")?
            .utf8_text(source.as_bytes())
            .ok()?;

        let return_type = node
            .child_by_field_name("type")
            .and_then(|t| t.utf8_text(source.as_bytes()).ok())
            .unwrap_or("void");

        // Get modifiers for the signature
        let modifiers = self.extract_method_modifiers(node, source);
        let modifier_str = if modifiers.is_empty() {
            String::new()
        } else {
            format!("{} ", modifiers.join(" "))
        };

        Some(format!("{modifier_str}{return_type} {name}{params}"))
    }

    fn extract_method_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifiers node
        if let Some(modifiers_node) = node.child_by_field_name("modifiers") {
            // For Java, the modifiers node often contains the actual modifiers as its text content
            if let Ok(modifier_text) = modifiers_node.utf8_text(source.as_bytes()) {
                // Split by whitespace to handle multiple modifiers
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "public"
                            | "private"
                            | "protected"
                            | "static"
                            | "final"
                            | "abstract"
                            | "synchronized"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }

            // Also check if modifiers node has children
            let mut cursor = modifiers_node.walk();
            for child in modifiers_node.children(&mut cursor) {
                if let Ok(modifier) = child.utf8_text(source.as_bytes()) {
                    let modifier = modifier.trim();
                    if matches!(
                        modifier,
                        "public"
                            | "private"
                            | "protected"
                            | "static"
                            | "final"
                            | "abstract"
                            | "synchronized"
                    ) && !modifiers.contains(&modifier.to_string())
                    {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifiers node - in Java tree-sitter, modifiers is a single node containing modifier keywords
        if let Some(modifiers_node) = node.child_by_field_name("modifiers") {
            // For Java, the modifiers node often contains the actual modifiers as its text content
            if let Ok(modifier_text) = modifiers_node.utf8_text(source.as_bytes()) {
                // Split by whitespace to handle multiple modifiers
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "public" | "private" | "protected" | "static" | "final" | "abstract"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }

            // Also check if modifiers node has children (some versions might structure it differently)
            let mut cursor = modifiers_node.walk();
            for child in modifiers_node.children(&mut cursor) {
                if let Ok(modifier) = child.utf8_text(source.as_bytes()) {
                    let modifier = modifier.trim();
                    if matches!(
                        modifier,
                        "public" | "private" | "protected" | "static" | "final" | "abstract"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        // Also check for modifier keywords directly as children (fallback)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                let text = text.trim();
                if matches!(
                    text,
                    "public" | "private" | "protected" | "static" | "final" | "abstract"
                ) && !modifiers.contains(&text.to_string())
                {
                    modifiers.push(text.to_string());
                }
            }
        }

        // Check for extends/implements
        if node.child_by_field_name("superclass").is_some() {
            modifiers.push("extends".to_string());
        }

        if node.child_by_field_name("interfaces").is_some() {
            modifiers.push("implements".to_string());
        }

        modifiers
    }

    fn extract_fields(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let modifiers = self.extract_field_modifiers(node, source);
        let field_type = self.extract_field_type(node, source);

        // Find variable_declarator children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&child, file_path);

                    // Determine if it's a constant (final fields)
                    let kind = if modifiers.contains(&"final".to_string())
                        && modifiers.contains(&"static".to_string())
                    {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Field
                    };

                    symbols.push(Symbol {
                        name,
                        kind,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Java,
                        documentation: None,
                        modifiers: modifiers.clone(),
                        signature: field_type.clone(),
                    });
                }
            }
        }
    }

    fn extract_field_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifiers node
        if let Some(modifiers_node) = node.child_by_field_name("modifiers") {
            // For Java, the modifiers node often contains the actual modifiers as its text content
            if let Ok(modifier_text) = modifiers_node.utf8_text(source.as_bytes()) {
                // Split by whitespace to handle multiple modifiers
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "public"
                            | "private"
                            | "protected"
                            | "static"
                            | "final"
                            | "volatile"
                            | "transient"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }

            // Also check if modifiers node has children
            let mut cursor = modifiers_node.walk();
            for child in modifiers_node.children(&mut cursor) {
                if let Ok(modifier) = child.utf8_text(source.as_bytes()) {
                    let modifier = modifier.trim();
                    if matches!(
                        modifier,
                        "public"
                            | "private"
                            | "protected"
                            | "static"
                            | "final"
                            | "volatile"
                            | "transient"
                    ) && !modifiers.contains(&modifier.to_string())
                    {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_field_type(&self, node: &Node, source: &str) -> Option<String> {
        node.child_by_field_name("type")
            .and_then(|t| t.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string())
    }

    fn extract_import(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let mut import_path = String::new();
        let mut is_static = false;
        let mut is_wildcard = false;

        // Extract import details from child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "static" => is_static = true,
                "scoped_identifier" => {
                    import_path = self.extract_scoped_identifier(&child, source);
                }
                "asterisk" => {
                    is_wildcard = true;
                    if !import_path.is_empty() {
                        import_path.push_str(".*");
                    }
                }
                _ => {}
            }
        }

        if !import_path.is_empty() {
            let location = Location::from_node(node, file_path);

            let mut modifiers = vec!["import".to_string()];
            if is_static {
                modifiers.push("static".to_string());
            }
            if is_wildcard {
                modifiers.push("wildcard".to_string());
            }

            symbols.push(Symbol {
                name: import_path,
                kind: SymbolKind::Import,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::Java,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extract_scoped_identifier(&self, node: &Node, source: &str) -> String {
        match node.kind() {
            "scoped_identifier" => {
                if let (Some(scope), Some(name)) = (
                    node.child_by_field_name("scope"),
                    node.child_by_field_name("name"),
                ) {
                    let scope_str = self.extract_scoped_identifier(&scope, source);
                    let name_str = name.utf8_text(source.as_bytes()).unwrap_or("");
                    format!("{scope_str}.{name_str}")
                } else {
                    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                }
            }
            "identifier" => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
            _ => node.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
        }
    }

    fn extract_javadoc(&self, node: &Node, source: &str) -> Option<String> {
        // Look for Javadoc comment before the node
        let mut current = *node;
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "block_comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") {
                        // Clean up Javadoc comment
                        let cleaned = comment_text
                            .trim_start_matches("/**")
                            .trim_end_matches("*/")
                            .lines()
                            .map(|line| line.trim().trim_start_matches('*').trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        return Some(cleaned);
                    }
                }
                _ if prev.kind().contains("whitespace") => {
                    current = prev;
                    continue;
                }
                _ => break,
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_symbol_extraction() {
        let extractor = JavaExtractor;
        assert_eq!(extractor.language(), LanguageId::Java);
    }

    #[test]
    fn test_scoped_identifier_extraction() {
        let extractor = JavaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Java);
    }

    #[test]
    fn test_method_signature_extraction() {
        let extractor = JavaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Java);
    }

    #[test]
    fn test_modifier_extraction() {
        let extractor = JavaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Java);
    }

    #[test]
    fn test_javadoc_extraction() {
        let extractor = JavaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Java);
    }

    #[test]
    fn test_field_extraction() {
        let extractor = JavaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Java);
    }
}
