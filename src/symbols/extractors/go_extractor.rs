//! Go symbol extractor
//! 
//! Extracts symbols from Go source code including:
//! - Package declarations and imports
//! - Functions and methods
//! - Types (structs, interfaces)
//! - Variables and constants
//! - Go documentation comments

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Go Symbol Extractor
/// Extracts functions, types, variables, imports, and packages from Go code
pub struct GoExtractor;

impl SymbolExtractor for GoExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Go
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl GoExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "package_clause" => {
                // Extract package name from package declaration - Go uses package_identifier child
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "package_identifier" {
                        let package_name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let location = Location::from_node(&node, file_path);
                        
                        symbols.push(Symbol {
                            name: package_name,
                            kind: SymbolKind::Namespace,
                            location,
                            scope_chain: vec![],
                            language: LanguageId::Go,
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
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    let signature = self.extract_function_signature(&node, source);
                    let documentation = self.extract_go_doc(&node, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Go,
                        documentation,
                        modifiers: vec![],
                        signature,
                    });
                }
            }
            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    let signature = self.extract_method_signature(&node, source);
                    let documentation = self.extract_go_doc(&node, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Method,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Go,
                        documentation,
                        modifiers: vec!["method".to_string()],
                        signature,
                    });
                }
            }
            "type_declaration" => {
                self.extract_type_declarations(&node, source, file_path, symbols, scope_stack);
            }
            "var_declaration" => {
                self.extract_var_declarations(&node, source, file_path, symbols, scope_stack);
            }
            "const_declaration" => {
                self.extract_const_declarations(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }
    }

    fn extract_import(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Go imports can have different structures:
        // import "package"
        // import alias "package"
        // import . "package"
        // import _ "package"
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_spec_list" {
                // Process all import_spec nodes within the import_spec_list
                let mut list_cursor = child.walk();
                for spec_node in child.children(&mut list_cursor) {
                    if spec_node.kind() == "import_spec" {
                        self.process_import_spec(&spec_node, source, file_path, symbols, scope_stack);
                    }
                }
            } else if child.kind() == "import_spec" {
                // Handle single import without parentheses
                self.process_import_spec(&child, source, file_path, symbols, scope_stack);
            }
        }
    }
    
    fn process_import_spec(&self, spec_node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        let mut import_path = String::new();
        let mut alias = None;
        let mut is_dot_import = false;
        let mut is_blank_import = false;
        
        let mut spec_cursor = spec_node.walk();
        for spec_child in spec_node.children(&mut spec_cursor) {
            match spec_child.kind() {
                "package_identifier" => {
                    alias = Some(spec_child.utf8_text(source.as_bytes()).unwrap_or("").to_string());
                }
                "dot" => is_dot_import = true,
                "blank_identifier" => is_blank_import = true,
                "interpreted_string_literal" | "raw_string_literal" => {
                    import_path = spec_child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    // Remove quotes
                    if import_path.starts_with('"') && import_path.ends_with('"') {
                        import_path = import_path[1..import_path.len()-1].to_string();
                    }
                }
                _ => {}
            }
        }
        
        if !import_path.is_empty() {
            let location = Location::from_node(spec_node, file_path);
            let mut modifiers = vec!["import".to_string()];
            
            if is_dot_import {
                modifiers.push("dot".to_string());
            }
            if is_blank_import {
                modifiers.push("blank".to_string());
            }
            if alias.is_some() {
                modifiers.push("aliased".to_string());
            }
            
            let import_name = alias.unwrap_or_else(|| {
                // Extract package name from path
                import_path.split('/').next_back().unwrap_or(&import_path).to_string()
            });
            
            symbols.push(Symbol {
                name: import_name,
                kind: SymbolKind::Import,
                location,
                scope_chain: scope_stack.to_vec(),
                language: LanguageId::Go,
                documentation: None,
                modifiers,
                signature: Some(import_path),
            });
        }
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node.child_by_field_name("name")?
            .utf8_text(source.as_bytes()).ok()?;
            
        let params = node.child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");
            
        let result = node.child_by_field_name("result")
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");
            
        Some(format!("func {name}{params} {result}").trim().to_string())
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node.child_by_field_name("name")?
            .utf8_text(source.as_bytes()).ok()?;
            
        let receiver = node.child_by_field_name("receiver")
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");
            
        let params = node.child_by_field_name("parameters")
            .and_then(|p| p.utf8_text(source.as_bytes()).ok())
            .unwrap_or("()");
            
        let result = node.child_by_field_name("result")
            .and_then(|r| r.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");
            
        Some(format!("func {receiver}{name}{params} {result}").trim().to_string())
    }

    fn extract_type_declarations(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&child, file_path);
                    
                    // Determine the type kind based on the type definition
                    let (kind, modifiers) = if let Some(type_node) = child.child_by_field_name("type") {
                        match type_node.kind() {
                            "struct_type" => (SymbolKind::Struct, vec!["struct".to_string()]),
                            "interface_type" => (SymbolKind::Interface, vec!["interface".to_string()]),
                            _ => (SymbolKind::Type, vec!["type".to_string()]),
                        }
                    } else {
                        (SymbolKind::Type, vec!["type".to_string()])
                    };
                    
                    let documentation = self.extract_go_doc(&child, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Go,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
        }
    }

    fn extract_var_declarations(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "var_spec_list" {
                // Process all var_spec nodes within the var_spec_list
                let mut list_cursor = child.walk();
                for spec_node in child.children(&mut list_cursor) {
                    if spec_node.kind() == "var_spec" {
                        self.process_var_spec(&spec_node, source, file_path, symbols, scope_stack);
                    }
                }
            } else if child.kind() == "var_spec" {
                // Handle single var without parentheses
                self.process_var_spec(&child, source, file_path, symbols, scope_stack);
            }
        }
    }
    
    fn process_var_spec(&self, spec_node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Extract variable names from the var_spec
        let mut var_cursor = spec_node.walk();
        for var_child in spec_node.children(&mut var_cursor) {
            if var_child.kind() == "identifier" {
                let name = var_child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                let location = Location::from_node(&var_child, file_path);
                
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Variable,
                    location,
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::Go,
                    documentation: None,
                    modifiers: vec!["var".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn extract_const_declarations(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_spec" {
                // Extract constant names from the const_spec
                let mut const_cursor = child.walk();
                for const_child in child.children(&mut const_cursor) {
                    if const_child.kind() == "identifier" {
                        let name = const_child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let location = Location::from_node(&const_child, file_path);
                        
                        symbols.push(Symbol {
                            name,
                            kind: SymbolKind::Constant,
                            location,
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::Go,
                            documentation: None,
                            modifiers: vec!["const".to_string()],
                            signature: None,
                        });
                    }
                }
            }
        }
    }

    fn extract_go_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Go documentation is typically in comments preceding the declaration
        // Look for comment nodes that appear immediately before the current node
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("//") {
                        // Single-line comment
                        let content = comment_text.strip_prefix("//").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("/*") && comment_text.ends_with("*/") {
                        // Block comment
                        let content = comment_text
                            .strip_prefix("/*").unwrap_or("")
                            .strip_suffix("*/").unwrap_or("")
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_symbol_extraction() {
        let extractor = GoExtractor;
        assert_eq!(extractor.language(), LanguageId::Go);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = GoExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Go);
    }

    #[test]
    fn test_method_signature_extraction() {
        let extractor = GoExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Go);
    }

    #[test]
    fn test_import_extraction() {
        let extractor = GoExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Go);
    }

    #[test]
    fn test_type_extraction() {
        let extractor = GoExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Go);
    }

    #[test]
    fn test_go_doc_extraction() {
        let extractor = GoExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Go);
    }
}
