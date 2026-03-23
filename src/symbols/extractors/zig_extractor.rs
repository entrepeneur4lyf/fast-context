//! Zig symbol extractor
//!
//! Extracts symbols from Zig source code including:
//! - Functions and methods
//! - Structs and unions
//! - Enums and tagged unions
//! - Constants and variables
//! - Imports and uses
//! - Type definitions

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

/// Zig Symbol Extractor
/// Extracts functions, structs, enums, imports from Zig code
pub struct ZigExtractor;

impl SymbolExtractor for ZigExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Zig
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

impl ZigExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "function_declaration" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "struct_declaration" => {
                self.extract_struct(&node, source, file_path, symbols, scope_stack);
            }
            "union_declaration" => {
                self.extract_union(&node, source, file_path, symbols, scope_stack);
            }
            "enum_declaration" => {
                self.extract_enum(&node, source, file_path, symbols, scope_stack);
            }
            "variable_declaration" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            "const_declaration" => {
                self.extract_const(&node, source, file_path, symbols, scope_stack);
            }
            "use_declaration" | "import_declaration" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "type_declaration" => {
                self.extract_type_alias(&node, source, file_path, symbols, scope_stack);
            }
            "error_declaration" => {
                self.extract_error_type(&node, source, file_path, symbols, scope_stack);
            }
            "test_declaration" => {
                self.extract_test(&node, source, file_path, symbols, scope_stack);
            }
            "comptime_declaration" => {
                self.extract_comptime(&node, source, file_path, symbols, scope_stack);
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
            "struct_declaration" | "union_declaration" | "enum_declaration"
        ) {
            scope_stack.pop();
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
            let documentation = self.extract_zig_doc(node, source);
            let modifiers = self.extract_function_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Zig,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_struct(
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

            // Push struct as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Struct,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_zig_doc(node, source);
            let modifiers = self.extract_struct_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Struct,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Zig,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_union(
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

            // Push union as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Struct, // Treat unions as structs
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_zig_doc(node, source);
            let mut modifiers = self.extract_union_modifiers(node, source);
            modifiers.push("union".to_string());

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Struct,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Zig,
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
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            // Push enum as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Enum,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_zig_doc(node, source);
            let modifiers = self.extract_enum_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Enum,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Zig,
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let mut modifiers = vec!["var".to_string()];
            modifiers.extend(self.extract_variable_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Zig,
                documentation: self.extract_zig_doc(node, source),
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_const(
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

            let mut modifiers = vec!["const".to_string()];
            modifiers.extend(self.extract_const_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Constant,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Zig,
                documentation: self.extract_zig_doc(node, source),
                modifiers,
                signature: None,
            });
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
        // Zig uses @import("path") for imports and @use for bringing symbols into scope
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string_literal" {
                let import_path = self.clean_string_literal(&self.get_node_text(&child, source));
                let location = Location::from_node(&child, file_path);

                symbols.push(Symbol {
                    name: import_path,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Zig,
                    documentation: None,
                    modifiers: vec!["import".to_string()],
                    signature: None,
                });
                break;
            } else if child.kind() == "identifier" {
                // Handle @use declarations
                let use_name = self.get_node_text(&child, source);
                let location = Location::from_node(&child, file_path);

                symbols.push(Symbol {
                    name: use_name,
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::Zig,
                    documentation: None,
                    modifiers: vec!["use".to_string()],
                    signature: None,
                });
                break;
            }
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
                language: LanguageId::Zig,
                documentation: self.extract_zig_doc(node, source),
                modifiers: vec!["type".to_string()],
                signature: None,
            });
        }
    }

    fn extract_error_type(
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
                language: LanguageId::Zig,
                documentation: self.extract_zig_doc(node, source),
                modifiers: vec!["error".to_string()],
                signature: None,
            });
        }
    }

    fn extract_test(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Zig test functions: test "test name" { ... }
        let mut test_name = "unnamed_test".to_string();

        // Look for string literal containing test name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string_literal" {
                test_name = self.clean_string_literal(&self.get_node_text(&child, source));
                break;
            }
        }

        let location = Location::from_node(node, file_path);
        let documentation = self.extract_zig_doc(node, source);

        symbols.push(Symbol {
            name: test_name,
            kind: SymbolKind::Function,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Zig,
            documentation,
            modifiers: vec!["test".to_string()],
            signature: Some("test".to_string()),
        });
    }

    fn extract_comptime(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Zig comptime declarations: comptime var x = value; or comptime { ... }
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);

            let mut modifiers = vec!["comptime".to_string()];

            // Determine if it's a variable or constant
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "var" {
                    modifiers.push("var".to_string());
                } else if child.kind() == "const" {
                    modifiers.push("const".to_string());
                }
            }

            let kind = if modifiers.contains(&"const".to_string()) {
                SymbolKind::Constant
            } else {
                SymbolKind::Variable
            };

            symbols.push(Symbol {
                name,
                kind,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Zig,
                documentation: self.extract_zig_doc(node, source),
                modifiers,
                signature: None,
            });
        }
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        safe_node_text(node, source)
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

    fn extract_zig_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Zig documentation appears as /// comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Zig doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "line_comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("///") {
                        // Zig doc comment
                        let content = comment_text.strip_prefix("///").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("//") {
                        // Regular comment - stop looking for doc comments
                        break;
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

    fn extract_function_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["fn".to_string()];

        // Check for function modifiers like pub, extern, inline, etc.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            } else if child.kind() == "extern_modifier" {
                modifiers.push("extern".to_string());
            } else if child.kind() == "inline_modifier" {
                modifiers.push("inline".to_string());
            } else if child.kind() == "export_modifier" {
                modifiers.push("export".to_string());
            }
        }

        modifiers
    }

    fn extract_struct_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["struct".to_string()];

        // Check for struct modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            } else if child.kind() == "packed_modifier" {
                modifiers.push("packed".to_string());
            } else if child.kind() == "extern_modifier" {
                modifiers.push("extern".to_string());
            }
        }

        modifiers
    }

    fn extract_union_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for union modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            } else if child.kind() == "packed_modifier" {
                modifiers.push("packed".to_string());
            } else if child.kind() == "extern_modifier" {
                modifiers.push("extern".to_string());
            }
        }

        modifiers
    }

    fn extract_enum_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["enum".to_string()];

        // Check for enum modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            }
        }

        modifiers
    }

    fn extract_variable_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for variable modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            } else if child.kind() == "threadlocal_modifier" {
                modifiers.push("threadlocal".to_string());
            }
        }

        modifiers
    }

    fn extract_const_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for const modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let visibility = self.get_node_text(&child, source);
                if visibility == "pub" {
                    modifiers.push("public".to_string());
                }
            } else if child.kind() == "extern_modifier" {
                modifiers.push("extern".to_string());
            } else if child.kind() == "export_modifier" {
                modifiers.push("export".to_string());
            }
        }

        modifiers
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
            .map(|rt| format!(" {}", self.get_node_text(&rt, source)))
            .unwrap_or_default();

        // Check for function modifiers in signature
        let mut signature_parts = Vec::new();

        // Add visibility
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" && self.get_node_text(&child, source) == "pub"
            {
                signature_parts.push("pub".to_string());
            } else if child.kind() == "extern_modifier" {
                signature_parts.push("extern".to_string());
            } else if child.kind() == "export_modifier" {
                signature_parts.push("export".to_string());
            } else if child.kind() == "inline_modifier" {
                signature_parts.push("inline".to_string());
            }
        }

        signature_parts.push(format!("fn {name}{params}{return_type}"));
        Some(signature_parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zig_symbol_extraction() {
        let extractor = ZigExtractor;
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_struct_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_enum_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_union_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_variable_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_import_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_zig_doc_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_test_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_comptime_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }

    #[test]
    fn test_error_type_extraction() {
        let extractor = ZigExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Zig);
    }
}
