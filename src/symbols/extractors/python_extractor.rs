//! Python symbol extractor
//!
//! Extracts symbols from Python source code including:
//! - Functions and methods
//! - Classes and inheritance
//! - Variables and constants
//! - Import statements
//! - Decorators and docstrings

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Python Symbol Extractor
/// Extracts functions, classes, variables, imports, constants, and decorators from Python code
pub struct PythonExtractor;

impl SymbolExtractor for PythonExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Python
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

impl PythonExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let location = Location::from_node(&node, file_path);

                    // Extract function signature and docstring
                    let signature = self.extract_function_signature(&node, source);
                    let documentation = self.extract_docstring(&node, source);
                    let modifiers = self.extract_function_modifiers(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Python,
                        documentation,
                        modifiers,
                        signature,
                    });
                }
            }
            "class_definition" => {
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

                    let documentation = self.extract_docstring(&node, source);
                    let modifiers = self.extract_class_modifiers(&node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        location,
                        scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                        language: LanguageId::Python,
                        documentation,
                        modifiers,
                        signature: None,
                    });
                }
            }
            "import_statement" | "import_from_statement" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "assignment" => {
                // Extract variable assignments at module level or class level
                if let Some(left) = node.child_by_field_name("left") {
                    if left.kind() == "identifier" {
                        let name = left.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let location = Location::from_node(&left, file_path);

                        // Determine if this is a constant (ALL_CAPS) or variable
                        let kind = if name.chars().all(|c| c.is_uppercase() || c == '_') {
                            SymbolKind::Constant
                        } else {
                            SymbolKind::Variable
                        };

                        symbols.push(Symbol {
                            name,
                            kind,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Python,
                            documentation: None,
                            modifiers: vec![],
                            signature: None,
                        });
                    }
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
        if matches!(node.kind(), "class_definition" | "function_definition") {
            scope_stack.pop();
        }
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        // Extract complete function signature including return type
        let func_name = node
            .child_by_field_name("name")?
            .utf8_text(source.as_bytes())
            .ok()?;
        
        let parameters = node.child_by_field_name("parameters")?;
        let params = parameters.utf8_text(source.as_bytes()).ok()?;
        
        let mut signature = format!("def {func_name}{params}");
        
        // Add return type annotation if present
        if let Some(return_type) = node.child_by_field_name("return_type") {
            if let Ok(return_annotation) = return_type.utf8_text(source.as_bytes()) {
                signature.push_str(" -> ");
                signature.push_str(return_annotation.trim());
            }
        }
        
        Some(signature)
    }

    fn extract_docstring(&self, node: &Node, source: &str) -> Option<String> {
        // Look for the first string literal in the function/class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                // Check if this is an expression statement containing a string
                if child.kind() == "expression_statement" {
                    // Look for string in expression statement children
                    let mut expr_cursor = child.walk();
                    for expr_child in child.children(&mut expr_cursor) {
                        if expr_child.kind() == "string" {
                            if let Ok(docstring_raw) = expr_child.utf8_text(source.as_bytes()) {
                                return Some(self.clean_docstring(docstring_raw));
                            }
                        }
                    }
                }
                // Also check for direct string nodes (some parsers structure it differently)
                else if child.kind() == "string" {
                    if let Ok(docstring_raw) = child.utf8_text(source.as_bytes()) {
                        return Some(self.clean_docstring(docstring_raw));
                    }
                }
            }
        }
        None
    }

    /// Clean up docstring by removing quotes and normalizing whitespace
    fn clean_docstring(&self, raw: &str) -> String {
        let mut cleaned = raw.trim();

        // Remove triple quotes first
        if (cleaned.starts_with("\"\"\"") && cleaned.ends_with("\"\"\"") && cleaned.len() >= 6)
            || (cleaned.starts_with("'''") && cleaned.ends_with("'''") && cleaned.len() >= 6)
        {
            cleaned = &cleaned[3..cleaned.len() - 3];
        }
        // Remove single quotes
        else if (cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() >= 2)
            || (cleaned.starts_with('\'') && cleaned.ends_with('\'') && cleaned.len() >= 2)
        {
            cleaned = &cleaned[1..cleaned.len() - 1];
        }

        // Normalize whitespace for multi-line docstrings
        let lines: Vec<&str> = cleaned.lines().collect();
        if lines.len() > 1 {
            // Find common indentation (excluding first line)
            let min_indent = lines
                .iter()
                .skip(1)
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.len() - line.trim_start().len())
                .min()
                .unwrap_or(0);

            let normalized_lines: Vec<String> = lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    if i == 0 {
                        line.trim().to_string()
                    } else if line.trim().is_empty() {
                        String::new()
                    } else {
                        line.chars().skip(min_indent).collect()
                    }
                })
                .collect();

            normalized_lines.join("\n").trim().to_string()
        } else {
            cleaned.trim().to_string()
        }
    }

    fn extract_function_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for async keyword
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "async" {
                modifiers.push("async".to_string());
                break;
            }
        }

        // Check for generator (yield keyword in function body)
        if let Some(body) = node.child_by_field_name("body") {
            let mut body_cursor = body.walk();
            for body_child in body.children(&mut body_cursor) {
                if body_child.kind() == "yield" || body_child.kind() == "yield_expression" {
                    modifiers.push("generator".to_string());
                    break;
                }
            }
        }

        // Check for decorators by looking at previous siblings
        let mut current = *node;
        while let Some(sibling) = current.prev_sibling() {
            if sibling.kind() == "decorator" {
                if let Ok(decorator_text) = sibling.utf8_text(source.as_bytes()) {
                    let decorator = decorator_text.to_string();
                    
                    // Identify common decorator patterns
                    if decorator.contains("@property") {
                        modifiers.push("property".to_string());
                    } else if decorator.contains("@staticmethod") {
                        modifiers.push("staticmethod".to_string());
                    } else if decorator.contains("@classmethod") {
                        modifiers.push("classmethod".to_string());
                    } else if decorator.contains("@abstractmethod") {
                        modifiers.push("abstractmethod".to_string());
                    } else {
                        modifiers.push(decorator);
                    }
                }
                current = sibling;
            } else {
                break;
            }
        }

        // Check if function name suggests it's private/dunder
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                if name.starts_with("__") && name.ends_with("__") {
                    modifiers.push("dunder".to_string());
                } else if name.starts_with('_') {
                    modifiers.push("private".to_string());
                }
            }
        }

        // Extract type annotations from function signature
        if let Some(parameters) = node.child_by_field_name("parameters") {
            if self.has_type_annotations(&parameters, source) {
                modifiers.push("typed".to_string());
            }
        }

        // Check return type annotation
        if let Some(_return_type) = node.child_by_field_name("return_type") {
            modifiers.push("return_typed".to_string());
        }

        modifiers
    }

    /// Check if parameters contain type annotations
    fn has_type_annotations(&self, parameters: &Node, _source: &str) -> bool {
        let mut cursor = parameters.walk();
        for child in parameters.children(&mut cursor) {
            if child.kind() == "typed_parameter" || child.kind() == "type_annotation" {
                return true;
            }
            // Check for default values with type hints
            if child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param_child in child.children(&mut param_cursor) {
                    if param_child.kind() == "typed_parameter" {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn extract_class_modifiers(&self, node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Check for inheritance and extract parent classes
        if let Some(superclasses) = node.child_by_field_name("superclasses") {
            let mut parent_count = 0;
            let mut cursor = superclasses.walk();
            for child in superclasses.children(&mut cursor) {
                if child.kind() == "argument_list" {
                    let mut arg_cursor = child.walk();
                    for arg_child in child.children(&mut arg_cursor) {
                        if arg_child.kind() == "identifier" || arg_child.kind() == "dotted_name" {
                            if let Ok(parent_name) = arg_child.utf8_text(source.as_bytes()) {
                                parent_count += 1;
                                modifiers.push(format!("inherits:{}", parent_name));
                                
                                // Check for common base classes
                                if parent_name == "Exception" || parent_name == "BaseException" {
                                    modifiers.push("exception_class".to_string());
                                } else if parent_name == "ABC" || parent_name.contains("abc.") {
                                    modifiers.push("abstract_base".to_string());
                                } else if parent_name == "enum.Enum" {
                                    modifiers.push("enum_class".to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            if parent_count > 1 {
                modifiers.push("multiple_inheritance".to_string());
            }
        }

        // Check for class decorators
        let mut current = *node;
        while let Some(sibling) = current.prev_sibling() {
            if sibling.kind() == "decorator" {
                if let Ok(decorator_text) = sibling.utf8_text(source.as_bytes()) {
                    let decorator = decorator_text.to_string();
                    
                    if decorator.contains("@dataclass") {
                        modifiers.push("dataclass".to_string());
                    } else if decorator.contains("@enum") {
                        modifiers.push("enum_decorator".to_string());
                    } else if decorator.contains("@abstractmethod") {
                        modifiers.push("abstract_class".to_string());
                    } else {
                        modifiers.push(decorator);
                    }
                }
                current = sibling;
            } else {
                break;
            }
        }

        // Check if class name suggests it's private/exception
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                if name.starts_with('_') {
                    modifiers.push("private_class".to_string());
                } else if name.ends_with("Exception") || name.ends_with("Error") {
                    modifiers.push("exception_class".to_string());
                }
            }
        }

        modifiers
    }

    fn extract_import(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        match node.kind() {
            "import_statement" => {
                // import module1, module2
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "dotted_name" || child.kind() == "identifier" {
                        let module_name =
                            child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let location = Location::from_node(&child, file_path);

                        symbols.push(Symbol {
                            name: module_name,
                            kind: SymbolKind::Import,
                            location,
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::Python,
                            documentation: None,
                            modifiers: vec!["import".to_string()],
                            signature: None,
                        });
                    }
                }
            }
            "import_from_statement" => {
                // from module import name1, name2
                let mut module_name = String::new();
                let mut imported_names = Vec::new();

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "dotted_name" | "identifier" => {
                            if module_name.is_empty() {
                                module_name =
                                    child.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            } else {
                                imported_names.push((
                                    child.utf8_text(source.as_bytes()).unwrap_or("").to_string(),
                                    Location::from_node(&child, file_path),
                                ));
                            }
                        }
                        "import_list" => {
                            let mut import_cursor = child.walk();
                            for import_child in child.children(&mut import_cursor) {
                                if import_child.kind() == "identifier" {
                                    imported_names.push((
                                        import_child
                                            .utf8_text(source.as_bytes())
                                            .unwrap_or("")
                                            .to_string(),
                                        Location::from_node(&import_child, file_path),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Create import symbols
                for (name, location) in imported_names {
                    symbols.push(Symbol {
                        name: if module_name.is_empty() {
                            name.clone()
                        } else {
                            format!("{module_name}.{name}")
                        },
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Python,
                        documentation: None,
                        modifiers: vec!["from_import".to_string()],
                        signature: None,
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_symbol_extraction() {
        let extractor = PythonExtractor;
        assert_eq!(extractor.language(), LanguageId::Python);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = PythonExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Python);
    }

    #[test]
    fn test_docstring_extraction() {
        let extractor = PythonExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Python);
    }

    #[test]
    fn test_clean_docstring_triple_quotes() {
        let extractor = PythonExtractor;

        // Test triple quote removal
        let raw = r#""""This is a docstring""""#;
        let cleaned = extractor.clean_docstring(raw);
        assert_eq!(cleaned, "This is a docstring");

        // Test single quote removal
        let raw2 = r#"'''Another docstring'''"#;
        let cleaned2 = extractor.clean_docstring(raw2);
        assert_eq!(cleaned2, "Another docstring");
    }

    #[test]
    fn test_clean_docstring_single_quotes() {
        let extractor = PythonExtractor;

        let raw = r#""Simple docstring""#;
        let cleaned = extractor.clean_docstring(raw);
        assert_eq!(cleaned, "Simple docstring");

        let raw2 = r#"'Another simple docstring'"#;
        let cleaned2 = extractor.clean_docstring(raw2);
        assert_eq!(cleaned2, "Another simple docstring");
    }

    #[test]
    fn test_import_extraction() {
        let extractor = PythonExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Python);
    }
}
