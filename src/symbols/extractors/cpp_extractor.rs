//! C++ symbol extractor
//!
//! Extracts symbols from C++ source code including:
//! - Classes, structs, and unions
//! - Functions and methods
//! - Variables and fields
//! - Namespaces and includes
//! - Templates and specializations

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// C++ Symbol Extractor
/// Extracts classes, functions, variables, namespaces, and includes from C++ code
pub struct CppExtractor;

impl SymbolExtractor for CppExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Cpp
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

impl CppExtractor {
    #[allow(clippy::ptr_arg)]
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "preproc_include" => {
                self.extract_include(&node, source, file_path, symbols, scope_stack);
            }
            "namespace_definition" => {
                self.extract_namespace(&node, source, file_path, symbols, scope_stack);
            }
            "class_specifier" => {
                self.extract_class(&node, source, file_path, symbols, scope_stack);
            }
            "struct_specifier" => {
                self.extract_struct(&node, source, file_path, symbols, scope_stack);
            }
            "union_specifier" => {
                self.extract_union(&node, source, file_path, symbols, scope_stack);
            }
            "function_definition" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "function_declarator" => {
                self.extract_function_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "field_declaration" => {
                self.extract_field(&node, source, file_path, symbols, scope_stack);
            }
            "declaration" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            "template_declaration" => {
                self.extract_template(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.extract_from_node(child, source, file_path, symbols, scope_stack);
            }
        }
    }

    #[allow(clippy::ptr_arg)]
    fn extract_include(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        if let Some(path_node) = node.child_by_field_name("path") {
            let include_path = self.get_node_text(&path_node, source);
            let clean_path = include_path.trim_matches('"').trim_matches('<').trim_matches('>');
            
            symbols.push(Symbol {
                name: clean_path.to_string(),
                kind: SymbolKind::Import,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers: vec!["include".to_string()],
                signature: Some(format!("#include {include_path}")),
                documentation: None,
            });
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            
            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Module,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers: vec!["namespace".to_string()],
                signature: Some(format!("namespace {name}")),
                documentation: None,
            });

            // Push namespace scope
            scope_stack.push(Scope {
                name: name.clone(),
                kind: SymbolKind::Module,
                location: self.node_to_location(node, file_path),
            });

            // Process namespace body
            if let Some(body) = node.child_by_field_name("body") {
                self.extract_from_node(body, source, file_path, symbols, scope_stack);
            }

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
            let mut modifiers = vec!["class".to_string()];
            
            // Check for template parameters
            if let Some(_template_params) = node.child_by_field_name("template_parameters") {
                modifiers.push("template".to_string());
            }
            
            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers,
                signature: Some(format!("class {name}")),
                documentation: None,
            });

            // Push class scope
            scope_stack.push(Scope {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: self.node_to_location(node, file_path),
            });

            // Process class body
            if let Some(body) = node.child_by_field_name("body") {
                self.extract_from_node(body, source, file_path, symbols, scope_stack);
            }

            scope_stack.pop();
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
            
            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Struct,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers: vec!["struct".to_string()],
                signature: Some(format!("struct {name}")),
                documentation: None,
            });

            // Push struct scope
            scope_stack.push(Scope {
                name: name.clone(),
                kind: SymbolKind::Struct,
                location: self.node_to_location(node, file_path),
            });

            // Process struct body
            if let Some(body) = node.child_by_field_name("body") {
                self.extract_from_node(body, source, file_path, symbols, scope_stack);
            }

            scope_stack.pop();
        }
    }

    #[allow(clippy::ptr_arg)]
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
            
            symbols.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Union,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers: vec!["union".to_string()],
                signature: Some(format!("union {name}")),
                documentation: None,
            });
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
        if let Some(declarator) = node.child_by_field_name("declarator") {
            self.extract_function_from_declarator(&declarator, source, file_path, symbols, scope_stack, true);
        }
    }

    fn extract_function_declaration(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        self.extract_function_from_declarator(node, source, file_path, symbols, scope_stack, false);
    }

    #[allow(clippy::ptr_arg)]
    fn extract_function_from_declarator(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
        is_definition: bool,
    ) {
        if let Some(name_node) = node.child_by_field_name("declarator") {
            let name = self.get_node_text(&name_node, source);
            let mut modifiers = vec![];
            
            if is_definition {
                modifiers.push("definition".to_string());
            } else {
                modifiers.push("declaration".to_string());
            }
            
            // Check if it's a method (inside a class/struct)
            let kind = if scope_stack.iter().any(|s| matches!(s.kind, SymbolKind::Class | SymbolKind::Struct)) {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            };
            
            symbols.push(Symbol {
                name: name.clone(),
                kind,
                location: self.node_to_location(node, file_path),
                scope_chain: scope_stack.clone(),
                language: LanguageId::Cpp,
                modifiers,
                signature: Some(self.get_node_text(node, source)),
                documentation: None,
            });
        }
    }

    #[allow(clippy::ptr_arg)]
    fn extract_field(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Extract field declarations (member variables)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "field_identifier" {
                    let name = self.get_node_text(&child, source);
                    
                    symbols.push(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Field,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Cpp,
                        modifiers: vec!["field".to_string()],
                        signature: Some(self.get_node_text(node, source)),
                        documentation: None,
                    });
                }
            }
        }
    }

    #[allow(clippy::ptr_arg)]
    fn extract_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Extract variable declarations
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    let name = self.get_node_text(&child, source);
                    
                    symbols.push(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        location: self.node_to_location(&child, file_path),
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Cpp,
                        modifiers: vec!["variable".to_string()],
                        signature: Some(self.get_node_text(node, source)),
                        documentation: None,
                    });
                }
            }
        }
    }

    fn extract_template(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Process the template declaration
        if let Some(declaration) = node.child_by_field_name("declaration") {
            self.extract_from_node(declaration, source, file_path, symbols, scope_stack);
        }
    }

    // Helper methods
    fn get_node_text(&self, node: &Node, source: &str) -> String {
        source[node.start_byte()..node.end_byte()].to_string()
    }

    fn node_to_location(&self, node: &Node, file_path: &str) -> Location {
        Location {
            file_path: file_path.to_string(),
            start_line: node.start_position().row + 1,
            start_column: node.start_position().column + 1,
            end_line: node.end_position().row + 1,
            end_column: node.end_position().column + 1,
        }
    }


}
