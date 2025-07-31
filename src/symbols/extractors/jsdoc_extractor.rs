//! JSDoc symbol extractor
//! 
//! Extracts symbols from JSDoc comments including:
//! - @param annotations
//! - @returns annotations  
//! - @type annotations
//! - @typedef definitions
//! - @namespace declarations
//! - @class definitions
//! - @function declarations

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// JSDoc symbol extractor
pub struct JSDocExtractor;

impl SymbolExtractor for JSDocExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::JSDoc
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl JSDocExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            // @typedef definitions
            "typedef" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        let type_info = self.extract_type_info(&node, source);
                        
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Type,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::JSDoc,
                            documentation: self.extract_description(&node, source),
                            modifiers: vec!["typedef".to_string()],
                            signature: type_info,
                        });
                    }
                }
            }
            
            // @namespace declarations
            "namespace" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        
                        // Push namespace as scope
                        let scope = Scope {
                            name: name.clone(),
                            kind: SymbolKind::Namespace,
                            location: location.clone(),
                        };
                        scope_stack.push(scope);
                        
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Namespace,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::JSDoc,
                            documentation: self.extract_description(&node, source),
                            modifiers: vec!["namespace".to_string()],
                            signature: None,
                        });
                    }
                }
            }
            
            // @class definitions
            "class" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        
                        // Push class as scope
                        let scope = Scope {
                            name: name.clone(),
                            kind: SymbolKind::Class,
                            location: location.clone(),
                        };
                        scope_stack.push(scope);
                        
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Class,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::JSDoc,
                            documentation: self.extract_description(&node, source),
                            modifiers: vec!["class".to_string()],
                            signature: None,
                        });
                    }
                }
            }
            
            // @function declarations
            "function" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        let signature = self.extract_function_signature(&node, source);
                        
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::JSDoc,
                            documentation: self.extract_description(&node, source),
                            modifiers: vec!["function".to_string()],
                            signature,
                        });
                    }
                }
            }
            
            // @param annotations
            "param" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        let type_info = self.extract_type_info(&node, source);
                        
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Parameter,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::JSDoc,
                            documentation: self.extract_description(&node, source),
                            modifiers: vec!["param".to_string()],
                            signature: type_info,
                        });
                    }
                }
            }
            
            // @type annotations
            "type" => {
                let type_text = self.get_node_text(node, source);
                if !type_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    
                    symbols.push(Symbol {
                        name: type_text.clone(),
                        kind: SymbolKind::Type,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::JSDoc,
                        documentation: None,
                        modifiers: vec!["type".to_string()],
                        signature: None,
                    });
                }
            }
            
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }
    }

    fn get_node_text(&self, node: Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }
    
    fn extract_description(&self, node: &Node, source: &str) -> Option<String> {
        // Look for description field or text content
        if let Some(desc_node) = node.child_by_field_name("description") {
            let desc = self.get_node_text(desc_node, source);
            if !desc.is_empty() {
                return Some(desc);
            }
        }
        None
    }
    
    fn extract_type_info(&self, node: &Node, source: &str) -> Option<String> {
        // Look for type field
        if let Some(type_node) = node.child_by_field_name("type") {
            let type_text = self.get_node_text(type_node, source);
            if !type_text.is_empty() {
                return Some(type_text);
            }
        }
        None
    }
    
    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract function signature from JSDoc
        let mut signature_parts = Vec::new();
        
        // Look for parameters
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "param" {
                if let Some(param_name) = child.child_by_field_name("name") {
                    let name = self.get_node_text(param_name, source);
                    let type_info = self.extract_type_info(&child, source);
                    
                    if let Some(type_str) = type_info {
                        signature_parts.push(format!("{name}: {type_str}"));
                    } else {
                        signature_parts.push(name);
                    }
                }
            }
        }
        
        if !signature_parts.is_empty() {
            Some(format!("({})", signature_parts.join(", ")))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsdoc_symbol_extraction() {
        let extractor = JSDocExtractor;
        assert_eq!(extractor.language(), LanguageId::JSDoc);
    }
}
