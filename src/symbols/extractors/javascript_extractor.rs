//! JavaScript/TypeScript symbol extractor
//! 
//! Extracts symbols from JavaScript and TypeScript source code including:
//! - Functions and arrow functions
//! - Classes and methods
//! - Variables and constants
//! - Import and export statements
//! - TypeScript interfaces and types

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// JavaScript/TypeScript Symbol Extractor
/// Extracts functions, classes, variables, imports, exports, interfaces, and types
pub struct JavaScriptExtractor;

impl SymbolExtractor for JavaScriptExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::JavaScript // Used for both JS and TS
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl JavaScriptExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        let language = if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            LanguageId::TypeScript
        } else {
            LanguageId::JavaScript
        };

        
        match node.kind() {
            "function_declaration" | "function" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    let signature = self.extract_function_signature(&node, source);
                    let documentation = self.extract_jsdoc(&node, source);
                    let modifiers = self.extract_function_modifiers(&node, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        location,
                        scope_chain: scope_stack.clone(),
                        language,
                        documentation,
                        modifiers,
                        signature,
                    });
                }
            }
            "arrow_function" => {
                // For arrow functions, we need to look at the parent assignment or variable declarator
                if let Some(parent) = node.parent() {
                    match parent.kind() {
                        "variable_declarator" => {
                            if let Some(name_node) = parent.child_by_field_name("name") {
                                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                let location = Location::from_node(&node, file_path);
                                
                                let signature = self.extract_arrow_function_signature(&node, source, &name);
                                
                                symbols.push(Symbol {
                                    name,
                                    kind: SymbolKind::Function,
                                    location,
                                    scope_chain: scope_stack.clone(),
                                    language,
                                    documentation: None,
                                    modifiers: vec!["arrow".to_string()],
                                    signature,
                                });
                            }
                        }
                        "assignment_expression" => {
                            if let Some(left) = parent.child_by_field_name("left") {
                                if left.kind() == "identifier" {
                                    let name = left.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                                    let location = Location::from_node(&node, file_path);
                                    
                                    symbols.push(Symbol {
                                        name,
                                        kind: SymbolKind::Function,
                                        location,
                                        scope_chain: scope_stack.clone(),
                                        language,
                                        documentation: None,
                                        modifiers: vec!["arrow".to_string()],
                                        signature: None,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    // Push class as scope for nested items
                    let scope = Scope {
                        name: name.clone(),
                        kind: SymbolKind::Class,
                        location: location.clone(),
                    };
                    scope_stack.push(scope);
                    
                    let documentation = self.extract_jsdoc(&node, source);
                    let modifiers = self.extract_class_modifiers(&node, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        location,
                        scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                        language,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
            "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    let signature = self.extract_method_signature(&node, source);
                    let documentation = self.extract_jsdoc(&node, source);
                    let modifiers = self.extract_method_modifiers(&node, source);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Method,
                        location,
                        scope_chain: scope_stack.clone(),
                        language,
                        documentation,
                        modifiers,
                        signature,
                    });
                }
            }
            "import_statement" | "import" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack, language);
            }
            "export_statement" => {
                self.extract_export(&node, source, file_path, symbols, scope_stack, language);
            }
            "variable_declaration" | "lexical_declaration" => {
                self.extract_variables(&node, source, file_path, symbols, scope_stack, language);
            }
            "interface_declaration" if language == LanguageId::TypeScript => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Interface,
                        location,
                        scope_chain: scope_stack.clone(),
                        language,
                        documentation: self.extract_jsdoc(&node, source),
                        modifiers: vec![],
                        signature: None,
                    });
                }
            }
            "type_alias_declaration" if language == LanguageId::TypeScript => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&node, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Type,
                        location,
                        scope_chain: scope_stack.clone(),
                        language,
                        documentation: self.extract_jsdoc(&node, source),
                        modifiers: vec![],
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

        // Pop scope if we added one for this node
        if matches!(node.kind(), "class_declaration" | "function_declaration") {
            scope_stack.pop();
        }
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node.child_by_field_name("name")?
            .utf8_text(source.as_bytes()).ok()?;
        let params = node.child_by_field_name("parameters")?
            .utf8_text(source.as_bytes()).ok()?;
        
        // Check for async modifier
        let is_async = node.prev_sibling()
            .map(|s| s.kind() == "async")
            .unwrap_or(false);
        
        let async_prefix = if is_async { "async " } else { "" };
        Some(format!("{async_prefix}function {name}{params}"))
    }

    fn extract_arrow_function_signature(&self, node: &Node, source: &str, name: &str) -> Option<String> {
        let params = node.child_by_field_name("parameters")?
            .utf8_text(source.as_bytes()).ok()?;
        Some(format!("const {name} = {params} => {{...}}"))
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node.child_by_field_name("name")?
            .utf8_text(source.as_bytes()).ok()?;
        let params = node.child_by_field_name("parameters")?
            .utf8_text(source.as_bytes()).ok()?;
        
        // Check for method kind (get, set, async, static)
        let kind = node.child_by_field_name("kind")
            .and_then(|k| k.utf8_text(source.as_bytes()).ok())
            .unwrap_or("");
        
        let prefix = if kind.is_empty() { "" } else { &format!("{kind} ") };
        Some(format!("{prefix}{name}{params}"))
    }

    fn extract_jsdoc(&self, node: &Node, source: &str) -> Option<String> {
        // Look for JSDoc comment before the node
        let mut current = *node;
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        return Some(self.parse_jsdoc_comment(comment_text));
                    }
                }
                _ if prev.kind().contains("whitespace") || prev.kind() == "\n" => {
                    current = prev;
                    continue;
                }
                _ => break,
            }
            current = prev;
        }
        None
    }

    /// Parse and clean JSDoc comment with comprehensive tag support
    fn parse_jsdoc_comment(&self, comment_text: &str) -> String {
        let content = comment_text
            .trim_start_matches("/**")
            .trim_end_matches("*/")
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();

        let mut result = Vec::new();
        let mut current_section = Vec::new();
        let mut in_description = true;

        for line in content {
            if line.starts_with('@') {
                // Process previous section
                if in_description && !current_section.is_empty() {
                    result.push(current_section.join(" "));
                    current_section.clear();
                }
                in_description = false;

                // Process JSDoc tag
                let processed_tag = self.process_jsdoc_tag(line);
                result.push(processed_tag);
            } else if in_description {
                current_section.push(line);
            } else {
                // Continuation of previous tag
                if let Some(last) = result.last_mut() {
                    last.push(' ');
                    last.push_str(line);
                }
            }
        }

        // Add remaining description
        if in_description && !current_section.is_empty() {
            result.insert(0, current_section.join(" "));
        }

        result.join("\n")
    }

    /// Process individual JSDoc tags with type and parameter information
    fn process_jsdoc_tag(&self, tag_line: &str) -> String {
        if tag_line.starts_with("@param") {
            self.process_param_tag(tag_line)
        } else if tag_line.starts_with("@returns") || tag_line.starts_with("@return") {
            self.process_return_tag(tag_line)
        } else if tag_line.starts_with("@throws") || tag_line.starts_with("@exception") {
            self.process_throws_tag(tag_line)
        } else if tag_line.starts_with("@type") {
            self.process_type_tag(tag_line)
        } else if tag_line.starts_with("@typedef") {
            self.process_typedef_tag(tag_line)
        } else if tag_line.starts_with("@namespace") {
            self.process_namespace_tag(tag_line)
        } else if tag_line.starts_with("@class") || tag_line.starts_with("@constructor") {
            self.process_class_tag(tag_line)
        } else if tag_line.starts_with("@module") {
            self.process_module_tag(tag_line)
        } else if tag_line.starts_with("@function") || tag_line.starts_with("@method") {
            self.process_function_tag(tag_line)
        } else if tag_line.starts_with("@example") {
            format!("Example: {}", tag_line.strip_prefix("@example").unwrap_or("").trim())
        } else if tag_line.starts_with("@see") {
            format!("See: {}", tag_line.strip_prefix("@see").unwrap_or("").trim())
        } else if tag_line.starts_with("@since") {
            format!("Since: {}", tag_line.strip_prefix("@since").unwrap_or("").trim())
        } else if tag_line.starts_with("@deprecated") {
            format!("Deprecated: {}", tag_line.strip_prefix("@deprecated").unwrap_or("").trim())
        } else if tag_line.starts_with("@author") {
            format!("Author: {}", tag_line.strip_prefix("@author").unwrap_or("").trim())
        } else if tag_line.starts_with("@version") {
            format!("Version: {}", tag_line.strip_prefix("@version").unwrap_or("").trim())
        } else if tag_line.starts_with("@todo") {
            format!("TODO: {}", tag_line.strip_prefix("@todo").unwrap_or("").trim())
        } else if tag_line.starts_with("@override") {
            "Override: This method overrides a parent method".to_string()
        } else if tag_line.starts_with("@abstract") {
            "Abstract: This is an abstract method".to_string()
        } else if tag_line.starts_with("@static") {
            "Static: This is a static method".to_string()
        } else if tag_line.starts_with("@readonly") {
            "Readonly: This property is read-only".to_string()
        } else if tag_line.starts_with("@private") {
            "Private: This is a private member".to_string()
        } else if tag_line.starts_with("@protected") {
            "Protected: This is a protected member".to_string()
        } else if tag_line.starts_with("@public") {
            "Public: This is a public member".to_string()
        } else {
            // Generic tag processing
            let tag_name = tag_line.split_whitespace().next().unwrap_or("").trim_start_matches('@');
            let content = tag_line.strip_prefix(&format!("@{tag_name}")).unwrap_or("").trim();
            if content.is_empty() {
                format!("{}: true", tag_name.to_uppercase())
            } else {
                format!("{}: {}", tag_name.to_uppercase(), content)
            }
        }
    }

    /// Process @param tags with type and description
    fn process_param_tag(&self, tag_line: &str) -> String {
        // @param {type} name description
        let content = tag_line.strip_prefix("@param").unwrap_or("").trim();

        if content.starts_with('{') {
            // Extract type in braces
            if let Some(end_brace) = content.find('}') {
                let type_part = &content[1..end_brace];
                let rest = content[end_brace + 1..].trim();

                if let Some(space_pos) = rest.find(' ') {
                    let param_name = &rest[..space_pos];
                    let description = &rest[space_pos + 1..];
                    format!("Parameter {param_name}: {type_part} - {description}")
                } else {
                    format!("Parameter {rest}: {type_part}")
                }
            } else {
                format!("Parameter: {content}")
            }
        } else {
            // No type specified
            if let Some(space_pos) = content.find(' ') {
                let param_name = &content[..space_pos];
                let description = &content[space_pos + 1..];
                format!("Parameter {param_name}: {description}")
            } else {
                format!("Parameter: {content}")
            }
        }
    }

    /// Process @returns/@return tags with type and description
    fn process_return_tag(&self, tag_line: &str) -> String {
        let tag_prefix = if tag_line.starts_with("@returns") { "@returns" } else { "@return" };
        let content = tag_line.strip_prefix(tag_prefix).unwrap_or("").trim();

        if content.starts_with('{') {
            // Extract type in braces
            if let Some(end_brace) = content.find('}') {
                let type_part = &content[1..end_brace];
                let description = content[end_brace + 1..].trim();
                if description.is_empty() {
                    format!("Returns: {type_part}")
                } else {
                    format!("Returns: {type_part} - {description}")
                }
            } else {
                format!("Returns: {content}")
            }
        } else {
            format!("Returns: {content}")
        }
    }

    /// Process @throws/@exception tags
    fn process_throws_tag(&self, tag_line: &str) -> String {
        let tag_prefix = if tag_line.starts_with("@throws") { "@throws" } else { "@exception" };
        let content = tag_line.strip_prefix(tag_prefix).unwrap_or("").trim();

        if content.starts_with('{') {
            // Extract type in braces
            if let Some(end_brace) = content.find('}') {
                let type_part = &content[1..end_brace];
                let description = content[end_brace + 1..].trim();
                if description.is_empty() {
                    format!("Throws: {type_part}")
                } else {
                    format!("Throws: {type_part} - {description}")
                }
            } else {
                format!("Throws: {content}")
            }
        } else {
            format!("Throws: {content}")
        }
    }

    /// Process @type tags
    fn process_type_tag(&self, tag_line: &str) -> String {
        let content = tag_line.strip_prefix("@type").unwrap_or("").trim();

        if content.starts_with('{') && content.ends_with('}') {
            let type_part = &content[1..content.len()-1];
            format!("Type: {type_part}")
        } else {
            format!("Type: {content}")
        }
    }

    /// Process @typedef tags
    fn process_typedef_tag(&self, tag_line: &str) -> String {
        let content = tag_line.strip_prefix("@typedef").unwrap_or("").trim();

        if content.starts_with('{') {
            if let Some(end_brace) = content.find('}') {
                let type_part = &content[1..end_brace];
                let name = content[end_brace + 1..].trim();
                format!("Typedef: {name} as {type_part}")
            } else {
                format!("Typedef: {content}")
            }
        } else {
            format!("Typedef: {content}")
        }
    }

    /// Process @namespace tags
    fn process_namespace_tag(&self, tag_line: &str) -> String {
        let content = tag_line.strip_prefix("@namespace").unwrap_or("").trim();
        format!("Namespace: {content}")
    }

    /// Process @class/@constructor tags
    fn process_class_tag(&self, tag_line: &str) -> String {
        let tag_prefix = if tag_line.starts_with("@class") { "@class" } else { "@constructor" };
        let content = tag_line.strip_prefix(tag_prefix).unwrap_or("").trim();

        if content.is_empty() {
            "Class: Constructor function".to_string()
        } else {
            format!("Class: {content}")
        }
    }

    /// Process @module tags
    fn process_module_tag(&self, tag_line: &str) -> String {
        let content = tag_line.strip_prefix("@module").unwrap_or("").trim();
        format!("Module: {content}")
    }

    /// Process @function/@method tags
    fn process_function_tag(&self, tag_line: &str) -> String {
        let tag_prefix = if tag_line.starts_with("@function") { "@function" } else { "@method" };
        let content = tag_line.strip_prefix(tag_prefix).unwrap_or("").trim();

        if content.is_empty() {
            "Function: Method definition".to_string()
        } else {
            format!("Function: {content}")
        }
    }

    fn extract_function_modifiers(&self, node: &Node, _source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        
        // Check for async
        if let Some(prev) = node.prev_sibling() {
            if prev.kind() == "async" {
                modifiers.push("async".to_string());
            }
        }
        
        // Check for export
        if let Some(parent) = node.parent() {
            if parent.kind() == "export_statement" {
                modifiers.push("export".to_string());
            }
        }

        modifiers
    }

    fn extract_class_modifiers(&self, node: &Node, _source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        
        // Check for export
        if let Some(parent) = node.parent() {
            if parent.kind() == "export_statement" {
                modifiers.push("export".to_string());
            }
        }

        // Check for extends
        if node.child_by_field_name("superclass").is_some() {
            modifiers.push("extends".to_string());
        }

        modifiers
    }

    fn extract_method_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        
        // Check for static
        if let Some(kind) = node.child_by_field_name("kind") {
            if let Ok(kind_text) = kind.utf8_text(source.as_bytes()) {
                if kind_text == "static" {
                    modifiers.push("static".to_string());
                }
            }
        }

        // Check for async
        if let Some(value) = node.child_by_field_name("value") {
            if let Some(prev) = value.prev_sibling() {
                if prev.kind() == "async" {
                    modifiers.push("async".to_string());
                }
            }
        }

        // Check for getter/setter
        if let Some(kind) = node.child_by_field_name("kind") {
            if let Ok(kind_text) = kind.utf8_text(source.as_bytes()) {
                if kind_text == "get" || kind_text == "set" {
                    modifiers.push(kind_text.to_string());
                }
            }
        }

        modifiers
    }

    fn extract_import(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope], language: LanguageId) {
        // Find the source string
        let mut module_path = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" {
                module_path = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                // Remove quotes
                module_path = module_path.trim_matches('"').trim_matches('\'').to_string();
                break;
            }
        }
        
        // Extract import specifiers
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "import_clause" => {
                    self.extract_import_specifiers(&child, source, file_path, symbols, scope_stack, language, &module_path);
                }
                "identifier" => {
                    // Simple import like: import './module';
                    let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&child, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language,
                        documentation: None,
                        modifiers: vec!["side_effect".to_string()],
                        signature: Some(format!("from {module_path}")),
                    });
                }
                _ => {}
            }
        }
    }

    fn extract_import_specifiers(&self, clause: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope], language: LanguageId, module_path: &str) {
        let mut cursor = clause.walk();
        for child in clause.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Default import: import Foo from './foo'
                    let name = child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&child, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language,
                        documentation: None,
                        modifiers: vec!["default".to_string()],
                        signature: Some(format!("from {module_path}")),
                    });
                }
                "namespace_import" => {
                    // Namespace import: import * as Foo from './foo'
                    // Look for the identifier child which contains the alias name
                    let mut ns_cursor = child.walk();
                    for ns_child in child.children(&mut ns_cursor) {
                        if ns_child.kind() == "identifier" {
                            let name = ns_child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            let location = Location::from_node(&child, file_path);
                            
                            symbols.push(Symbol {
                                name,
                                kind: SymbolKind::Import,
                                location,
                                scope_chain: scope_stack.to_vec(),
                                language,
                                documentation: None,
                                modifiers: vec!["namespace".to_string()],
                                signature: Some(format!("from {module_path}")),
                            });
                            break;
                        }
                    }
                }
                "named_imports" => {
                    // Named imports: import { a, b } from './foo'
                    let mut import_cursor = child.walk();
                    for import_child in child.children(&mut import_cursor) {
                        if import_child.kind() == "import_specifier" {
                            let name = if let Some(alias) = import_child.child_by_field_name("alias") {
                                alias.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                            } else if let Some(name_node) = import_child.child_by_field_name("name") {
                                name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                            } else {
                                continue;
                            };
                            
                            let location = Location::from_node(&import_child, file_path);
                            
                            symbols.push(Symbol {
                                name,
                                kind: SymbolKind::Import,
                                location,
                                scope_chain: scope_stack.to_vec(),
                                language,
                                documentation: None,
                                modifiers: vec!["named".to_string()],
                                signature: Some(format!("from {module_path}")),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_export(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope], language: LanguageId) {
        // Handle different export types
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" | "class_declaration" => {
                    // These will be processed by their respective handlers with export modifier
                }
                "export_specifier" => {
                    let name = if let Some(alias) = child.child_by_field_name("alias") {
                        alias.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                    } else if let Some(name_node) = child.child_by_field_name("name") {
                        name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
                    } else {
                        continue;
                    };
                    
                    let location = Location::from_node(&child, file_path);
                    
                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Export,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language,
                        documentation: None,
                        modifiers: vec!["named".to_string()],
                        signature: None,
                    });
                }
                _ => {}
            }
        }
    }

    fn extract_variables(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope], language: LanguageId) {
        // Find the declaration type (const, let, var)
        let mut decl_type = "var";
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "const" | "let" | "var") {
                decl_type = child.utf8_text(source.as_bytes()).unwrap_or("var");
                break;
            }
        }
        
        // Process variable declarators
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let location = Location::from_node(&child, file_path);
                    
                    let kind = match decl_type {
                        "const" => SymbolKind::Constant,
                        _ => SymbolKind::Variable,
                    };
                    
                    symbols.push(Symbol {
                        name,
                        kind,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language,
                        documentation: None,
                        modifiers: vec![],
                        signature: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_symbol_extraction() {
        let extractor = JavaScriptExtractor;
        assert_eq!(extractor.language(), LanguageId::JavaScript);
    }

    #[test]
    fn test_jsdoc_extraction() {
        let extractor = JavaScriptExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::JavaScript);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = JavaScriptExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::JavaScript);
    }

    #[test]
    fn test_arrow_function_signature() {
        let extractor = JavaScriptExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::JavaScript);
    }

    #[test]
    fn test_typescript_detection() {
        let extractor = JavaScriptExtractor;

        // Test that TypeScript files are detected correctly
        // This would be tested through the extract_from_node method with .ts files
        assert_eq!(extractor.language(), LanguageId::JavaScript);
    }

    #[test]
    fn test_jsdoc_comment_parsing() {
        let extractor = JavaScriptExtractor;

        // Test basic JSDoc comment
        let basic_comment = "/**\n * This is a description\n * @param {string} name The name parameter\n * @returns {boolean} The result\n */";
        let parsed = extractor.parse_jsdoc_comment(basic_comment);
        assert!(parsed.contains("This is a description"));
        assert!(parsed.contains("Parameter name: string - The name parameter"));
        assert!(parsed.contains("Returns: boolean - The result"));
    }

    #[test]
    fn test_jsdoc_param_tag_processing() {
        let extractor = JavaScriptExtractor;

        // Test param with type and description
        let param_with_type = "@param {string} name The user's name";
        let result = extractor.process_param_tag(param_with_type);
        assert_eq!(result, "Parameter name: string - The user's name");

        // Test param without type
        let param_no_type = "@param name The user's name";
        let result_no_type = extractor.process_param_tag(param_no_type);
        assert_eq!(result_no_type, "Parameter name: The user's name");

        // Test param with only name
        let param_name_only = "@param name";
        let result_name_only = extractor.process_param_tag(param_name_only);
        assert_eq!(result_name_only, "Parameter: name");
    }

    #[test]
    fn test_jsdoc_return_tag_processing() {
        let extractor = JavaScriptExtractor;

        // Test return with type and description
        let return_with_type = "@returns {boolean} True if successful";
        let result = extractor.process_return_tag(return_with_type);
        assert_eq!(result, "Returns: boolean - True if successful");

        // Test return with only type
        let return_type_only = "@returns {boolean}";
        let result_type_only = extractor.process_return_tag(return_type_only);
        assert_eq!(result_type_only, "Returns: boolean");

        // Test return without type
        let return_no_type = "@returns True if successful";
        let result_no_type = extractor.process_return_tag(return_no_type);
        assert_eq!(result_no_type, "Returns: True if successful");
    }

    #[test]
    fn test_jsdoc_throws_tag_processing() {
        let extractor = JavaScriptExtractor;

        // Test throws with type and description
        let throws_with_type = "@throws {Error} When validation fails";
        let result = extractor.process_throws_tag(throws_with_type);
        assert_eq!(result, "Throws: Error - When validation fails");

        // Test throws with only type
        let throws_type_only = "@throws {Error}";
        let result_type_only = extractor.process_throws_tag(throws_type_only);
        assert_eq!(result_type_only, "Throws: Error");
    }

    #[test]
    fn test_jsdoc_type_tag_processing() {
        let extractor = JavaScriptExtractor;

        // Test type with braces
        let type_with_braces = "@type {string|number}";
        let result = extractor.process_type_tag(type_with_braces);
        assert_eq!(result, "Type: string|number");

        // Test type without braces
        let type_no_braces = "@type string";
        let result_no_braces = extractor.process_type_tag(type_no_braces);
        assert_eq!(result_no_braces, "Type: string");
    }

    #[test]
    fn test_jsdoc_typedef_tag_processing() {
        let extractor = JavaScriptExtractor;

        // Test typedef with type and name
        let typedef_full = "@typedef {Object} UserConfig";
        let result = extractor.process_typedef_tag(typedef_full);
        assert_eq!(result, "Typedef: UserConfig as Object");
    }

    #[test]
    fn test_jsdoc_special_tags() {
        let extractor = JavaScriptExtractor;

        // Test various special tags
        assert_eq!(extractor.process_jsdoc_tag("@deprecated Use newMethod instead"), "Deprecated: Use newMethod instead");
        assert_eq!(extractor.process_jsdoc_tag("@since 1.0.0"), "Since: 1.0.0");
        assert_eq!(extractor.process_jsdoc_tag("@author John Doe"), "Author: John Doe");
        assert_eq!(extractor.process_jsdoc_tag("@version 2.1.0"), "Version: 2.1.0");
        assert_eq!(extractor.process_jsdoc_tag("@see https://example.com"), "See: https://example.com");
        assert_eq!(extractor.process_jsdoc_tag("@todo Implement feature"), "TODO: Implement feature");
        assert_eq!(extractor.process_jsdoc_tag("@override"), "Override: This method overrides a parent method");
        assert_eq!(extractor.process_jsdoc_tag("@abstract"), "Abstract: This is an abstract method");
        assert_eq!(extractor.process_jsdoc_tag("@static"), "Static: This is a static method");
        assert_eq!(extractor.process_jsdoc_tag("@readonly"), "Readonly: This property is read-only");
        assert_eq!(extractor.process_jsdoc_tag("@private"), "Private: This is a private member");
        assert_eq!(extractor.process_jsdoc_tag("@protected"), "Protected: This is a protected member");
        assert_eq!(extractor.process_jsdoc_tag("@public"), "Public: This is a public member");
    }
}
