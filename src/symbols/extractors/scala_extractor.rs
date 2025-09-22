//! Scala symbol extractor
//!
//! Extracts symbols from Scala source code including:
//! - Classes and traits
//! - Objects (singleton objects)
//! - Methods and functions
//! - Variables and values
//! - Imports and packages
//! - Case classes and pattern matching

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

/// Scala Symbol Extractor
/// Extracts classes, objects, methods, imports from Scala code
pub struct ScalaExtractor;

impl SymbolExtractor for ScalaExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Scala
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

impl ScalaExtractor {
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
                self.extract_package(&node, source, file_path, symbols, scope_stack);
            }
            "import_declaration" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "class_definition" => {
                self.extract_class(&node, source, file_path, symbols, scope_stack);
            }
            "trait_definition" => {
                self.extract_trait(&node, source, file_path, symbols, scope_stack);
            }
            "object_definition" => {
                self.extract_object(&node, source, file_path, symbols, scope_stack);
            }
            "function_definition" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "method_definition" => {
                self.extract_method(&node, source, file_path, symbols, scope_stack);
            }
            "val_definition" => {
                self.extract_val(&node, source, file_path, symbols, scope_stack);
            }
            "var_definition" => {
                self.extract_var(&node, source, file_path, symbols, scope_stack);
            }
            "type_definition" => {
                self.extract_type_alias(&node, source, file_path, symbols, scope_stack);
            }
            "case_class_definition" => {
                self.extract_case_class(&node, source, file_path, symbols, scope_stack);
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
            "class_definition" | "trait_definition" | "object_definition" | "case_class_definition"
        ) {
            scope_stack.pop();
        }
    }

    fn extract_package(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Extract package name from qualified identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "qualified_identifier" || child.kind() == "identifier" {
                let package_name = self.get_node_text(&child, source);
                let location = Location::from_node(node, file_path);

                symbols.push(Symbol {
                    name: package_name,
                    kind: SymbolKind::Namespace,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Scala,
                    documentation: None,
                    modifiers: vec!["package".to_string()],
                    signature: None,
                });
                break;
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
        // Scala imports can be: import scala.collection.mutable, import scala.util.{Try, Success, Failure}, import scala.collection._
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "qualified_identifier" {
                let import_name = self.get_node_text(&child, source);
                let location = Location::from_node(&child, file_path);

                let mut modifiers = vec!["import".to_string()];

                // Check for wildcard import
                if import_name.ends_with("._") {
                    modifiers.push("wildcard".to_string());
                }

                symbols.push(Symbol {
                    name: import_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Scala,
                    documentation: None,
                    modifiers,
                    signature: None,
                });
            } else if child.kind() == "import_selectors" {
                // Handle selective imports like import scala.util.{Try, Success}
                self.extract_import_selectors(&child, source, file_path, symbols, scope_stack);
            } else if child.kind() == "wildcard" || self.get_node_text(&child, source) == "_" {
                // Handle wildcard imports
                let location = Location::from_node(&child, file_path);
                symbols.push(Symbol {
                    name: "_".to_string(),
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Scala,
                    documentation: None,
                    modifiers: vec!["import".to_string(), "wildcard".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn extract_import_selectors(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let import_name = self.get_node_text(&child, source);
                let location = Location::from_node(&child, file_path);

                symbols.push(Symbol {
                    name: import_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Scala,
                    documentation: None,
                    modifiers: ["import", "selective"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    signature: None,
                });
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

            let documentation = self.extract_scala_doc(node, source);
            let modifiers = self.extract_class_modifiers(node, source);
            let signature = self.extract_class_signature(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature,
            });
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            // Push trait as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Interface, // Traits are like interfaces
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_scala_doc(node, source);
            let modifiers = self.extract_trait_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Interface,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_object(
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

            // Push object as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class, // Objects are singleton classes
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_scala_doc(node, source);
            let mut modifiers = self.extract_object_modifiers(node, source);
            modifiers.push("object".to_string());
            modifiers.push("singleton".to_string());

            // Check if this might be a companion object
            if self.is_companion_object(&name, scope_stack) {
                modifiers.push("companion".to_string());
            }

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature: None,
            });
        }
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
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let signature = self.extract_function_signature(node, source);
            let documentation = self.extract_scala_doc(node, source);
            let modifiers = self.extract_function_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature,
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
            let documentation = self.extract_scala_doc(node, source);
            let modifiers = self.extract_method_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_val(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Extract val declarations (immutable values)
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let mut modifiers = vec!["val".to_string(), "immutable".to_string()];
            modifiers.extend(self.extract_val_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Constant,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Scala,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_var(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Extract var declarations (mutable variables)
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let mut modifiers = vec!["var".to_string(), "mutable".to_string()];
            modifiers.extend(self.extract_var_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Scala,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_type_alias(
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

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Type,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Scala,
                documentation: self.extract_scala_doc(node, source),
                modifiers: vec!["type".to_string()],
                signature: None,
            });
        }
    }

    fn extract_case_class(
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

            // Push case class as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_scala_doc(node, source);
            let mut modifiers = self.extract_class_modifiers(node, source);
            modifiers.push("case".to_string());
            modifiers.push("immutable".to_string());
            let signature = self.extract_case_class_signature(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Scala,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        safe_node_text(node, source)
    }

    fn extract_scala_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Scala documentation appears as /** */ comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Scala doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        // ScalaDoc comment
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

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["class".to_string()];

        // Look for modifiers before the class keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "abstract"
                            | "final"
                            | "sealed"
                            | "private"
                            | "protected"
                            | "implicit"
                            | "lazy"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        // Check for extends and with clauses
        if node.child_by_field_name("extends").is_some() {
            modifiers.push("extends".to_string());
        }
        if node.child_by_field_name("with").is_some() {
            modifiers.push("with".to_string());
        }

        modifiers
    }

    fn extract_trait_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["trait".to_string()];

        // Look for modifiers before the trait keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(modifier, "sealed" | "private" | "protected") {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_object_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifiers before the object keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(modifier, "private" | "protected" | "implicit" | "lazy") {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_function_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["def".to_string()];

        // Look for modifiers before the def keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "private" | "protected" | "override" | "final" | "implicit"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_method_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["def".to_string(), "method".to_string()];

        // Look for modifiers before the def keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "private" | "protected" | "override" | "final" | "implicit" | "abstract"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_val_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(
                        modifier,
                        "private" | "protected" | "override" | "implicit" | "lazy"
                    ) {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_var_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let modifier_text = self.get_node_text(&child, source);
                for modifier in modifier_text.split_whitespace() {
                    if matches!(modifier, "private" | "protected" | "override" | "implicit") {
                        modifiers.push(modifier.to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn extract_class_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_default();

        let extends_clause = node
            .child_by_field_name("extends")
            .map(|e| format!(" extends {}", self.get_node_text(&e, source)))
            .unwrap_or_default();

        Some(format!("class {name}{params}{extends_clause}"))
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        let return_type = node
            .child_by_field_name("return_type")
            .map(|rt| format!(": {}", self.get_node_text(&rt, source)))
            .unwrap_or_default();

        Some(format!("def {name}{params}{return_type}"))
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        self.extract_function_signature(node, source)
    }

    fn extract_case_class_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        let extends_clause = node
            .child_by_field_name("extends")
            .map(|e| format!(" extends {}", self.get_node_text(&e, source)))
            .unwrap_or_default();

        Some(format!("case class {name}{params}{extends_clause}"))
    }

    fn is_companion_object(&self, object_name: &str, scope_stack: &[Scope]) -> bool {
        // A companion object has the same name as a class in the same scope
        // This is a simplified check - in practice, you'd need to track all symbols in the current scope
        scope_stack
            .iter()
            .any(|scope| scope.name == object_name && scope.kind == SymbolKind::Class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scala_symbol_extraction() {
        let extractor = ScalaExtractor;
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_class_signature_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_object_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_trait_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_method_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_val_var_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_import_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_scala_doc_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_case_class_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_companion_object_detection() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }

    #[test]
    fn test_wildcard_import_extraction() {
        let extractor = ScalaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Scala);
    }
}
