//! Dependency extraction system
//!
//! Extracts dependency relationships between symbols across all supported languages.
//! This module provides the core architecture for tracking how symbols relate to each other
//! through calls, references, imports, inheritance, and other relationships.

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType, Location, Symbol, SymbolKind};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};

/// Context information for dependency extraction
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// File path being analyzed
    pub file_path: String,
    /// Programming language
    pub language: LanguageId,
    /// All symbols extracted from this file
    pub symbols: Vec<Symbol>,
    /// Symbol lookup by name for quick resolution
    pub symbol_map: HashMap<String, Vec<usize>>, // name -> indices in symbols vec
    /// Current scope stack during traversal
    pub scope_stack: Vec<String>,
    /// Whether we're inside a conditional block
    pub in_conditional: bool,
    /// Depth of conditional nesting
    pub conditional_depth: usize,
    /// Reference to global symbol registry for cross-file resolution
    pub global_registry: Option<GlobalSymbolRegistry>,
}

impl ExtractionContext {
    pub fn new(file_path: String, language: LanguageId, symbols: Vec<Symbol>) -> Self {
        let mut symbol_map = HashMap::new();

        // Build symbol lookup map
        for (idx, symbol) in symbols.iter().enumerate() {
            symbol_map
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(idx);

            // Also add qualified name for scoped symbols
            let qualified = symbol.qualified_name();
            if qualified != symbol.name {
                symbol_map
                    .entry(qualified)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        Self {
            file_path,
            language,
            symbols,
            symbol_map,
            scope_stack: Vec::new(),
            in_conditional: false,
            conditional_depth: 0,
            global_registry: None,
        }
    }

    /// Find symbols by name, preferring those in current scope
    pub fn find_symbols(&self, name: &str) -> Vec<&Symbol> {
        if let Some(indices) = self.symbol_map.get(name) {
            let mut symbols: Vec<&Symbol> = indices.iter().map(|&idx| &self.symbols[idx]).collect();

            // Sort by scope relevance (prefer current scope)
            symbols.sort_by(|a, b| {
                let a_scope_match = self.scope_relevance(a);
                let b_scope_match = self.scope_relevance(b);
                b_scope_match.cmp(&a_scope_match)
            });

            symbols
        } else {
            Vec::new()
        }
    }

    /// Find symbols across all files using global registry
    pub fn find_symbols_global(&self, name: &str) -> Vec<&Symbol> {
        if let Some(registry) = &self.global_registry {
            let symbol_refs = registry.resolve_symbol(name, &self.file_path, &self.current_scope());
            symbol_refs.iter().map(|sr| &sr.symbol).collect()
        } else {
            self.find_symbols(name)
        }
    }

    /// Set global registry for cross-file resolution
    pub fn with_global_registry(mut self, registry: GlobalSymbolRegistry) -> Self {
        self.global_registry = Some(registry);
        self
    }

    /// Calculate how relevant a symbol is to the current scope
    fn scope_relevance(&self, symbol: &Symbol) -> usize {
        let symbol_scope = symbol.qualified_name();
        let current_scope = self.scope_stack.join("::");

        if symbol_scope == current_scope {
            return 1000; // Exact match
        }

        // Count common scope prefixes
        let symbol_parts: Vec<&str> = symbol_scope.split("::").collect();
        let current_parts: Vec<&str> = current_scope.split("::").collect();

        let mut common = 0;
        for (a, b) in symbol_parts.iter().zip(current_parts.iter()) {
            if a == b {
                common += 1;
            } else {
                break;
            }
        }

        common
    }

    /// Enter a new scope
    pub fn push_scope(&mut self, scope_name: String) {
        self.scope_stack.push(scope_name);
    }

    /// Exit current scope
    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Enter conditional context
    pub fn enter_conditional(&mut self) {
        self.in_conditional = true;
        self.conditional_depth += 1;
    }

    /// Exit conditional context
    pub fn exit_conditional(&mut self) {
        if self.conditional_depth > 0 {
            self.conditional_depth -= 1;
            if self.conditional_depth == 0 {
                self.in_conditional = false;
            }
        }
    }

    /// Get current scope as qualified name
    pub fn current_scope(&self) -> String {
        self.scope_stack.join("::")
    }
}

/// Global symbol registry for cross-file reference resolution
#[derive(Debug, Clone)]
pub struct GlobalSymbolRegistry {
    /// All symbols indexed by file path
    pub symbols_by_file: HashMap<String, Vec<Symbol>>,
    /// Global symbol lookup by qualified name
    pub global_symbol_map: HashMap<String, Vec<SymbolReference>>,
    /// Module import mappings (import path -> actual file path)
    pub import_mappings: HashMap<String, String>,
    /// External dependencies (crate/package names)
    pub external_dependencies: HashSet<String>,
    /// File dependency graph (file -> files it depends on)
    pub file_dependencies: HashMap<String, HashSet<String>>,
}

/// Reference to a symbol with location information
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub symbol: Symbol,
    pub file_path: String,
    pub is_exported: bool,
    pub visibility: SymbolVisibility,
}

/// Symbol visibility levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolVisibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

impl Default for GlobalSymbolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalSymbolRegistry {
    pub fn new() -> Self {
        Self {
            symbols_by_file: HashMap::new(),
            global_symbol_map: HashMap::new(),
            import_mappings: HashMap::new(),
            external_dependencies: HashSet::new(),
            file_dependencies: HashMap::new(),
        }
    }

    /// Add symbols from a file to the global registry
    pub fn add_file_symbols(&mut self, file_path: String, symbols: Vec<Symbol>) {
        // Store symbols by file
        self.symbols_by_file
            .insert(file_path.clone(), symbols.clone());

        // Index symbols globally
        for symbol in symbols {
            let visibility = self.determine_visibility(&symbol);
            let is_exported = self.is_symbol_exported(&symbol);

            let symbol_ref = SymbolReference {
                symbol: symbol.clone(),
                file_path: file_path.clone(),
                is_exported,
                visibility,
            };

            // Add to global map by simple name
            self.global_symbol_map
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol_ref.clone());

            // Add to global map by qualified name
            let qualified_name = symbol.qualified_name();
            if qualified_name != symbol.name {
                self.global_symbol_map
                    .entry(qualified_name)
                    .or_default()
                    .push(symbol_ref);
            }
        }
    }

    /// Resolve a symbol reference across all files
    pub fn resolve_symbol(
        &self,
        name: &str,
        context_file: &str,
        current_scope: &str,
    ) -> Vec<&SymbolReference> {
        let mut candidates = Vec::new();

        // 1. Look for exact matches in global registry
        if let Some(refs) = self.global_symbol_map.get(name) {
            candidates.extend(refs.iter());
        }

        // 2. Look for qualified matches
        let qualified_candidates = self.find_qualified_matches(name, current_scope);
        candidates.extend(qualified_candidates);

        // 3. Sort by relevance
        candidates.sort_by(|a, b| {
            self.calculate_symbol_relevance(a, context_file, current_scope)
                .partial_cmp(&self.calculate_symbol_relevance(b, context_file, current_scope))
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        });

        candidates
    }

    /// Find symbols that match with qualified names
    fn find_qualified_matches(&self, name: &str, current_scope: &str) -> Vec<&SymbolReference> {
        let mut matches = Vec::new();

        // Try different scope combinations
        let scope_parts: Vec<&str> = current_scope.split("::").collect();

        for i in 0..scope_parts.len() {
            let partial_scope = scope_parts[0..=i].join("::");
            let qualified_name = format!("{partial_scope}::{name}");

            if let Some(refs) = self.global_symbol_map.get(&qualified_name) {
                matches.extend(refs.iter());
            }
        }

        matches
    }

    /// Calculate relevance score for symbol resolution
    fn calculate_symbol_relevance(
        &self,
        symbol_ref: &SymbolReference,
        context_file: &str,
        current_scope: &str,
    ) -> f32 {
        let mut score = 0.0;

        // Same file gets highest priority
        if symbol_ref.file_path == context_file {
            score += 100.0;
        }

        // Public symbols are more accessible
        if symbol_ref.visibility == SymbolVisibility::Public {
            score += 50.0;
        }

        // Exported symbols are more likely to be referenced
        if symbol_ref.is_exported {
            score += 25.0;
        }

        // Scope proximity
        let symbol_scope = symbol_ref.symbol.qualified_name();
        let scope_similarity = self.calculate_scope_similarity(&symbol_scope, current_scope);
        score += scope_similarity * 20.0;

        // File dependency relationship
        if let Some(deps) = self.file_dependencies.get(context_file) {
            if deps.contains(&symbol_ref.file_path) {
                score += 30.0;
            }
        }

        score
    }

    /// Calculate similarity between two scopes
    fn calculate_scope_similarity(&self, scope1: &str, scope2: &str) -> f32 {
        let parts1: Vec<&str> = scope1.split("::").collect();
        let parts2: Vec<&str> = scope2.split("::").collect();

        let mut common = 0;
        for (a, b) in parts1.iter().zip(parts2.iter()) {
            if a == b {
                common += 1;
            } else {
                break;
            }
        }

        if parts1.is_empty() && parts2.is_empty() {
            1.0
        } else {
            common as f32 / (parts1.len().max(parts2.len()) as f32)
        }
    }

    /// Determine symbol visibility from modifiers
    fn determine_visibility(&self, symbol: &Symbol) -> SymbolVisibility {
        for modifier in &symbol.modifiers {
            match modifier.as_str() {
                "public" | "pub" => return SymbolVisibility::Public,
                "private" | "priv" => return SymbolVisibility::Private,
                "protected" => return SymbolVisibility::Protected,
                "internal" => return SymbolVisibility::Internal,
                "package" => return SymbolVisibility::Package,
                _ => {}
            }
        }

        // Default visibility rules by language
        match symbol.language {
            LanguageId::Rust => {
                // Rust is private by default
                SymbolVisibility::Private
            }
            LanguageId::Java | LanguageId::CSharp => {
                // Java/C# package/internal by default
                SymbolVisibility::Package
            }
            _ => {
                // Most languages are public by default
                SymbolVisibility::Public
            }
        }
    }

    /// Check if a symbol is exported (available for external use)
    fn is_symbol_exported(&self, symbol: &Symbol) -> bool {
        // Check for export modifiers
        for modifier in &symbol.modifiers {
            if modifier == "export" || modifier == "pub" || modifier == "public" {
                return true;
            }
        }

        // Language-specific export rules
        match symbol.language {
            LanguageId::JavaScript | LanguageId::TypeScript => {
                // In JS/TS, functions and classes at module level are often exported
                symbol.scope_chain.is_empty()
                    && matches!(symbol.kind, SymbolKind::Function | SymbolKind::Class)
            }
            LanguageId::Python => {
                // In Python, symbols not starting with _ are typically exported
                !symbol.name.starts_with('_')
            }
            _ => false,
        }
    }

    /// Add import mapping (import path -> actual file path)
    pub fn add_import_mapping(&mut self, import_path: String, file_path: String) {
        self.import_mappings.insert(import_path, file_path);
    }

    /// Add external dependency
    pub fn add_external_dependency(&mut self, dependency: String) {
        self.external_dependencies.insert(dependency);
    }

    /// Add file dependency relationship
    pub fn add_file_dependency(&mut self, from_file: String, to_file: String) {
        self.file_dependencies
            .entry(from_file)
            .or_default()
            .insert(to_file);
    }

    /// Get all files that depend on a given file
    pub fn get_dependents(&self, file_path: &str) -> Vec<String> {
        self.file_dependencies
            .iter()
            .filter_map(|(from, deps)| {
                if deps.contains(file_path) {
                    Some(from.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all files that a given file depends on
    pub fn get_dependencies(&self, file_path: &str) -> Vec<String> {
        self.file_dependencies
            .get(file_path)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Trait for language-specific dependency extractors
pub trait DependencyExtractor {
    /// Extract dependencies from a parsed tree
    fn extract_dependencies(
        &self,
        tree: &Tree,
        source: &str,
        context: &mut ExtractionContext,
    ) -> Vec<Dependency>;

    /// Language this extractor handles
    fn language(&self) -> LanguageId;

    /// Extract dependencies from a specific node
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Check if a node represents a function call
    fn is_function_call(&self, node: &Node) -> bool;

    /// Check if a node represents a variable reference
    fn is_variable_reference(&self, node: &Node) -> bool;

    /// Check if a node represents an import/include statement
    fn is_import_statement(&self, node: &Node) -> bool;

    /// Check if a node represents inheritance
    fn is_inheritance(&self, node: &Node) -> bool;

    /// Check if a node represents an assignment
    fn is_assignment(&self, node: &Node) -> bool;

    /// Extract function call dependencies
    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Extract variable reference dependencies
    fn extract_variable_references(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Extract import dependencies
    fn extract_imports(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Extract inheritance dependencies
    fn extract_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Extract assignment dependencies
    fn extract_assignments(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Extract control flow dependencies
    fn extract_control_flow(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    );

    /// Check if a node represents a conditional statement
    fn is_conditional_statement(&self, node: &Node) -> bool;

    /// Check if a node represents a loop statement
    fn is_loop_statement(&self, node: &Node) -> bool;

    /// Check if a node represents exception handling
    fn is_exception_handling(&self, node: &Node) -> bool;

    /// Check if a node represents a switch/match statement
    fn is_switch_statement(&self, node: &Node) -> bool;

    /// Check if a node represents a return statement
    fn is_return_statement(&self, node: &Node) -> bool;

    /// Check if a node represents break/continue
    fn is_break_continue(&self, node: &Node) -> bool;

    /// Get text content of a node
    fn get_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    /// Create a dependency with proper context
    fn create_dependency(
        &self,
        from_symbol: String,
        to_symbol: String,
        dependency_type: DependencyType,
        node: &Node,
        context: &ExtractionContext,
    ) -> Dependency {
        let location = Location::from_node(node, &context.file_path);

        let mut dependency = if context.in_conditional {
            Dependency::conditional(
                from_symbol,
                to_symbol,
                dependency_type,
                location,
                context.language,
            )
        } else {
            Dependency::new(
                from_symbol,
                to_symbol,
                dependency_type,
                location,
                context.language,
            )
        };

        // Add context information
        let node_text = self.get_node_text(node, ""); // Would need source parameter
        if !node_text.trim().is_empty() && node_text.len() < 200 {
            dependency = dependency.with_context(node_text);
        }

        dependency
    }
}

/// Factory for creating dependency extractors
pub struct DependencyExtractorFactory {
    extractors: HashMap<LanguageId, Box<dyn DependencyExtractor>>,
}

impl DependencyExtractorFactory {
    pub fn new() -> Self {
        let mut extractors: HashMap<LanguageId, Box<dyn DependencyExtractor>> = HashMap::new();

        // Register language-specific extractors
        extractors.insert(LanguageId::Rust, Box::new(RustDependencyExtractor));
        extractors.insert(LanguageId::Python, Box::new(PythonDependencyExtractor));
        extractors.insert(
            LanguageId::JavaScript,
            Box::new(JavaScriptDependencyExtractor),
        );
        extractors.insert(
            LanguageId::TypeScript,
            Box::new(JavaScriptDependencyExtractor),
        ); // Same as JS
        extractors.insert(LanguageId::Java, Box::new(JavaDependencyExtractor));
        extractors.insert(LanguageId::Go, Box::new(GoDependencyExtractor));
        extractors.insert(LanguageId::CSharp, Box::new(CSharpDependencyExtractor));
        extractors.insert(LanguageId::Swift, Box::new(SwiftDependencyExtractor));
        extractors.insert(LanguageId::PHP, Box::new(PhpDependencyExtractor));
        extractors.insert(LanguageId::Ruby, Box::new(RubyDependencyExtractor));
        extractors.insert(
            LanguageId::ObjectiveC,
            Box::new(ObjectiveCDependencyExtractor),
        );
        extractors.insert(LanguageId::Scala, Box::new(ScalaDependencyExtractor));
        extractors.insert(LanguageId::Zig, Box::new(ZigDependencyExtractor));
        extractors.insert(LanguageId::Dart, Box::new(DartDependencyExtractor));
        extractors.insert(LanguageId::Lua, Box::new(LuaDependencyExtractor));
        extractors.insert(LanguageId::Bash, Box::new(BashDependencyExtractor));
        // Additional extractors will be added as we implement them

        Self { extractors }
    }

    /// Extract dependencies for a given language
    pub fn extract_dependencies(
        &self,
        tree: &Tree,
        source: &str,
        symbols: Vec<Symbol>,
        file_path: &str,
        language: LanguageId,
    ) -> Vec<Dependency> {
        if let Some(extractor) = self.extractors.get(&language) {
            let mut context = ExtractionContext::new(file_path.to_string(), language, symbols);
            extractor.extract_dependencies(tree, source, &mut context)
        } else {
            Vec::new()
        }
    }
}

impl Default for DependencyExtractorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Base implementation for common dependency extraction patterns
pub struct BaseDependencyExtractor;

impl BaseDependencyExtractor {
    /// Generic traversal that calls language-specific handlers
    pub fn traverse_node<T: DependencyExtractor>(
        extractor: &T,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Track scope changes
        let entered_scope = Self::maybe_enter_scope(&node, source, context);

        // Track conditional context
        let entered_conditional = Self::maybe_enter_conditional(&node, context);

        // Extract dependencies from this node
        extractor.extract_from_node(node, source, context, dependencies);

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::traverse_node(extractor, child, source, context, dependencies);
        }

        // Clean up context
        if entered_conditional {
            context.exit_conditional();
        }

        if entered_scope {
            context.pop_scope();
        }
    }

    /// Check if we should enter a new scope for this node
    fn maybe_enter_scope(node: &Node, source: &str, context: &mut ExtractionContext) -> bool {
        match node.kind() {
            // Rust patterns
            "function_item" | "impl_item" | "struct_item" | "enum_item" | "trait_item" |
            // Python patterns  
            "function_definition" | "class_definition" |
            // JavaScript/TypeScript patterns
            "function_declaration" | "class_declaration" | "interface_declaration" |
            // Java patterns (method_declaration for Java specifically)
            "method_declaration" |
            // Go patterns (type_declaration for Go specifically)
            "type_declaration" => {
                if let Some(name) = Self::extract_name_from_declaration(node, source) {
                    context.push_scope(name);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Check if we should enter conditional context
    fn maybe_enter_conditional(node: &Node, context: &mut ExtractionContext) -> bool {
        match node.kind() {
            "if_expression" | "if_statement" | "match_expression" | "while_expression" |
            "for_expression" | "try_expression" | "conditional_expression" |
            // Add more conditional patterns for different languages
            "if_stmt" | "while_stmt" | "for_stmt" | "try_stmt" | "with_stmt" => {
                context.enter_conditional();
                true
            }
            _ => false,
        }
    }

    /// Extract name from a declaration node
    fn extract_name_from_declaration(node: &Node, source: &str) -> Option<String> {
        // Look for name field or identifier child
        if let Some(name_node) = node.child_by_field_name("name") {
            return Some(
                name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .to_string(),
            );
        }

        // Fallback: look for first identifier child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(child.utf8_text(source.as_bytes()).unwrap_or("").to_string());
            }
        }

        None
    }
}

// Language-specific extractors
pub mod bash_extractor;
pub mod csharp_extractor;
pub mod dart_extractor;
pub mod go_extractor;
pub mod java_extractor;
pub mod javascript_extractor;
pub mod lua_extractor;
pub mod objc_extractor;
pub mod php_extractor;
pub mod python_extractor;
pub mod ruby_extractor;
pub mod rust_extractor;
pub mod scala_extractor;
pub mod swift_extractor;
pub mod zig_extractor;

pub use bash_extractor::BashDependencyExtractor;
pub use csharp_extractor::CSharpDependencyExtractor;
pub use dart_extractor::DartDependencyExtractor;
pub use go_extractor::GoDependencyExtractor;
pub use java_extractor::JavaDependencyExtractor;
pub use javascript_extractor::JavaScriptDependencyExtractor;
pub use lua_extractor::LuaDependencyExtractor;
pub use objc_extractor::ObjectiveCDependencyExtractor;
pub use php_extractor::PhpDependencyExtractor;
pub use python_extractor::PythonDependencyExtractor;
pub use ruby_extractor::RubyDependencyExtractor;
pub use rust_extractor::RustDependencyExtractor;
pub use scala_extractor::ScalaDependencyExtractor;
pub use swift_extractor::SwiftDependencyExtractor;
pub use zig_extractor::ZigDependencyExtractor;

// All major language extractors are now implemented

// All major language extractors are implemented in their respective modules

// All major language extractors are implemented in their respective modules

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_context() {
        let symbols = vec![Symbol {
            name: "test_func".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        }];

        let context = ExtractionContext::new("test.rs".to_string(), LanguageId::Rust, symbols);
        assert_eq!(context.file_path, "test.rs");
        assert_eq!(context.language, LanguageId::Rust);
        assert_eq!(context.symbols.len(), 1);
        assert!(context.symbol_map.contains_key("test_func"));
    }

    #[test]
    fn test_dependency_extractor_factory() {
        let factory = DependencyExtractorFactory::new();
        assert!(factory.extractors.contains_key(&LanguageId::Rust));
        assert!(factory.extractors.contains_key(&LanguageId::Python));
        assert!(factory.extractors.contains_key(&LanguageId::JavaScript));
        assert!(factory.extractors.contains_key(&LanguageId::TypeScript));
        assert!(factory.extractors.contains_key(&LanguageId::Java));
        assert!(factory.extractors.contains_key(&LanguageId::Go));
        assert!(factory.extractors.contains_key(&LanguageId::CSharp));
    }

    #[test]
    fn test_global_symbol_registry() {
        let mut registry = GlobalSymbolRegistry::new();

        // Create test symbols
        let symbol1 = Symbol {
            name: "test_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "file1.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: None,
        };

        let symbol2 = Symbol {
            name: "test_class".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "file2.py".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Python,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        // Add symbols to registry
        registry.add_file_symbols("file1.rs".to_string(), vec![symbol1]);
        registry.add_file_symbols("file2.py".to_string(), vec![symbol2]);

        // Test symbol resolution
        let resolved = registry.resolve_symbol("test_function", "file1.rs", "");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].symbol.name, "test_function");

        let resolved = registry.resolve_symbol("test_class", "file2.py", "");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].symbol.name, "test_class");

        // Test cross-file resolution
        let resolved = registry.resolve_symbol("test_function", "file2.py", "");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].symbol.name, "test_function");
    }

    #[test]
    fn test_extraction_context_with_global_registry() {
        let symbols = vec![Symbol {
            name: "local_var".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        }];

        let mut registry = GlobalSymbolRegistry::new();
        registry.add_file_symbols("test.rs".to_string(), symbols.clone());

        let context = ExtractionContext::new("test.rs".to_string(), LanguageId::Rust, symbols)
            .with_global_registry(registry);

        // Test global symbol resolution
        let resolved = context.find_symbols_global("local_var");
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].name, "local_var");
    }
}
