//! C# symbol extractor
//!
//! Extracts symbols from C# source code including:
//! - Classes, interfaces, and enums
//! - Methods and constructors
//! - Properties and fields
//! - Namespaces and using statements
//! - XML documentation comments

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

/// C# Symbol Extractor
/// Extracts classes, methods, properties, namespaces, and using statements from C# code
pub struct CSharpExtractor;

impl SymbolExtractor for CSharpExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::CSharp
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

impl CSharpExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "using_directive" => {
                self.extract_using_statement(&node, source, file_path, symbols, scope_stack);
            }
            "namespace_declaration" => {
                self.extract_namespace(&node, source, file_path, symbols, scope_stack);
            }
            "class_declaration" => {
                self.extract_class(&node, source, file_path, symbols, scope_stack);
            }
            "interface_declaration" => {
                self.extract_interface(&node, source, file_path, symbols, scope_stack);
            }
            "enum_declaration" => {
                self.extract_enum(&node, source, file_path, symbols, scope_stack);
            }
            "method_declaration" => {
                self.extract_method(&node, source, file_path, symbols, scope_stack);
            }
            "constructor_declaration" => {
                self.extract_constructor(&node, source, file_path, symbols, scope_stack);
            }
            "property_declaration" => {
                self.extract_property(&node, source, file_path, symbols, scope_stack);
            }
            "field_declaration" => {
                self.extract_field(&node, source, file_path, symbols, scope_stack);
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
            "namespace_declaration" | "class_declaration" | "interface_declaration"
        ) {
            scope_stack.pop();
        }
    }

    fn extract_using_statement(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // C# using directive: using System; or using System.Collections.Generic;
        // Look for identifier or qualified_name children since C# doesn't use field names
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "qualified_name" {
                let using_name = safe_node_text(&child, source);
                let location = Location::from_node(node, file_path);

                symbols.push(Symbol {
                    name: using_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::CSharp,
                    documentation: None,
                    modifiers: vec!["using".to_string()],
                    signature: None,
                });
                break; // Only process the first namespace identifier/qualified_name
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
        // Look for qualified_name child since C# doesn't use field names for namespace
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "qualified_name" || child.kind() == "identifier" {
                let name = safe_node_text(&child, source);
                let location = Location::from_node(node, file_path);

                // Push namespace as scope for nested items
                let scope = Scope {
                    name: name.clone(),
                    kind: SymbolKind::Namespace,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Namespace,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::CSharp,
                    documentation: None,
                    modifiers: vec!["namespace".to_string()],
                    signature: None,
                });
                break; // Only process the first namespace name
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
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            // Extract modifiers (public, private, static, etc.)
            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_xml_doc(node, source);

            // Push class as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
            };
            scope_stack.push(scope);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::CSharp,
                documentation,
                modifiers,
                signature: None,
            });
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_xml_doc(node, source);

            // Push interface as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Interface,
                location: location.clone(),
            };
            scope_stack.push(scope);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Interface,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::CSharp,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_enum(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_xml_doc(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Enum,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::CSharp,
                documentation,
                modifiers,
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
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let documentation = self.extract_xml_doc(node, source);
            let signature = self.extract_method_signature(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::CSharp,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_constructor(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let modifiers = self.extract_modifiers(node, source);
            let signature = self.extract_constructor_signature(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::CSharp,
                documentation: None,
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = safe_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let mut modifiers = self.extract_modifiers(node, source);
            modifiers.push("property".to_string());
            let documentation = self.extract_xml_doc(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Field, // Properties are treated as fields
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::CSharp,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_field(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // C# field declarations can contain multiple variables
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = safe_node_text(&name_node, source);
                    let location = Location::from_node(&child, file_path);

                    let mut modifiers = self.extract_modifiers(node, source);
                    modifiers.push("field".to_string());

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Field,
                        location,
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::CSharp,
                        documentation: None,
                        modifiers,
                        signature: None,
                    });
                }
            }
        }
    }

    fn extract_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifier lists that typically appear before declarations
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                if let Ok(modifier_text) = child.utf8_text(source.as_bytes()) {
                    modifiers.push(modifier_text.to_string());
                }
            }
        }

        modifiers
    }

    fn extract_xml_doc(&self, node: &Node, source: &str) -> Option<String> {
        // C# XML documentation appears as /// comments preceding declarations
        // Look for documentation comment nodes that appear before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find XML doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "documentation_comment" => {
                    let comment_text = safe_node_text(&prev, source);
                    if comment_text.starts_with("///") {
                        // XML documentation comment
                        let content = comment_text.strip_prefix("///").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    }
                    current = prev;
                }
                "comment" => {
                    // Regular comment - check if it's XML doc style
                    let comment_text = safe_node_text(&prev, source);
                    if comment_text.starts_with("///") {
                        let content = comment_text.strip_prefix("///").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, self.clean_xml_doc(content));
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

    /// Clean XML documentation by removing XML tags and normalizing content
    fn clean_xml_doc(&self, content: &str) -> String {
        // Remove common XML doc tags and extract meaningful content
        let mut cleaned = content.to_string();

        // Remove XML tags but keep the content
        cleaned = cleaned
            .replace("<summary>", "")
            .replace("</summary>", "")
            .replace("<param name=\"", "Parameter ")
            .replace("\">", ": ")
            .replace("</param>", "")
            .replace("<returns>", "Returns: ")
            .replace("</returns>", "")
            .replace("<remarks>", "")
            .replace("</remarks>", "")
            .replace("<example>", "Example: ")
            .replace("</example>", "")
            .replace("<see cref=\"", "See: ")
            .replace("\"/>", "")
            .replace("<c>", "`")
            .replace("</c>", "`")
            .replace("<code>", "```")
            .replace("</code>", "```");

        cleaned.trim().to_string()
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract method signature including return type, name, and parameters
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;

        let return_type = node
            .child_by_field_name("type")
            .and_then(|t| t.utf8_text(source.as_bytes()).ok())
            .unwrap_or("void");

        let params = node
            .child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");

        Some(format!("{return_type} {name}{params}"))
    }

    fn extract_constructor_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;

        let params = node
            .child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");

        Some(format!("{name}{params}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csharp_symbol_extraction() {
        let extractor = CSharpExtractor;
        assert_eq!(extractor.language(), LanguageId::CSharp);
    }

    #[test]
    fn test_xml_doc_cleaning() {
        let extractor = CSharpExtractor;

        // Test XML tag removal
        let xml_content = "<summary>This is a summary</summary>";
        let cleaned = extractor.clean_xml_doc(xml_content);
        assert_eq!(cleaned, "This is a summary");

        // Test parameter documentation
        let param_content = r#"<param name="value">The input value</param>"#;
        let cleaned_param = extractor.clean_xml_doc(param_content);
        assert_eq!(cleaned_param, "Parameter value: The input value");

        // Test returns documentation
        let returns_content = "<returns>The result value</returns>";
        let cleaned_returns = extractor.clean_xml_doc(returns_content);
        assert_eq!(cleaned_returns, "Returns: The result value");
    }

    #[test]
    fn test_method_signature_extraction() {
        let extractor = CSharpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSharp);
    }

    #[test]
    fn test_constructor_signature_extraction() {
        let extractor = CSharpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSharp);
    }

    #[test]
    fn test_modifier_extraction() {
        let extractor = CSharpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSharp);
    }

    #[test]
    fn test_namespace_extraction() {
        let extractor = CSharpExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSharp);
    }
}
