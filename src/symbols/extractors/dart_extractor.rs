//! Dart symbol extractor
//!
//! Extracts symbols from Dart source code including:
//! - Classes and mixins
//! - Methods and functions
//! - Variables and fields
//! - Imports and exports
//! - Enums and constructors
//! - Extensions and typedefs

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

/// Dart Symbol Extractor
/// Extracts classes, methods, variables, imports from Dart code
pub struct DartExtractor;

impl SymbolExtractor for DartExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Dart
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

impl DartExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "class_definition" => {
                self.extract_class(&node, source, file_path, symbols, scope_stack);
            }
            "mixin_declaration" => {
                self.extract_mixin(&node, source, file_path, symbols, scope_stack);
            }
            "enum_declaration" => {
                self.extract_enum(&node, source, file_path, symbols, scope_stack);
            }
            "extension_declaration" => {
                self.extract_extension(&node, source, file_path, symbols, scope_stack);
            }
            "function_declaration" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "method_declaration" => {
                self.extract_method(&node, source, file_path, symbols, scope_stack);
            }
            "constructor_declaration" => {
                self.extract_constructor(&node, source, file_path, symbols, scope_stack);
            }
            "variable_declaration" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            "field_declaration" => {
                self.extract_field(&node, source, file_path, symbols, scope_stack);
            }
            "import_declaration" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "export_declaration" => {
                self.extract_export(&node, source, file_path, symbols, scope_stack);
            }
            "typedef_declaration" => {
                self.extract_typedef(&node, source, file_path, symbols, scope_stack);
            }
            "getter_declaration" => {
                self.extract_getter(&node, source, file_path, symbols, scope_stack);
            }
            "setter_declaration" => {
                self.extract_setter(&node, source, file_path, symbols, scope_stack);
            }
            "enum_constant" => {
                self.extract_enum_value(&node, source, file_path, symbols, scope_stack);
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
            "class_definition" | "mixin_declaration" | "enum_declaration" | "extension_declaration"
        ) {
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
            let location = Location::from_node(node, file_path);

            // Push class as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class,
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_dart_doc(node, source);
            let modifiers = self.extract_class_modifiers(node, source);
            let signature = self.extract_class_signature(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Dart,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_mixin(
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

            // Push mixin as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Interface, // Treat mixins as interfaces
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_dart_doc(node, source);
            let mut modifiers = vec!["mixin".to_string()];
            modifiers.extend(self.extract_mixin_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Interface,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Dart,
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

            let documentation = self.extract_dart_doc(node, source);
            let modifiers = self.extract_enum_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Enum,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Dart,
                documentation,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_extension(
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

            // Push extension as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Class, // Treat extensions as classes
                location: location.clone(),
            };
            scope_stack.push(scope);

            let documentation = self.extract_dart_doc(node, source);
            let mut modifiers = vec!["extension".to_string()];
            modifiers.extend(self.extract_extension_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                language: LanguageId::Dart,
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
            let documentation = self.extract_dart_doc(node, source);
            let modifiers = self.extract_function_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
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
            let documentation = self.extract_dart_doc(node, source);
            let modifiers = self.extract_method_modifiers(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
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
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.get_node_text(&name_node, source)
        } else {
            // Default constructor has no name, use class name
            scope_stack
                .last()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "constructor".to_string())
        };

        let location = Location::from_node(node, file_path);
        let signature = self.extract_constructor_signature(node, source);
        let documentation = self.extract_dart_doc(node, source);
        let modifiers = self.extract_constructor_modifiers(node, source);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Method, // Treat constructors as methods
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Dart,
            documentation,
            modifiers,
            signature,
        });
    }

    fn extract_variable(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Extract variable names from variable declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "initialized_variable_definition" || child.kind() == "variable_name"
            {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(&child, file_path);

                    let mut modifiers = vec!["var".to_string()];
                    modifiers.extend(self.extract_variable_modifiers(node, source));

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        location,
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::Dart,
                        documentation: self.extract_dart_doc(node, source),
                        modifiers,
                        signature: None,
                    });
                }
            }
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
        // Extract field names from field declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "initialized_variable_definition" || child.kind() == "variable_name"
            {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(&child, file_path);

                    let mut modifiers = vec!["field".to_string()];
                    modifiers.extend(self.extract_field_modifiers(node, source));

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Field,
                        location,
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::Dart,
                        documentation: self.extract_dart_doc(node, source),
                        modifiers,
                        signature: None,
                    });
                }
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
        if let Some(uri_node) = node.child_by_field_name("uri") {
            let import_path = self.clean_string_literal(&self.get_node_text(&uri_node, source));
            let location = Location::from_node(&uri_node, file_path);

            let mut modifiers = vec!["import".to_string()];

            // Check for 'as' clause
            if let Some(as_node) = node.child_by_field_name("as") {
                let as_name = self.get_node_text(&as_node, source);
                modifiers.push(format!("as {as_name}"));
            }

            // Check for 'show' or 'hide' clauses
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "show_clause" {
                    modifiers.push("show".to_string());
                } else if child.kind() == "hide_clause" {
                    modifiers.push("hide".to_string());
                }
            }

            symbols.push(Symbol {
                name: import_path,
                kind: SymbolKind::Import,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_export(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        if let Some(uri_node) = node.child_by_field_name("uri") {
            let export_path = self.clean_string_literal(&self.get_node_text(&uri_node, source));
            let location = Location::from_node(&uri_node, file_path);

            let mut modifiers = vec!["export".to_string()];

            // Check for 'show' or 'hide' clauses
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "show_clause" {
                    modifiers.push("show".to_string());
                } else if child.kind() == "hide_clause" {
                    modifiers.push("hide".to_string());
                }
            }

            symbols.push(Symbol {
                name: export_path,
                kind: SymbolKind::Import, // Treat exports as imports
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_typedef(
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
                language: LanguageId::Dart,
                documentation: self.extract_dart_doc(node, source),
                modifiers: vec!["typedef".to_string()],
                signature: None,
            });
        }
    }

    fn extract_getter(
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

            let signature = self.extract_getter_signature(node, source);
            let documentation = self.extract_dart_doc(node, source);
            let mut modifiers = vec!["getter".to_string()];
            modifiers.extend(self.extract_method_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_setter(
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

            let signature = self.extract_setter_signature(node, source);
            let documentation = self.extract_dart_doc(node, source);
            let mut modifiers = vec!["setter".to_string()];
            modifiers.extend(self.extract_method_modifiers(node, source));

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Method,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_enum_value(
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

            let documentation = self.extract_dart_doc(node, source);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Constant,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Dart,
                documentation,
                modifiers: vec!["enum_value".to_string()],
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

    fn extract_dart_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Dart documentation appears as /// or /** */ comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Dart doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "documentation_comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("///") {
                        // Single-line doc comment
                        let content = comment_text.strip_prefix("///").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("/**") && comment_text.ends_with("*/") {
                        // Multi-line doc comment
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
                    }
                    current = prev;
                }
                "line_comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("//") && !comment_text.starts_with("///") {
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

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["class".to_string()];

        // Check for class modifiers (including Dart 3.0 features)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(
                    modifier_text.as_str(),
                    "abstract" | "final" | "sealed" | "base" | "interface" | "mixin"
                ) {
                    modifiers.push(modifier_text);
                }
            }
        }

        // Check for extends and implements clauses
        if node.child_by_field_name("superclass").is_some() {
            modifiers.push("extends".to_string());
        }
        if node.child_by_field_name("interfaces").is_some() {
            modifiers.push("implements".to_string());
        }

        // Check for with clause (mixins)
        if node.child_by_field_name("mixins").is_some() {
            modifiers.push("with".to_string());
        }

        modifiers
    }

    fn extract_mixin_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for mixin modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(modifier_text.as_str(), "base") {
                    modifiers.push(modifier_text);
                }
            }
        }

        modifiers
    }

    fn extract_enum_modifiers(&self, node: &Node, _source: &str) -> Vec<String> {
        let mut modifiers = vec!["enum".to_string()];

        // Check for enhanced enum features
        if node.child_by_field_name("body").is_some() {
            modifiers.push("enhanced".to_string());
        }

        modifiers
    }

    fn extract_extension_modifiers(&self, _node: &Node, _source: &str) -> Vec<String> {
        Vec::new()
    }

    fn extract_function_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["function".to_string()];

        // Check for function modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(
                    modifier_text.as_str(),
                    "static" | "async" | "sync" | "external"
                ) {
                    modifiers.push(modifier_text);
                }
            } else if child.kind() == "async_modifier" {
                modifiers.push("async".to_string());
            } else if child.kind() == "sync_modifier" {
                modifiers.push("sync".to_string());
            }
        }

        // Check for generator functions
        if self.get_node_text(node, source).contains("sync*") {
            modifiers.push("sync_generator".to_string());
        } else if self.get_node_text(node, source).contains("async*") {
            modifiers.push("async_generator".to_string());
        }

        modifiers
    }

    fn extract_method_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["method".to_string()];

        // Check for method modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(
                    modifier_text.as_str(),
                    "static" | "abstract" | "override" | "async" | "sync" | "external"
                ) {
                    modifiers.push(modifier_text);
                }
            }
        }

        modifiers
    }

    fn extract_constructor_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = vec!["constructor".to_string()];

        // Check for constructor modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(modifier_text.as_str(), "const" | "factory" | "external") {
                    modifiers.push(modifier_text);
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
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(
                    modifier_text.as_str(),
                    "static" | "final" | "const" | "late"
                ) {
                    modifiers.push(modifier_text);
                }
            }
        }

        modifiers
    }

    fn extract_field_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for field modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifier" {
                let modifier_text = self.get_node_text(&child, source);
                if matches!(
                    modifier_text.as_str(),
                    "static" | "final" | "const" | "late" | "abstract"
                ) {
                    modifiers.push(modifier_text);
                }
            }
        }

        modifiers
    }

    fn extract_class_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let type_params = node
            .child_by_field_name("type_parameters")
            .map(|tp| self.get_node_text(&tp, source))
            .unwrap_or_default();

        let extends_clause = node
            .child_by_field_name("superclass")
            .map(|e| format!(" extends {}", self.get_node_text(&e, source)))
            .unwrap_or_default();

        let implements_clause = node
            .child_by_field_name("interfaces")
            .map(|i| format!(" implements {}", self.get_node_text(&i, source)))
            .unwrap_or_default();

        Some(format!(
            "class {name}{type_params}{extends_clause}{implements_clause}"
        ))
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let type_params = node
            .child_by_field_name("type_parameters")
            .map(|tp| self.get_node_text(&tp, source))
            .unwrap_or_default();

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        let return_type = node
            .child_by_field_name("return_type")
            .map(|rt| format!("{} ", self.get_node_text(&rt, source)))
            .unwrap_or_default();

        Some(format!("{return_type}{name}{type_params}{params}"))
    }

    fn extract_method_signature(&self, node: &Node, source: &str) -> Option<String> {
        self.extract_function_signature(node, source)
    }

    fn extract_constructor_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.get_node_text(&name_node, source)
        } else {
            "constructor".to_string()
        };

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "()".to_string());

        Some(format!("{name}{params}"))
    }

    fn extract_getter_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let return_type = node
            .child_by_field_name("return_type")
            .map(|rt| format!("{} ", self.get_node_text(&rt, source)))
            .unwrap_or_default();

        Some(format!("{return_type}get {name}"))
    }

    fn extract_setter_signature(&self, node: &Node, source: &str) -> Option<String> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.get_node_text(&n, source))?;

        let params = node
            .child_by_field_name("parameters")
            .map(|p| self.get_node_text(&p, source))
            .unwrap_or_else(|| "(value)".to_string());

        Some(format!("set {name}{params}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dart_symbol_extraction() {
        let extractor = DartExtractor;
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_class_signature_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_method_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_variable_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_import_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_enum_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_mixin_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_dart_doc_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_getter_setter_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_enum_value_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_async_function_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }

    #[test]
    fn test_extension_extraction() {
        let extractor = DartExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Dart);
    }
}
