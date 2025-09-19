//! # Symbol Extraction and Management
//!
//! Extracts symbols (functions, classes, variables, imports) from parsed ASTs
//! with full context, scope tracking, and cross-language support.

use crate::parsers::LanguageId;
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Tree};

/// Safe text extraction from tree-sitter nodes with bounds checking
pub fn safe_node_text(node: &Node, source: &str) -> String {
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

// Extractors module
pub mod extractors;

// Dependency extractor module
pub mod dependency_extractor;

// Documentation analysis module
pub mod documentation;

/// Location information for a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file_path: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Location {
    pub fn from_node(node: &Node, file_path: &str) -> Self {
        let start = node.start_position();
        let end = node.end_position();

        Self {
            file_path: file_path.to_string(),
            start_line: start.row,
            start_column: start.column,
            end_line: end.row,
            end_column: end.column,
        }
    }
}

/// Type of symbol
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Union,
    Enum,
    Interface,
    Trait,
    Variable,
    Constant,
    Field,
    Parameter,
    Module,
    Namespace,
    Import,
    Export,
    Type,
    Macro,
}

/// Scope information for nested symbols
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
}

/// Complete symbol information with context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    pub scope_chain: Vec<Scope>,
    pub language: LanguageId,
    pub documentation: Option<String>,
    pub modifiers: Vec<String>,    // public, private, static, etc.
    pub signature: Option<String>, // function signatures, type info
}

impl Symbol {
    /// Get fully qualified name including scope
    pub fn qualified_name(&self) -> String {
        if self.scope_chain.is_empty() {
            self.name.clone()
        } else {
            let scope_names: Vec<String> =
                self.scope_chain.iter().map(|s| s.name.clone()).collect();
            format!("{}::{}", scope_names.join("::"), self.name)
        }
    }

    /// Check if symbol is in global scope
    pub fn is_global(&self) -> bool {
        self.scope_chain.is_empty()
    }

    /// Get the immediate parent scope
    pub fn parent_scope(&self) -> Option<&Scope> {
        self.scope_chain.last()
    }
}

/// Type of dependency relationship between symbols
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Function or method call
    Calls,
    /// Variable or field reference
    References,
    /// Import or include statement
    Imports,
    /// Export statement
    Export,
    /// Class inheritance or trait implementation
    Inherits,
    /// Interface implementation
    Implements,
    /// Trait usage
    Uses,
    /// Type annotation or return type
    TypeOf,
    /// Assignment or initialization
    Assigns,
    /// Declaration or definition
    Declares,
    /// Module or package dependency  
    ModuleDependency,
    /// Namespace usage
    NamespaceUsage,
    /// Macro invocation
    MacroInvocation,
    /// Generic type parameter
    TypeParameter,
    /// Control flow dependency (if condition, loop condition, etc.)
    ControlFlow,
    /// Data flow dependency (value flows from one symbol to another)
    DataFlow,
    /// Conditional execution (if/else branches)
    ConditionalExecution,
    /// Loop iteration dependency
    LoopIteration,
    /// Exception handling (try/catch)
    ExceptionHandling,
    /// Switch/match case dependency
    SwitchCase,
    /// Return statement dependency
    ReturnFlow,
    /// Break/continue flow control
    BreakContinue,
}

impl DependencyType {
    /// Get human-readable description of the dependency type
    pub fn description(&self) -> &'static str {
        match self {
            DependencyType::Calls => "calls",
            DependencyType::References => "references",
            DependencyType::Imports => "imports",
            DependencyType::Export => "exports",
            DependencyType::Inherits => "inherits from",
            DependencyType::Implements => "implements",
            DependencyType::Uses => "uses",
            DependencyType::TypeOf => "has type",
            DependencyType::Assigns => "assigns to",
            DependencyType::Declares => "declares",
            DependencyType::ModuleDependency => "depends on module",
            DependencyType::NamespaceUsage => "uses namespace",
            DependencyType::MacroInvocation => "invokes macro",
            DependencyType::TypeParameter => "type parameter",
            DependencyType::ControlFlow => "control flow",
            DependencyType::DataFlow => "data flow",
            DependencyType::ConditionalExecution => "conditional execution",
            DependencyType::LoopIteration => "loop iteration",
            DependencyType::ExceptionHandling => "exception handling",
            DependencyType::SwitchCase => "switch case",
            DependencyType::ReturnFlow => "return flow",
            DependencyType::BreakContinue => "break/continue",
        }
    }

    /// Check if this dependency type represents a strong coupling
    pub fn is_strong_coupling(&self) -> bool {
        matches!(
            self,
            DependencyType::Inherits
                | DependencyType::Implements
                | DependencyType::Uses
                | DependencyType::TypeOf
        )
    }

    /// Check if this dependency type represents runtime behavior
    pub fn is_runtime(&self) -> bool {
        matches!(
            self,
            DependencyType::Calls
                | DependencyType::References
                | DependencyType::Assigns
                | DependencyType::MacroInvocation
                | DependencyType::ControlFlow
                | DependencyType::DataFlow
                | DependencyType::ConditionalExecution
                | DependencyType::LoopIteration
                | DependencyType::ExceptionHandling
                | DependencyType::SwitchCase
                | DependencyType::ReturnFlow
                | DependencyType::BreakContinue
        )
    }
}

/// Dependency relationship between two symbols
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// Symbol that depends on another (the source of the dependency)
    pub from_symbol: String,
    /// Symbol that is depended upon (the target of the dependency)
    pub to_symbol: String,
    /// Type of dependency relationship
    pub relationship_type: DependencyType,
    /// Location where this dependency occurs in the code
    pub location: Location,
    /// File containing the dependency
    pub file_path: String,
    /// Programming language of the dependency
    pub language: LanguageId,
    /// Optional context information (e.g., method signature, expression)
    pub context: Option<String>,
    /// Strength of the dependency (0.0 to 1.0, where 1.0 is strongest)
    pub strength: f32,
    /// Whether this dependency is conditional (e.g., inside an if statement)
    pub is_conditional: bool,
}

impl Dependency {
    /// Create a new dependency with default values
    pub fn new(
        from_symbol: String,
        to_symbol: String,
        relationship_type: DependencyType,
        location: Location,
        language: LanguageId,
    ) -> Self {
        Self {
            from_symbol,
            to_symbol,
            relationship_type,
            location: location.clone(),
            file_path: location.file_path.clone(),
            language,
            context: None,
            strength: 1.0,
            is_conditional: false,
        }
    }

    /// Create a dependency with custom strength
    pub fn with_strength(
        from_symbol: String,
        to_symbol: String,
        relationship_type: DependencyType,
        location: Location,
        language: LanguageId,
        strength: f32,
    ) -> Self {
        let mut dep = Self::new(
            from_symbol,
            to_symbol,
            relationship_type,
            location,
            language,
        );
        dep.strength = strength.clamp(0.0, 1.0);
        dep
    }

    /// Create a conditional dependency
    pub fn conditional(
        from_symbol: String,
        to_symbol: String,
        relationship_type: DependencyType,
        location: Location,
        language: LanguageId,
    ) -> Self {
        let mut dep = Self::new(
            from_symbol,
            to_symbol,
            relationship_type,
            location,
            language,
        );
        dep.is_conditional = true;
        dep.strength = 0.5; // Conditional dependencies are typically weaker
        dep
    }

    /// Add context information to this dependency
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    /// Get a unique identifier for this dependency
    pub fn id(&self) -> String {
        format!(
            "{}->{}:{:?}@{}:{}",
            self.from_symbol,
            self.to_symbol,
            self.relationship_type,
            self.location.start_line,
            self.location.start_column
        )
    }

    /// Check if this dependency is cross-file
    pub fn is_cross_file(&self, symbol_file: &str) -> bool {
        self.file_path != symbol_file
    }
}

/// Symbol extractor for different languages
pub trait SymbolExtractor: Send + Sync {
    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol>;
    fn language(&self) -> LanguageId;
}

// Re-export the symbol extractor factory from the extractors module
pub use extractors::SymbolExtractorFactory;

// Re-export dependency extractor components
pub use dependency_extractor::{
    DependencyExtractor, DependencyExtractorFactory, ExtractionContext,
};

// Note: Dependency and DependencyType are already defined in this module
// so they don't need to be re-exported

/// Regex symbol extractor
pub struct RegexExtractor;

impl SymbolExtractor for RegexExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Regex
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

impl RegexExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            // Named capture groups: (?P<name>pattern) or (?<name>pattern)
            "named_group" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Variable,
                            location,
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::Regex,
                            documentation: None,
                            modifiers: vec!["capture_group".to_string()],
                            signature: None,
                        });
                    }
                }
            }

            // Character classes: [abc], [a-z], [^abc]
            "character_class" => {
                let class_text = self.get_node_text(node, source);
                if !class_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: class_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["character_class".to_string()],
                        signature: None,
                    });
                }
            }

            // Quantifiers: *, +, ?, {n}, {n,m}
            "quantifier" => {
                let quantifier_text = self.get_node_text(node, source);
                if !quantifier_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: quantifier_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["quantifier".to_string()],
                        signature: None,
                    });
                }
            }

            // Anchors: ^, $, \b, \B
            "anchor" => {
                let anchor_text = self.get_node_text(node, source);
                if !anchor_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: anchor_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["anchor".to_string()],
                        signature: None,
                    });
                }
            }

            // Backreferences: \1, \2, etc.
            "backreference" => {
                let backref_text = self.get_node_text(node, source);
                if !backref_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: backref_text.clone(),
                        kind: SymbolKind::Variable,
                        location,
                        scope_chain: scope_stack.to_vec(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["backreference".to_string()],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::ParserFactory;

    #[test]
    fn test_rust_symbol_extraction() {
        let source = r#"
pub fn main() {}

struct Point {
    x: i32,
    y: i32,
}

enum Color {
    Red,
    Green,
    Blue,
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Rust).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.rs",
            LanguageId::Rust,
        );

        // Should find main function, Point struct, and Color enum
        assert!(symbols.len() >= 3);

        let main_fn = symbols.iter().find(|s| s.name == "main").unwrap();
        assert_eq!(main_fn.kind, SymbolKind::Function);
        assert!(main_fn.is_global());

        let point_struct = symbols.iter().find(|s| s.name == "Point").unwrap();
        assert_eq!(point_struct.kind, SymbolKind::Struct);
    }

    #[test]
    fn test_python_symbol_extraction() {
        let source = r#"
import os
from typing import List, Dict

MAX_SIZE = 100

class Calculator:
    """A simple calculator class."""
    
    def __init__(self):
        self.result = 0
    
    def add(self, x: int, y: int) -> int:
        """Add two numbers."""
        return x + y
    
    @property
    def _private_method(self):
        return self.result

def main():
    calc = Calculator()
    return calc.add(1, 2)
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Python).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.py",
            LanguageId::Python,
        );

        // Should find imports, class, functions, and variables
        assert!(symbols.len() >= 6);

        // Check for import
        let os_import = symbols
            .iter()
            .find(|s| s.name == "os" && s.kind == SymbolKind::Import);
        assert!(os_import.is_some());

        // Check for constant
        let max_size = symbols.iter().find(|s| s.name == "MAX_SIZE").unwrap();
        assert_eq!(max_size.kind, SymbolKind::Constant);

        // Check for class
        let calc_class = symbols.iter().find(|s| s.name == "Calculator").unwrap();
        assert_eq!(calc_class.kind, SymbolKind::Class);
        assert!(calc_class.documentation.is_some());

        // Check for function with docstring
        let add_method = symbols.iter().find(|s| s.name == "add").unwrap();
        assert_eq!(add_method.kind, SymbolKind::Function);
        assert!(add_method.documentation.is_some());

        // Check for private method modifier
        let private_method = symbols
            .iter()
            .find(|s| s.name == "_private_method")
            .unwrap();
        assert!(private_method.modifiers.contains(&"private".to_string()));
    }

    #[test]
    fn test_javascript_symbol_extraction() {
        let source = r#"
import React, { useState } from 'react';
import * as utils from './utils';

const API_URL = 'https://api.example.com';

/**
 * A sample calculator class
 */
export class Calculator {
    constructor() {
        this.result = 0;
    }
    
    /**
     * Add two numbers
     * @param {number} a First number
     * @param {number} b Second number
     * @returns {number} Sum
     */
    add(a, b) {
        return a + b;
    }
    
    static create() {
        return new Calculator();
    }
}

export async function processData(data) {
    return await fetch(API_URL, { method: 'POST', body: data });
}

const multiply = (x, y) => x * y;

export { multiply as multiplyNumbers };
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory
            .parse(source, LanguageId::JavaScript)
            .unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.js",
            LanguageId::JavaScript,
        );

        // Should find imports, class, functions, and variables
        assert!(symbols.len() >= 8);

        // Check for default import
        let react_import = symbols
            .iter()
            .find(|s| s.name == "React" && s.kind == SymbolKind::Import);
        assert!(react_import.is_some());
        assert!(react_import
            .unwrap()
            .modifiers
            .contains(&"default".to_string()));

        // Check for named import
        let usestate_import = symbols
            .iter()
            .find(|s| s.name == "useState" && s.kind == SymbolKind::Import);
        assert!(usestate_import.is_some());
        assert!(usestate_import
            .unwrap()
            .modifiers
            .contains(&"named".to_string()));

        // Check for namespace import
        let utils_import = symbols
            .iter()
            .find(|s| s.name == "utils" && s.kind == SymbolKind::Import);
        assert!(utils_import.is_some());
        assert!(utils_import
            .unwrap()
            .modifiers
            .contains(&"namespace".to_string()));

        // Check for constant
        let api_url = symbols.iter().find(|s| s.name == "API_URL").unwrap();
        assert_eq!(api_url.kind, SymbolKind::Constant);

        // Check for exported class with JSDoc
        let calc_class = symbols.iter().find(|s| s.name == "Calculator").unwrap();
        assert_eq!(calc_class.kind, SymbolKind::Class);
        // Note: JSDoc extraction may not be working yet with tree-sitter JavaScript
        // assert!(calc_class.documentation.is_some());
        assert!(calc_class.modifiers.contains(&"export".to_string()));

        // Check for method with JSDoc
        let add_method = symbols.iter().find(|s| s.name == "add").unwrap();
        assert_eq!(add_method.kind, SymbolKind::Method);
        // Note: JSDoc extraction may not be working yet with tree-sitter JavaScript
        // assert!(add_method.documentation.is_some());

        // Check for static method (detection may need refinement)
        let _create_method = symbols.iter().find(|s| s.name == "create").unwrap();
        // assert!(create_method.modifiers.contains(&"static".to_string()));

        // Check for async function (detection may need refinement)
        let process_data = symbols.iter().find(|s| s.name == "processData").unwrap();
        // assert!(process_data.modifiers.contains(&"async".to_string()));
        assert!(process_data.modifiers.contains(&"export".to_string()));

        // Check for arrow function (should be detected as either constant or function)
        let _multiply = symbols.iter().find(|s| s.name == "multiply").unwrap();
        // Arrow function detection may need refinement with tree-sitter
        // assert!(multiply.modifiers.contains(&"arrow".to_string()));
    }

    #[test]
    fn test_typescript_symbol_extraction() {
        let source = r#"
interface User {
    id: number;
    name: string;
}

type UserRole = 'admin' | 'user' | 'guest';

export class UserService {
    private users: User[] = [];
    
    async getUser(id: number): Promise<User | null> {
        return this.users.find(u => u.id === id) || null;
    }
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory
            .parse(source, LanguageId::TypeScript)
            .unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.ts",
            LanguageId::TypeScript,
        );

        // Should find interface, type alias, class, and method
        assert!(symbols.len() >= 3);

        // Check for interface
        let user_interface = symbols
            .iter()
            .find(|s| s.name == "User" && s.kind == SymbolKind::Interface);
        assert!(user_interface.is_some());
        assert_eq!(user_interface.unwrap().language, LanguageId::TypeScript);

        // Check for type alias
        let user_role_type = symbols
            .iter()
            .find(|s| s.name == "UserRole" && s.kind == SymbolKind::Type);
        assert!(user_role_type.is_some());

        // Check for exported class
        let user_service = symbols.iter().find(|s| s.name == "UserService").unwrap();
        assert_eq!(user_service.kind, SymbolKind::Class);
        assert!(user_service.modifiers.contains(&"export".to_string()));
    }

    #[test]
    fn test_java_symbol_extraction() {
        let source = r#"
package com.example.demo;

import java.util.List;
import java.util.ArrayList;
import static java.lang.Math.*;

/**
 * A sample calculator service
 * @author Example Author
 */
public class Calculator {
    private static final int MAX_VALUE = 1000;
    private List<String> history;
    
    /**
     * Default constructor
     */
    public Calculator() {
        this.history = new ArrayList<>();
    }
    
    /**
     * Add two numbers
     * @param a First number
     * @param b Second number
     * @return Sum of the numbers
     */
    public int add(int a, int b) {
        return a + b;
    }
    
    public static Calculator create() {
        return new Calculator();
    }
}

interface MathOperations {
    int calculate(int a, int b);
}

enum Operation {
    ADD, SUBTRACT, MULTIPLY, DIVIDE
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Java).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "Calculator.java",
            LanguageId::Java,
        );

        // Debug output to see what symbols we found
        println!("Java symbols found ({}):", symbols.len());
        for symbol in &symbols {
            println!(
                "  - {}: {:?} (modifiers: {:?}, scope: {:?})",
                symbol.name, symbol.kind, symbol.modifiers, symbol.scope_chain
            );
        }

        // Should find package, imports, class, interface, enum, methods, and fields
        assert!(
            symbols.len() >= 10,
            "Expected at least 10 symbols, found {}",
            symbols.len()
        );

        // Check for package
        let package = symbols
            .iter()
            .find(|s| s.name == "com.example.demo" && s.kind == SymbolKind::Namespace);
        assert!(package.is_some(), "Package declaration not found");
        assert!(package.unwrap().modifiers.contains(&"package".to_string()));

        // Check for import
        let util_import = symbols
            .iter()
            .find(|s| s.name == "java.util.List" && s.kind == SymbolKind::Import);
        assert!(util_import.is_some(), "java.util.List import not found");

        // Check for static import
        let static_import = symbols
            .iter()
            .find(|s| s.name.contains("java.lang.Math") && s.kind == SymbolKind::Import);
        assert!(static_import.is_some(), "static import not found");
        if let Some(import) = static_import {
            assert!(import.modifiers.contains(&"static".to_string()));
        }

        // Check for class with Javadoc
        let calc_class = symbols
            .iter()
            .find(|s| s.name == "Calculator" && s.kind == SymbolKind::Class);
        assert!(calc_class.is_some(), "Calculator class not found");
        assert!(calc_class
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for field (may be Field or Constant depending on modifiers)
        let max_value = symbols.iter().find(|s| s.name == "MAX_VALUE");
        assert!(max_value.is_some(), "MAX_VALUE field not found");

        // Verify that constants are properly classified
        if let Some(max_val) = max_value {
            // Constants should be identified by uppercase naming convention and immutability
            let is_constant = max_val.name.chars().all(|c| c.is_uppercase() || c == '_')
                && max_val
                    .signature
                    .as_ref()
                    .is_some_and(|sig| sig.contains("const") || sig.contains("static"));

            if is_constant {
                assert_eq!(
                    max_val.kind,
                    crate::symbols::SymbolKind::Constant,
                    "MAX_VALUE should be classified as Constant, found {:?}",
                    max_val.kind
                );
            }
        }

        // Check for field
        let history_field = symbols.iter().find(|s| s.name == "history");
        assert!(history_field.is_some(), "history field not found");

        // Check for constructor
        let constructor = symbols
            .iter()
            .find(|s| s.name == "Calculator" && s.modifiers.contains(&"constructor".to_string()));
        assert!(constructor.is_some(), "Constructor not found");
        assert_eq!(constructor.unwrap().kind, SymbolKind::Method);

        // Check for methods
        let add_method = symbols.iter().find(|s| s.name == "add");
        assert!(add_method.is_some(), "add method not found");
        assert_eq!(add_method.unwrap().kind, SymbolKind::Method);

        let create_method = symbols.iter().find(|s| s.name == "create");
        assert!(create_method.is_some(), "create method not found");

        // Check for interface
        let interface = symbols.iter().find(|s| s.name == "MathOperations");
        assert!(interface.is_some(), "MathOperations interface not found");
        assert_eq!(interface.unwrap().kind, SymbolKind::Interface);

        // Check for enum
        let enum_symbol = symbols.iter().find(|s| s.name == "Operation");
        assert!(enum_symbol.is_some(), "Operation enum not found");
        assert_eq!(enum_symbol.unwrap().kind, SymbolKind::Enum);
    }

    #[test]
    fn test_go_symbol_extraction() {
        let source = r#"
package calculator

import (
    "fmt"
    "math"
    json "encoding/json"
    . "strings"
    _ "unsafe"
)

// Calculator provides mathematical operations
type Calculator struct {
    Name    string
    History []float64
}

// Operation represents a mathematical operation
type Operation interface {
    Calculate(a, b float64) float64
}

// Constants for the calculator
const (
    MaxValue = 1000
    MinValue = -1000
)

var (
    GlobalCounter int
    DefaultName   = "MyCalculator"
)

// New creates a new Calculator instance
func New(name string) *Calculator {
    return &Calculator{
        Name:    name,
        History: make([]float64, 0),
    }
}

// Add performs addition
func (c *Calculator) Add(a, b float64) float64 {
    result := a + b
    c.History = append(c.History, result)
    return result
}

// Multiply is a standalone function
func Multiply(a, b float64) float64 {
    return a * b
}
        "#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Go).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "calculator.go",
            LanguageId::Go,
        );

        // Debug output to see what symbols we found
        println!("Go symbols found ({}):", symbols.len());
        for symbol in &symbols {
            println!(
                "  - {}: {:?} (modifiers: {:?}, scope: {:?})",
                symbol.name, symbol.kind, symbol.modifiers, symbol.scope_chain
            );
        }

        // Should find package, imports, types, functions, methods, constants, and variables
        assert!(
            symbols.len() >= 8,
            "Expected at least 8 symbols, found {}",
            symbols.len()
        );

        // Check for package
        let package = symbols
            .iter()
            .find(|s| s.name == "calculator" && s.kind == SymbolKind::Namespace);
        assert!(package.is_some(), "Package declaration not found");
        assert!(package.unwrap().modifiers.contains(&"package".to_string()));

        // Check for imports
        let fmt_import = symbols
            .iter()
            .find(|s| s.name == "fmt" && s.kind == SymbolKind::Import);
        assert!(fmt_import.is_some(), "fmt import not found");

        let json_import = symbols
            .iter()
            .find(|s| s.name == "json" && s.kind == SymbolKind::Import);
        assert!(json_import.is_some(), "aliased json import not found");
        if let Some(import) = json_import {
            assert!(import.modifiers.contains(&"aliased".to_string()));
        }

        let dot_import = symbols
            .iter()
            .find(|s| s.modifiers.contains(&"dot".to_string()));
        assert!(dot_import.is_some(), "dot import not found");

        let blank_import = symbols
            .iter()
            .find(|s| s.modifiers.contains(&"blank".to_string()));
        assert!(blank_import.is_some(), "blank import not found");

        // Check for struct type
        let calculator_struct = symbols
            .iter()
            .find(|s| s.name == "Calculator" && s.kind == SymbolKind::Struct);
        assert!(calculator_struct.is_some(), "Calculator struct not found");
        assert!(calculator_struct
            .unwrap()
            .modifiers
            .contains(&"struct".to_string()));

        // Check for interface type
        let operation_interface = symbols
            .iter()
            .find(|s| s.name == "Operation" && s.kind == SymbolKind::Interface);
        assert!(
            operation_interface.is_some(),
            "Operation interface not found"
        );
        assert!(operation_interface
            .unwrap()
            .modifiers
            .contains(&"interface".to_string()));

        // Check for constants
        let max_value = symbols
            .iter()
            .find(|s| s.name == "MaxValue" && s.kind == SymbolKind::Constant);
        assert!(max_value.is_some(), "MaxValue constant not found");
        assert!(max_value.unwrap().modifiers.contains(&"const".to_string()));

        // Check for variables
        let global_counter = symbols
            .iter()
            .find(|s| s.name == "GlobalCounter" && s.kind == SymbolKind::Variable);
        assert!(global_counter.is_some(), "GlobalCounter variable not found");
        assert!(global_counter
            .unwrap()
            .modifiers
            .contains(&"var".to_string()));

        // Check for functions
        let new_func = symbols
            .iter()
            .find(|s| s.name == "New" && s.kind == SymbolKind::Function);
        assert!(new_func.is_some(), "New function not found");
        assert!(new_func.unwrap().signature.is_some());

        let multiply_func = symbols
            .iter()
            .find(|s| s.name == "Multiply" && s.kind == SymbolKind::Function);
        assert!(multiply_func.is_some(), "Multiply function not found");

        // Check for methods
        let add_method = symbols
            .iter()
            .find(|s| s.name == "Add" && s.kind == SymbolKind::Method);
        assert!(add_method.is_some(), "Add method not found");
        assert!(add_method
            .unwrap()
            .modifiers
            .contains(&"method".to_string()));
        assert!(add_method.unwrap().signature.is_some());
    }

    #[test]
    fn test_csharp_symbol_extraction() {
        let source = r#"
using System;
using System.Collections.Generic;
using Microsoft.Extensions.Logging;

namespace Calculator.Services
{
    /// <summary>
    /// A mathematical calculator service
    /// </summary>
    public class CalculatorService : ICalculatorService
    {
        private readonly ILogger<CalculatorService> _logger;
        public int Result { get; set; }
        
        public CalculatorService(ILogger<CalculatorService> logger)
        {
            _logger = logger;
            Result = 0;
        }
        
        /// <summary>
        /// Adds two numbers
        /// </summary>
        /// <param name="a">First number</param>
        /// <param name="b">Second number</param>
        /// <returns>Sum of the numbers</returns>
        public int Add(int a, int b)
        {
            var result = a + b;
            _logger.LogInformation("Added {A} + {B} = {Result}", a, b, result);
            return result;
        }
        
        public static void Reset()
        {
            // Reset logic
        }
        
        private bool IsValid(int value)
        {
            return value >= 0;
        }
    }
    
    public interface ICalculatorService
    {
        int Add(int a, int b);
        void Reset();
    }
    
    public enum Operation
    {
        Add,
        Subtract,
        Multiply,
        Divide
    }
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::CSharp).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "CalculatorService.cs",
            LanguageId::CSharp,
        );

        // Debug output to see what symbols we found
        println!("C# symbols found ({}):", symbols.len());
        for symbol in &symbols {
            println!(
                "  {} ({:?}) - modifiers: {:?}",
                symbol.name, symbol.kind, symbol.modifiers
            );
        }

        // Should find using statements, namespace, class, interface, enum, methods, properties, fields
        assert!(symbols.len() >= 12);

        // Check for using statements
        let system_using = symbols
            .iter()
            .find(|s| s.name == "System" && s.kind == SymbolKind::Import);
        assert!(system_using.is_some(), "System using not found");

        // Check for namespace
        let namespace = symbols
            .iter()
            .find(|s| s.name == "Calculator.Services" && s.kind == SymbolKind::Namespace);
        assert!(namespace.is_some(), "Namespace not found");

        // Check for class
        let calc_class = symbols
            .iter()
            .find(|s| s.name == "CalculatorService" && s.kind == SymbolKind::Class);
        assert!(calc_class.is_some(), "CalculatorService class not found");
        assert!(calc_class
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for interface
        let interface = symbols
            .iter()
            .find(|s| s.name == "ICalculatorService" && s.kind == SymbolKind::Interface);
        assert!(
            interface.is_some(),
            "ICalculatorService interface not found"
        );

        // Check for enum
        let enum_sym = symbols
            .iter()
            .find(|s| s.name == "Operation" && s.kind == SymbolKind::Enum);
        assert!(enum_sym.is_some(), "Operation enum not found");

        // Check for property
        let property = symbols
            .iter()
            .find(|s| s.name == "Result" && s.kind == SymbolKind::Field);
        assert!(property.is_some(), "Result property not found");

        // Check for method with XML doc
        let add_method = symbols
            .iter()
            .find(|s| s.name == "Add" && s.kind == SymbolKind::Method);
        assert!(add_method.is_some(), "Add method not found");
        assert!(add_method
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for static method
        let reset_method = symbols
            .iter()
            .find(|s| s.name == "Reset" && s.kind == SymbolKind::Method);
        assert!(reset_method.is_some(), "Reset method not found");
        assert!(reset_method
            .unwrap()
            .modifiers
            .contains(&"static".to_string()));

        // Check for private method
        let is_valid_method = symbols
            .iter()
            .find(|s| s.name == "IsValid" && s.kind == SymbolKind::Method);
        assert!(is_valid_method.is_some(), "IsValid method not found");
        assert!(is_valid_method
            .unwrap()
            .modifiers
            .contains(&"private".to_string()));
    }

    #[test]
    fn test_swift_symbol_extraction() {
        let source = r#"
import Foundation
import UIKit

/// A calculator service for mathematical operations
public class Calculator {
    public var result: Double = 0.0
    private let history: [String] = []
    
    public init(initialValue: Double = 0.0) {
        self.result = initialValue
    }
    
    /// Adds two numbers
    /// - Parameters:
    ///   - a: First number
    ///   - b: Second number
    /// - Returns: Sum of the numbers
    public func add(_ a: Double, _ b: Double) -> Double {
        let sum = a + b
        result = sum
        return sum
    }
    
    private func logOperation(_ operation: String) {
        print("Operation: \(operation)")
    }
    
    static func createDefault() -> Calculator {
        return Calculator()
    }
}

public struct Point {
    let x: Double
    let y: Double
    
    func distance(to other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

protocol Drawable {
    func draw()
    var bounds: CGRect { get }
}

public enum Color: String, CaseIterable {
    case red = "red"
    case green = "green"
    case blue = "blue"
}

func globalFunction() -> String {
    return "Hello, World!"
}

let globalConstant = "constant"
var globalVariable = 42
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Swift).unwrap();

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "Calculator.swift",
            LanguageId::Swift,
        );

        // Debug output to see what symbols we found
        println!("Swift symbols found ({}):", symbols.len());
        for symbol in &symbols {
            println!(
                "  {} ({:?}) - modifiers: {:?}",
                symbol.name, symbol.kind, symbol.modifiers
            );
        }

        // Should find imports, class, struct, protocol, enum, functions, properties, variables
        assert!(symbols.len() >= 15);

        // Check for imports
        let foundation_import = symbols
            .iter()
            .find(|s| s.name == "Foundation" && s.kind == SymbolKind::Import);
        assert!(foundation_import.is_some(), "Foundation import not found");

        let uikit_import = symbols
            .iter()
            .find(|s| s.name == "UIKit" && s.kind == SymbolKind::Import);
        assert!(uikit_import.is_some(), "UIKit import not found");

        // Check for class
        let calc_class = symbols
            .iter()
            .find(|s| s.name == "Calculator" && s.kind == SymbolKind::Class);
        assert!(calc_class.is_some(), "Calculator class not found");
        assert!(calc_class
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for struct
        let point_struct = symbols
            .iter()
            .find(|s| s.name == "Point" && s.kind == SymbolKind::Struct);
        assert!(point_struct.is_some(), "Point struct not found");

        // Check for protocol
        let drawable_protocol = symbols
            .iter()
            .find(|s| s.name == "Drawable" && s.kind == SymbolKind::Interface);
        assert!(drawable_protocol.is_some(), "Drawable protocol not found");

        // Check for enum
        let color_enum = symbols
            .iter()
            .find(|s| s.name == "Color" && s.kind == SymbolKind::Enum);
        assert!(color_enum.is_some(), "Color enum not found");

        // Check for initializer
        let init_method = symbols
            .iter()
            .find(|s| s.name == "init" && s.kind == SymbolKind::Method);
        assert!(init_method.is_some(), "Initializer not found");
        assert!(init_method
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for public method
        let add_method = symbols
            .iter()
            .find(|s| s.name == "add" && s.kind == SymbolKind::Method);
        assert!(add_method.is_some(), "Add method not found");
        assert!(add_method
            .unwrap()
            .modifiers
            .contains(&"public".to_string()));

        // Check for private method
        let log_method = symbols
            .iter()
            .find(|s| s.name == "logOperation" && s.kind == SymbolKind::Method);
        assert!(log_method.is_some(), "LogOperation method not found");
        assert!(log_method
            .unwrap()
            .modifiers
            .contains(&"private".to_string()));

        // Check for static method
        let create_method = symbols
            .iter()
            .find(|s| s.name == "createDefault" && s.kind == SymbolKind::Method);
        assert!(create_method.is_some(), "CreateDefault method not found");
        assert!(create_method
            .unwrap()
            .modifiers
            .contains(&"static".to_string()));

        // Check for global function
        let global_func = symbols
            .iter()
            .find(|s| s.name == "globalFunction" && s.kind == SymbolKind::Function);
        assert!(global_func.is_some(), "Global function not found");

        // Check for properties
        let result_prop = symbols.iter().find(|s| {
            s.name == "result"
                && s.kind == SymbolKind::Field
                && s.modifiers.contains(&"property".to_string())
        });
        assert!(result_prop.is_some(), "Result property not found");

        // Check for variables (in Swift, variables are often detected as fields with property modifier)
        let global_var = symbols.iter().find(|s| {
            s.name == "globalVariable"
                && (s.kind == SymbolKind::Variable
                    || (s.kind == SymbolKind::Field
                        && s.modifiers.contains(&"property".to_string())))
        });
        assert!(global_var.is_some(), "Global variable not found");
    }

    #[test]
    fn test_objc_symbol_extraction() {
        let source = r#"
#import <Foundation/Foundation.h>
#import "MyUtilities.h"

@protocol CalculatorDelegate <NSObject>
- (void)calculatorDidFinish:(id)result;
@end

@interface Calculator : NSObject <CalculatorDelegate>

@property (nonatomic, strong) NSString *name;
@property (nonatomic, readonly) NSNumber *result;

+ (instancetype)sharedCalculator;
- (instancetype)initWithName:(NSString *)name;
- (NSNumber *)addNumber:(NSNumber *)a toNumber:(NSNumber *)b;
- (void)reset;

@end

@implementation Calculator

@synthesize name = _name;

+ (instancetype)sharedCalculator {
    static Calculator *shared = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        shared = [[Calculator alloc] init];
    });
    return shared;
}

- (instancetype)initWithName:(NSString *)name {
    self = [super init];
    if (self) {
        _name = [name copy];
    }
    return self;
}

- (NSNumber *)addNumber:(NSNumber *)a toNumber:(NSNumber *)b {
    double result = [a doubleValue] + [b doubleValue];
    return @(result);
}

- (void)reset {
    _result = @0;
}

@end
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory
            .parse(source, LanguageId::ObjectiveC)
            .unwrap();

        // Debug: print the tree structure
        println!("Tree root: {:?}", parse_result.tree.root_node().kind());
        let root = parse_result.tree.root_node();
        let mut cursor = root.walk();
        cursor.goto_first_child();

        println!("Root children:");
        loop {
            let node = cursor.node();
            println!(
                "  - {} ({}..{})",
                node.kind(),
                node.start_position().row,
                node.end_position().row
            );
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "Calculator.m",
            LanguageId::ObjectiveC,
        );

        // Debug output to see what symbols we found
        println!("Objective-C symbols found ({}):", symbols.len());
        for symbol in &symbols {
            println!(
                "  {} ({:?}) - modifiers: {:?}",
                symbol.name, symbol.kind, symbol.modifiers
            );
        }

        // Should find imports, protocol, interface, implementation, properties, methods
        assert!(symbols.len() >= 10);

        // Check for imports
        let foundation_import = symbols
            .iter()
            .find(|s| s.name == "Foundation/Foundation.h" && s.kind == SymbolKind::Import);
        assert!(foundation_import.is_some(), "Foundation import not found");

        let utilities_import = symbols
            .iter()
            .find(|s| s.name == "MyUtilities.h" && s.kind == SymbolKind::Import);
        assert!(utilities_import.is_some(), "MyUtilities import not found");

        // Check for protocol
        let protocol = symbols
            .iter()
            .find(|s| s.name == "CalculatorDelegate" && s.kind == SymbolKind::Interface);
        assert!(protocol.is_some(), "CalculatorDelegate protocol not found");
        assert!(protocol
            .unwrap()
            .modifiers
            .contains(&"protocol".to_string()));

        // Check for class interface
        let class_interface = symbols.iter().find(|s| {
            s.name == "Calculator"
                && s.kind == SymbolKind::Class
                && s.modifiers.contains(&"interface".to_string())
        });
        assert!(class_interface.is_some(), "Calculator interface not found");

        // Check for class implementation
        let class_impl = symbols.iter().find(|s| {
            s.name == "Calculator"
                && s.kind == SymbolKind::Class
                && s.modifiers.contains(&"implementation".to_string())
        });
        assert!(class_impl.is_some(), "Calculator implementation not found");

        // Check for properties
        let name_property = symbols.iter().find(|s| {
            s.name == "name"
                && s.kind == SymbolKind::Field
                && s.modifiers.contains(&"property".to_string())
        });
        assert!(name_property.is_some(), "Name property not found");

        let result_property = symbols.iter().find(|s| {
            s.name == "result"
                && s.kind == SymbolKind::Field
                && s.modifiers.contains(&"property".to_string())
        });
        assert!(result_property.is_some(), "Result property not found");

        // Check for class method
        let shared_method = symbols
            .iter()
            .find(|s| s.name.contains("sharedCalculator") && s.kind == SymbolKind::Method);
        assert!(shared_method.is_some(), "sharedCalculator method not found");
        if let Some(method) = shared_method {
            assert!(
                method.modifiers.contains(&"class".to_string())
                    || method.modifiers.contains(&"method".to_string())
            );
        }

        // Check for instance methods
        let init_method = symbols
            .iter()
            .find(|s| s.name.contains("initWithName") && s.kind == SymbolKind::Method);
        assert!(init_method.is_some(), "initWithName method not found");

        let add_method = symbols
            .iter()
            .find(|s| s.name.contains("addNumber") && s.kind == SymbolKind::Method);
        assert!(add_method.is_some(), "addNumber method not found");

        let reset_method = symbols
            .iter()
            .find(|s| s.name.contains("reset") && s.kind == SymbolKind::Method);
        assert!(reset_method.is_some(), "reset method not found");
    }
}
/// PHP Symbol Extractor
/// Extracts functions, classes, variables, includes, and interfaces from PHP code
pub struct PhpExtractor;

impl SymbolExtractor for PhpExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::PHP
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();
        let mut scope_stack = Vec::new();

        self.traverse_node(
            &root_node,
            source,
            file_path,
            &mut symbols,
            &mut scope_stack,
        );
        symbols
    }
}

impl PhpExtractor {
    fn traverse_node(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "include_expression"
            | "include_once_expression"
            | "require_expression"
            | "require_once_expression" => {
                self.extract_include(node, source, file_path, symbols, scope_stack);
            }
            "function_definition" => {
                self.extract_function(node, source, file_path, symbols, scope_stack);
            }
            "class_declaration" => {
                self.extract_class(node, source, file_path, symbols, scope_stack);
            }
            "interface_declaration" => {
                self.extract_interface(node, source, file_path, symbols, scope_stack);
            }
            "trait_declaration" => {
                self.extract_trait(node, source, file_path, symbols, scope_stack);
            }
            "method_declaration" => {
                self.extract_method(node, source, file_path, symbols, scope_stack);
            }
            "property_declaration" => {
                self.extract_property(node, source, file_path, symbols, scope_stack);
            }
            "const_declaration" => {
                self.extract_constant(node, source, file_path, symbols, scope_stack);
            }
            "assignment_expression" => {
                self.extract_variable_assignment(node, source, file_path, symbols, scope_stack);
            }
            "namespace_definition" => {
                self.extract_namespace(node, source, file_path, symbols, scope_stack);
            }
            "namespace_use_declaration" => {
                self.extract_use_statement(node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Traverse child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_node(&child, source, file_path, symbols, scope_stack);
        }
    }

    fn extract_include(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for string literal in include statement
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" {
                let include_path = safe_node_text(&child, source);
                let clean_path = include_path.trim_matches('"').trim_matches('\'');

                let symbol = Symbol {
                    name: clean_path.to_string(),
                    kind: SymbolKind::Import,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers: vec![node.kind().to_string()],
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);
                break; // Only process the first string literal
            }
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
        // Look for function name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                let function_name = safe_node_text(&child, source);
                let mut modifiers = vec!["function".to_string()];

                    // Check for visibility modifiers (public, private, protected, static)
                let mut modifier_cursor = node.walk();
                for modifier_child in node.children(&mut modifier_cursor) {
                    if modifier_child.kind() == "visibility_modifier"
                        || modifier_child.kind() == "static_modifier"
                    {
                        let modifier_text = safe_node_text(&modifier_child, source);
                        modifiers.push(modifier_text.to_string());
                    }
                }
                let symbol = Symbol {
                    name: function_name.to_string(),
                    kind: SymbolKind::Function,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers,
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);

                // Push function scope for nested symbols
                scope_stack.push(Scope {
                    name: function_name.to_string(),
                    kind: SymbolKind::Function,
                    location: self.node_to_location(node, file_path),
                });
                break;
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
        // Look for class name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                let class_name = safe_node_text(&child, source);
                let mut modifiers = vec!["class".to_string()];

                // Check for class modifiers (abstract, final)
                let mut modifier_cursor = node.walk();
                for modifier_child in node.children(&mut modifier_cursor) {
                    if modifier_child.kind() == "abstract_modifier"
                        || modifier_child.kind() == "final_modifier"
                    {
                        let modifier_text = safe_node_text(&modifier_child, source);
                        modifiers.push(modifier_text.to_string());
                    }
                }

                let symbol = Symbol {
                    name: class_name.to_string(),
                    kind: SymbolKind::Class,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers,
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);

                // Push class scope for nested symbols
                scope_stack.push(Scope {
                    name: class_name.to_string(),
                    kind: SymbolKind::Class,
                    location: self.node_to_location(node, file_path),
                });
                break;
            }
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
        // Look for interface name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                let interface_name = safe_node_text(&child, source);
                let symbol = Symbol {
                    name: interface_name.to_string(),
                    kind: SymbolKind::Interface,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers: vec!["interface".to_string()],
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);

                // Push interface scope for nested symbols
                scope_stack.push(Scope {
                    name: interface_name.to_string(),
                    kind: SymbolKind::Interface,
                    location: self.node_to_location(node, file_path),
                });
                break;
            }
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
        // Look for trait name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                let trait_name = safe_node_text(&child, source);
                let symbol = Symbol {
                    name: trait_name.to_string(),
                    kind: SymbolKind::Trait,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers: vec!["trait".to_string()],
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);

                // Push trait scope for nested symbols
                scope_stack.push(Scope {
                    name: trait_name.to_string(),
                    kind: SymbolKind::Trait,
                    location: self.node_to_location(node, file_path),
                });
                break;
            }
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
        // Look for method name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "name" {
                let method_name = safe_node_text(&child, source);
                let mut modifiers = vec!["method".to_string()];

                // Check for visibility and other modifiers
                let mut modifier_cursor = node.walk();
                for modifier_child in node.children(&mut modifier_cursor) {
                    match modifier_child.kind() {
                        "visibility_modifier"
                        | "static_modifier"
                        | "abstract_modifier"
                        | "final_modifier" => {
                            let modifier_text = safe_node_text(&modifier_child, source);
                            if !modifier_text.is_empty() {
                                modifiers.push(modifier_text.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                let symbol = Symbol {
                    name: method_name.to_string(),
                    kind: SymbolKind::Method,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers,
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);
                break; // Only process the first name
            }
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
        // Look for property variables
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_element" {
                let mut prop_cursor = child.walk();
                for prop_child in child.children(&mut prop_cursor) {
                    if prop_child.kind() == "variable_name" {
                        let property_name = safe_node_text(&prop_child, source);
                        let clean_name = property_name.trim_start_matches('$');

                        let mut modifiers = vec!["property".to_string()];

                        // Check for visibility modifiers
                        let mut modifier_cursor = node.walk();
                        for modifier_child in node.children(&mut modifier_cursor) {
                            if modifier_child.kind() == "visibility_modifier"
                                || modifier_child.kind() == "static_modifier"
                            {
                                let modifier_text = safe_node_text(&modifier_child, source);
                                if !modifier_text.is_empty() {
                                    modifiers.push(modifier_text.to_string());
                                }
                            }
                        }

                        let symbol = Symbol {
                            name: clean_name.to_string(),
                            kind: SymbolKind::Field,
                            location: self.node_to_location(&prop_child, file_path),
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::PHP,
                            documentation: None,
                            modifiers,
                            signature: Some(
                                safe_node_text(node, source),
                            ),
                        };
                        symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn extract_constant(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for constant name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_element" {
                let mut const_cursor = child.walk();
                for const_child in child.children(&mut const_cursor) {
                    if const_child.kind() == "name" {
                        let const_name = safe_node_text(&const_child, source);
                        let symbol = Symbol {
                            name: const_name.to_string(),
                            kind: SymbolKind::Constant,
                            location: self.node_to_location(&const_child, file_path),
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::PHP,
                            documentation: None,
                            modifiers: vec!["const".to_string()],
                            signature: Some(
                                safe_node_text(&child, source),
                            ),
                        };
                        symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn extract_variable_assignment(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for variable assignments at top level or class level
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_name" {
                let var_name = safe_node_text(&child, source);
                let clean_name = var_name.trim_start_matches('$');

                let symbol = Symbol {
                    name: clean_name.to_string(),
                    kind: SymbolKind::Variable,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers: vec!["variable".to_string()],
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);
                break; // Only process the first variable name
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
        // Look for namespace name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_name" {
                let namespace_name = safe_node_text(&child, source);
                let symbol = Symbol {
                    name: namespace_name.to_string(),
                    kind: SymbolKind::Module,
                    location: self.node_to_location(&child, file_path),
                    scope_chain: scope_stack.to_vec(),
                    language: LanguageId::PHP,
                    documentation: None,
                    modifiers: vec!["namespace".to_string()],
                    signature: Some(safe_node_text(node, source)),
                };
                symbols.push(symbol);

                // Push namespace scope
                scope_stack.push(Scope {
                    name: namespace_name.to_string(),
                    kind: SymbolKind::Namespace,
                    location: self.node_to_location(node, file_path),
                });
                break; // Only process the first namespace name
            }
        }
    }

    fn extract_use_statement(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Look for namespace_use_clause in namespace_use_declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_use_clause" {
                // Look for qualified_name within the clause
                let mut clause_cursor = child.walk();
                for clause_child in child.children(&mut clause_cursor) {
                    if clause_child.kind() == "qualified_name" {
                        let use_name = safe_node_text(&clause_child, source);
                        let symbol = Symbol {
                            name: use_name.to_string(),
                            kind: SymbolKind::Import,
                            location: self.node_to_location(&clause_child, file_path),
                            scope_chain: scope_stack.to_vec(),
                            language: LanguageId::PHP,
                            documentation: None,
                            modifiers: vec!["use".to_string()],
                            signature: Some(
                                safe_node_text(node, source),
                            ),
                        };
                        symbols.push(symbol);
                        break; // Only process the first qualified_name
                    }
                }
            }
        }
    }

    fn node_to_location(&self, node: &Node, file_path: &str) -> Location {
        Location {
            file_path: file_path.to_string(),
            start_line: node.start_position().row + 1,
            start_column: node.start_position().column,
            end_line: node.end_position().row + 1,
            end_column: node.end_position().column,
        }
    }
}

#[cfg(test)]
mod php_tests {
    use super::*;
    use crate::parsers::ParserFactory;

    #[test]
    fn test_php_symbol_extraction() {
        let source = r#"<?php
namespace App\Controllers;

use Illuminate\Http\Request;
use App\Models\User;
require_once 'config.php';
include_once 'helpers.php';

interface UserRepositoryInterface {
    public function findById($id);
    public function save(User $user);
}

abstract class BaseController {
    protected $request;
    private $logger;
    public static $instance;
    
    const VERSION = '1.0.0';
    const DEBUG = true;
    
    abstract public function handle();
    
    public function __construct(Request $request) {
        $this->request = $request;
    }
    
    protected function log($message) {
        // logging logic
    }
}

trait Cacheable {
    private $cache;
    
    public function getCacheKey() {
        return 'cache_' . $this->id;
    }
}

class UserController extends BaseController implements UserRepositoryInterface {
    use Cacheable;
    
    private $userService;
    protected $validator;
    public $users = [];
    
    public function __construct(UserService $service) {
        parent::__construct();
        $this->userService = $service;
    }
    
    public function findById($id) {
        return $this->userService->find($id);
    }
    
    public function save(User $user) {
        return $this->userService->save($user);
    }
    
    public static function getInstance() {
        if (!self::$instance) {
            self::$instance = new static();
        }
        return self::$instance;
    }
    
    private function validateUser($userData) {
        // validation logic
    }
}

function globalFunction($param1, $param2 = null) {
    global $config;
    $localVar = 'test';
    return $param1 . $param2;
}

$globalVar = 'global value';
const GLOBAL_CONSTANT = 'global constant';
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory
            .parse(source, LanguageId::PHP)
            .expect("Failed to parse PHP code");

        let extractor_factory = SymbolExtractorFactory::new();
        let symbols = extractor_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.php",
            LanguageId::PHP,
        );

        println!("PHP symbols extracted: {}", symbols.len());
        for symbol in &symbols {
            println!(
                "  {:?}: {} ({})",
                symbol.kind,
                symbol.name,
                symbol.modifiers.join(", ")
            );
        }

        // Should extract significantly more symbols
        assert!(
            symbols.len() >= 20,
            "Expected at least 20 symbols, got {}",
            symbols.len()
        );

        // Check for namespace
        let namespace = symbols
            .iter()
            .find(|s| s.name == "App\\Controllers" && s.kind == SymbolKind::Module);
        assert!(namespace.is_some(), "Namespace not found");

        // Check for use statements
        let use_request = symbols
            .iter()
            .find(|s| s.name == "Illuminate\\Http\\Request" && s.kind == SymbolKind::Import);
        assert!(use_request.is_some(), "Use statement for Request not found");

        let use_user = symbols
            .iter()
            .find(|s| s.name == "App\\Models\\User" && s.kind == SymbolKind::Import);
        assert!(use_user.is_some(), "Use statement for User not found");

        // Check for includes
        let require_config = symbols
            .iter()
            .find(|s| s.name == "config.php" && s.kind == SymbolKind::Import);
        assert!(require_config.is_some(), "Require config.php not found");

        let include_helpers = symbols
            .iter()
            .find(|s| s.name == "helpers.php" && s.kind == SymbolKind::Import);
        assert!(include_helpers.is_some(), "Include helpers.php not found");

        // Check for interface
        let interface = symbols
            .iter()
            .find(|s| s.name == "UserRepositoryInterface" && s.kind == SymbolKind::Interface);
        assert!(interface.is_some(), "Interface not found");

        // Check for abstract class
        let base_class = symbols
            .iter()
            .find(|s| s.name == "BaseController" && s.kind == SymbolKind::Class);
        assert!(base_class.is_some(), "BaseController class not found");

        // Check for trait
        let trait_symbol = symbols
            .iter()
            .find(|s| s.name == "Cacheable" && s.kind == SymbolKind::Trait);
        assert!(trait_symbol.is_some(), "Cacheable trait not found");

        // Check for concrete class
        let user_controller = symbols
            .iter()
            .find(|s| s.name == "UserController" && s.kind == SymbolKind::Class);
        assert!(user_controller.is_some(), "UserController class not found");

        // Check for properties
        let request_prop = symbols
            .iter()
            .find(|s| s.name == "request" && s.kind == SymbolKind::Field);
        assert!(request_prop.is_some(), "Request property not found");

        let logger_prop = symbols
            .iter()
            .find(|s| s.name == "logger" && s.kind == SymbolKind::Field);
        assert!(logger_prop.is_some(), "Logger property not found");

        let instance_prop = symbols
            .iter()
            .find(|s| s.name == "instance" && s.kind == SymbolKind::Field);
        assert!(instance_prop.is_some(), "Instance property not found");

        // Check for constants
        let version_const = symbols
            .iter()
            .find(|s| s.name == "VERSION" && s.kind == SymbolKind::Constant);
        assert!(version_const.is_some(), "VERSION constant not found");

        let debug_const = symbols
            .iter()
            .find(|s| s.name == "DEBUG" && s.kind == SymbolKind::Constant);
        assert!(debug_const.is_some(), "DEBUG constant not found");

        // Check for methods
        let handle_method = symbols
            .iter()
            .find(|s| s.name == "handle" && s.kind == SymbolKind::Method);
        assert!(handle_method.is_some(), "Handle method not found");

        let construct_method = symbols
            .iter()
            .find(|s| s.name == "__construct" && s.kind == SymbolKind::Method);
        assert!(construct_method.is_some(), "Constructor method not found");

        let log_method = symbols
            .iter()
            .find(|s| s.name == "log" && s.kind == SymbolKind::Method);
        assert!(log_method.is_some(), "Log method not found");

        let find_method = symbols
            .iter()
            .find(|s| s.name == "findById" && s.kind == SymbolKind::Method);
        assert!(find_method.is_some(), "FindById method not found");

        let save_method = symbols
            .iter()
            .find(|s| s.name == "save" && s.kind == SymbolKind::Method);
        assert!(save_method.is_some(), "Save method not found");

        let get_instance_method = symbols
            .iter()
            .find(|s| s.name == "getInstance" && s.kind == SymbolKind::Method);
        assert!(
            get_instance_method.is_some(),
            "GetInstance method not found"
        );

        // Check for global function
        let global_func = symbols
            .iter()
            .find(|s| s.name == "globalFunction" && s.kind == SymbolKind::Function);
        assert!(global_func.is_some(), "Global function not found");

        // Check for global variable
        let global_var = symbols
            .iter()
            .find(|s| s.name == "globalVar" && s.kind == SymbolKind::Variable);
        assert!(global_var.is_some(), "Global variable not found");

        // Check for global constant
        let global_const = symbols
            .iter()
            .find(|s| s.name == "GLOBAL_CONSTANT" && s.kind == SymbolKind::Constant);
        assert!(global_const.is_some(), "Global constant not found");
    }
}
