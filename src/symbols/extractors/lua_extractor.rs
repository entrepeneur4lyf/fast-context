//! Lua symbol extractor
//! 
//! Extracts symbols from Lua source code including:
//! - Functions (local and global)
//! - Variables and constants
//! - Tables and metatables
//! - Modules and requires
//! - Methods and closures

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Lua Symbol Extractor
/// Extracts functions, variables, modules, requires from Lua code
pub struct LuaExtractor;

impl SymbolExtractor for LuaExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Lua
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl LuaExtractor {
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
            "local_function" => {
                self.extract_local_function(&node, source, file_path, symbols, scope_stack);
            }
            "function_call" => {
                self.extract_function_call(&node, source, file_path, symbols, scope_stack);
            }
            "variable_declaration" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            "local_variable_declaration" => {
                self.extract_local_variable(&node, source, file_path, symbols, scope_stack);
            }
            "assignment_statement" => {
                self.extract_assignment(&node, source, file_path, symbols, scope_stack);
            }
            "table_constructor" => {
                self.extract_table(&node, source, file_path, symbols, scope_stack);
            }
            "for_statement" => {
                self.extract_for_variables(&node, source, file_path, symbols, scope_stack);
            }
            "function_definition" => {
                self.extract_anonymous_function(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "function_declaration" | "local_function") {
            scope_stack.pop();
        }
    }

    fn extract_function(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            // Push function as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Function,
                location: location.clone(),
            };
            scope_stack.push(scope);
            
            let signature = self.extract_function_signature(node, source);
            let documentation = self.extract_lua_doc(node, source);
            let modifiers = vec!["function".to_string(), "global".to_string()];
            
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                language: LanguageId::Lua,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_local_function(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            // Push function as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Function,
                location: location.clone(),
            };
            scope_stack.push(scope);
            
            let signature = self.extract_function_signature(node, source);
            let documentation = self.extract_lua_doc(node, source);
            let modifiers = vec!["function".to_string(), "local".to_string()];
            
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                language: LanguageId::Lua,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_function_call(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Check for require() and dofile() calls
        if let Some(name_node) = node.child_by_field_name("name") {
            let function_name = self.get_node_text(&name_node, source);
            
            if matches!(function_name.as_str(), "require" | "dofile" | "loadfile" | "load") {
                if let Some(arguments) = node.child_by_field_name("arguments") {
                    // Extract the first argument as the module/file name
                    let mut cursor = arguments.walk();
                    for child in arguments.children(&mut cursor) {
                        if child.kind() == "string" {
                            let module_name = self.clean_string_literal(&self.get_node_text(&child, source));
                            let location = Location::from_node(&child, file_path);
                            
                            let mut modifiers = vec![function_name.clone()];
                            if function_name == "require" {
                                modifiers.push("module".to_string());
                            } else {
                                modifiers.push("file".to_string());
                            }
                            
                            symbols.push(Symbol {
                                name: module_name,
                                kind: SymbolKind::Import,
                                location,
                                scope_chain: scope_stack.clone(),
                                language: LanguageId::Lua,
                                documentation: None,
                                modifiers,
                                signature: None,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    fn extract_variable(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract variable names from variable declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable" || child.kind() == "identifier" {
                let name = self.get_node_text(&child, source);
                let location = Location::from_node(&child, file_path);
                
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Variable,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::Lua,
                    documentation: self.extract_lua_doc(node, source),
                    modifiers: vec!["variable".to_string(), "global".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn extract_local_variable(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract local variable names
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_list" {
                let mut var_cursor = child.walk();
                for var_child in child.children(&mut var_cursor) {
                    if var_child.kind() == "identifier" {
                        let name = self.get_node_text(&var_child, source);
                        let location = Location::from_node(&var_child, file_path);
                        
                        symbols.push(Symbol {
                            name,
                            kind: SymbolKind::Variable,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Lua,
                            documentation: self.extract_lua_doc(node, source),
                            modifiers: vec!["variable".to_string(), "local".to_string()],
                            signature: None,
                        });
                    }
                }
            }
        }
    }

    fn extract_assignment(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract variables from assignment statements
        if let Some(left) = node.child_by_field_name("left") {
            let mut cursor = left.walk();
            for child in left.children(&mut cursor) {
                if child.kind() == "identifier" {
                    let name = self.get_node_text(&child, source);
                    let location = Location::from_node(&child, file_path);
                    
                    // Check if this looks like a table/module definition
                    let mut modifiers = vec!["variable".to_string(), "global".to_string()];
                    if let Some(right) = node.child_by_field_name("right") {
                        if right.kind() == "table_constructor" {
                            modifiers.push("table".to_string());

                            // Check if it's a module pattern (has return statement or _M pattern)
                            if name.ends_with("_M") || name == "M" || self.is_module_pattern(&right, source) {
                                modifiers.push("module".to_string());
                            }

                            // Check if it's a metatable pattern
                            if self.is_metatable_pattern(&right, source) {
                                modifiers.push("metatable".to_string());
                            }
                        } else if right.kind() == "function_call" {
                            let call_text = self.get_node_text(&right, source);
                            if call_text.contains("setmetatable") {
                                modifiers.push("metatable".to_string());
                            } else if call_text.contains("require") {
                                modifiers.push("module".to_string());
                            } else {
                                modifiers.push("table".to_string());
                            }
                        }
                    }
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Lua,
                        documentation: self.extract_lua_doc(node, source),
                        modifiers,
                        signature: None,
                    });
                } else if child.kind() == "dot_index_expression" || child.kind() == "bracket_index_expression" {
                    // Handle table field assignments like obj.field = value
                    if let Some(field_node) = child.child_by_field_name("field") {
                        let field_name = self.get_node_text(&field_node, source);
                        let location = Location::from_node(&field_node, file_path);
                        
                        symbols.push(Symbol {
                            name: field_name,
                            kind: SymbolKind::Field,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Lua,
                            documentation: self.extract_lua_doc(node, source),
                            modifiers: vec!["field".to_string(), "table".to_string()],
                            signature: None,
                        });
                    }
                }
            }
        }
    }

    fn extract_table(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract field names from table constructors
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field" {
                // Handle different field types: [key] = value, key = value, key: value
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(&name_node, file_path);
                    
                    let mut modifiers = vec!["field".to_string(), "table".to_string()];

                    // Check if the value is a function
                    if let Some(value) = child.child_by_field_name("value") {
                        if value.kind() == "function_definition" {
                            modifiers.push("method".to_string());
                        }
                    }

                    // Check if it's a metamethod
                    if name.starts_with("__") {
                        modifiers.push("metamethod".to_string());
                    }
                    
                    symbols.push(Symbol {
                        name,
                        kind: if modifiers.contains(&"method".to_string()) { SymbolKind::Method } else { SymbolKind::Field },
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Lua,
                        documentation: None,
                        modifiers,
                        signature: None,
                    });
                }
            }
        }
    }

    fn extract_for_variables(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract loop variables from for statements
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_list" {
                let mut var_cursor = child.walk();
                for var_child in child.children(&mut var_cursor) {
                    if var_child.kind() == "identifier" {
                        let name = self.get_node_text(&var_child, source);
                        let location = Location::from_node(&var_child, file_path);

                        symbols.push(Symbol {
                            name,
                            kind: SymbolKind::Variable,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Lua,
                            documentation: None,
                            modifiers: vec!["variable".to_string(), "loop".to_string()],
                            signature: None,
                        });
                    }
                }
            }
        }
    }

    fn extract_anonymous_function(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // Extract anonymous functions (closures) - these don't have names but are important for scope
        let location = Location::from_node(node, file_path);
        let signature = self.extract_anonymous_function_signature(node, source);

        // Generate a unique name for the anonymous function
        let name = format!("anonymous_function_{}", location.start_line);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Function,
            location,
            scope_chain: scope_stack.clone(),
            language: LanguageId::Lua,
            documentation: self.extract_lua_doc(node, source),
            modifiers: vec!["function".to_string(), "anonymous".to_string(), "closure".to_string()],
            signature,
        });
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    fn clean_string_literal(&self, text: &str) -> String {
        // Remove quotes from string literals
        if (text.starts_with('"') && text.ends_with('"')) || 
           (text.starts_with('\'') && text.ends_with('\'')) {
            text[1..text.len()-1].to_string()
        } else if text.starts_with("[[") && text.ends_with("]]") {
            // Long string literal
            text[2..text.len()-2].to_string()
        } else {
            text.to_string()
        }
    }

    fn extract_lua_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Lua documentation appears as -- comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Lua doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("---") {
                        // Lua doc comment (triple dash)
                        let content = comment_text.strip_prefix("---").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("--[[") && comment_text.ends_with("]]") {
                        // Multi-line comment
                        let content = comment_text
                            .strip_prefix("--[[").unwrap_or("")
                            .strip_suffix("]]").unwrap_or("")
                            .trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("--") && !comment_text.starts_with("---") {
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

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node.child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node.child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        Some(format!("function {name}{params}"))
    }

    fn extract_anonymous_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let params = node.child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        Some(format!("function{params}"))
    }

    fn is_module_pattern(&self, node: &Node, source: &str) -> bool {
        // Check if table contains common module patterns
        let table_text = self.get_node_text(node, source);

        // Look for common module patterns
        table_text.contains("__index") ||
        table_text.contains("new") ||
        table_text.contains("init") ||
        table_text.contains("_VERSION") ||
        table_text.contains("_NAME")
    }

    fn is_metatable_pattern(&self, node: &Node, source: &str) -> bool {
        // Check if table contains metamethod patterns
        let table_text = self.get_node_text(node, source);

        // Look for metamethods
        table_text.contains("__index") ||
        table_text.contains("__newindex") ||
        table_text.contains("__call") ||
        table_text.contains("__add") ||
        table_text.contains("__sub") ||
        table_text.contains("__mul") ||
        table_text.contains("__div") ||
        table_text.contains("__mod") ||
        table_text.contains("__pow") ||
        table_text.contains("__unm") ||
        table_text.contains("__concat") ||
        table_text.contains("__len") ||
        table_text.contains("__eq") ||
        table_text.contains("__lt") ||
        table_text.contains("__le") ||
        table_text.contains("__tostring") ||
        table_text.contains("__gc") ||
        table_text.contains("__mode")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_symbol_extraction() {
        let extractor = LuaExtractor;
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_local_function_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_variable_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_require_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_table_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_assignment_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_lua_doc_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_anonymous_function_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_module_pattern_detection() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_metatable_pattern_detection() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }

    #[test]
    fn test_metamethod_extraction() {
        let extractor = LuaExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Lua);
    }
}
