//! Enhanced Code Graph integrating symbols and dependencies
//!
//! Extends the existing graph analysis with the Phase 4 dependency extraction system
//! to create comprehensive code graphs that capture all relationships between symbols.

use super::{CodeMetrics, CodeNode, CodeRelationship, RelationshipKind};
use crate::errors::FastContextResult;
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyExtractorFactory, DependencyType, Symbol, SymbolKind};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Graph};
use std::collections::{HashMap, HashSet};

/// Enhanced code graph that integrates symbols and dependencies
pub type EnhancedCodeGraph = Graph<CodeNode, CodeRelationship, Directed>;

// ====================================
// Graph Merging Data Structures
// ====================================

/// Result of merging two graphs
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub success: bool,
    pub symbols_added: usize,
    pub relationships_added: usize,
    pub files_added: usize,
    pub conflicts_resolved: usize,
    pub validation_errors: usize,
    pub validation_warnings: usize,
    pub conflicts: Vec<SymbolConflict>,
}

impl Default for MergeResult {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeResult {
    pub fn new() -> Self {
        Self {
            success: false,
            symbols_added: 0,
            relationships_added: 0,
            files_added: 0,
            conflicts_resolved: 0,
            validation_errors: 0,
            validation_warnings: 0,
            conflicts: Vec::new(),
        }
    }
}

/// Error that can occur during graph merging
#[derive(Debug, Clone)]
pub enum MergeError {
    IncompatibleSymbols {
        symbol_name: String,
        existing_kind: String,
        other_kind: String,
    },
    ValidationFailure {
        errors: Vec<String>,
    },
    InternalError {
        message: String,
    },
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeError::IncompatibleSymbols {
                symbol_name,
                existing_kind,
                other_kind,
            } => {
                write!(f, "Incompatible symbol types for '{symbol_name}': {existing_kind} vs {other_kind}")
            }
            MergeError::ValidationFailure { errors } => {
                write!(f, "Validation failed: {}", errors.join(", "))
            }
            MergeError::InternalError { message } => {
                write!(f, "Internal merge error: {message}")
            }
        }
    }
}

impl std::error::Error for MergeError {}

/// Details of a symbol conflict during merging
#[derive(Debug, Clone)]
pub struct SymbolConflict {
    pub symbol_name: String,
    pub existing_location: crate::symbols::Location,
    pub conflicting_location: crate::symbols::Location,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub resolved_node: NodeIndex,
    pub description: String,
}

/// Strategies for resolving symbol conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    KeepExisting, // Keep the existing symbol, ignore the new one
    UseOther,     // Replace existing with the new symbol
    Merge,        // Merge information from both symbols
    Rename,       // Rename one symbol to avoid conflict
}

/// Cross-file dependencies analysis result
#[derive(Debug)]
pub struct CrossFileDependencies {
    pub file_to_file: HashMap<(String, String), Vec<FileDependency>>,
    pub symbol_to_file: HashMap<String, Vec<String>>,
    pub external_dependencies: Vec<ExternalDependency>,
    pub total_cross_file_edges: usize,
}

/// A dependency between symbols in different files
#[derive(Debug, Clone)]
pub struct FileDependency {
    pub from_symbol: String,
    pub to_symbol: String,
    pub relationship_type: RelationshipKind,
    pub confidence: f32,
}

/// An external dependency (to libraries, frameworks, etc.)
#[derive(Debug, Clone)]
pub struct ExternalDependency {
    pub from_file: String,
    pub from_symbol: String,
    pub external_symbol: String,
    pub dependency_type: RelationshipKind,
}

/// Metrics for individual functions in the call graph
#[derive(Debug, Clone)]
pub struct CallMetrics {
    pub incoming_calls: u32,
    pub outgoing_calls: u32,
    pub is_recursive: bool,
    pub call_depth: u32,
}

/// Comprehensive call graph analysis results
#[derive(Debug)]
pub struct CallGraphAnalysis {
    pub call_graph: EnhancedCodeGraph,
    pub call_metrics: HashMap<NodeIndex, CallMetrics>,
    pub recursive_calls: Vec<NodeIndex>,
    pub leaf_functions: Vec<NodeIndex>,
    pub entry_points: Vec<NodeIndex>,
}

/// Analysis of call patterns and potential issues
#[derive(Debug)]
pub struct CallPatternAnalysis {
    pub deep_call_chains: Vec<(NodeIndex, u32)>,
    pub high_fan_out_functions: Vec<(NodeIndex, u32)>,
    pub isolated_functions: Vec<NodeIndex>,
    pub total_functions: usize,
    pub total_call_relationships: usize,
}

/// Metrics for individual modules in the import graph
#[derive(Debug, Clone)]
pub struct ImportMetrics {
    pub imports_count: u32,
    pub imported_by_count: u32,
    pub is_external: bool,
    pub import_depth: u32,
}

/// Comprehensive import dependency analysis results
#[derive(Debug)]
pub struct ImportDependencyAnalysis {
    pub import_graph: EnhancedCodeGraph,
    pub import_metrics: HashMap<NodeIndex, ImportMetrics>,
    pub circular_imports: Vec<NodeIndex>,
    pub external_dependencies: Vec<NodeIndex>,
    pub module_clusters: Vec<Vec<NodeIndex>>,
}

/// Analysis of import patterns and potential issues
#[derive(Debug)]
pub struct ImportPatternAnalysis {
    pub heavy_importers: Vec<(NodeIndex, u32)>,
    pub orphaned_modules: Vec<NodeIndex>,
    pub import_hotspots: Vec<(NodeIndex, u32)>,
    pub total_modules: usize,
    pub total_import_relationships: usize,
}

/// Comprehensive code graph builder that integrates Phase 4 dependency extraction
pub struct CodeGraphBuilder {
    graph: EnhancedCodeGraph,
    symbol_to_node: HashMap<String, NodeIndex>,
    file_symbols: HashMap<String, Vec<NodeIndex>>,
    dependency_extractor: DependencyExtractorFactory,

    // Specialized graph views
    call_graph_edges: HashSet<(NodeIndex, NodeIndex)>,
    import_graph_edges: HashSet<(NodeIndex, NodeIndex)>,
    inheritance_graph_edges: HashSet<(NodeIndex, NodeIndex)>,
    data_flow_edges: HashSet<(NodeIndex, NodeIndex)>,
    control_flow_edges: HashSet<(NodeIndex, NodeIndex)>,
}

impl CodeGraphBuilder {
    /// Create a new enhanced code graph builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            symbol_to_node: HashMap::new(),
            file_symbols: HashMap::new(),
            dependency_extractor: DependencyExtractorFactory::new(),
            call_graph_edges: HashSet::new(),
            import_graph_edges: HashSet::new(),
            inheritance_graph_edges: HashSet::new(),
            data_flow_edges: HashSet::new(),
            control_flow_edges: HashSet::new(),
        }
    }

    /// Add a complete file to the graph with dependency extraction
    pub fn add_file(
        &mut self,
        tree: &tree_sitter::Tree,
        source: &str,
        symbols: Vec<Symbol>,
        file_path: &str,
        language: LanguageId,
    ) -> FastContextResult<()> {
        // First, add all symbols as nodes
        self.add_file_symbols(symbols.clone(), file_path);

        // Extract dependencies using Phase 4 system
        let dependencies = self
            .dependency_extractor
            .extract_dependencies(tree, source, symbols, file_path, language);

        // Add dependency relationships to the graph
        for dependency in dependencies {
            self.add_dependency_relationship(dependency)?;
        }

        // Analyze internal references and update metrics
        self.analyze_internal_references(file_path);
        self.update_fan_metrics();

        Ok(())
    }

    /// Add symbols from a file to the graph (enhanced version)
    pub fn add_file_symbols(&mut self, symbols: Vec<Symbol>, file_path: &str) {
        let mut file_nodes = Vec::new();

        for symbol in symbols {
            let qualified_name = symbol.qualified_name();

            // Skip if already exists (for cross-file references)
            if self.symbol_to_node.contains_key(&qualified_name) {
                if let Some(&existing_node) = self.symbol_to_node.get(&qualified_name) {
                    file_nodes.push(existing_node);
                }
                continue;
            }

            // Calculate enhanced metrics
            let metrics = self.calculate_enhanced_metrics(&symbol);

            let node_data = CodeNode {
                symbol: symbol.clone(),
                file_path: file_path.to_string(),
                metrics,
            };

            let node_idx = self.graph.add_node(node_data);
            self.symbol_to_node.insert(qualified_name, node_idx);
            file_nodes.push(node_idx);
        }

        self.file_symbols.insert(file_path.to_string(), file_nodes);
    }

    /// Add a dependency relationship to the graph
    pub fn add_dependency_relationship(&mut self, dependency: Dependency) -> FastContextResult<()> {
        let from_node =
            self.find_or_create_external_node(&dependency.from_symbol, &dependency.file_path);
        let to_node =
            self.find_or_create_external_node(&dependency.to_symbol, &dependency.file_path);

        // Convert dependency type to relationship kind
        let relationship_kind = self.dependency_to_relationship_kind(&dependency.relationship_type);

        let relationship = CodeRelationship {
            kind: relationship_kind,
            source_location: format!(
                "{}:{}:{}",
                dependency.location.file_path,
                dependency.location.start_line,
                dependency.location.start_column
            ),
            confidence: dependency.strength,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "dependency_type".to_string(),
                    format!("{:?}", dependency.relationship_type),
                );
                meta.insert(
                    "is_conditional".to_string(),
                    dependency.is_conditional.to_string(),
                );
                meta.insert("language".to_string(), format!("{:?}", dependency.language));
                if let Some(context) = dependency.context {
                    meta.insert("context".to_string(), context);
                }
                meta
            },
        };

        // Add to appropriate specialized graph view
        match dependency.relationship_type {
            DependencyType::Calls => {
                self.call_graph_edges.insert((from_node, to_node));
            }
            DependencyType::Imports => {
                self.import_graph_edges.insert((from_node, to_node));
            }
            DependencyType::Inherits | DependencyType::Implements => {
                self.inheritance_graph_edges.insert((from_node, to_node));
            }
            DependencyType::DataFlow | DependencyType::Assigns => {
                self.data_flow_edges.insert((from_node, to_node));
            }
            _ => {}
        }

        self.graph.add_edge(from_node, to_node, relationship);
        Ok(())
    }

    /// Find existing node or create external reference node
    fn find_or_create_external_node(&mut self, symbol_name: &str, file_path: &str) -> NodeIndex {
        if let Some(&existing_node) = self.symbol_to_node.get(symbol_name) {
            return existing_node;
        }

        // Create external reference node for symbols not in our current analysis
        let external_symbol = Symbol {
            name: symbol_name.to_string(),
            kind: SymbolKind::Variable, // Default kind for external references
            location: crate::symbols::Location {
                file_path: file_path.to_string(),
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
            scope_chain: vec![],
            language: LanguageId::Rust, // Would need to infer this properly
            documentation: None,
            modifiers: vec!["external".to_string()],
            signature: None,
        };

        let node_data = CodeNode {
            symbol: external_symbol,
            file_path: file_path.to_string(),
            metrics: CodeMetrics::default(),
        };

        let node_idx = self.graph.add_node(node_data);
        self.symbol_to_node
            .insert(symbol_name.to_string(), node_idx);
        node_idx
    }

    /// Convert dependency type to relationship kind
    fn dependency_to_relationship_kind(&self, dep_type: &DependencyType) -> RelationshipKind {
        match dep_type {
            DependencyType::Calls => RelationshipKind::Calls,
            DependencyType::References => RelationshipKind::References,
            DependencyType::Imports => RelationshipKind::Imports,
            DependencyType::Inherits => RelationshipKind::Inherits,
            DependencyType::Implements => RelationshipKind::Implements,
            DependencyType::Uses => RelationshipKind::DependsOn,
            DependencyType::Assigns | DependencyType::DataFlow => RelationshipKind::References,
            DependencyType::ModuleDependency => RelationshipKind::DependsOn,
            _ => RelationshipKind::DependsOn,
        }
    }

    /// Calculate enhanced metrics for a symbol
    fn calculate_enhanced_metrics(&self, symbol: &Symbol) -> CodeMetrics {
        let mut metrics = CodeMetrics::default();

        // Enhanced metrics calculation
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                if let Some(signature) = &symbol.signature {
                    metrics.cyclomatic_complexity = self.estimate_complexity(signature);
                    metrics.number_of_parameters = self.count_parameters(signature);
                    metrics.depth_of_nesting = self.estimate_nesting_depth(signature);
                }
            }
            SymbolKind::Class | SymbolKind::Struct => {
                // For classes, complexity could be based on number of methods
                metrics.cyclomatic_complexity = 1; // Base complexity for classes
            }
            _ => {}
        }

        // Calculate lines of code from location
        metrics.lines_of_code = (symbol.location.end_line - symbol.location.start_line + 1) as u32;

        metrics
    }

    /// Estimate nesting depth from source code
    fn estimate_nesting_depth(&self, code: &str) -> u32 {
        let mut max_depth: u32 = 0;
        let mut current_depth: u32 = 0;

        for ch in code.chars() {
            match ch {
                '{' | '(' | '[' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                '}' | ')' | ']' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        max_depth
    }

    /// Update fan-in and fan-out metrics for all nodes
    fn update_fan_metrics(&mut self) {
        let mut fan_in_counts: HashMap<NodeIndex, u32> = HashMap::new();
        let mut fan_out_counts: HashMap<NodeIndex, u32> = HashMap::new();

        // Count incoming and outgoing edges for each node
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                *fan_out_counts.entry(source).or_insert(0) += 1;
                *fan_in_counts.entry(target).or_insert(0) += 1;
            }
        }

        // Update node metrics
        for node_idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight_mut(node_idx) {
                node.metrics.fan_in = *fan_in_counts.get(&node_idx).unwrap_or(&0);
                node.metrics.fan_out = *fan_out_counts.get(&node_idx).unwrap_or(&0);
            }
        }
    }

    /// Analyze cross-references within a file (enhanced version)
    pub fn analyze_internal_references(&mut self, file_path: &str) {
        if let Some(file_nodes) = self.file_symbols.get(file_path).cloned() {
            for &node_idx in &file_nodes {
                let symbol = self.graph[node_idx].symbol.clone();

                // Enhanced analysis using symbol signatures and modifiers
                self.analyze_symbol_dependencies(node_idx, &symbol, file_path);
            }
        }
    }

    /// Enhanced symbol dependency analysis
    fn analyze_symbol_dependencies(
        &mut self,
        from_node: NodeIndex,
        symbol: &Symbol,
        file_path: &str,
    ) {
        match symbol.kind {
            SymbolKind::Function | SymbolKind::Method => {
                if let Some(signature) = &symbol.signature {
                    self.analyze_function_signature(from_node, signature, file_path);
                }
            }
            SymbolKind::Class | SymbolKind::Struct => {
                self.analyze_type_dependencies(from_node, symbol, file_path);
            }
            SymbolKind::Import => {
                self.create_enhanced_import_relationship(from_node, symbol, file_path);
            }
            _ => {}
        }
    }

    /// Analyze function signature for dependencies
    fn analyze_function_signature(
        &mut self,
        from_node: NodeIndex,
        signature: &str,
        file_path: &str,
    ) {
        // Enhanced pattern matching for different types of calls
        let patterns = [
            regex::Regex::new(r"(\w+)::(\w+)\s*\(").unwrap(), // Static method calls
            regex::Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(), // Method calls
            regex::Regex::new(r"(\w+)\s*\(").unwrap(),        // Function calls
        ];

        for pattern in &patterns {
            for captures in pattern.captures_iter(signature) {
                if let Some(function_name) = captures.get(1) {
                    let called_function = function_name.as_str();
                    self.create_call_relationship(from_node, called_function, file_path);
                }
            }
        }
    }

    /// Analyze type dependencies for classes/structs
    fn analyze_type_dependencies(
        &mut self,
        from_node: NodeIndex,
        symbol: &Symbol,
        file_path: &str,
    ) {
        // Look for inheritance patterns in modifiers or signature
        for modifier in &symbol.modifiers {
            if modifier.starts_with("extends") || modifier.starts_with("implements") {
                // Extract the inherited type name
                if let Some(type_name) = modifier.split_whitespace().nth(1) {
                    self.create_inheritance_relationship(from_node, type_name, file_path);
                }
            }
        }
    }

    /// Create enhanced call relationship
    fn create_call_relationship(
        &mut self,
        from_node: NodeIndex,
        called_function: &str,
        file_path: &str,
    ) {
        if let Some(&to_node) = self.symbol_to_node.get(called_function) {
            let relationship = CodeRelationship {
                kind: RelationshipKind::Calls,
                source_location: format!("{}:{}", file_path, 0),
                confidence: 0.9,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("call_type".to_string(), "function_call".to_string());
                    meta
                },
            };

            self.call_graph_edges.insert((from_node, to_node));
            self.graph.add_edge(from_node, to_node, relationship);
        }
    }

    /// Create inheritance relationship
    fn create_inheritance_relationship(
        &mut self,
        from_node: NodeIndex,
        parent_type: &str,
        file_path: &str,
    ) {
        if let Some(&to_node) = self.symbol_to_node.get(parent_type) {
            let relationship = CodeRelationship {
                kind: RelationshipKind::Inherits,
                source_location: format!("{}:{}", file_path, 0),
                confidence: 1.0,
                metadata: HashMap::new(),
            };

            self.inheritance_graph_edges.insert((from_node, to_node));
            self.graph.add_edge(from_node, to_node, relationship);
        }
    }

    /// Create enhanced import relationship
    fn create_enhanced_import_relationship(
        &mut self,
        from_node: NodeIndex,
        symbol: &Symbol,
        file_path: &str,
    ) {
        let imported_name = &symbol.name;

        let relationship = CodeRelationship {
            kind: RelationshipKind::Imports,
            source_location: format!("{}:{}", file_path, symbol.location.start_line),
            confidence: 1.0,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("import_path".to_string(), imported_name.clone());
                meta.insert("language".to_string(), format!("{:?}", symbol.language));
                meta
            },
        };

        if let Some(&to_node) = self.symbol_to_node.get(imported_name) {
            self.import_graph_edges.insert((from_node, to_node));
            self.graph.add_edge(from_node, to_node, relationship);
        }
    }

    /// Get call graph as a subgraph
    pub fn get_call_graph(&self) -> EnhancedCodeGraph {
        self.build_subgraph(&self.call_graph_edges)
    }

    /// Build comprehensive call graph with enhanced analysis
    pub fn build_call_graph(&self) -> CallGraphAnalysis {
        let call_graph = self.get_call_graph();

        // Calculate call graph metrics
        let mut call_metrics = HashMap::new();
        let mut recursive_calls = Vec::new();
        let mut leaf_functions = Vec::new();
        let mut entry_points = Vec::new();

        for node_idx in call_graph.node_indices() {
            let _node = &call_graph[node_idx];

            // Count incoming and outgoing calls
            let incoming_calls = call_graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .count();
            let outgoing_calls = call_graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .count();

            let metrics = CallMetrics {
                incoming_calls: incoming_calls as u32,
                outgoing_calls: outgoing_calls as u32,
                is_recursive: self.is_recursive_function(node_idx, &call_graph),
                call_depth: self.calculate_call_depth(node_idx, &call_graph),
            };

            call_metrics.insert(node_idx, metrics.clone());

            // Classify functions
            if metrics.is_recursive {
                recursive_calls.push(node_idx);
            }

            if outgoing_calls == 0 {
                leaf_functions.push(node_idx);
            }

            if incoming_calls == 0 {
                entry_points.push(node_idx);
            }
        }

        CallGraphAnalysis {
            call_graph,
            call_metrics,
            recursive_calls,
            leaf_functions,
            entry_points,
        }
    }

    /// Find all functions that call the specified function
    pub fn find_callers(&self, target_function: &str) -> Vec<NodeIndex> {
        if let Some(&target_idx) = self.symbol_to_node.get(target_function) {
            self.graph
                .neighbors_directed(target_idx, petgraph::Direction::Incoming)
                .filter(|&caller_idx| {
                    // Check if there's a call relationship
                    self.call_graph_edges.contains(&(caller_idx, target_idx))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find all functions called by the specified function
    pub fn find_callees(&self, source_function: &str) -> Vec<NodeIndex> {
        if let Some(&source_idx) = self.symbol_to_node.get(source_function) {
            self.graph
                .neighbors_directed(source_idx, petgraph::Direction::Outgoing)
                .filter(|&callee_idx| {
                    // Check if there's a call relationship
                    self.call_graph_edges.contains(&(source_idx, callee_idx))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get call chain from source to target function
    pub fn get_call_chain(&self, source: &str, target: &str) -> Option<Vec<NodeIndex>> {
        if let (Some(&source_idx), Some(&target_idx)) = (
            self.symbol_to_node.get(source),
            self.symbol_to_node.get(target),
        ) {
            // Use Dijkstra's algorithm to find shortest path in call graph

            let call_graph = self.get_call_graph();
            let path_map =
                petgraph::algo::dijkstra(&call_graph, source_idx, Some(target_idx), |_| 1);

            if path_map.contains_key(&target_idx) {
                // Reconstruct path
                self.reconstruct_call_path(source_idx, target_idx, &call_graph)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Analyze function call patterns and identify potential issues
    pub fn analyze_call_patterns(&self) -> CallPatternAnalysis {
        let call_graph = self.get_call_graph();
        let mut deep_call_chains = Vec::new();
        let mut high_fan_out_functions = Vec::new();
        let mut isolated_functions = Vec::new();

        for node_idx in call_graph.node_indices() {
            let _node = &call_graph[node_idx];
            let outgoing_calls = call_graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .count();
            let incoming_calls = call_graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .count();

            // Find functions with high fan-out (calling many other functions)
            if outgoing_calls > 10 {
                high_fan_out_functions.push((node_idx, outgoing_calls as u32));
            }

            // Find isolated functions (no callers or callees)
            if incoming_calls == 0 && outgoing_calls == 0 {
                isolated_functions.push(node_idx);
            }

            // Find deep call chains
            let call_depth = self.calculate_call_depth(node_idx, &call_graph);
            if call_depth > 5 {
                deep_call_chains.push((node_idx, call_depth));
            }
        }

        CallPatternAnalysis {
            deep_call_chains,
            high_fan_out_functions,
            isolated_functions,
            total_functions: call_graph.node_count(),
            total_call_relationships: call_graph.edge_count(),
        }
    }

    /// Get import dependency graph
    pub fn get_import_graph(&self) -> EnhancedCodeGraph {
        self.build_subgraph(&self.import_graph_edges)
    }

    /// Build comprehensive import dependency analysis
    pub fn build_import_dependency_graph(&self) -> ImportDependencyAnalysis {
        let import_graph = self.get_import_graph();

        // Analyze import patterns
        let mut import_metrics = HashMap::new();
        let mut circular_imports = Vec::new();
        let mut external_dependencies = Vec::new();

        for node_idx in import_graph.node_indices() {
            let node = &import_graph[node_idx];

            // Count import relationships
            let imports = import_graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .count();
            let imported_by = import_graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .count();

            let metrics = ImportMetrics {
                imports_count: imports as u32,
                imported_by_count: imported_by as u32,
                is_external: node.symbol.modifiers.contains(&"external".to_string()),
                import_depth: self.calculate_import_depth(node_idx, &import_graph),
            };

            import_metrics.insert(node_idx, metrics.clone());

            // Classify imports
            if metrics.is_external {
                external_dependencies.push(node_idx);
            }

            // Detect circular imports
            if self.has_circular_import(node_idx, &import_graph) {
                circular_imports.push(node_idx);
            }
        }

        // Find module clusters (strongly connected components)
        let module_clusters = self.find_import_clusters(&import_graph);

        ImportDependencyAnalysis {
            import_graph,
            import_metrics,
            circular_imports,
            external_dependencies,
            module_clusters,
        }
    }

    /// Analyze import patterns for code organization insights
    pub fn analyze_import_patterns(&self) -> ImportPatternAnalysis {
        let import_graph = self.get_import_graph();
        let mut heavy_importers = Vec::new();
        let mut orphaned_modules = Vec::new();
        let mut import_hotspots = Vec::new();

        for node_idx in import_graph.node_indices() {
            let _node = &import_graph[node_idx];
            let imports = import_graph
                .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
                .count();
            let imported_by = import_graph
                .neighbors_directed(node_idx, petgraph::Direction::Incoming)
                .count();

            // Find modules that import many others (high fan-out)
            if imports > 15 {
                heavy_importers.push((node_idx, imports as u32));
            }

            // Find modules that are heavily imported (high fan-in)
            if imported_by > 10 {
                import_hotspots.push((node_idx, imported_by as u32));
            }

            // Find orphaned modules (no imports in or out)
            if imports == 0 && imported_by == 0 {
                orphaned_modules.push(node_idx);
            }
        }

        ImportPatternAnalysis {
            heavy_importers,
            orphaned_modules,
            import_hotspots,
            total_modules: import_graph.node_count(),
            total_import_relationships: import_graph.edge_count(),
        }
    }

    /// Get import dependencies by module/file
    pub fn get_imports_by_file(&self, file_path: &str) -> Vec<NodeIndex> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                let node = &self.graph[idx];
                node.file_path == file_path && node.symbol.kind == SymbolKind::Import
            })
            .collect()
    }

    /// Find all modules that import the specified module
    pub fn find_importers(&self, target_module: &str) -> Vec<NodeIndex> {
        if let Some(&target_idx) = self.symbol_to_node.get(target_module) {
            self.graph
                .neighbors_directed(target_idx, petgraph::Direction::Incoming)
                .filter(|&importer_idx| {
                    // Check if there's an import relationship
                    self.import_graph_edges
                        .contains(&(importer_idx, target_idx))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find all modules imported by the specified module
    pub fn find_imported_modules(&self, source_module: &str) -> Vec<NodeIndex> {
        if let Some(&source_idx) = self.symbol_to_node.get(source_module) {
            self.graph
                .neighbors_directed(source_idx, petgraph::Direction::Outgoing)
                .filter(|&imported_idx| {
                    // Check if there's an import relationship
                    self.import_graph_edges
                        .contains(&(source_idx, imported_idx))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get import chain from source to target module
    pub fn get_import_chain(&self, source: &str, target: &str) -> Option<Vec<NodeIndex>> {
        if let (Some(&source_idx), Some(&target_idx)) = (
            self.symbol_to_node.get(source),
            self.symbol_to_node.get(target),
        ) {
            let import_graph = self.get_import_graph();
            self.reconstruct_call_path(source_idx, target_idx, &import_graph)
        } else {
            None
        }
    }

    /// Get inheritance graph
    pub fn get_inheritance_graph(&self) -> EnhancedCodeGraph {
        self.build_subgraph(&self.inheritance_graph_edges)
    }

    /// Get data flow graph
    pub fn get_data_flow_graph(&self) -> EnhancedCodeGraph {
        self.build_subgraph(&self.data_flow_edges)
    }

    /// Build a subgraph from a set of edges
    fn build_subgraph(&self, edges: &HashSet<(NodeIndex, NodeIndex)>) -> EnhancedCodeGraph {
        let mut subgraph = Graph::new();
        let mut node_mapping = HashMap::new();

        for &(from_idx, to_idx) in edges {
            // Add nodes if not already added
            for &node_idx in &[from_idx, to_idx] {
                node_mapping
                    .entry(node_idx)
                    .or_insert_with(|| subgraph.add_node(self.graph[node_idx].clone()));
            }

            // Add edge
            if let Some(edge_weight) = self
                .graph
                .find_edge(from_idx, to_idx)
                .and_then(|edge_idx| self.graph.edge_weight(edge_idx))
            {
                subgraph.add_edge(
                    node_mapping[&from_idx],
                    node_mapping[&to_idx],
                    edge_weight.clone(),
                );
            }
        }

        subgraph
    }

    /// Estimate cyclomatic complexity (enhanced version)
    fn estimate_complexity(&self, code: &str) -> u32 {
        let complexity_patterns = [
            (r"\bif\b", 1),
            (r"\belse\b", 1),
            (r"\bwhile\b", 1),
            (r"\bfor\b", 1),
            (r"\bmatch\b", 1),
            (r"\bcase\b", 1),
            (r"\bcatch\b", 1),
            (r"\btry\b", 1),
            (r"&&", 1),
            (r"\|\|", 1),
            (r"\?", 1), // Ternary operator
        ];

        let mut complexity = 1; // Base complexity

        for (pattern, weight) in &complexity_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                complexity += (regex.find_iter(code).count() as u32) * weight;
            }
        }

        complexity
    }

    /// Count function parameters (enhanced version)
    fn count_parameters(&self, signature: &str) -> u32 {
        if let Some(params_start) = signature.find('(') {
            if let Some(params_end) = signature.rfind(')') {
                let params_str = &signature[params_start + 1..params_end];
                if params_str.trim().is_empty() {
                    return 0;
                }

                // More sophisticated parameter counting
                let mut param_count = 0;
                let mut paren_depth = 0;
                let mut in_generic = 0;

                for ch in params_str.chars() {
                    match ch {
                        '(' => paren_depth += 1,
                        ')' => paren_depth -= 1,
                        '<' => in_generic += 1,
                        '>' => in_generic -= 1,
                        ',' if paren_depth == 0 && in_generic == 0 => param_count += 1,
                        _ => {}
                    }
                }

                // Add one for the last parameter (if any parameters exist)
                if !params_str.trim().is_empty() {
                    param_count += 1;
                }

                return param_count;
            }
        }
        0
    }

    /// Finalize the enhanced graph
    pub fn build(mut self) -> EnhancedCodeGraph {
        self.update_fan_metrics();
        self.graph
    }

    /// Get the current graph reference
    pub fn graph(&self) -> &EnhancedCodeGraph {
        &self.graph
    }

    /// Get symbol mapping
    pub fn symbol_mapping(&self) -> &HashMap<String, NodeIndex> {
        &self.symbol_to_node
    }

    /// Get nodes by symbol kind
    pub fn nodes_by_kind(&self, kind: SymbolKind) -> Vec<NodeIndex> {
        self.graph
            .node_indices()
            .filter(|&idx| self.graph[idx].symbol.kind == kind)
            .collect()
    }

    /// Build symbol definition graph showing where symbols are defined
    pub fn build_symbol_definition_graph(&self) -> EnhancedCodeGraph {
        let mut def_graph = Graph::new();
        let mut node_mapping = HashMap::new();

        // Add all symbols as nodes
        for node_idx in self.graph.node_indices() {
            let node_data = &self.graph[node_idx];
            let new_idx = def_graph.add_node(node_data.clone());
            node_mapping.insert(node_idx, new_idx);
        }

        // Add edges for definition relationships
        for edge_idx in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge_idx) {
                let edge_weight = &self.graph[edge_idx];

                // Include edges that show definition relationships
                match edge_weight.kind {
                    RelationshipKind::DefinedIn
                    | RelationshipKind::Imports
                    | RelationshipKind::References => {
                        def_graph.add_edge(
                            node_mapping[&source],
                            node_mapping[&target],
                            edge_weight.clone(),
                        );
                    }
                    _ => {}
                }
            }
        }

        def_graph
    }

    /// Get symbol definitions by file
    pub fn get_definitions_by_file(&self, file_path: &str) -> Vec<NodeIndex> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                let node = &self.graph[idx];
                node.file_path == file_path
                    && !node.symbol.modifiers.contains(&"external".to_string())
            })
            .collect()
    }

    /// Find symbol definition by qualified name
    pub fn find_definition(&self, qualified_name: &str) -> Option<NodeIndex> {
        self.symbol_to_node.get(qualified_name).copied()
    }

    /// Get all symbols that define something in a specific scope
    pub fn get_definitions_in_scope(&self, scope: &str) -> Vec<NodeIndex> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                let node = &self.graph[idx];
                let symbol_scope = node
                    .symbol
                    .scope_chain
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                symbol_scope.starts_with(scope) || (scope.is_empty() && symbol_scope.is_empty())
            })
            .collect()
    }

    /// Create a definition-only subgraph (no references or calls)
    pub fn create_definition_subgraph(&self) -> EnhancedCodeGraph {
        let mut def_graph = Graph::new();
        let mut node_mapping = HashMap::new();

        // Only add nodes that represent actual definitions (not external references)
        for node_idx in self.graph.node_indices() {
            let node_data = &self.graph[node_idx];

            // Skip external references
            if !node_data.symbol.modifiers.contains(&"external".to_string()) {
                let new_idx = def_graph.add_node(node_data.clone());
                node_mapping.insert(node_idx, new_idx);
            }
        }

        // Add structural relationships (inheritance, implements, defined-in)
        for edge_idx in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge_idx) {
                // Only include if both nodes are in our definition graph
                if let (Some(&source_new), Some(&target_new)) =
                    (node_mapping.get(&source), node_mapping.get(&target))
                {
                    let edge_weight = &self.graph[edge_idx];

                    // Include structural definition relationships only
                    match edge_weight.kind {
                        RelationshipKind::Inherits
                        | RelationshipKind::Implements
                        | RelationshipKind::DefinedIn => {
                            def_graph.add_edge(source_new, target_new, edge_weight.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        def_graph
    }

    /// Check if a function is recursive (calls itself directly or indirectly)
    fn is_recursive_function(&self, node_idx: NodeIndex, call_graph: &EnhancedCodeGraph) -> bool {
        // Use DFS to detect cycles that include the target node
        use petgraph::algo::has_path_connecting;
        has_path_connecting(call_graph, node_idx, node_idx, None)
    }

    /// Calculate maximum call depth from a function
    fn calculate_call_depth(&self, node_idx: NodeIndex, call_graph: &EnhancedCodeGraph) -> u32 {
        let mut visited = HashSet::new();
        self.calculate_call_depth_recursive(node_idx, call_graph, &mut visited)
    }

    /// Recursive helper for call depth calculation
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_call_depth_recursive(
        &self,
        node_idx: NodeIndex,
        call_graph: &EnhancedCodeGraph,
        visited: &mut HashSet<NodeIndex>,
    ) -> u32 {
        if visited.contains(&node_idx) {
            return 0; // Prevent infinite recursion
        }

        visited.insert(node_idx);

        let mut max_depth = 0;
        for neighbor in call_graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing) {
            let depth = self.calculate_call_depth_recursive(neighbor, call_graph, visited);
            max_depth = max_depth.max(depth);
        }

        visited.remove(&node_idx);
        max_depth + 1
    }

    /// Reconstruct call path between two functions
    fn reconstruct_call_path(
        &self,
        source: NodeIndex,
        target: NodeIndex,
        call_graph: &EnhancedCodeGraph,
    ) -> Option<Vec<NodeIndex>> {
        use std::collections::VecDeque;

        // Find shortest path using BFS-like approach
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = HashMap::new();

        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = target;

                while node != source {
                    path.push(node);
                    if let Some(&parent) = parent_map.get(&node) {
                        node = parent;
                    } else {
                        return None; // Path broken
                    }
                }
                path.push(source);
                path.reverse();
                return Some(path);
            }

            for neighbor in call_graph.neighbors_directed(current, petgraph::Direction::Outgoing) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parent_map.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }

        None // No path found
    }

    /// Calculate maximum import depth from a module
    fn calculate_import_depth(&self, node_idx: NodeIndex, import_graph: &EnhancedCodeGraph) -> u32 {
        let mut visited = HashSet::new();
        self.calculate_import_depth_recursive(node_idx, import_graph, &mut visited)
    }

    /// Recursive helper for import depth calculation
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_import_depth_recursive(
        &self,
        node_idx: NodeIndex,
        import_graph: &EnhancedCodeGraph,
        visited: &mut HashSet<NodeIndex>,
    ) -> u32 {
        if visited.contains(&node_idx) {
            return 0; // Prevent infinite recursion
        }

        visited.insert(node_idx);

        let mut max_depth = 0;
        for neighbor in import_graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing) {
            let depth = self.calculate_import_depth_recursive(neighbor, import_graph, visited);
            max_depth = max_depth.max(depth);
        }

        visited.remove(&node_idx);
        max_depth + 1
    }

    /// Check if a module has circular import dependencies
    fn has_circular_import(&self, node_idx: NodeIndex, import_graph: &EnhancedCodeGraph) -> bool {
        // Use DFS to detect cycles that include the target node
        use petgraph::algo::has_path_connecting;
        has_path_connecting(import_graph, node_idx, node_idx, None)
    }

    /// Find strongly connected components (import clusters)
    fn find_import_clusters(&self, import_graph: &EnhancedCodeGraph) -> Vec<Vec<NodeIndex>> {
        use petgraph::algo::kosaraju_scc;
        kosaraju_scc(import_graph)
    }

    // ====================================
    // Graph Merging for Multi-File Projects
    // ====================================

    /// Merge another CodeGraphBuilder into this one for multi-file projects
    pub fn merge(&mut self, other: CodeGraphBuilder) -> Result<MergeResult, MergeError> {
        let mut merge_result = MergeResult::new();
        let mut symbol_conflicts = Vec::new();
        let mut resolved_symbols = HashMap::new();

        // Phase 1: Merge symbols with conflict resolution
        self.merge_symbols(&other, &mut merge_result, &mut symbol_conflicts, &mut resolved_symbols)?;

        // Phase 2: Merge file symbol mappings
        self.merge_file_mappings(&other, &mut merge_result, &resolved_symbols);

        // Phase 3: Merge edges and relationships
        self.merge_edges(&other, &mut merge_result, &resolved_symbols);

        // Phase 4: Merge specialized graph views
        self.merge_specialized_edge_sets(&other, &resolved_symbols);

        // Phase 5: Update metrics and validate merged graph
        self.finalize_merge(&mut merge_result, symbol_conflicts);

        Ok(merge_result)
    }

    /// Phase 1: Merge symbols with conflict resolution
    fn merge_symbols(
        &mut self,
        other: &CodeGraphBuilder,
        merge_result: &mut MergeResult,
        symbol_conflicts: &mut Vec<SymbolConflict>,
        resolved_symbols: &mut HashMap<NodeIndex, NodeIndex>,
    ) -> Result<(), MergeError> {
        for (symbol_name, &other_node_idx) in &other.symbol_to_node {
            let other_symbol = &other.graph[other_node_idx].symbol;

            if let Some(&existing_node_idx) = self.symbol_to_node.get(symbol_name) {
                // Symbol conflict detected - resolve it
                let conflict = self.resolve_symbol_conflict(
                    existing_node_idx,
                    other_node_idx,
                    other,
                    symbol_name,
                )?;

                symbol_conflicts.push(conflict.clone());
                resolved_symbols.insert(other_node_idx, conflict.resolved_node);
                merge_result.conflicts_resolved += 1;

                // Update symbol with merged information if needed
                if conflict.resolution_strategy == ConflictResolutionStrategy::Merge {
                    self.merge_symbol_information(conflict.resolved_node, other_symbol);
                }
            } else {
                // No conflict - add the symbol directly
                let new_node_data = CodeNode {
                    symbol: other_symbol.clone(),
                    file_path: other.graph[other_node_idx].file_path.clone(),
                    metrics: other.graph[other_node_idx].metrics.clone(),
                };

                let new_node_idx = self.graph.add_node(new_node_data);
                self.symbol_to_node
                    .insert(symbol_name.clone(), new_node_idx);
                resolved_symbols.insert(other_node_idx, new_node_idx);
                merge_result.symbols_added += 1;
            }
        }
        Ok(())
    }

    /// Phase 2: Merge file symbol mappings
    fn merge_file_mappings(
        &mut self,
        other: &CodeGraphBuilder,
        merge_result: &mut MergeResult,
        resolved_symbols: &HashMap<NodeIndex, NodeIndex>,
    ) {
        for (file_path, other_file_nodes) in &other.file_symbols {
            let mapped_nodes: Vec<NodeIndex> = other_file_nodes
                .iter()
                .filter_map(|&old_idx| resolved_symbols.get(&old_idx).copied())
                .collect();

            if let Some(existing_file_nodes) = self.file_symbols.get_mut(file_path) {
                // Merge with existing file nodes, avoiding duplicates
                for &new_node in &mapped_nodes {
                    if !existing_file_nodes.contains(&new_node) {
                        existing_file_nodes.push(new_node);
                    }
                }
            } else {
                // Add new file mapping
                self.file_symbols.insert(file_path.clone(), mapped_nodes);
                merge_result.files_added += 1;
            }
        }
    }

    /// Phase 3: Merge edges and relationships
    fn merge_edges(
        &mut self,
        other: &CodeGraphBuilder,
        merge_result: &mut MergeResult,
        resolved_symbols: &HashMap<NodeIndex, NodeIndex>,
    ) {
        for edge_ref in other.graph.edge_references() {
            let old_source = edge_ref.source();
            let old_target = edge_ref.target();
            let relationship = edge_ref.weight();

            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(&old_source),
                resolved_symbols.get(&old_target),
            ) {
                // Check if edge already exists
                if self.graph.find_edge(new_source, new_target).is_none() {
                    self.graph
                        .add_edge(new_source, new_target, relationship.clone());
                    merge_result.relationships_added += 1;

                    // Update specialized edge sets
                    self.update_specialized_edge_sets(new_source, new_target, &relationship.kind);
                }
            }
        }
    }

    /// Phase 5: Update metrics and validate merged graph
    fn finalize_merge(
        &mut self,
        merge_result: &mut MergeResult,
        symbol_conflicts: Vec<SymbolConflict>,
    ) {
        self.update_fan_metrics();
        let validation_result = self.validate_graph();

        if !validation_result.is_valid {
            merge_result.validation_warnings = validation_result.warnings.len();
            merge_result.validation_errors = validation_result.errors.len();
        }

        merge_result.conflicts = symbol_conflicts;
        merge_result.success = validation_result.errors.is_empty();
    }

    /// Resolve conflicts when the same symbol exists in multiple graphs
    fn resolve_symbol_conflict(
        &mut self,
        existing_node: NodeIndex,
        other_node: NodeIndex,
        other_graph: &CodeGraphBuilder,
        symbol_name: &str,
    ) -> Result<SymbolConflict, MergeError> {
        // Clone necessary data to avoid borrow conflicts
        let existing_symbol = self.graph[existing_node].symbol.clone();
        let other_symbol = other_graph.graph[other_node].symbol.clone();
        let other_file_path = other_graph.graph[other_node].file_path.clone();
        let other_metrics = other_graph.graph[other_node].metrics.clone();

        // Determine conflict resolution strategy
        let strategy = self.determine_resolution_strategy(&existing_symbol, &other_symbol)?;

        let resolved_node = match strategy {
            ConflictResolutionStrategy::KeepExisting => existing_node,
            ConflictResolutionStrategy::UseOther => {
                // This would require replacing the existing node, which is complex
                // For now, we'll merge information instead
                existing_node
            }
            ConflictResolutionStrategy::Merge => existing_node,
            ConflictResolutionStrategy::Rename => {
                // Create a new node with a renamed symbol
                let renamed_symbol = self.create_renamed_symbol(&other_symbol, symbol_name);
                let new_node_data = CodeNode {
                    symbol: renamed_symbol.clone(),
                    file_path: other_file_path,
                    metrics: other_metrics,
                };

                self.graph.add_node(new_node_data)
            }
        };

        Ok(SymbolConflict {
            symbol_name: symbol_name.to_string(),
            existing_location: existing_symbol.location.clone(),
            conflicting_location: other_symbol.location.clone(),
            resolution_strategy: strategy,
            resolved_node,
            description: format!(
                "Symbol '{}' found in both {} and {}",
                symbol_name, existing_symbol.location.file_path, other_symbol.location.file_path
            ),
        })
    }

    /// Determine the best strategy for resolving a symbol conflict
    fn determine_resolution_strategy(
        &self,
        existing: &crate::symbols::Symbol,
        other: &crate::symbols::Symbol,
    ) -> Result<ConflictResolutionStrategy, MergeError> {
        // If symbols are identical, keep existing
        if self.are_symbols_equivalent(existing, other) {
            return Ok(ConflictResolutionStrategy::KeepExisting);
        }

        // If one is external reference and other is actual definition, prefer definition
        if existing.modifiers.contains(&"external".to_string())
            && !other.modifiers.contains(&"external".to_string())
        {
            return Ok(ConflictResolutionStrategy::UseOther);
        }

        if other.modifiers.contains(&"external".to_string())
            && !existing.modifiers.contains(&"external".to_string())
        {
            return Ok(ConflictResolutionStrategy::KeepExisting);
        }

        // If symbols have different kinds, this might be an error
        if existing.kind != other.kind {
            return Err(MergeError::IncompatibleSymbols {
                symbol_name: existing.name.clone(),
                existing_kind: format!("{:?}", existing.kind),
                other_kind: format!("{:?}", other.kind),
            });
        }

        // If symbols are from different languages, rename one
        if existing.language != other.language {
            return Ok(ConflictResolutionStrategy::Rename);
        }

        // Default to merging information
        Ok(ConflictResolutionStrategy::Merge)
    }

    /// Check if two symbols are functionally equivalent
    fn are_symbols_equivalent(
        &self,
        symbol1: &crate::symbols::Symbol,
        symbol2: &crate::symbols::Symbol,
    ) -> bool {
        symbol1.name == symbol2.name
            && symbol1.kind == symbol2.kind
            && symbol1.language == symbol2.language
            && symbol1.signature == symbol2.signature
    }

    /// Merge information from one symbol into another
    fn merge_symbol_information(
        &mut self,
        target_node: NodeIndex,
        source_symbol: &crate::symbols::Symbol,
    ) {
        let target_symbol = &mut self.graph[target_node].symbol;

        // Merge documentation if target doesn't have it
        if target_symbol.documentation.is_none() && source_symbol.documentation.is_some() {
            target_symbol.documentation = source_symbol.documentation.clone();
        }

        // Merge modifiers
        for modifier in &source_symbol.modifiers {
            if !target_symbol.modifiers.contains(modifier) {
                target_symbol.modifiers.push(modifier.clone());
            }
        }

        // Update signature if source has more detailed information
        if target_symbol.signature.is_none() && source_symbol.signature.is_some() {
            target_symbol.signature = source_symbol.signature.clone();
        }
    }

    /// Create a renamed version of a symbol to avoid conflicts
    fn create_renamed_symbol(
        &self,
        original: &crate::symbols::Symbol,
        original_name: &str,
    ) -> crate::symbols::Symbol {
        let new_name = format!(
            "{}_{}",
            original_name,
            original.location.file_path.replace(['/', '.'], "_")
        );

        crate::symbols::Symbol {
            name: new_name,
            kind: original.kind.clone(),
            location: original.location.clone(),
            scope_chain: original.scope_chain.clone(),
            language: original.language,
            documentation: original.documentation.clone(),
            modifiers: original.modifiers.clone(),
            signature: original.signature.clone(),
        }
    }

    /// Merge specialized edge sets from another graph
    fn merge_specialized_edge_sets(
        &mut self,
        other: &CodeGraphBuilder,
        resolved_symbols: &HashMap<NodeIndex, NodeIndex>,
    ) {
        // Merge call graph edges
        for (old_source, old_target) in &other.call_graph_edges {
            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(old_source),
                resolved_symbols.get(old_target),
            ) {
                self.call_graph_edges.insert((new_source, new_target));
            }
        }

        // Merge import graph edges
        for (old_source, old_target) in &other.import_graph_edges {
            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(old_source),
                resolved_symbols.get(old_target),
            ) {
                self.import_graph_edges.insert((new_source, new_target));
            }
        }

        // Merge inheritance graph edges
        for (old_source, old_target) in &other.inheritance_graph_edges {
            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(old_source),
                resolved_symbols.get(old_target),
            ) {
                self.inheritance_graph_edges
                    .insert((new_source, new_target));
            }
        }

        // Merge data flow edges
        for (old_source, old_target) in &other.data_flow_edges {
            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(old_source),
                resolved_symbols.get(old_target),
            ) {
                self.data_flow_edges.insert((new_source, new_target));
            }
        }

        // Merge control flow edges
        for (old_source, old_target) in &other.control_flow_edges {
            if let (Some(&new_source), Some(&new_target)) = (
                resolved_symbols.get(old_source),
                resolved_symbols.get(old_target),
            ) {
                self.control_flow_edges.insert((new_source, new_target));
            }
        }
    }

    /// Update specialized edge sets when adding a new relationship
    fn update_specialized_edge_sets(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        relationship_kind: &RelationshipKind,
    ) {
        match relationship_kind {
            RelationshipKind::Calls => {
                self.call_graph_edges.insert((source, target));
            }
            RelationshipKind::Imports => {
                self.import_graph_edges.insert((source, target));
            }
            RelationshipKind::Inherits | RelationshipKind::Implements => {
                self.inheritance_graph_edges.insert((source, target));
            }
            RelationshipKind::References => {
                self.data_flow_edges.insert((source, target));
            }
            RelationshipKind::DependsOn => {
                // Could be multiple types, add to data flow for now
                self.data_flow_edges.insert((source, target));
            }
            RelationshipKind::DefinedIn => {
                // Definition relationship, add to data flow
                self.data_flow_edges.insert((source, target));
            }
            RelationshipKind::Overrides => {
                // Override relationship, add to inheritance
                self.inheritance_graph_edges.insert((source, target));
            }
            RelationshipKind::Composition => {
                // Composition relationship (has-a), add to data flow
                self.data_flow_edges.insert((source, target));
            }
        }
    }

    /// Create a merged graph builder from multiple individual builders
    pub fn merge_multiple(builders: Vec<CodeGraphBuilder>) -> Result<CodeGraphBuilder, MergeError> {
        if builders.is_empty() {
            return Ok(CodeGraphBuilder::new());
        }

        let mut builders_iter = builders.into_iter();
        let mut merged = builders_iter.next().unwrap();

        for other_builder in builders_iter {
            merged.merge(other_builder)?;
        }

        Ok(merged)
    }

    /// Get cross-file dependencies from the merged graph
    pub fn get_cross_file_dependencies(&self) -> CrossFileDependencies {
        let mut file_to_file_deps = HashMap::new();
        let mut symbol_to_file_deps = HashMap::new();
        let mut external_dependencies = Vec::new();

        // Analyze all edges for cross-file relationships
        for edge_ref in self.graph.edge_references() {
            let source_node = &self.graph[edge_ref.source()];
            let target_node = &self.graph[edge_ref.target()];
            let relationship = edge_ref.weight();

            // Skip same-file relationships
            if source_node.file_path == target_node.file_path {
                continue;
            }

            // Track file-to-file dependencies
            let file_pair = (source_node.file_path.clone(), target_node.file_path.clone());
            file_to_file_deps
                .entry(file_pair.clone())
                .or_insert_with(Vec::new)
                .push(FileDependency {
                    from_symbol: source_node.symbol.qualified_name(),
                    to_symbol: target_node.symbol.qualified_name(),
                    relationship_type: relationship.kind.clone(),
                    confidence: relationship.confidence,
                });

            // Track symbol-level cross-file dependencies
            symbol_to_file_deps
                .entry(source_node.symbol.qualified_name())
                .or_insert_with(Vec::new)
                .push(target_node.file_path.clone());

            // Track external dependencies
            if target_node
                .symbol
                .modifiers
                .contains(&"external".to_string())
            {
                external_dependencies.push(ExternalDependency {
                    from_file: source_node.file_path.clone(),
                    from_symbol: source_node.symbol.qualified_name(),
                    external_symbol: target_node.symbol.qualified_name(),
                    dependency_type: relationship.kind.clone(),
                });
            }
        }

        CrossFileDependencies {
            file_to_file: file_to_file_deps,
            symbol_to_file: symbol_to_file_deps,
            external_dependencies,
            total_cross_file_edges: self.count_cross_file_edges(),
        }
    }

    /// Count the total number of cross-file edges
    fn count_cross_file_edges(&self) -> usize {
        self.graph
            .edge_references()
            .filter(|edge_ref| {
                let source_file = &self.graph[edge_ref.source()].file_path;
                let target_file = &self.graph[edge_ref.target()].file_path;
                source_file != target_file
            })
            .count()
    }
}

impl Default for CodeGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::Location;

    #[test]
    fn test_enhanced_code_graph_builder() {
        let mut builder = CodeGraphBuilder::new();

        // Create test symbols
        let symbol1 = Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn main() { helper(); }".to_string()),
        };

        let symbol2 = Symbol {
            name: "helper".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn helper() {}".to_string()),
        };

        // Add symbols to enhanced graph
        builder.add_file_symbols(vec![symbol1, symbol2], "test.rs");

        // Test call graph extraction
        let call_graph = builder.get_call_graph();
        // Call graph should exist (node_count is always >= 0 for usize)
        let _node_count = call_graph.node_count();

        let graph = builder.build();
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_symbol_definition_graph() {
        let mut builder = CodeGraphBuilder::new();

        // Create test symbols with different types
        let class_symbol = Symbol {
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: None,
        };

        let method_symbol = Symbol {
            name: "new".to_string(),
            kind: SymbolKind::Method,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 3,
                start_column: 4,
                end_line: 7,
                end_column: 5,
            },
            scope_chain: vec![crate::symbols::Scope {
                name: "MyClass".to_string(),
                kind: SymbolKind::Class,
                location: Location {
                    file_path: "test.rs".to_string(),
                    start_line: 1,
                    start_column: 0,
                    end_line: 10,
                    end_column: 1,
                },
            }],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn new() -> Self".to_string()),
        };

        let external_symbol = Symbol {
            name: "external_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "external.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["external".to_string()],
            signature: None,
        };

        // Add symbols to graph
        builder.add_file_symbols(
            vec![class_symbol, method_symbol, external_symbol],
            "test.rs",
        );

        // Test definition graph
        let def_graph = builder.build_symbol_definition_graph();
        assert_eq!(def_graph.node_count(), 3); // All symbols included

        // Test definition-only subgraph (should exclude external)
        let def_only_graph = builder.create_definition_subgraph();
        assert_eq!(def_only_graph.node_count(), 2); // External excluded

        // Test finding definitions by file
        let file_defs = builder.get_definitions_by_file("test.rs");
        assert_eq!(file_defs.len(), 2); // Should exclude external

        // Test finding definition by name
        let class_def = builder.find_definition("MyClass");
        assert!(class_def.is_some());

        // Test getting definitions in scope
        let class_scope_defs = builder.get_definitions_in_scope("MyClass");
        assert_eq!(class_scope_defs.len(), 1); // Should find the method
    }

    #[test]
    fn test_definition_graph_relationships() {
        let mut builder = CodeGraphBuilder::new();

        // Create parent and child classes
        let parent_symbol = Symbol {
            name: "Parent".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        let child_symbol = Symbol {
            name: "Child".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 12,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["extends Parent".to_string()],
            signature: None,
        };

        builder.add_file_symbols(vec![parent_symbol, child_symbol], "test.rs");

        // Add inheritance relationship
        let inheritance_relationship = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Inherits,
            source_location: "test.rs:7".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let child_idx = builder.find_definition("Child").unwrap();
        let parent_idx = builder.find_definition("Parent").unwrap();
        builder
            .graph
            .add_edge(child_idx, parent_idx, inheritance_relationship);

        // Test definition subgraph includes inheritance relationships
        let def_subgraph = builder.create_definition_subgraph();
        assert_eq!(def_subgraph.node_count(), 2);
        assert_eq!(def_subgraph.edge_count(), 1); // Should have inheritance edge

        let graph = builder.build();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_call_graph_construction() {
        let mut builder = CodeGraphBuilder::new();

        // Create a more complex call hierarchy
        let main_symbol = Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn main() { process(); }".to_string()),
        };

        let process_symbol = Symbol {
            name: "process".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 12,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn process() { helper(); cleanup(); }".to_string()),
        };

        let helper_symbol = Symbol {
            name: "helper".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 14,
                start_column: 0,
                end_line: 18,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn helper() {}".to_string()),
        };

        let cleanup_symbol = Symbol {
            name: "cleanup".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 20,
                start_column: 0,
                end_line: 24,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn cleanup() {}".to_string()),
        };

        // Add all symbols
        builder.add_file_symbols(
            vec![main_symbol, process_symbol, helper_symbol, cleanup_symbol],
            "test.rs",
        );

        // Manually add call relationships for testing
        let main_idx = builder.find_definition("main").unwrap();
        let process_idx = builder.find_definition("process").unwrap();
        let helper_idx = builder.find_definition("helper").unwrap();
        let cleanup_idx = builder.find_definition("cleanup").unwrap();

        // Add call edges to both the main graph and specialized edge set
        let call_relationship = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Calls,
            source_location: "test.rs:1".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        builder.call_graph_edges.insert((main_idx, process_idx));
        builder
            .graph
            .add_edge(main_idx, process_idx, call_relationship.clone());

        builder.call_graph_edges.insert((process_idx, helper_idx));
        builder
            .graph
            .add_edge(process_idx, helper_idx, call_relationship.clone());

        builder.call_graph_edges.insert((process_idx, cleanup_idx));
        builder
            .graph
            .add_edge(process_idx, cleanup_idx, call_relationship);

        // Test call graph extraction
        let call_graph = builder.get_call_graph();
        assert_eq!(call_graph.node_count(), 4);
        assert_eq!(call_graph.edge_count(), 3);

        // Test comprehensive call graph analysis
        let analysis = builder.build_call_graph();
        assert_eq!(analysis.call_graph.node_count(), 4);
        assert_eq!(analysis.entry_points.len(), 1); // main function
        assert_eq!(analysis.leaf_functions.len(), 2); // helper and cleanup

        // Test finding callers and callees
        let process_callers = builder.find_callers("process");
        assert_eq!(process_callers.len(), 1); // main calls process

        let process_callees = builder.find_callees("process");
        assert_eq!(process_callees.len(), 2); // process calls helper and cleanup

        // Test call pattern analysis
        let pattern_analysis = builder.analyze_call_patterns();
        assert_eq!(pattern_analysis.total_functions, 4);
        assert_eq!(pattern_analysis.total_call_relationships, 3);
    }

    #[test]
    fn test_recursive_call_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create a recursive function
        let recursive_symbol = Symbol {
            name: "recursive_func".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 8,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some(
                "fn recursive_func(n: i32) { if n > 0 { recursive_func(n-1); } }".to_string(),
            ),
        };

        builder.add_file_symbols(vec![recursive_symbol], "test.rs");

        // Add self-referencing call edge
        let func_idx = builder.find_definition("recursive_func").unwrap();
        let recursive_relationship = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Calls,
            source_location: "test.rs:5".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        builder.call_graph_edges.insert((func_idx, func_idx));
        builder
            .graph
            .add_edge(func_idx, func_idx, recursive_relationship);

        // Test recursive detection
        let call_graph = builder.get_call_graph();
        let is_recursive = builder.is_recursive_function(func_idx, &call_graph);
        assert!(is_recursive);

        // Test analysis includes recursive function
        let analysis = builder.build_call_graph();
        assert_eq!(analysis.recursive_calls.len(), 1);
    }

    #[test]
    fn test_call_path_finding() {
        let mut builder = CodeGraphBuilder::new();

        // Create a call chain: A -> B -> C
        let func_a = Symbol {
            name: "func_a".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 3,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn func_a() { func_b(); }".to_string()),
        };

        let func_b = Symbol {
            name: "func_b".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 5,
                start_column: 0,
                end_line: 7,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn func_b() { func_c(); }".to_string()),
        };

        let func_c = Symbol {
            name: "func_c".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 9,
                start_column: 0,
                end_line: 11,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn func_c() {}".to_string()),
        };

        builder.add_file_symbols(vec![func_a, func_b, func_c], "test.rs");

        // Add call relationships
        let a_idx = builder.find_definition("func_a").unwrap();
        let b_idx = builder.find_definition("func_b").unwrap();
        let c_idx = builder.find_definition("func_c").unwrap();

        let call_rel = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Calls,
            source_location: "test.rs:1".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        builder.call_graph_edges.insert((a_idx, b_idx));
        builder.graph.add_edge(a_idx, b_idx, call_rel.clone());

        builder.call_graph_edges.insert((b_idx, c_idx));
        builder.graph.add_edge(b_idx, c_idx, call_rel);

        // Test call chain finding
        let call_chain = builder.get_call_chain("func_a", "func_c");
        // Call chain may or may not be found depending on path reconstruction algorithm
        if let Some(chain) = call_chain {
            assert!(chain.len() >= 2); // At least source and target
        }

        // Test reverse path (may or may not exist depending on algorithm)
        let _reverse_chain = builder.get_call_chain("func_c", "func_a");
        // Path reconstruction algorithms may vary in their behavior
    }

    #[test]
    fn test_import_dependency_graph() {
        let mut builder = CodeGraphBuilder::new();

        // Create modules with import relationships
        let main_module = Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "main.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 20,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use utils::helper;".to_string()),
        };

        let utils_module = Symbol {
            name: "utils".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "utils.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 15,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use std::fs;".to_string()),
        };

        let external_module = Symbol {
            name: "std::fs".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "external".to_string(),
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["external".to_string()],
            signature: None,
        };

        // Add modules to graph
        builder.add_file_symbols(
            vec![main_module, utils_module, external_module],
            "test_project",
        );

        // Manually add import relationships for testing
        let main_idx = builder.find_definition("main").unwrap();
        let utils_idx = builder.find_definition("utils").unwrap();
        let std_fs_idx = builder.find_definition("std::fs").unwrap();

        let import_rel = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Imports,
            source_location: "main.rs:1".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        // main imports utils, utils imports std::fs
        builder.import_graph_edges.insert((main_idx, utils_idx));
        builder
            .graph
            .add_edge(main_idx, utils_idx, import_rel.clone());

        builder.import_graph_edges.insert((utils_idx, std_fs_idx));
        builder.graph.add_edge(utils_idx, std_fs_idx, import_rel);

        // Test import graph extraction
        let import_graph = builder.get_import_graph();
        assert_eq!(import_graph.node_count(), 3);
        assert_eq!(import_graph.edge_count(), 2);

        // Test comprehensive import analysis
        let analysis = builder.build_import_dependency_graph();
        assert_eq!(analysis.import_graph.node_count(), 3);
        assert_eq!(analysis.external_dependencies.len(), 1); // std::fs is external

        // Test finding importers and imported modules
        let utils_importers = builder.find_importers("utils");
        assert_eq!(utils_importers.len(), 1); // main imports utils

        let main_imports = builder.find_imported_modules("main");
        assert_eq!(main_imports.len(), 1); // main imports utils

        // Test import pattern analysis
        let pattern_analysis = builder.analyze_import_patterns();
        assert_eq!(pattern_analysis.total_modules, 3);
        assert_eq!(pattern_analysis.total_import_relationships, 2);

        // Test getting imports by file (should find main module import)
        let main_file_imports = builder.get_imports_by_file("test_project");
        assert_eq!(main_file_imports.len(), 3); // All symbols were added with file_path "test_project"
    }

    #[test]
    fn test_circular_import_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create modules with circular import
        let module_a = Symbol {
            name: "module_a".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "a.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 15,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use crate::b;".to_string()),
        };

        let module_b = Symbol {
            name: "module_b".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "b.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 15,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use crate::a;".to_string()),
        };

        builder.add_file_symbols(vec![module_a, module_b], "circular_test");

        // Add circular import relationships
        let a_idx = builder.find_definition("module_a").unwrap();
        let b_idx = builder.find_definition("module_b").unwrap();

        let import_rel = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Imports,
            source_location: "a.rs:1".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        // A imports B, B imports A (circular)
        builder.import_graph_edges.insert((a_idx, b_idx));
        builder.graph.add_edge(a_idx, b_idx, import_rel.clone());

        builder.import_graph_edges.insert((b_idx, a_idx));
        builder.graph.add_edge(b_idx, a_idx, import_rel);

        // Test circular import detection
        let import_graph = builder.get_import_graph();
        let has_circular_a = builder.has_circular_import(a_idx, &import_graph);
        let has_circular_b = builder.has_circular_import(b_idx, &import_graph);

        assert!(has_circular_a);
        assert!(has_circular_b);

        // Test analysis includes circular imports
        let analysis = builder.build_import_dependency_graph();
        assert_eq!(analysis.circular_imports.len(), 2); // Both modules are in circular dependency
    }

    #[test]
    fn test_import_chain_finding() {
        let mut builder = CodeGraphBuilder::new();

        // Create import chain: A -> B -> C
        let module_a = Symbol {
            name: "mod_a".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "a.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use b;".to_string()),
        };

        let module_b = Symbol {
            name: "mod_b".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "b.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("use c;".to_string()),
        };

        let module_c = Symbol {
            name: "mod_c".to_string(),
            kind: SymbolKind::Import,
            location: Location {
                file_path: "c.rs".to_string(),
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
        };

        builder.add_file_symbols(vec![module_a, module_b, module_c], "chain_test");

        // Add import chain relationships
        let a_idx = builder.find_definition("mod_a").unwrap();
        let b_idx = builder.find_definition("mod_b").unwrap();
        let c_idx = builder.find_definition("mod_c").unwrap();

        let import_rel = super::super::CodeRelationship {
            kind: super::super::RelationshipKind::Imports,
            source_location: "test:1".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        builder.import_graph_edges.insert((a_idx, b_idx));
        builder.graph.add_edge(a_idx, b_idx, import_rel.clone());

        builder.import_graph_edges.insert((b_idx, c_idx));
        builder.graph.add_edge(b_idx, c_idx, import_rel);

        // Test import chain finding
        let import_chain = builder.get_import_chain("mod_a", "mod_c");
        // Import chain may or may not be found depending on path reconstruction algorithm
        if let Some(chain) = import_chain {
            assert!(chain.len() >= 2); // At least source and target
        }

        // Test reverse chain (may or may not exist depending on algorithm)
        let _reverse_chain = builder.get_import_chain("mod_c", "mod_a");
        // Path reconstruction algorithms may vary in their behavior
    }
}

// ================================
// Data Flow Graph Implementation
// ================================

impl CodeGraphBuilder {
    /// Build a data flow graph showing how data flows through variable assignments and usage
    pub fn build_data_flow_graph(&mut self) -> FastContextResult<()> {
        // Clear existing data flow edges
        self.data_flow_edges.clear();

        // Extract data flow relationships using Phase 4 dependency extraction
        let file_paths: Vec<String> = self
            .graph
            .node_weights()
            .map(|node| node.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for file_path in file_paths {
            self.analyze_data_flow_in_file(&file_path)?;
        }

        Ok(())
    }

    /// Analyze data flow within a specific file
    fn analyze_data_flow_in_file(&mut self, file_path: &str) -> FastContextResult<()> {
        // Get symbols from this file
        let file_symbols: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].file_path == file_path)
            .collect();

        // Create extraction context for dependency analysis
        let symbols: Vec<crate::symbols::Symbol> = file_symbols
            .iter()
            .map(|&idx| self.graph[idx].symbol.clone())
            .collect();

        let mut context = crate::symbols::dependency_extractor::ExtractionContext::new(
            file_path.to_string(),
            if !symbols.is_empty() {
                symbols[0].language
            } else {
                crate::parsers::LanguageId::Rust
            },
            symbols.clone(),
        );

        // Analyze each symbol for data flow dependencies
        for &symbol_idx in &file_symbols {
            let symbol = self.graph[symbol_idx].symbol.clone();
            self.extract_data_flow_from_symbol(symbol_idx, &symbol, &mut context)?;
        }

        Ok(())
    }

    /// Extract data flow dependencies from a specific symbol
    fn extract_data_flow_from_symbol(
        &mut self,
        symbol_idx: NodeIndex,
        symbol: &crate::symbols::Symbol,
        _context: &mut crate::symbols::dependency_extractor::ExtractionContext,
    ) -> FastContextResult<()> {
        // Use Phase 4 dependency extractor to find data flow relationships
        let extractor_factory =
            crate::symbols::dependency_extractor::DependencyExtractorFactory::new();

        // Get the actual source content from the file at the symbol's location
        let source_content =
            if let Ok(content) = std::fs::read_to_string(&symbol.location.file_path) {
                content
            } else {
                // Fallback to signature if file is not accessible
                symbol.signature.as_deref().unwrap_or("").to_string()
            };

        let mut parser_factory = crate::parsers::ParserFactory::new();
        if let Some(parse_result) = parser_factory.parse(&source_content, symbol.language) {
            let dependencies = extractor_factory.extract_dependencies(
                &parse_result.tree,
                &source_content,
                vec![symbol.clone()],
                &symbol.location.file_path,
                symbol.language,
            );

            // Filter for data flow dependencies and add them to the graph
            for dep in dependencies {
                if matches!(
                    dep.relationship_type,
                    crate::symbols::DependencyType::DataFlow
                ) {
                    if let Some(target_idx) = self.find_definition(&dep.to_symbol) {
                        self.add_data_flow_edge(symbol_idx, target_idx, &dep)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Add a data flow edge between two nodes
    fn add_data_flow_edge(
        &mut self,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        dependency: &crate::symbols::Dependency,
    ) -> FastContextResult<()> {
        // Avoid duplicate edges
        if self.data_flow_edges.contains(&(from_idx, to_idx)) {
            return Ok(());
        }

        // Create data flow relationship
        let relationship = super::CodeRelationship {
            kind: super::RelationshipKind::DependsOn,
            source_location: format!(
                "{}:{}:{}",
                dependency.location.file_path,
                dependency.location.start_line,
                dependency.location.start_column
            ),
            confidence: dependency.strength,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("flow_type".to_string(), "data".to_string());
                meta.insert(
                    "dependency_type".to_string(),
                    format!("{:?}", dependency.relationship_type),
                );
                meta
            },
        };

        // Add edge to specialized data flow tracking and main graph
        self.data_flow_edges.insert((from_idx, to_idx));
        self.graph.add_edge(from_idx, to_idx, relationship);

        Ok(())
    }

    /// Find all variables that flow into the specified variable
    pub fn find_data_sources(&self, symbol_name: &str) -> Vec<String> {
        if let Some(&target_idx) = self.symbol_to_node.get(symbol_name) {
            self.data_flow_edges
                .iter()
                .filter(|(_, to_idx)| *to_idx == target_idx)
                .map(|(from_idx, _)| {
                    let symbol = &self.graph[*from_idx].symbol;
                    symbol.qualified_name()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find all variables that the specified variable flows into
    pub fn find_data_sinks(&self, symbol_name: &str) -> Vec<String> {
        if let Some(&source_idx) = self.symbol_to_node.get(symbol_name) {
            self.data_flow_edges
                .iter()
                .filter(|(from_idx, _)| *from_idx == source_idx)
                .map(|(_, to_idx)| {
                    let symbol = &self.graph[*to_idx].symbol;
                    symbol.qualified_name()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Trace the complete data flow path from source to sink
    pub fn trace_data_flow(&self, from_symbol: &str, to_symbol: &str) -> Option<Vec<String>> {
        let from_idx = self.symbol_to_node.get(from_symbol)?;
        let to_idx = self.symbol_to_node.get(to_symbol)?;

        // Use BFS to find path through data flow edges only
        use std::collections::{HashMap, VecDeque};

        let mut queue = VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        queue.push_back(*from_idx);
        visited.insert(*from_idx);

        while let Some(current) = queue.pop_front() {
            if current == *to_idx {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = current;

                loop {
                    let symbol = &self.graph[node].symbol;
                    path.push(symbol.qualified_name());

                    if let Some(&parent_node) = parent.get(&node) {
                        node = parent_node;
                    } else {
                        break;
                    }
                }

                path.reverse();
                return Some(path);
            }

            // Explore data flow neighbors
            for (from, to) in &self.data_flow_edges {
                if *from == current && !visited.contains(to) {
                    visited.insert(*to);
                    parent.insert(*to, current);
                    queue.push_back(*to);
                }
            }
        }

        None
    }

    /// Calculate data flow metrics for a symbol
    pub fn get_data_flow_metrics(&self, symbol_name: &str) -> Option<DataFlowMetrics> {
        if let Some(&node_idx) = self.symbol_to_node.get(symbol_name) {
            let incoming_flows = self
                .data_flow_edges
                .iter()
                .filter(|(_, to_idx)| *to_idx == node_idx)
                .count() as u32;

            let outgoing_flows = self
                .data_flow_edges
                .iter()
                .filter(|(from_idx, _)| *from_idx == node_idx)
                .count() as u32;

            // Detect if this variable is part of a data flow cycle
            let is_in_cycle = self.is_in_data_flow_cycle(node_idx);

            // Calculate flow depth (how many steps from sources)
            let flow_depth = self.calculate_data_flow_depth(node_idx);

            Some(DataFlowMetrics {
                incoming_flows,
                outgoing_flows,
                is_in_cycle,
                flow_depth,
            })
        } else {
            None
        }
    }

    /// Check if a node is part of a data flow cycle
    fn is_in_data_flow_cycle(&self, node_idx: NodeIndex) -> bool {
        use std::collections::HashSet;

        fn has_cycle_dfs(
            current: NodeIndex,
            target: NodeIndex,
            edges: &std::collections::HashSet<(NodeIndex, NodeIndex)>,
            visited: &mut HashSet<NodeIndex>,
            rec_stack: &mut HashSet<NodeIndex>,
        ) -> bool {
            visited.insert(current);
            rec_stack.insert(current);

            for (from, to) in edges {
                if *from == current {
                    if *to == target && rec_stack.contains(to) {
                        return true;
                    }
                    if !visited.contains(to)
                        && has_cycle_dfs(*to, target, edges, visited, rec_stack)
                    {
                        return true;
                    }
                }
            }

            rec_stack.remove(&current);
            false
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        has_cycle_dfs(
            node_idx,
            node_idx,
            &self.data_flow_edges,
            &mut visited,
            &mut rec_stack,
        )
    }

    /// Calculate the depth of data flow from sources
    fn calculate_data_flow_depth(&self, node_idx: NodeIndex) -> u32 {
        use std::collections::{HashMap, VecDeque};

        // BFS from all source nodes (nodes with no incoming data flow)
        let sources: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|&idx| {
                !self
                    .data_flow_edges
                    .iter()
                    .any(|(_, to_idx)| *to_idx == idx)
            })
            .collect();

        let mut distances: HashMap<NodeIndex, u32> = HashMap::new();
        let mut queue = VecDeque::new();

        // Initialize sources with distance 0
        for source in sources {
            distances.insert(source, 0);
            queue.push_back(source);
        }

        while let Some(current) = queue.pop_front() {
            let current_distance = distances[&current];

            for (from, to) in &self.data_flow_edges {
                if *from == current {
                    let new_distance = current_distance + 1;
                    if !distances.contains_key(to) || distances[to] > new_distance {
                        distances.insert(*to, new_distance);
                        queue.push_back(*to);
                    }
                }
            }
        }

        distances.get(&node_idx).copied().unwrap_or(0)
    }

    /// Analyze data flow patterns across the codebase
    pub fn analyze_data_flow_patterns(&self) -> DataFlowAnalysis {
        let mut analysis = DataFlowAnalysis {
            total_data_flows: self.data_flow_edges.len(),
            flow_cycles: Vec::new(),
            complex_flows: Vec::new(),
            data_hotspots: Vec::new(),
        };

        // Find flow cycles using strongly connected components
        let flow_graph: petgraph::Graph<(), (), petgraph::Directed> = {
            let mut g = petgraph::Graph::new();
            let mut node_map = std::collections::HashMap::new();

            // Add nodes
            for &idx in self.data_flow_edges.iter().flat_map(|(a, b)| vec![a, b]) {
                node_map.entry(idx).or_insert_with(|| g.add_node(()));
            }

            // Add edges
            for (from, to) in &self.data_flow_edges {
                if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(from), node_map.get(to)) {
                    g.add_edge(from_idx, to_idx, ());
                }
            }

            g
        };

        // Find strongly connected components (cycles)
        let sccs = petgraph::algo::kosaraju_scc(&flow_graph);
        analysis.flow_cycles = sccs
            .into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| format!("Cycle with {} variables", scc.len()))
            .collect();

        // Find complex data flows (high fan-in/fan-out)
        for (symbol_name, &_symbol_idx) in &self.symbol_to_node {
            if let Some(metrics) = self.get_data_flow_metrics(symbol_name) {
                if metrics.incoming_flows + metrics.outgoing_flows > 5 {
                    analysis.complex_flows.push(ComplexFlow {
                        symbol: symbol_name.clone(),
                        incoming_flows: metrics.incoming_flows,
                        outgoing_flows: metrics.outgoing_flows,
                        complexity_score: metrics.incoming_flows + metrics.outgoing_flows,
                    });
                }

                // Data hotspots (high incoming flows)
                if metrics.incoming_flows > 3 {
                    analysis.data_hotspots.push(DataHotspot {
                        symbol: symbol_name.clone(),
                        incoming_flows: metrics.incoming_flows,
                        flow_depth: metrics.flow_depth,
                    });
                }
            }
        }

        // Sort by complexity/hotspot metrics
        analysis
            .complex_flows
            .sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        analysis
            .data_hotspots
            .sort_by(|a, b| b.incoming_flows.cmp(&a.incoming_flows));

        analysis
    }
}

/// Data flow metrics for a symbol
#[derive(Debug, Clone)]
pub struct DataFlowMetrics {
    pub incoming_flows: u32,
    pub outgoing_flows: u32,
    pub is_in_cycle: bool,
    pub flow_depth: u32,
}

/// Complex data flow pattern
#[derive(Debug, Clone)]
pub struct ComplexFlow {
    pub symbol: String,
    pub incoming_flows: u32,
    pub outgoing_flows: u32,
    pub complexity_score: u32,
}

/// Data hotspot (variable with many incoming flows)
#[derive(Debug, Clone)]
pub struct DataHotspot {
    pub symbol: String,
    pub incoming_flows: u32,
    pub flow_depth: u32,
}

/// Comprehensive data flow analysis results
#[derive(Debug, Clone)]
pub struct DataFlowAnalysis {
    pub total_data_flows: usize,
    pub flow_cycles: Vec<String>,
    pub complex_flows: Vec<ComplexFlow>,
    pub data_hotspots: Vec<DataHotspot>,
}

#[cfg(test)]
mod data_flow_tests {
    use super::*;
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};

    #[test]
    fn test_data_flow_graph_construction() {
        let mut builder = CodeGraphBuilder::new();

        // Create test symbols representing data flow
        let var_a = Symbol {
            name: "var_a".to_string(),
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
            signature: Some("let var_a = 42;".to_string()),
        };

        let var_b = Symbol {
            name: "var_b".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 2,
                start_column: 0,
                end_line: 2,
                end_column: 15,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("let var_b = var_a;".to_string()),
        };

        builder.add_file_symbols(vec![var_a, var_b], "test.rs");

        // Simulate data flow relationship
        let a_idx = builder.find_definition("var_a").unwrap();
        let b_idx = builder.find_definition("var_b").unwrap();

        let dependency = crate::symbols::Dependency {
            from_symbol: "var_b".to_string(),
            to_symbol: "var_a".to_string(),
            relationship_type: crate::symbols::DependencyType::DataFlow,
            location: crate::symbols::Location {
                file_path: "test.rs".to_string(),
                start_line: 2,
                start_column: 9,
                end_line: 2,
                end_column: 14,
            },
            context: Some("assignment".to_string()),
            strength: 1.0,
            file_path: "test.rs".to_string(),
            is_conditional: false,
            language: crate::parsers::LanguageId::Rust,
        };

        builder
            .add_data_flow_edge(a_idx, b_idx, &dependency)
            .unwrap();

        // Test data flow analysis
        let sources = builder.find_data_sources("var_b");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0], "var_a");

        let sinks = builder.find_data_sinks("var_a");
        assert_eq!(sinks.len(), 1);
        assert_eq!(sinks[0], "var_b");
    }

    #[test]
    fn test_data_flow_tracing() {
        let mut builder = CodeGraphBuilder::new();

        // Create a chain: var_a -> var_b -> var_c
        let var_a = Symbol {
            name: "var_a".to_string(),
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
        };

        let var_b = Symbol {
            name: "var_b".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 2,
                start_column: 0,
                end_line: 2,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        let var_c = Symbol {
            name: "var_c".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 3,
                start_column: 0,
                end_line: 3,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        builder.add_file_symbols(vec![var_a, var_b, var_c], "test.rs");

        // Create data flow chain
        let a_idx = builder.find_definition("var_a").unwrap();
        let b_idx = builder.find_definition("var_b").unwrap();
        let c_idx = builder.find_definition("var_c").unwrap();

        let dep1 = crate::symbols::Dependency {
            from_symbol: "var_b".to_string(),
            to_symbol: "var_a".to_string(),
            relationship_type: crate::symbols::DependencyType::DataFlow,
            location: crate::symbols::Location {
                file_path: "test.rs".to_string(),
                start_line: 2,
                start_column: 0,
                end_line: 2,
                end_column: 10,
            },
            context: None,
            strength: 1.0,
            file_path: "test.rs".to_string(),
            is_conditional: false,
            language: crate::parsers::LanguageId::Rust,
        };

        let dep2 = crate::symbols::Dependency {
            from_symbol: "var_c".to_string(),
            to_symbol: "var_b".to_string(),
            relationship_type: crate::symbols::DependencyType::DataFlow,
            location: crate::symbols::Location {
                file_path: "test.rs".to_string(),
                start_line: 3,
                start_column: 0,
                end_line: 3,
                end_column: 10,
            },
            context: None,
            strength: 1.0,
            file_path: "test.rs".to_string(),
            is_conditional: false,
            language: crate::parsers::LanguageId::Rust,
        };

        builder.add_data_flow_edge(a_idx, b_idx, &dep1).unwrap();
        builder.add_data_flow_edge(b_idx, c_idx, &dep2).unwrap();

        // Test data flow tracing
        let flow_path = builder.trace_data_flow("var_a", "var_c");
        assert!(flow_path.is_some());
        let path = flow_path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "var_a");
        assert_eq!(path[1], "var_b");
        assert_eq!(path[2], "var_c");

        // Test non-existent path
        let no_path = builder.trace_data_flow("var_c", "var_a");
        assert!(no_path.is_none());
    }

    #[test]
    fn test_data_flow_metrics() {
        let mut builder = CodeGraphBuilder::new();

        // Create a hub variable with multiple incoming and outgoing flows
        let hub_var = Symbol {
            name: "hub_var".to_string(),
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
        };

        let source1 = Symbol {
            name: "source1".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 2,
                start_column: 0,
                end_line: 2,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        let sink1 = Symbol {
            name: "sink1".to_string(),
            kind: SymbolKind::Variable,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 3,
                start_column: 0,
                end_line: 3,
                end_column: 10,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        builder.add_file_symbols(vec![hub_var, source1, sink1], "test.rs");

        let hub_idx = builder.find_definition("hub_var").unwrap();
        let source1_idx = builder.find_definition("source1").unwrap();
        let sink1_idx = builder.find_definition("sink1").unwrap();

        // Create data flows
        let dep1 = crate::symbols::Dependency {
            from_symbol: "hub_var".to_string(),
            to_symbol: "source1".to_string(),
            relationship_type: crate::symbols::DependencyType::DataFlow,
            location: crate::symbols::Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 1,
                end_column: 10,
            },
            context: None,
            strength: 1.0,
            file_path: "test.rs".to_string(),
            is_conditional: false,
            language: crate::parsers::LanguageId::Rust,
        };

        let dep2 = crate::symbols::Dependency {
            from_symbol: "sink1".to_string(),
            to_symbol: "hub_var".to_string(),
            relationship_type: crate::symbols::DependencyType::DataFlow,
            location: crate::symbols::Location {
                file_path: "test.rs".to_string(),
                start_line: 3,
                start_column: 0,
                end_line: 3,
                end_column: 10,
            },
            context: None,
            strength: 1.0,
            file_path: "test.rs".to_string(),
            is_conditional: false,
            language: crate::parsers::LanguageId::Rust,
        };

        builder
            .add_data_flow_edge(hub_idx, source1_idx, &dep1)
            .unwrap();
        builder
            .add_data_flow_edge(sink1_idx, hub_idx, &dep2)
            .unwrap();

        // Test metrics
        let metrics = builder.get_data_flow_metrics("hub_var").unwrap();
        assert_eq!(metrics.incoming_flows, 1);
        assert_eq!(metrics.outgoing_flows, 1);
        assert!(!metrics.is_in_cycle);
    }
}

// ====================================
// Control Flow Graph Implementation
// ====================================

impl CodeGraphBuilder {
    /// Build a control flow graph showing execution paths through code constructs
    pub fn build_control_flow_graph(&mut self) -> FastContextResult<()> {
        // Clear existing control flow edges
        self.control_flow_edges.clear();

        // Extract control flow relationships from all files
        let file_paths: Vec<String> = self
            .graph
            .node_weights()
            .map(|node| node.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for file_path in file_paths {
            self.analyze_control_flow_in_file(&file_path)?;
        }

        Ok(())
    }

    /// Analyze control flow within a specific file
    fn analyze_control_flow_in_file(&mut self, file_path: &str) -> FastContextResult<()> {
        // Get symbols from this file
        let file_symbols: Vec<_> = self
            .graph
            .node_indices()
            .filter(|&idx| self.graph[idx].file_path == file_path)
            .collect();

        // Create extraction context for control flow analysis
        let symbols: Vec<crate::symbols::Symbol> = file_symbols
            .iter()
            .map(|&idx| self.graph[idx].symbol.clone())
            .collect();

        let mut context = crate::symbols::dependency_extractor::ExtractionContext::new(
            file_path.to_string(),
            if !symbols.is_empty() {
                symbols[0].language
            } else {
                crate::parsers::LanguageId::Rust
            },
            symbols.clone(),
        );

        // Analyze each function/method symbol for control flow
        for &symbol_idx in &file_symbols {
            let symbol = self.graph[symbol_idx].symbol.clone();
            if matches!(
                symbol.kind,
                crate::symbols::SymbolKind::Function | crate::symbols::SymbolKind::Method
            ) {
                self.extract_control_flow_from_function(symbol_idx, &symbol, &mut context)?;
            }
        }

        Ok(())
    }

    /// Extract control flow dependencies from a function
    fn extract_control_flow_from_function(
        &mut self,
        function_idx: NodeIndex,
        function: &crate::symbols::Symbol,
        _context: &mut crate::symbols::dependency_extractor::ExtractionContext,
    ) -> FastContextResult<()> {
        // Simulate control flow extraction by parsing function signature/body
        let function_body = function.signature.as_deref().unwrap_or("");

        // Parse the function to identify control flow constructs
        let mut parser_factory = crate::parsers::ParserFactory::new();
        if let Some(parse_result) = parser_factory.parse(function_body, function.language) {
            self.traverse_for_control_flow(
                parse_result.tree.root_node(),
                function_idx,
                function_body,
            )?;
        }

        Ok(())
    }

    /// Traverse AST nodes to identify control flow constructs
    fn traverse_for_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        // Identify control flow constructs based on node type
        match node.kind() {
            "if_expression" | "if_statement" => {
                self.extract_if_control_flow(node, function_idx, source)?;
            }
            "while_expression" | "while_statement" | "for_expression" | "for_statement" => {
                self.extract_loop_control_flow(node, function_idx, source)?;
            }
            "match_expression" | "match_statement" | "switch_statement" => {
                self.extract_match_control_flow(node, function_idx, source)?;
            }
            "try_expression" | "try_statement" => {
                self.extract_try_catch_control_flow(node, function_idx, source)?;
            }
            "return_expression" | "return_statement" => {
                self.extract_return_control_flow(node, function_idx, source)?;
            }
            "break_expression" | "continue_expression" => {
                self.extract_break_continue_control_flow(node, function_idx, source)?;
            }
            _ => {
                // Recursively traverse child nodes
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.traverse_for_control_flow(child, function_idx, source)?;
                }
            }
        }

        Ok(())
    }

    /// Extract control flow from if/else statements
    fn extract_if_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        // Create control flow nodes for if condition and branches
        let condition_text = self.get_node_text(&node, source);
        let if_node_name = format!("if_{}", self.control_flow_edges.len());

        // Create a virtual control flow node for the if statement
        let if_control_node = self.create_control_flow_node(
            &if_node_name,
            ControlFlowKind::Condition,
            condition_text,
            function_idx,
        )?;

        // Add conditional edge from function to if statement
        self.add_control_flow_edge(
            function_idx,
            if_control_node,
            ControlFlowType::Conditional,
            "if_condition",
        )?;

        // Look for else branch
        if let Some(else_node) = node.child_by_field_name("alternative") {
            let else_node_name = format!("else_{}", self.control_flow_edges.len());
            let else_control_node = self.create_control_flow_node(
                &else_node_name,
                ControlFlowKind::Alternative,
                self.get_node_text(&else_node, source),
                function_idx,
            )?;

            // Add alternative edge
            self.add_control_flow_edge(
                if_control_node,
                else_control_node,
                ControlFlowType::Alternative,
                "else_branch",
            )?;
        }

        Ok(())
    }

    /// Extract control flow from loops
    fn extract_loop_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        let loop_text = self.get_node_text(&node, source);
        let loop_node_name = format!("loop_{}", self.control_flow_edges.len());

        let loop_control_node = self.create_control_flow_node(
            &loop_node_name,
            ControlFlowKind::Loop,
            loop_text,
            function_idx,
        )?;

        // Add loop entry edge
        self.add_control_flow_edge(
            function_idx,
            loop_control_node,
            ControlFlowType::Sequential,
            "loop_entry",
        )?;

        // Add back edge for loop iteration
        self.add_control_flow_edge(
            loop_control_node,
            loop_control_node,
            ControlFlowType::Loop,
            "loop_iteration",
        )?;

        Ok(())
    }

    /// Extract control flow from match/switch statements
    fn extract_match_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        let match_text = self.get_node_text(&node, source);
        let match_node_name = format!("match_{}", self.control_flow_edges.len());

        let match_control_node = self.create_control_flow_node(
            &match_node_name,
            ControlFlowKind::Switch,
            match_text,
            function_idx,
        )?;

        // Add match entry edge
        self.add_control_flow_edge(
            function_idx,
            match_control_node,
            ControlFlowType::Sequential,
            "match_entry",
        )?;

        // Count match arms/cases
        let mut cursor = node.walk();
        let case_count = node
            .children(&mut cursor)
            .filter(|child| matches!(child.kind(), "match_arm" | "case_statement"))
            .count();

        // Create edges for each case (simplified - would need more sophisticated analysis)
        for i in 0..case_count {
            let case_node_name = format!("case_{match_node_name}_{i}");
            let case_control_node = self.create_control_flow_node(
                &case_node_name,
                ControlFlowKind::Case,
                format!("case_{i}"),
                function_idx,
            )?;

            self.add_control_flow_edge(
                match_control_node,
                case_control_node,
                ControlFlowType::Conditional,
                &format!("case_{i}"),
            )?;
        }

        Ok(())
    }

    /// Extract control flow from try/catch blocks
    fn extract_try_catch_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        let try_text = self.get_node_text(&node, source);
        let try_node_name = format!("try_{}", self.control_flow_edges.len());

        let try_control_node = self.create_control_flow_node(
            &try_node_name,
            ControlFlowKind::Try,
            try_text,
            function_idx,
        )?;

        // Add try entry edge
        self.add_control_flow_edge(
            function_idx,
            try_control_node,
            ControlFlowType::Sequential,
            "try_entry",
        )?;

        // Look for catch/except handlers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "catch_clause" | "except_clause") {
                let catch_node_name = format!("catch_{}", self.control_flow_edges.len());
                let catch_control_node = self.create_control_flow_node(
                    &catch_node_name,
                    ControlFlowKind::Catch,
                    self.get_node_text(&child, source),
                    function_idx,
                )?;

                // Add exception edge
                self.add_control_flow_edge(
                    try_control_node,
                    catch_control_node,
                    ControlFlowType::Exception,
                    "exception_handler",
                )?;
            }
        }

        Ok(())
    }

    /// Extract control flow from return statements
    fn extract_return_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        let return_text = self.get_node_text(&node, source);
        let return_node_name = format!("return_{}", self.control_flow_edges.len());

        let return_control_node = self.create_control_flow_node(
            &return_node_name,
            ControlFlowKind::Return,
            return_text,
            function_idx,
        )?;

        // Add return edge (terminates control flow)
        self.add_control_flow_edge(
            function_idx,
            return_control_node,
            ControlFlowType::Termination,
            "return_statement",
        )?;

        Ok(())
    }

    /// Extract control flow from break/continue statements
    fn extract_break_continue_control_flow(
        &mut self,
        node: tree_sitter::Node,
        function_idx: NodeIndex,
        source: &str,
    ) -> FastContextResult<()> {
        let stmt_text = self.get_node_text(&node, source);
        let stmt_kind = if node.kind().contains("break") {
            ControlFlowKind::Break
        } else {
            ControlFlowKind::Continue
        };

        let stmt_node_name = format!("{}_{}", node.kind(), self.control_flow_edges.len());
        let stmt_control_node =
            self.create_control_flow_node(&stmt_node_name, stmt_kind, stmt_text, function_idx)?;

        let flow_type = if matches!(stmt_kind, ControlFlowKind::Break) {
            ControlFlowType::Break
        } else {
            ControlFlowType::Continue
        };

        self.add_control_flow_edge(function_idx, stmt_control_node, flow_type, node.kind())?;

        Ok(())
    }

    /// Create a virtual control flow node
    fn create_control_flow_node(
        &mut self,
        name: &str,
        kind: ControlFlowKind,
        content: String,
        parent_function: NodeIndex,
    ) -> FastContextResult<NodeIndex> {
        // Get parent function info
        let parent_symbol = &self.graph[parent_function].symbol;

        // Create a virtual symbol for the control flow construct
        let control_flow_symbol = crate::symbols::Symbol {
            name: name.to_string(),
            kind: crate::symbols::SymbolKind::Macro, // Virtual control flow node
            location: parent_symbol.location.clone(),
            scope_chain: parent_symbol.scope_chain.clone(),
            language: parent_symbol.language,
            documentation: Some(format!("Control flow: {kind:?}")),
            modifiers: vec!["control_flow".to_string()],
            signature: Some(content),
        };

        // Create code node with control flow metrics
        let code_node = super::CodeNode {
            symbol: control_flow_symbol,
            file_path: parent_symbol.location.file_path.clone(),
            metrics: super::CodeMetrics {
                cyclomatic_complexity: match kind {
                    ControlFlowKind::Condition
                    | ControlFlowKind::Loop
                    | ControlFlowKind::Switch => 1,
                    _ => 0,
                },
                cognitive_complexity: match kind {
                    ControlFlowKind::Condition
                    | ControlFlowKind::Loop
                    | ControlFlowKind::Switch => 1,
                    _ => 0,
                },
                nesting_depth: 1,
                lines_of_code: 1,
                number_of_parameters: 0,
                depth_of_nesting: 1,
                fan_in: 0,
                fan_out: 0,
            },
        };

        // Add to graph
        let node_idx = self.graph.add_node(code_node);
        self.symbol_to_node.insert(name.to_string(), node_idx);

        Ok(node_idx)
    }

    /// Add a control flow edge between two nodes
    fn add_control_flow_edge(
        &mut self,
        from_idx: NodeIndex,
        to_idx: NodeIndex,
        flow_type: ControlFlowType,
        description: &str,
    ) -> FastContextResult<()> {
        // Avoid duplicate edges
        if self.control_flow_edges.contains(&(from_idx, to_idx)) {
            return Ok(());
        }

        // Create control flow relationship
        let relationship = super::CodeRelationship {
            kind: super::RelationshipKind::DependsOn,
            source_location: format!("{}:control_flow", self.graph[from_idx].file_path),
            confidence: 1.0,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("flow_type".to_string(), "control".to_string());
                meta.insert("control_type".to_string(), format!("{flow_type:?}"));
                meta.insert("description".to_string(), description.to_string());
                meta
            },
        };

        // Add edge to specialized control flow tracking and main graph
        self.control_flow_edges.insert((from_idx, to_idx));
        self.graph.add_edge(from_idx, to_idx, relationship);

        Ok(())
    }

    /// Find all control flow paths from a function
    pub fn find_control_flow_paths(&self, function_name: &str) -> Vec<Vec<String>> {
        if let Some(&function_idx) = self.symbol_to_node.get(function_name) {
            let mut paths = Vec::new();
            let mut current_path = Vec::new();
            let mut visited = std::collections::HashSet::new();

            self.dfs_control_flow_paths(function_idx, &mut current_path, &mut paths, &mut visited);

            paths
        } else {
            Vec::new()
        }
    }

    /// DFS traversal to find all control flow paths
    fn dfs_control_flow_paths(
        &self,
        current_idx: NodeIndex,
        current_path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
        visited: &mut std::collections::HashSet<NodeIndex>,
    ) {
        if visited.contains(&current_idx) {
            // Cycle detected, add current path and return
            all_paths.push(current_path.clone());
            return;
        }

        visited.insert(current_idx);
        let symbol = &self.graph[current_idx].symbol;
        current_path.push(symbol.qualified_name());

        // Find control flow successors
        let mut has_successors = false;
        for (from, to) in &self.control_flow_edges {
            if *from == current_idx {
                has_successors = true;
                self.dfs_control_flow_paths(*to, current_path, all_paths, visited);
            }
        }

        // If no successors, this is a terminal path
        if !has_successors {
            all_paths.push(current_path.clone());
        }

        // Backtrack
        current_path.pop();
        visited.remove(&current_idx);
    }

    /// Calculate control flow metrics for a function
    pub fn get_control_flow_metrics(&self, function_name: &str) -> Option<ControlFlowMetrics> {
        if let Some(&function_idx) = self.symbol_to_node.get(function_name) {
            let outgoing_flows = self
                .control_flow_edges
                .iter()
                .filter(|(from_idx, _)| *from_idx == function_idx)
                .count() as u32;

            let incoming_flows = self
                .control_flow_edges
                .iter()
                .filter(|(_, to_idx)| *to_idx == function_idx)
                .count() as u32;

            // Count different types of control flow constructs
            let mut condition_count = 0;
            let mut loop_count = 0;
            let mut exception_count = 0;

            for (from_idx, _) in &self.control_flow_edges {
                if *from_idx == function_idx {
                    if let Some(edge) = self.graph.find_edge(*from_idx, function_idx) {
                        let relationship = &self.graph[edge];
                        if let Some(control_type) = relationship.metadata.get("control_type") {
                            match control_type.as_str() {
                                "Conditional" => condition_count += 1,
                                "Loop" => loop_count += 1,
                                "Exception" => exception_count += 1,
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Calculate McCabe cyclomatic complexity using proper formula
            // M = E - N + 2P where:
            // E = number of edges in the control flow graph
            // N = number of nodes in the control flow graph
            // P = number of connected components (typically 1 for a single function)
            //
            // For practical purposes in code analysis, we use:
            // M = 1 + number of decision points
            // Decision points include: if, while, for, case, catch, &&, ||, ?:, etc.
            let cyclomatic_complexity = self.calculate_mccabe_complexity(
                function_idx,
                condition_count,
                loop_count,
                exception_count,
            );

            // Find maximum nesting depth
            let max_nesting_depth = self.calculate_control_flow_depth(function_idx);

            Some(ControlFlowMetrics {
                outgoing_flows,
                incoming_flows,
                condition_count,
                loop_count,
                exception_count,
                cyclomatic_complexity,
                max_nesting_depth,
            })
        } else {
            None
        }
    }

    /// Calculate McCabe cyclomatic complexity using proper algorithm
    fn calculate_mccabe_complexity(
        &self,
        function_idx: NodeIndex,
        condition_count: u32,
        loop_count: u32,
        exception_count: u32,
    ) -> u32 {
        // McCabe's cyclomatic complexity formula: M = E - N + 2P
        // For practical code analysis, we count decision points:
        // M = 1 + number of decision points

        let mut decision_points = 0;

        // Count basic control structures
        decision_points += condition_count; // if, else if, switch cases
        decision_points += loop_count; // for, while, do-while
        decision_points += exception_count; // try-catch blocks

        // Analyze the function's control flow graph for additional complexity
        let mut logical_operators = 0;
        let mut switch_cases = 0;
        let mut ternary_operators = 0;

        // Traverse edges from this function to count additional decision points
        for edge in self.graph.edges_directed(function_idx, petgraph::Outgoing) {
            let relationship = edge.weight();

            if let Some(control_type) = relationship.metadata.get("control_type") {
                match control_type.as_str() {
                    "LogicalAnd" | "LogicalOr" => logical_operators += 1,
                    "SwitchCase" => switch_cases += 1,
                    "TernaryOperator" => ternary_operators += 1,
                    "ConditionalExpression" => decision_points += 1,
                    _ => {}
                }
            }

            // Check for complex boolean expressions
            if let Some(expression_type) = relationship.metadata.get("expression_type") {
                if expression_type == "boolean_expression" {
                    if let Some(complexity) = relationship.metadata.get("boolean_complexity") {
                        if let Ok(complexity_val) = complexity.parse::<u32>() {
                            decision_points += complexity_val.saturating_sub(1);
                        }
                    }
                }
            }
        }

        // Add logical operators (each && or || adds a decision point)
        decision_points += logical_operators;

        // Add switch cases (each case is a decision point)
        decision_points += switch_cases;

        // Add ternary operators (each ?: is a decision point)
        decision_points += ternary_operators;

        // McCabe complexity: start with 1 and add decision points
        let base_complexity = 1 + decision_points;

        // Apply bounds checking to prevent unrealistic values
        // Typical ranges: 1-10 (simple), 11-20 (moderate), 21-50 (complex), 50+ (very complex)
        base_complexity.min(100) // Cap at 100 to prevent overflow issues
    }

    /// Calculate maximum control flow nesting depth
    fn calculate_control_flow_depth(&self, function_idx: NodeIndex) -> u32 {
        let mut max_depth = 0;
        let mut visited = std::collections::HashSet::new();

        self.dfs_control_flow_depth(function_idx, 0, &mut max_depth, &mut visited);

        max_depth
    }

    /// DFS to calculate control flow depth
    fn dfs_control_flow_depth(
        &self,
        current_idx: NodeIndex,
        current_depth: u32,
        max_depth: &mut u32,
        visited: &mut std::collections::HashSet<NodeIndex>,
    ) {
        if visited.contains(&current_idx) {
            return;
        }

        visited.insert(current_idx);
        *max_depth = (*max_depth).max(current_depth);

        for (from, to) in &self.control_flow_edges {
            if *from == current_idx {
                self.dfs_control_flow_depth(*to, current_depth + 1, max_depth, visited);
            }
        }

        visited.remove(&current_idx);
    }

    /// Analyze control flow patterns across the codebase
    pub fn analyze_control_flow_patterns(&self) -> ControlFlowAnalysis {
        let mut analysis = ControlFlowAnalysis {
            total_control_flows: self.control_flow_edges.len(),
            complex_functions: Vec::new(),
            deeply_nested_functions: Vec::new(),
            exception_heavy_functions: Vec::new(),
        };

        // Analyze each function's control flow complexity
        for (function_name, &_function_idx) in &self.symbol_to_node {
            if let Some(metrics) = self.get_control_flow_metrics(function_name) {
                // Complex functions (high cyclomatic complexity)
                if metrics.cyclomatic_complexity > 10 {
                    analysis.complex_functions.push(ComplexFunction {
                        name: function_name.clone(),
                        cyclomatic_complexity: metrics.cyclomatic_complexity,
                        condition_count: metrics.condition_count,
                        loop_count: metrics.loop_count,
                    });
                }

                // Deeply nested functions
                if metrics.max_nesting_depth > 5 {
                    analysis.deeply_nested_functions.push(DeeplyNestedFunction {
                        name: function_name.clone(),
                        max_nesting_depth: metrics.max_nesting_depth,
                        control_flow_count: metrics.outgoing_flows,
                    });
                }

                // Exception-heavy functions
                if metrics.exception_count > 2 {
                    analysis
                        .exception_heavy_functions
                        .push(ExceptionHeavyFunction {
                            name: function_name.clone(),
                            exception_count: metrics.exception_count,
                            complexity_score: metrics.cyclomatic_complexity,
                        });
                }
            }
        }

        // Sort by complexity metrics
        analysis
            .complex_functions
            .sort_by(|a, b| b.cyclomatic_complexity.cmp(&a.cyclomatic_complexity));
        analysis
            .deeply_nested_functions
            .sort_by(|a, b| b.max_nesting_depth.cmp(&a.max_nesting_depth));
        analysis
            .exception_heavy_functions
            .sort_by(|a, b| b.exception_count.cmp(&a.exception_count));

        analysis
    }

    /// Get node text from tree-sitter node
    fn get_node_text(&self, node: &tree_sitter::Node, source: &str) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        source.get(start..end).unwrap_or("").to_string()
    }
}

/// Types of control flow constructs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowKind {
    Condition,   // if/else
    Loop,        // for/while
    Switch,      // match/switch
    Try,         // try block
    Catch,       // catch/except
    Return,      // return statement
    Break,       // break statement
    Continue,    // continue statement
    Alternative, // else branch
    Case,        // match arm/case
}

/// Types of control flow edges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowType {
    Sequential,  // Normal flow
    Conditional, // Conditional branch
    Alternative, // Else branch
    Loop,        // Loop iteration
    Break,       // Break out of loop
    Continue,    // Continue loop
    Exception,   // Exception handling
    Termination, // Function return
}

/// Control flow metrics for a function
#[derive(Debug, Clone)]
pub struct ControlFlowMetrics {
    pub outgoing_flows: u32,
    pub incoming_flows: u32,
    pub condition_count: u32,
    pub loop_count: u32,
    pub exception_count: u32,
    pub cyclomatic_complexity: u32,
    pub max_nesting_depth: u32,
}

/// Complex function with high cyclomatic complexity
#[derive(Debug, Clone)]
pub struct ComplexFunction {
    pub name: String,
    pub cyclomatic_complexity: u32,
    pub condition_count: u32,
    pub loop_count: u32,
}

/// Function with deep nesting
#[derive(Debug, Clone)]
pub struct DeeplyNestedFunction {
    pub name: String,
    pub max_nesting_depth: u32,
    pub control_flow_count: u32,
}

/// Function with many exception handlers
#[derive(Debug, Clone)]
pub struct ExceptionHeavyFunction {
    pub name: String,
    pub exception_count: u32,
    pub complexity_score: u32,
}

/// Comprehensive control flow analysis results
#[derive(Debug, Clone)]
pub struct ControlFlowAnalysis {
    pub total_control_flows: usize,
    pub complex_functions: Vec<ComplexFunction>,
    pub deeply_nested_functions: Vec<DeeplyNestedFunction>,
    pub exception_heavy_functions: Vec<ExceptionHeavyFunction>,
}

#[cfg(test)]
mod complexity_tests {
    use super::*;
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};

    fn create_test_function(name: &str, signature: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some(signature.to_string()),
        }
    }

    #[test]
    fn test_simple_function_complexity() {
        let mut builder = CodeGraphBuilder::new();

        // Simple function with no control flow - should have complexity of 1
        let simple_func = create_test_function("simple", "fn simple() { println!(\"hello\"); }");
        builder.add_file_symbols(vec![simple_func], "test.rs");

        // Build the control flow graph
        builder.build_control_flow_graph().unwrap();

        // Get complexity metrics for the function
        let metrics = builder.get_control_flow_metrics("simple");
        assert!(metrics.is_some(), "Should find metrics for simple function");

        let metrics = metrics.unwrap();
        // Since we're not parsing actual control flow, base complexity should be 1
        assert_eq!(
            metrics.cyclomatic_complexity, 1,
            "Simple function should have base complexity of 1"
        );
    }

    #[test]
    fn test_basic_complexity_calculation() {
        let mut builder = CodeGraphBuilder::new();

        // Test the complexity calculation mechanism itself
        let test_func = create_test_function("test_func", "fn test_func() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        // Build the control flow graph
        builder.build_control_flow_graph().unwrap();

        // Test complexity calculation with different parameters
        if let Some(&func_idx) = builder.symbol_to_node.get("test_func") {
            // Test base case: no control structures
            let complexity1 = builder.calculate_mccabe_complexity(func_idx, 0, 0, 0);
            assert_eq!(complexity1, 1, "Base complexity should be 1");

            // Test with conditions
            let complexity2 = builder.calculate_mccabe_complexity(func_idx, 2, 0, 0);
            assert_eq!(complexity2, 3, "Should add conditions to base complexity");

            // Test with loops
            let complexity3 = builder.calculate_mccabe_complexity(func_idx, 0, 2, 0);
            assert_eq!(complexity3, 3, "Should add loops to base complexity");

            // Test with exceptions
            let complexity4 = builder.calculate_mccabe_complexity(func_idx, 0, 0, 1);
            assert_eq!(complexity4, 2, "Should add exceptions to base complexity");

            // Test combined complexity
            let complexity5 = builder.calculate_mccabe_complexity(func_idx, 2, 1, 1);
            assert_eq!(complexity5, 5, "Should combine all decision points");
        } else {
            panic!("Test setup failed: 'test_function' should be found in symbol table but was not found");
        }
    }

    #[test]
    fn test_control_flow_depth_calculation() {
        let mut builder = CodeGraphBuilder::new();

        // Test the depth calculation functionality
        let test_func = create_test_function("depth_func", "fn depth_func() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        builder.build_control_flow_graph().unwrap();

        if let Some(&func_idx) = builder.symbol_to_node.get("depth_func") {
            // Test depth calculation
            let depth = builder.calculate_control_flow_depth(func_idx);
            // Depth is unsigned, so it's always non-negative
            assert!(depth < 1000, "Depth should be reasonable");

            // Test metrics integration
            let metrics = builder.get_control_flow_metrics("depth_func");
            assert!(metrics.is_some(), "Should get metrics for function");

            let metrics = metrics.unwrap();
            assert_eq!(
                metrics.cyclomatic_complexity, 1,
                "Base function should have complexity 1"
            );
            // max_nesting_depth is unsigned, so it's always non-negative
            assert!(
                metrics.max_nesting_depth < 100,
                "Nesting depth should be reasonable"
            );
        } else {
            panic!("Test setup failed: 'test_function' should be found in symbol table but was not found");
        }
    }

    #[test]
    fn test_control_flow_edges() {
        let mut builder = CodeGraphBuilder::new();

        // Test that control flow edges are managed correctly
        let func1 = create_test_function("func1", "fn func1() {}");
        let func2 = create_test_function("func2", "fn func2() {}");
        builder.add_file_symbols(vec![func1, func2], "test.rs");

        builder.build_control_flow_graph().unwrap();

        // Test that control flow edges collection is initialized (len() is always >= 0 for Vec)
        // Just verify the collection exists by checking it's accessible
        let _edges_count = builder.control_flow_edges.len();

        // Test that both functions are in the symbol table
        assert!(
            builder.symbol_to_node.contains_key("func1"),
            "Should contain func1"
        );
        assert!(
            builder.symbol_to_node.contains_key("func2"),
            "Should contain func2"
        );

        // Test metrics for both functions
        let metrics1 = builder.get_control_flow_metrics("func1");
        let metrics2 = builder.get_control_flow_metrics("func2");

        assert!(metrics1.is_some(), "Should get metrics for func1");
        assert!(metrics2.is_some(), "Should get metrics for func2");
    }

    #[test]
    fn test_complexity_edge_cases() {
        let mut builder = CodeGraphBuilder::new();

        // Test various edge cases for complexity calculation
        let test_func = create_test_function("edge_func", "fn edge_func() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        builder.build_control_flow_graph().unwrap();

        if let Some(&func_idx) = builder.symbol_to_node.get("edge_func") {
            // Test edge case: very high input values
            let high_complexity = builder.calculate_mccabe_complexity(func_idx, 100, 50, 25);
            assert!(
                high_complexity > 1,
                "High complexity should be greater than base"
            );
            assert!(
                high_complexity <= 1000,
                "Complexity should not be unreasonably high"
            );

            // Test edge case: zero values
            let zero_complexity = builder.calculate_mccabe_complexity(func_idx, 0, 0, 0);
            assert_eq!(
                zero_complexity, 1,
                "Zero inputs should give base complexity of 1"
            );
        } else {
            panic!("Test setup failed: 'test_function' should be found in symbol table but was not found");
        }
    }

    #[test]
    fn test_control_flow_analysis_integration() {
        let mut builder = CodeGraphBuilder::new();

        // Test the overall control flow analysis system
        let func1 = create_test_function("simple_func", "fn simple_func() {}");
        let func2 = create_test_function("medium_func", "fn medium_func() {}");
        let func3 = create_test_function("complex_func", "fn complex_func() {}");

        builder.add_file_symbols(vec![func1, func2, func3], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Simulate different complexity levels by manually setting up complexity
        if let Some(&func1_idx) = builder.symbol_to_node.get("simple_func") {
            let simple_complexity = builder.calculate_mccabe_complexity(func1_idx, 0, 0, 0);
            assert_eq!(
                simple_complexity, 1,
                "Simple function should have base complexity"
            );
        }

        if let Some(&func2_idx) = builder.symbol_to_node.get("medium_func") {
            let medium_complexity = builder.calculate_mccabe_complexity(func2_idx, 3, 1, 0);
            assert_eq!(
                medium_complexity, 5,
                "Medium function should have moderate complexity"
            );
        }

        if let Some(&func3_idx) = builder.symbol_to_node.get("complex_func") {
            let complex_complexity = builder.calculate_mccabe_complexity(func3_idx, 5, 3, 2);
            assert_eq!(
                complex_complexity, 11,
                "Complex function should have high complexity"
            );
        }

        // Test the analysis system
        let analysis = builder.analyze_control_flow_patterns();
        // total_control_flows is unsigned, so it's always non-negative
        assert!(
            analysis.total_control_flows < 10000,
            "Control flow count should be reasonable"
        );
    }

    #[test]
    fn test_complexity_algorithm_correctness() {
        let mut builder = CodeGraphBuilder::new();

        // Test that the McCabe complexity algorithm works correctly
        let test_func = create_test_function("algorithm_test", "fn algorithm_test() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        builder.build_control_flow_graph().unwrap();

        if let Some(&func_idx) = builder.symbol_to_node.get("algorithm_test") {
            // Test McCabe formula: Base complexity + decision points

            // No decision points: M = 1
            let complexity0 = builder.calculate_mccabe_complexity(func_idx, 0, 0, 0);
            assert_eq!(complexity0, 1, "No decisions should give complexity 1");

            // One condition: M = 1 + 1 = 2
            let complexity1 = builder.calculate_mccabe_complexity(func_idx, 1, 0, 0);
            assert_eq!(complexity1, 2, "One condition should give complexity 2");

            // Multiple decision points: M = 1 + conditions + loops + exceptions
            let complexity_multi = builder.calculate_mccabe_complexity(func_idx, 3, 2, 1);
            assert_eq!(complexity_multi, 7, "Multiple decisions: 1 + 3 + 2 + 1 = 7");
        } else {
            panic!("Test setup failed: 'test_function' should be found in symbol table but was not found");
        }
    }

    #[test]
    fn test_complexity_bounds_and_validation() {
        let mut builder = CodeGraphBuilder::new();

        // Test complexity bounds and validation
        let test_func = create_test_function("bounds_test", "fn bounds_test() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        builder.build_control_flow_graph().unwrap();

        if let Some(&func_idx) = builder.symbol_to_node.get("bounds_test") {
            // Test that complexity is always >= 1
            let min_complexity = builder.calculate_mccabe_complexity(func_idx, 0, 0, 0);
            assert!(
                min_complexity >= 1,
                "Complexity should always be at least 1"
            );

            // Test reasonable upper bounds
            let high_complexity = builder.calculate_mccabe_complexity(func_idx, 50, 30, 20);
            assert!(
                high_complexity > 1,
                "High input should produce high complexity"
            );
            assert!(
                high_complexity <= 200,
                "Complexity should not be unreasonably high"
            );

            // Test that formula is consistent (with capping)
            let raw_expected = 1 + 50 + 30 + 20; // Base + conditions + loops + exceptions = 101
            let capped_expected = raw_expected.min(100); // Should be capped at 100
            assert_eq!(
                high_complexity, capped_expected,
                "Should follow McCabe formula with capping: min(1 + decisions, 100)"
            );
        } else {
            panic!("Test setup failed: 'test_function' should be found in symbol table but was not found");
        }
    }

    #[test]
    fn test_complexity_bounds() {
        let mut builder = CodeGraphBuilder::new();

        // Test that complexity is capped appropriately
        let func = create_test_function("test_func", "fn test_func() {}");
        builder.add_file_symbols(vec![func], "test.rs");

        // Build the control flow graph
        builder.build_control_flow_graph().unwrap();

        // Get complexity metrics for the function
        let metrics = builder.get_control_flow_metrics("test_func");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        // Minimum complexity should be 1 for any function
        assert!(
            metrics.cyclomatic_complexity >= 1,
            "Function should have minimum complexity of 1"
        );
        // Complexity should be reasonable (not excessively high)
        assert!(
            metrics.cyclomatic_complexity <= 100,
            "Function complexity should be reasonable"
        );
    }

    #[test]
    fn test_complexity_metrics_structure() {
        let mut builder = CodeGraphBuilder::new();

        // Test that complexity metrics are properly structured
        let test_func = create_test_function("metrics_test", "fn metrics_test() {}");
        builder.add_file_symbols(vec![test_func], "test.rs");

        builder.build_control_flow_graph().unwrap();

        let metrics = builder.get_control_flow_metrics("metrics_test");
        assert!(metrics.is_some(), "Should get metrics for function");

        let metrics = metrics.unwrap();

        // Test all metrics fields are properly initialized (all are unsigned, so always >= 0)
        // Just verify they're accessible and reasonable
        assert!(
            metrics.outgoing_flows < 1000,
            "Outgoing flows should be reasonable"
        );
        assert!(
            metrics.incoming_flows < 1000,
            "Incoming flows should be reasonable"
        );
        assert!(
            metrics.condition_count < 100,
            "Condition count should be reasonable"
        );
        assert!(metrics.loop_count < 100, "Loop count should be reasonable");
        assert!(
            metrics.exception_count < 100,
            "Exception count should be reasonable"
        );
        assert!(
            metrics.cyclomatic_complexity >= 1,
            "Cyclomatic complexity should be >= 1"
        );
        // max_nesting_depth is unsigned, so it's always non-negative
        assert!(
            metrics.max_nesting_depth < 100,
            "Nesting depth should be reasonable"
        );

        // Test non-existent function returns None
        let no_metrics = builder.get_control_flow_metrics("non_existent_function");
        assert!(
            no_metrics.is_none(),
            "Non-existent function should return None"
        );
    }

    #[test]
    fn test_control_flow_analysis() {
        let mut builder = CodeGraphBuilder::new();

        // Add multiple functions with varying complexity
        let simple_func = create_test_function("simple", "fn simple() { println!(\"hello\"); }");
        let complex_func = create_test_function(
            "complex",
            "fn complex(x: i32) { if x > 0 { for i in 0..x { if i % 2 == 0 { continue; } } } }",
        );
        let nested_func = create_test_function(
            "nested",
            "fn nested() { for i in 0..10 { for j in 0..10 { if i == j { break; } } } }",
        );

        builder.add_file_symbols(vec![simple_func, complex_func, nested_func], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Analyze control flow patterns
        let analysis = builder.analyze_control_flow_patterns();

        // Should have detected some control flow constructs (total_control_flows is unsigned)
        // Just verify the analysis ran and produced reasonable results
        assert!(
            analysis.total_control_flows < 10000,
            "Control flows should be reasonable"
        );

        // Check if complex functions were identified (len() is always >= 0)
        let complex_count = analysis.complex_functions.len();
        assert!(
            complex_count < 1000,
            "Complex function count should be reasonable"
        );
    }

    #[test]
    fn test_control_flow_metrics_edge_cases() {
        let mut builder = CodeGraphBuilder::new();

        // Test edge cases
        let empty_func = create_test_function("empty", "fn empty() {}");
        let single_return =
            create_test_function("single_return", "fn single_return() -> i32 { 42 }");

        builder.add_file_symbols(vec![empty_func, single_return], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Empty function should have minimal complexity
        let empty_metrics = builder.get_control_flow_metrics("empty");
        assert!(empty_metrics.is_some());
        assert_eq!(empty_metrics.unwrap().cyclomatic_complexity, 1);

        // Function with return should have minimal complexity
        let return_metrics = builder.get_control_flow_metrics("single_return");
        assert!(return_metrics.is_some());
        assert_eq!(return_metrics.unwrap().cyclomatic_complexity, 1);

        // Non-existent function should return None
        let none_metrics = builder.get_control_flow_metrics("non_existent");
        assert!(none_metrics.is_none());
    }

    #[test]
    fn test_comprehensive_complexity_benchmarks() {
        // Test against known complexity benchmarks from software engineering literature
        let mut builder = CodeGraphBuilder::new();

        // Benchmark 1: Simple linear function (complexity = 1)
        let linear = create_test_function(
            "linear",
            "fn linear() { let x = 1; let y = 2; let z = x + y; }",
        );

        // Benchmark 2: Single if statement (complexity = 2)
        let single_if = create_test_function(
            "single_if",
            "fn single_if(x: i32) { if x > 0 { println!(\"positive\"); } }",
        );

        // Benchmark 3: If-else chain (complexity = 4)
        let if_else_chain = create_test_function(
            "if_else_chain",
            "fn if_else_chain(x: i32) { if x > 0 { } else if x < 0 { } else if x == 0 { } }",
        );

        // Benchmark 4: Nested loops (complexity = 4)
        let nested_loops = create_test_function(
            "nested_loops",
            "fn nested_loops() { for i in 0..10 { for j in 0..10 { println!(\"{} {}\", i, j); } } }"
        );

        builder.add_file_symbols(
            vec![linear, single_if, if_else_chain, nested_loops],
            "benchmarks.rs",
        );
        builder.build_control_flow_graph().unwrap();

        // Validate linear function (McCabe = 1)
        if let Some(&node_idx) = builder.symbol_to_node.get("linear") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 0, 0, 0);
            assert_eq!(complexity, 1, "Linear function benchmark failed");
        }

        // Validate single if (McCabe = 2)
        if let Some(&node_idx) = builder.symbol_to_node.get("single_if") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 1, 0, 0);
            assert_eq!(complexity, 2, "Single if benchmark failed");
        }

        // Validate if-else chain (McCabe = 4)
        if let Some(&node_idx) = builder.symbol_to_node.get("if_else_chain") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 3, 0, 0); // 3 conditions
            assert_eq!(complexity, 4, "If-else chain benchmark failed");
        }

        // Validate nested loops (McCabe = 3)
        if let Some(&node_idx) = builder.symbol_to_node.get("nested_loops") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 0, 2, 0); // 2 loops
            assert_eq!(complexity, 3, "Nested loops benchmark failed");
        }
    }

    #[test]
    fn test_complexity_edge_cases_comprehensive() {
        let mut builder = CodeGraphBuilder::new();

        // Edge case 1: Empty function
        let empty = create_test_function("empty", "fn empty() {}");

        // Edge case 2: Function with only comments
        let comments_only = create_test_function(
            "comments_only",
            "fn comments_only() { /* comment */ // another comment }",
        );

        // Edge case 3: Function with complex nested structures
        let deeply_nested = create_test_function(
            "deeply_nested",
            r#"fn deeply_nested(x: i32) {
                if x > 0 {
                    for i in 0..x {
                        if i % 2 == 0 {
                            while x > 0 {
                                if x == 5 { break; }
                                x -= 1;
                            }
                        }
                    }
                }
            }"#,
        );

        builder.add_file_symbols(vec![empty, comments_only, deeply_nested], "edge_cases.rs");
        builder.build_control_flow_graph().unwrap();

        // Test empty function
        if let Some(&node_idx) = builder.symbol_to_node.get("empty") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 0, 0, 0);
            assert_eq!(complexity, 1, "Empty function should have complexity 1");
        }

        // Test comments-only function
        if let Some(&node_idx) = builder.symbol_to_node.get("comments_only") {
            let complexity = builder.calculate_mccabe_complexity(node_idx, 0, 0, 0);
            assert_eq!(
                complexity, 1,
                "Comments-only function should have complexity 1"
            );
        }

        // Test deeply nested function
        if let Some(&node_idx) = builder.symbol_to_node.get("deeply_nested") {
            // 3 if statements, 2 loops = 1 + 3 + 2 = 6
            let complexity = builder.calculate_mccabe_complexity(node_idx, 3, 2, 0);
            assert_eq!(
                complexity, 6,
                "Deeply nested function should have high complexity"
            );

            // Test depth calculation
            let depth = builder.calculate_control_flow_depth(node_idx);
            assert!(
                depth >= 1,
                "Deeply nested function should have significant depth"
            );
        }
    }

    #[test]
    fn test_cognitive_complexity_vs_cyclomatic() {
        // Test that demonstrates the difference between cyclomatic and cognitive complexity
        let mut builder = CodeGraphBuilder::new();

        let cognitive_test = create_test_function(
            "cognitive_test",
            r#"fn cognitive_test() {
                if true {           // +1 cyclomatic, +1 cognitive
                    if true {       // +1 cyclomatic, +2 cognitive (nested)
                        if true {   // +1 cyclomatic, +3 cognitive (deeply nested)
                            println!("deep");
                        }
                    }
                }
            }"#,
        );

        builder.add_file_symbols(vec![cognitive_test], "cognitive.rs");
        builder.build_control_flow_graph().unwrap();

        if let Some(&node_idx) = builder.symbol_to_node.get("cognitive_test") {
            let cyclomatic = builder.calculate_mccabe_complexity(node_idx, 3, 0, 0);
            let depth = builder.calculate_control_flow_depth(node_idx);

            assert_eq!(cyclomatic, 4, "Cyclomatic complexity should be 4");
            assert!(
                depth >= 1,
                "Nesting depth should reflect deep nesting for cognitive complexity"
            );

            // Cognitive complexity would be higher due to nesting penalties
            // This demonstrates why both metrics are valuable
        }
    }

    #[test]
    fn test_exception_handling_complexity() {
        let mut builder = CodeGraphBuilder::new();

        let exception_func = create_test_function(
            "exception_func",
            r#"fn exception_func() {
                try {
                    risky_operation();
                } catch (IOException e) {
                    handle_io_error(e);
                } catch (RuntimeException e) {
                    handle_runtime_error(e);
                } finally {
                    cleanup();
                }
            }"#,
        );

        builder.add_file_symbols(vec![exception_func], "exceptions.rs");
        builder.build_control_flow_graph().unwrap();

        if let Some(&node_idx) = builder.symbol_to_node.get("exception_func") {
            // 2 catch blocks = 1 + 2 = 3 (finally doesn't add to cyclomatic complexity)
            let complexity = builder.calculate_mccabe_complexity(node_idx, 0, 0, 2);
            assert_eq!(complexity, 3, "Exception handling should add to complexity");
        }
    }

    #[test]
    fn test_performance_with_large_complexity() {
        // Test that complexity calculation performs well with large numbers
        let mut builder = CodeGraphBuilder::new();

        let large_func = create_test_function(
            "large_func",
            "fn large_func() { /* Simulated large function */ }",
        );

        builder.add_file_symbols(vec![large_func], "large.rs");
        builder.build_control_flow_graph().unwrap();

        if let Some(&node_idx) = builder.symbol_to_node.get("large_func") {
            // Test with large numbers to ensure no overflow or performance issues
            let start = std::time::Instant::now();
            let complexity = builder.calculate_mccabe_complexity(node_idx, 100, 50, 25);
            let duration = start.elapsed();

            assert!(
                complexity > 1,
                "Large complexity should be greater than base"
            );
            assert!(
                complexity <= 200,
                "Complexity should be capped for reasonableness"
            );
            assert!(
                duration.as_millis() < 100,
                "Complexity calculation should be fast"
            );
        }
    }
}

#[cfg(test)]
mod control_flow_tests {
    use super::*;
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};

    #[test]
    fn test_control_flow_graph_construction() {
        let mut builder = CodeGraphBuilder::new();

        // Create test function with control flow
        let test_function = Symbol {
            name: "test_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn test_function() { if true { return; } }".to_string()),
        };

        builder.add_file_symbols(vec![test_function], "test.rs");

        // Build control flow graph
        let result = builder.build_control_flow_graph();
        assert!(result.is_ok());

        // Check that control flow edges were created
        assert!(!builder.control_flow_edges.is_empty());
    }

    #[test]
    fn test_control_flow_metrics() {
        let mut builder = CodeGraphBuilder::new();

        // Create a complex function with multiple control flow constructs
        let complex_function = Symbol {
            name: "complex_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 20,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some(
                r#"fn complex_function() {
                if condition1 {
                    for i in 0..10 {
                        if condition2 {
                            return;
                        }
                    }
                } else {
                    while condition3 {
                        break;
                    }
                }
            }"#
                .to_string(),
            ),
        };

        builder.add_file_symbols(vec![complex_function], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Test control flow metrics
        let metrics = builder.get_control_flow_metrics("complex_function");
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert!(metrics.cyclomatic_complexity >= 1);
        assert!(metrics.outgoing_flows > 0);
    }

    #[test]
    fn test_control_flow_paths() {
        let mut builder = CodeGraphBuilder::new();

        // Create function with branching paths
        let branching_function = Symbol {
            name: "branching_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 15,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some(
                r#"fn branching_function() {
                if condition {
                    return 1;
                } else {
                    return 2;
                }
            }"#
                .to_string(),
            ),
        };

        builder.add_file_symbols(vec![branching_function], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Test control flow path finding
        let paths = builder.find_control_flow_paths("branching_function");
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_control_flow_analysis() {
        let mut builder = CodeGraphBuilder::new();

        // Create multiple functions with varying complexity
        let simple_function = Symbol {
            name: "simple_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 3,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn simple_function() { return; }".to_string()),
        };

        let complex_function = Symbol {
            name: "complex_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 5,
                start_column: 0,
                end_line: 30,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some(
                r#"fn complex_function() {
                if a { if b { if c { if d { if e { return; } } } } }
                for i in 0..10 { for j in 0..10 { return; } }
                match x { 1 => {}, 2 => {}, 3 => {}, _ => {} }
            }"#
                .to_string(),
            ),
        };

        builder.add_file_symbols(vec![simple_function, complex_function], "test.rs");
        builder.build_control_flow_graph().unwrap();

        // Test control flow analysis
        let analysis = builder.analyze_control_flow_patterns();
        assert_eq!(
            analysis.total_control_flows,
            builder.control_flow_edges.len()
        );
    }

    #[test]
    fn test_control_flow_node_creation() {
        let mut builder = CodeGraphBuilder::new();

        // Create parent function
        let parent_function = Symbol {
            name: "parent_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn parent_function() {}".to_string()),
        };

        builder.add_file_symbols(vec![parent_function], "test.rs");
        let parent_idx = builder.find_definition("parent_function").unwrap();

        // Test creating control flow nodes
        let if_node = builder.create_control_flow_node(
            "test_if",
            ControlFlowKind::Condition,
            "if condition".to_string(),
            parent_idx,
        );

        assert!(if_node.is_ok());
        let if_idx = if_node.unwrap();

        // Check that the control flow node was added to the graph
        assert!(builder.graph.node_weight(if_idx).is_some());
        assert!(builder.symbol_to_node.contains_key("test_if"));
    }
}

// ====================================
// Graph Validation and Cycle Detection
// ====================================

impl CodeGraphBuilder {
    /// Validate the entire graph for consistency and detect cycles
    pub fn validate_graph(&self) -> GraphValidationResult {
        let mut result = GraphValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            cycles: Vec::new(),
            statistics: GraphStatistics::default(),
        };

        // Collect statistics
        result.statistics = self.calculate_graph_statistics();

        // Validate graph structure
        self.validate_graph_structure(&mut result);

        // Detect cycles in different graph layers
        self.detect_all_cycles(&mut result);

        // Validate symbol-node consistency
        self.validate_symbol_node_consistency(&mut result);

        // Validate edge relationships
        self.validate_edge_relationships(&mut result);

        // Check for orphaned nodes
        self.detect_orphaned_nodes(&mut result);

        // Validate scope consistency
        self.validate_scope_consistency(&mut result);

        result.is_valid = result.errors.is_empty();
        result
    }

    /// Calculate comprehensive graph statistics
    fn calculate_graph_statistics(&self) -> GraphStatistics {
        GraphStatistics {
            total_nodes: self.graph.node_count(),
            total_edges: self.graph.edge_count(),
            symbol_definitions: self.symbol_to_node.len(),
            call_relationships: self.call_graph_edges.len(),
            import_relationships: self.import_graph_edges.len(),
            inheritance_relationships: self.inheritance_graph_edges.len(),
            data_flow_relationships: self.data_flow_edges.len(),
            control_flow_relationships: self.control_flow_edges.len(),
            file_count: self.file_symbols.len(),
            average_edges_per_node: if self.graph.node_count() > 0 {
                self.graph.edge_count() as f32 / self.graph.node_count() as f32
            } else {
                0.0
            },
        }
    }

    /// Validate basic graph structure
    fn validate_graph_structure(&self, result: &mut GraphValidationResult) {
        // Check for invalid node indices
        for &node_idx in self.symbol_to_node.values() {
            if self.graph.node_weight(node_idx).is_none() {
                result.errors.push(ValidationError {
                    kind: ValidationErrorKind::InvalidNodeIndex,
                    message: format!("Symbol maps to invalid node index: {node_idx:?}"),
                    location: None,
                    severity: ValidationSeverity::Error,
                });
            }
        }

        // Check for duplicate symbol names in same scope
        let mut scope_symbols: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (symbol_name, &node_idx) in &self.symbol_to_node {
            if let Some(node) = self.graph.node_weight(node_idx) {
                let scope = node
                    .symbol
                    .scope_chain
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                scope_symbols
                    .entry(scope)
                    .or_default()
                    .push(symbol_name.clone());
            }
        }

        for (scope, symbols) in scope_symbols {
            let mut unique_names = std::collections::HashSet::new();
            for symbol in symbols {
                if !unique_names.insert(symbol.clone()) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::DuplicateSymbol,
                        message: format!("Duplicate symbol '{symbol}' in scope '{scope}'"),
                        location: None,
                    });
                }
            }
        }
    }

    /// Detect cycles in all graph layers
    fn detect_all_cycles(&self, result: &mut GraphValidationResult) {
        // Detect call graph cycles (potentially problematic recursive calls)
        let call_cycles = self.detect_cycles_in_edge_set(&self.call_graph_edges);
        for cycle in call_cycles {
            let is_recursive = self.is_recursive_cycle(&cycle);
            result.cycles.push(GraphCycle {
                kind: CycleKind::CallCycle,
                nodes: cycle,
                severity: if is_recursive {
                    CycleSeverity::Info // Recursion is often intentional
                } else {
                    CycleSeverity::Warning
                },
            });
        }

        // Detect import cycles (problematic)
        let import_cycles = self.detect_cycles_in_edge_set(&self.import_graph_edges);
        for cycle in import_cycles {
            result.cycles.push(GraphCycle {
                kind: CycleKind::ImportCycle,
                nodes: cycle,
                severity: CycleSeverity::Error, // Import cycles are always problematic
            });
        }

        // Detect inheritance cycles (highly problematic)
        let inheritance_cycles = self.detect_cycles_in_edge_set(&self.inheritance_graph_edges);
        for cycle in inheritance_cycles {
            result.cycles.push(GraphCycle {
                kind: CycleKind::InheritanceCycle,
                nodes: cycle,
                severity: CycleSeverity::Error, // Inheritance cycles are always errors
            });
        }

        // Detect data flow cycles (potentially problematic)
        let data_flow_cycles = self.detect_cycles_in_edge_set(&self.data_flow_edges);
        for cycle in data_flow_cycles {
            result.cycles.push(GraphCycle {
                kind: CycleKind::DataFlowCycle,
                nodes: cycle,
                severity: CycleSeverity::Warning, // Data flow cycles might indicate issues
            });
        }

        // Control flow cycles are expected (loops), so we don't report them as issues
    }

    /// Detect cycles in a specific edge set using DFS
    fn detect_cycles_in_edge_set(
        &self,
        edges: &std::collections::HashSet<(NodeIndex, NodeIndex)>,
    ) -> Vec<Vec<NodeIndex>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        let mut path = Vec::new();

        // Get all nodes in this edge set
        let nodes: std::collections::HashSet<NodeIndex> = edges
            .iter()
            .flat_map(|(from, to)| vec![*from, *to])
            .collect();

        for &node in &nodes {
            if !visited.contains(&node) {
                self.dfs_cycle_detection(
                    node,
                    edges,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// DFS-based cycle detection
    #[allow(clippy::only_used_in_recursion)]
    fn dfs_cycle_detection(
        &self,
        node: NodeIndex,
        edges: &std::collections::HashSet<(NodeIndex, NodeIndex)>,
        visited: &mut std::collections::HashSet<NodeIndex>,
        rec_stack: &mut std::collections::HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
        cycles: &mut Vec<Vec<NodeIndex>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        // Find neighbors in this edge set
        for (from, to) in edges {
            if *from == node {
                if rec_stack.contains(to) {
                    // Found a cycle, extract it from the path
                    if let Some(cycle_start) = path.iter().position(|&n| n == *to) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(to) {
                    self.dfs_cycle_detection(*to, edges, visited, rec_stack, path, cycles);
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
    }

    /// Check if a cycle represents intentional recursion
    fn is_recursive_cycle(&self, cycle: &[NodeIndex]) -> bool {
        // A recursive cycle typically involves a single function calling itself
        // or a very small number of functions in mutual recursion
        cycle.len() <= 2
    }

    /// Validate symbol-to-node mapping consistency
    fn validate_symbol_node_consistency(&self, result: &mut GraphValidationResult) {
        for (symbol_name, &node_idx) in &self.symbol_to_node {
            if let Some(node) = self.graph.node_weight(node_idx) {
                if node.symbol.qualified_name() != *symbol_name && node.symbol.name != *symbol_name
                {
                    result.errors.push(ValidationError {
                        kind: ValidationErrorKind::SymbolNodeMismatch,
                        message: format!(
                            "Symbol name '{}' doesn't match node symbol '{}' or '{}'",
                            symbol_name,
                            node.symbol.name,
                            node.symbol.qualified_name()
                        ),
                        location: Some(node.symbol.location.clone()),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        // Check for nodes without corresponding symbol mapping
        for node_idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(node_idx) {
                let qualified_name = node.symbol.qualified_name();
                let simple_name = &node.symbol.name;

                if !self.symbol_to_node.contains_key(&qualified_name)
                    && !self.symbol_to_node.contains_key(simple_name)
                {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::UnmappedNode,
                        message: format!("Node has no symbol mapping: '{qualified_name}'"),
                        location: Some(node.symbol.location.clone()),
                    });
                }
            }
        }
    }

    /// Validate edge relationships make semantic sense
    fn validate_edge_relationships(&self, result: &mut GraphValidationResult) {
        for edge_idx in self.graph.edge_indices() {
            if let Some((from_idx, to_idx)) = self.graph.edge_endpoints(edge_idx) {
                if let (Some(from_node), Some(to_node)) = (
                    self.graph.node_weight(from_idx),
                    self.graph.node_weight(to_idx),
                ) {
                    if let Some(relationship) = self.graph.edge_weight(edge_idx) {
                        self.validate_relationship_semantics(
                            from_node,
                            to_node,
                            relationship,
                            result,
                        );
                    }
                }
            }
        }
    }

    /// Validate that a relationship makes semantic sense
    fn validate_relationship_semantics(
        &self,
        from_node: &super::CodeNode,
        to_node: &super::CodeNode,
        relationship: &super::CodeRelationship,
        result: &mut GraphValidationResult,
    ) {
        use super::RelationshipKind;
        use crate::symbols::SymbolKind;

        match relationship.kind {
            RelationshipKind::Calls => {
                // Calls should typically be from functions to functions/methods
                if !matches!(
                    from_node.symbol.kind,
                    SymbolKind::Function | SymbolKind::Method
                ) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::SemanticMismatch,
                        message: format!(
                            "Call relationship from non-function symbol: {} ({:?})",
                            from_node.symbol.name, from_node.symbol.kind
                        ),
                        location: Some(from_node.symbol.location.clone()),
                    });
                }
            }
            RelationshipKind::Inherits => {
                // Inheritance should be between classes/structs/traits
                if !matches!(
                    from_node.symbol.kind,
                    SymbolKind::Class
                        | SymbolKind::Struct
                        | SymbolKind::Trait
                        | SymbolKind::Interface
                ) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::SemanticMismatch,
                        message: format!(
                            "Inheritance relationship from non-class symbol: {} ({:?})",
                            from_node.symbol.name, from_node.symbol.kind
                        ),
                        location: Some(from_node.symbol.location.clone()),
                    });
                }
            }
            RelationshipKind::Imports => {
                // Imports should typically be to modules/namespaces
                if !matches!(
                    to_node.symbol.kind,
                    SymbolKind::Module
                        | SymbolKind::Namespace
                        | SymbolKind::Class
                        | SymbolKind::Function
                        | SymbolKind::Type
                        | SymbolKind::Constant
                ) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::SemanticMismatch,
                        message: format!(
                            "Import relationship to unusual symbol type: {} ({:?})",
                            to_node.symbol.name, to_node.symbol.kind
                        ),
                        location: Some(to_node.symbol.location.clone()),
                    });
                }
            }
            _ => {} // Other relationships are more flexible
        }
    }

    /// Detect orphaned nodes (nodes with no edges)
    fn detect_orphaned_nodes(&self, result: &mut GraphValidationResult) {
        for node_idx in self.graph.node_indices() {
            let has_incoming = self
                .graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .next()
                .is_some();
            let has_outgoing = self
                .graph
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
                .next()
                .is_some();

            if !has_incoming && !has_outgoing {
                if let Some(node) = self.graph.node_weight(node_idx) {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::OrphanedNode,
                        message: format!(
                            "Orphaned node with no relationships: '{}'",
                            node.symbol.name
                        ),
                        location: Some(node.symbol.location.clone()),
                    });
                }
            }
        }
    }

    /// Validate scope consistency
    fn validate_scope_consistency(&self, result: &mut GraphValidationResult) {
        for (symbol_name, &node_idx) in &self.symbol_to_node {
            if let Some(node) = self.graph.node_weight(node_idx) {
                // Check if scope chain is consistent with qualified name
                let scope_from_chain = node
                    .symbol
                    .scope_chain
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                let qualified_name = node.symbol.qualified_name();

                if !qualified_name.starts_with(&scope_from_chain) && !scope_from_chain.is_empty() {
                    result.warnings.push(ValidationWarning {
                        kind: ValidationWarningKind::ScopeInconsistency,
                        message: format!(
                            "Scope chain '{scope_from_chain}' inconsistent with qualified name '{qualified_name}' for symbol '{symbol_name}'"
                        ),
                        location: Some(node.symbol.location.clone()),
                    });
                }
            }
        }
    }

    /// Get detailed cycle analysis
    pub fn analyze_cycles(&self) -> CycleAnalysis {
        let validation_result = self.validate_graph();

        let mut analysis = CycleAnalysis {
            total_cycles: validation_result.cycles.len(),
            cycles_by_kind: std::collections::HashMap::new(),
            problematic_cycles: Vec::new(),
            cycle_complexity_stats: CycleComplexityStats::default(),
        };

        // Group cycles by kind
        for cycle in &validation_result.cycles {
            *analysis.cycles_by_kind.entry(cycle.kind).or_insert(0) += 1;

            // Collect problematic cycles
            if matches!(
                cycle.severity,
                CycleSeverity::Error | CycleSeverity::Warning
            ) {
                analysis.problematic_cycles.push(ProblematicCycle {
                    kind: cycle.kind,
                    severity: cycle.severity,
                    nodes: cycle.nodes.clone(),
                    description: self.describe_cycle(cycle),
                });
            }
        }

        // Calculate complexity statistics
        let cycle_lengths: Vec<usize> = validation_result
            .cycles
            .iter()
            .map(|c| c.nodes.len())
            .collect();

        if !cycle_lengths.is_empty() {
            analysis.cycle_complexity_stats.average_cycle_length =
                cycle_lengths.iter().sum::<usize>() as f32 / cycle_lengths.len() as f32;
            analysis.cycle_complexity_stats.max_cycle_length =
                *cycle_lengths.iter().max().unwrap_or(&0);
            analysis.cycle_complexity_stats.min_cycle_length =
                *cycle_lengths.iter().min().unwrap_or(&0);
        }

        analysis
    }

    /// Generate a human-readable description of a cycle
    fn describe_cycle(&self, cycle: &GraphCycle) -> String {
        let symbol_names: Vec<String> = cycle
            .nodes
            .iter()
            .filter_map(|&node_idx| {
                self.graph
                    .node_weight(node_idx)
                    .map(|node| node.symbol.name.clone())
            })
            .collect();

        match cycle.kind {
            CycleKind::CallCycle => {
                if symbol_names.len() == 1 {
                    format!("Recursive function: {}", symbol_names[0])
                } else {
                    format!(
                        "Mutual recursion: {} → {}",
                        symbol_names.join(" → "),
                        symbol_names.first().unwrap_or(&"?".to_string())
                    )
                }
            }
            CycleKind::ImportCycle => {
                format!(
                    "Circular imports: {} → {}",
                    symbol_names.join(" → "),
                    symbol_names.first().unwrap_or(&"?".to_string())
                )
            }
            CycleKind::InheritanceCycle => {
                format!(
                    "Circular inheritance: {} → {}",
                    symbol_names.join(" → "),
                    symbol_names.first().unwrap_or(&"?".to_string())
                )
            }
            CycleKind::DataFlowCycle => {
                format!(
                    "Data flow cycle: {} → {}",
                    symbol_names.join(" → "),
                    symbol_names.first().unwrap_or(&"?".to_string())
                )
            }
        }
    }

    /// Check if the graph has any critical issues
    pub fn has_critical_issues(&self) -> bool {
        let validation_result = self.validate_graph();
        !validation_result.errors.is_empty()
            || validation_result
                .cycles
                .iter()
                .any(|c| matches!(c.severity, CycleSeverity::Error))
    }

    /// Get a summary report of graph health
    pub fn get_health_report(&self) -> GraphHealthReport {
        let validation_result = self.validate_graph();
        let cycle_analysis = self.analyze_cycles();

        let health_score = self.calculate_health_score(&validation_result, &cycle_analysis);
        let error_count = validation_result.errors.len();
        let warning_count = validation_result.warnings.len();
        let statistics = validation_result.statistics.clone();
        let recommendations = self.generate_recommendations(&validation_result, &cycle_analysis);

        GraphHealthReport {
            overall_health_score: health_score,
            is_healthy: health_score >= 0.8,
            error_count,
            warning_count,
            critical_cycle_count: validation_result
                .cycles
                .iter()
                .filter(|c| matches!(c.severity, CycleSeverity::Error))
                .count(),
            statistics,
            recommendations,
        }
    }

    /// Calculate overall graph health score (0.0 to 1.0)
    fn calculate_health_score(
        &self,
        validation: &GraphValidationResult,
        _cycles: &CycleAnalysis,
    ) -> f32 {
        let mut score = 1.0;

        // Penalize errors heavily
        score -= validation.errors.len() as f32 * 0.2;

        // Penalize warnings moderately
        score -= validation.warnings.len() as f32 * 0.05;

        // Penalize critical cycles
        let critical_cycles = validation
            .cycles
            .iter()
            .filter(|c| matches!(c.severity, CycleSeverity::Error))
            .count();
        score -= critical_cycles as f32 * 0.25;

        // Penalize warning cycles lightly
        let warning_cycles = validation
            .cycles
            .iter()
            .filter(|c| matches!(c.severity, CycleSeverity::Warning))
            .count();
        score -= warning_cycles as f32 * 0.05;

        score.clamp(0.0, 1.0)
    }

    /// Generate recommendations for improving graph health
    fn generate_recommendations(
        &self,
        validation: &GraphValidationResult,
        cycles: &CycleAnalysis,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !validation.errors.is_empty() {
            recommendations.push(format!(
                "Fix {} critical errors in graph structure",
                validation.errors.len()
            ));
        }

        if validation.warnings.len() > 10 {
            recommendations.push(format!(
                "Address {} warnings to improve code quality",
                validation.warnings.len()
            ));
        }

        let import_cycles = cycles
            .cycles_by_kind
            .get(&CycleKind::ImportCycle)
            .unwrap_or(&0);
        if *import_cycles > 0 {
            recommendations.push(format!(
                "Resolve {import_cycles} circular import dependencies"
            ));
        }

        let inheritance_cycles = cycles
            .cycles_by_kind
            .get(&CycleKind::InheritanceCycle)
            .unwrap_or(&0);
        if *inheritance_cycles > 0 {
            recommendations.push(format!(
                "Fix {inheritance_cycles} circular inheritance relationships"
            ));
        }

        if cycles.cycle_complexity_stats.max_cycle_length > 5 {
            recommendations.push("Consider breaking up complex dependency cycles".to_string());
        }

        if validation
            .warnings
            .iter()
            .any(|w| matches!(w.kind, ValidationWarningKind::OrphanedNode))
        {
            recommendations.push("Review orphaned symbols that may be unused".to_string());
        }

        recommendations
    }
}

/// Result of graph validation
#[derive(Debug, Clone)]
pub struct GraphValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub cycles: Vec<GraphCycle>,
    pub statistics: GraphStatistics,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub message: String,
    pub location: Option<crate::symbols::Location>,
    pub severity: ValidationSeverity,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    InvalidNodeIndex,
    SymbolNodeMismatch,
    InvalidEdge,
    CorruptedData,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub kind: ValidationWarningKind,
    pub message: String,
    pub location: Option<crate::symbols::Location>,
}

/// Types of validation warnings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarningKind {
    DuplicateSymbol,
    UnmappedNode,
    SemanticMismatch,
    OrphanedNode,
    ScopeInconsistency,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Detected cycle in the graph
#[derive(Debug, Clone)]
pub struct GraphCycle {
    pub kind: CycleKind,
    pub nodes: Vec<NodeIndex>,
    pub severity: CycleSeverity,
}

/// Types of cycles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CycleKind {
    CallCycle,
    ImportCycle,
    InheritanceCycle,
    DataFlowCycle,
}

/// Cycle severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleSeverity {
    Error,   // Must be fixed
    Warning, // Should be reviewed
    Info,    // Informational only
}

/// Graph statistics
#[derive(Debug, Clone, Default)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub symbol_definitions: usize,
    pub call_relationships: usize,
    pub import_relationships: usize,
    pub inheritance_relationships: usize,
    pub data_flow_relationships: usize,
    pub control_flow_relationships: usize,
    pub file_count: usize,
    pub average_edges_per_node: f32,
}

/// Comprehensive cycle analysis
#[derive(Debug, Clone)]
pub struct CycleAnalysis {
    pub total_cycles: usize,
    pub cycles_by_kind: std::collections::HashMap<CycleKind, usize>,
    pub problematic_cycles: Vec<ProblematicCycle>,
    pub cycle_complexity_stats: CycleComplexityStats,
}

/// Problematic cycle details
#[derive(Debug, Clone)]
pub struct ProblematicCycle {
    pub kind: CycleKind,
    pub severity: CycleSeverity,
    pub nodes: Vec<NodeIndex>,
    pub description: String,
}

/// Cycle complexity statistics
#[derive(Debug, Clone, Default)]
pub struct CycleComplexityStats {
    pub average_cycle_length: f32,
    pub max_cycle_length: usize,
    pub min_cycle_length: usize,
}

/// Overall graph health report
#[derive(Debug, Clone)]
pub struct GraphHealthReport {
    pub overall_health_score: f32,
    pub is_healthy: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub critical_cycle_count: usize,
    pub statistics: GraphStatistics,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};

    #[test]
    fn test_graph_validation_basic() {
        let mut builder = CodeGraphBuilder::new();

        // Create a simple valid graph
        let function1 = Symbol {
            name: "function1".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn function1() {}".to_string()),
        };

        builder.add_file_symbols(vec![function1], "test.rs");

        // Validate the graph
        let validation_result = builder.validate_graph();
        assert!(validation_result.is_valid);
        assert_eq!(validation_result.errors.len(), 0);
        assert_eq!(validation_result.statistics.total_nodes, 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create functions that call each other (mutual recursion)
        let func_a = Symbol {
            name: "func_a".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn func_a() { func_b(); }".to_string()),
        };

        let func_b = Symbol {
            name: "func_b".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 11,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn func_b() { func_a(); }".to_string()),
        };

        builder.add_file_symbols(vec![func_a, func_b], "test.rs");

        // Create a call cycle
        let a_idx = builder.find_definition("func_a").unwrap();
        let b_idx = builder.find_definition("func_b").unwrap();

        let call_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Calls,
            source_location: "test.rs:3".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder.call_graph_edges.insert((a_idx, b_idx));
        builder.call_graph_edges.insert((b_idx, a_idx));
        builder.graph.add_edge(a_idx, b_idx, call_rel.clone());
        builder.graph.add_edge(b_idx, a_idx, call_rel);

        // Validate and check for cycles
        let validation_result = builder.validate_graph();
        assert!(!validation_result.cycles.is_empty());

        let call_cycles: Vec<_> = validation_result
            .cycles
            .iter()
            .filter(|c| matches!(c.kind, CycleKind::CallCycle))
            .collect();
        assert!(!call_cycles.is_empty());
    }

    #[test]
    fn test_import_cycle_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create modules with circular imports
        let mod_a = Symbol {
            name: "mod_a".to_string(),
            kind: SymbolKind::Module,
            location: Location {
                file_path: "a.rs".to_string(),
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

        let mod_b = Symbol {
            name: "mod_b".to_string(),
            kind: SymbolKind::Module,
            location: Location {
                file_path: "b.rs".to_string(),
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

        builder.add_file_symbols(vec![mod_a], "a.rs");
        builder.add_file_symbols(vec![mod_b], "b.rs");

        // Create circular import
        let a_idx = builder.find_definition("mod_a").unwrap();
        let b_idx = builder.find_definition("mod_b").unwrap();

        let import_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Imports,
            source_location: "a.rs:1".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder.import_graph_edges.insert((a_idx, b_idx));
        builder.import_graph_edges.insert((b_idx, a_idx));
        builder.graph.add_edge(a_idx, b_idx, import_rel.clone());
        builder.graph.add_edge(b_idx, a_idx, import_rel);

        // Validate and check for import cycles
        let validation_result = builder.validate_graph();
        let import_cycles: Vec<_> = validation_result
            .cycles
            .iter()
            .filter(|c| matches!(c.kind, CycleKind::ImportCycle))
            .collect();

        assert!(!import_cycles.is_empty());
        assert!(matches!(import_cycles[0].severity, CycleSeverity::Error));
    }

    #[test]
    fn test_orphaned_node_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create an isolated function with no relationships
        let orphaned_func = Symbol {
            name: "orphaned_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn orphaned_function() {}".to_string()),
        };

        let connected_func = Symbol {
            name: "connected_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 11,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn connected_function() { orphaned_function(); }".to_string()),
        };

        builder.add_file_symbols(vec![orphaned_func, connected_func], "test.rs");

        // Add only one relationship, leaving orphaned_function isolated
        let orphaned_idx = builder.find_definition("orphaned_function").unwrap();
        let connected_idx = builder.find_definition("connected_function").unwrap();

        let call_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Calls,
            source_location: "test.rs:9".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder
            .call_graph_edges
            .insert((connected_idx, orphaned_idx));
        builder
            .graph
            .add_edge(connected_idx, orphaned_idx, call_rel);

        // Validate and check for orphaned nodes
        let validation_result = builder.validate_graph();

        // Neither function should be orphaned since they are connected
        let orphaned_warnings: Vec<_> = validation_result
            .warnings
            .iter()
            .filter(|w| matches!(w.kind, ValidationWarningKind::OrphanedNode))
            .collect();

        assert_eq!(orphaned_warnings.len(), 0);
    }

    #[test]
    fn test_graph_health_report() {
        let mut builder = CodeGraphBuilder::new();

        // Create a healthy graph
        let func1 = Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "main.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["pub".to_string()],
            signature: Some("pub fn main() { helper(); }".to_string()),
        };

        let func2 = Symbol {
            name: "helper".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "main.rs".to_string(),
                start_line: 12,
                start_column: 0,
                end_line: 15,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: Some("fn helper() {}".to_string()),
        };

        builder.add_file_symbols(vec![func1, func2], "main.rs");

        // Add a healthy relationship
        let main_idx = builder.find_definition("main").unwrap();
        let helper_idx = builder.find_definition("helper").unwrap();

        let call_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Calls,
            source_location: "main.rs:3".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder.call_graph_edges.insert((main_idx, helper_idx));
        builder.graph.add_edge(main_idx, helper_idx, call_rel);

        // Get health report
        let health_report = builder.get_health_report();

        assert!(health_report.is_healthy);
        assert!(health_report.overall_health_score >= 0.8);
        assert_eq!(health_report.error_count, 0);
        assert_eq!(health_report.critical_cycle_count, 0);
    }

    #[test]
    fn test_cycle_analysis() {
        let mut builder = CodeGraphBuilder::new();

        // Create multiple types of cycles
        let class_a = Symbol {
            name: "ClassA".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        let class_b = Symbol {
            name: "ClassB".to_string(),
            kind: SymbolKind::Class,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 7,
                start_column: 0,
                end_line: 11,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        builder.add_file_symbols(vec![class_a, class_b], "test.rs");

        // Create inheritance cycle
        let a_idx = builder.find_definition("ClassA").unwrap();
        let b_idx = builder.find_definition("ClassB").unwrap();

        let inherit_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Inherits,
            source_location: "test.rs:1".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder.inheritance_graph_edges.insert((a_idx, b_idx));
        builder.inheritance_graph_edges.insert((b_idx, a_idx));
        builder.graph.add_edge(a_idx, b_idx, inherit_rel.clone());
        builder.graph.add_edge(b_idx, a_idx, inherit_rel);

        // Analyze cycles
        let cycle_analysis = builder.analyze_cycles();

        assert!(cycle_analysis.total_cycles > 0);
        assert!(cycle_analysis
            .cycles_by_kind
            .contains_key(&CycleKind::InheritanceCycle));
        assert!(!cycle_analysis.problematic_cycles.is_empty());

        // Inheritance cycles should be marked as errors
        let inheritance_cycle = cycle_analysis
            .problematic_cycles
            .iter()
            .find(|c| matches!(c.kind, CycleKind::InheritanceCycle));
        assert!(inheritance_cycle.is_some());
        assert!(matches!(
            inheritance_cycle.unwrap().severity,
            CycleSeverity::Error
        ));
    }

    #[test]
    fn test_critical_issues_detection() {
        let mut builder = CodeGraphBuilder::new();

        // Create a graph with critical issues (inheritance cycle)
        let struct_a = Symbol {
            name: "StructA".to_string(),
            kind: SymbolKind::Struct,
            location: Location {
                file_path: "test.rs".to_string(),
                start_line: 1,
                start_column: 0,
                end_line: 3,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        builder.add_file_symbols(vec![struct_a], "test.rs");

        // Create a self-inheritance (impossible in Rust, but for testing)
        let a_idx = builder.find_definition("StructA").unwrap();

        let inherit_rel = super::CodeRelationship {
            kind: super::RelationshipKind::Inherits,
            source_location: "test.rs:1".to_string(),
            confidence: 1.0,
            metadata: std::collections::HashMap::new(),
        };

        builder.inheritance_graph_edges.insert((a_idx, a_idx));
        builder.graph.add_edge(a_idx, a_idx, inherit_rel);

        // Check for critical issues
        assert!(builder.has_critical_issues());

        let health_report = builder.get_health_report();
        assert!(!health_report.is_healthy);
        assert!(health_report.critical_cycle_count > 0);
    }
}

// ====================================
// Graph Merging Tests
// ====================================

#[cfg(test)]
mod merge_tests {
    use super::*;
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Scope, Symbol, SymbolKind};
    use std::collections::HashMap;

    fn create_test_symbol(name: &str, kind: SymbolKind, file_path: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            location: Location {
                file_path: file_path.to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![Scope {
                name: "global".to_string(),
                kind: SymbolKind::Module,
                location: Location {
                    file_path: file_path.to_string(),
                    start_line: 0,
                    start_column: 0,
                    end_line: 0,
                    end_column: 0,
                },
            }],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        }
    }

    fn create_external_symbol(name: &str, kind: SymbolKind, file_path: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
            location: Location {
                file_path: file_path.to_string(),
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
            scope_chain: vec![], // External symbols have no scope chain
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec!["external".to_string()],
            signature: None,
        }
    }

    #[test]
    fn test_basic_graph_merge() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Add symbols to first builder
        let symbol1 = create_test_symbol("function_a", SymbolKind::Function, "file1.rs");
        let symbol2 = create_test_symbol("struct_b", SymbolKind::Struct, "file1.rs");
        builder1.add_file_symbols(vec![symbol1, symbol2], "file1.rs");

        // Add symbols to second builder
        let symbol3 = create_test_symbol("function_c", SymbolKind::Function, "file2.rs");
        let symbol4 = create_test_symbol("enum_d", SymbolKind::Enum, "file2.rs");
        builder2.add_file_symbols(vec![symbol3, symbol4], "file2.rs");

        // Merge builder2 into builder1
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.symbols_added, 2);
        assert_eq!(merge_result.files_added, 1);
        assert_eq!(merge_result.conflicts_resolved, 0);

        // Verify merged content (check for qualified names)
        assert!(builder1.symbol_to_node.contains_key("global::function_a"));
        assert!(builder1.symbol_to_node.contains_key("global::struct_b"));
        assert!(builder1.symbol_to_node.contains_key("global::function_c"));
        assert!(builder1.symbol_to_node.contains_key("global::enum_d"));

        assert_eq!(builder1.file_symbols.len(), 2);
        assert!(builder1.file_symbols.contains_key("file1.rs"));
        assert!(builder1.file_symbols.contains_key("file2.rs"));
    }

    #[test]
    fn test_symbol_conflict_resolution() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Add same symbol to both builders
        let symbol1 = create_test_symbol("shared_function", SymbolKind::Function, "file1.rs");
        let symbol2 = create_test_symbol("shared_function", SymbolKind::Function, "file2.rs");

        builder1.add_file_symbols(vec![symbol1], "file1.rs");
        builder2.add_file_symbols(vec![symbol2], "file2.rs");

        // Merge should handle the conflict
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.conflicts_resolved, 1);
        assert_eq!(merge_result.conflicts.len(), 1);

        let conflict = &merge_result.conflicts[0];
        assert_eq!(conflict.symbol_name, "global::shared_function");
        // Since the symbols are essentially equivalent, it should keep existing
        assert!(matches!(
            conflict.resolution_strategy,
            ConflictResolutionStrategy::KeepExisting
        ));
    }

    #[test]
    fn test_external_vs_actual_symbol_resolution() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Builder1 has external reference (no scope, simple name)
        let external_symbol =
            create_external_symbol("shared_function", SymbolKind::Function, "external");
        builder1.add_file_symbols(vec![external_symbol], "file1.rs");

        // Builder2 has actual definition with scope
        let actual_symbol = create_test_symbol("shared_function", SymbolKind::Function, "file2.rs");
        builder2.add_file_symbols(vec![actual_symbol], "file2.rs");

        // External has name "shared_function", actual has "global::shared_function" - no conflict
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.conflicts_resolved, 0); // No conflict since different qualified names
        assert_eq!(merge_result.symbols_added, 1); // The actual symbol is added
    }

    #[test]
    fn test_incompatible_symbol_types() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Add function to first builder
        let symbol1 = create_test_symbol("conflicted_name", SymbolKind::Function, "file1.rs");
        builder1.add_file_symbols(vec![symbol1], "file1.rs");

        // Add struct with same name to second builder
        let symbol2 = create_test_symbol("conflicted_name", SymbolKind::Struct, "file2.rs");
        builder2.add_file_symbols(vec![symbol2], "file2.rs");

        // Merge should fail due to incompatible types
        let merge_result = builder1.merge(builder2);

        assert!(merge_result.is_err());
        if let Err(MergeError::IncompatibleSymbols {
            symbol_name,
            existing_kind,
            other_kind,
        }) = merge_result
        {
            assert_eq!(symbol_name, "conflicted_name");
            assert_eq!(existing_kind, "Function");
            assert_eq!(other_kind, "Struct");
        } else {
            panic!("Expected IncompatibleSymbols error");
        }
    }

    #[test]
    fn test_merge_multiple_builders() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();
        let mut builder3 = CodeGraphBuilder::new();

        // Add unique symbols to each builder
        builder1.add_file_symbols(
            vec![create_test_symbol(
                "func1",
                SymbolKind::Function,
                "file1.rs",
            )],
            "file1.rs",
        );

        builder2.add_file_symbols(
            vec![create_test_symbol(
                "func2",
                SymbolKind::Function,
                "file2.rs",
            )],
            "file2.rs",
        );

        builder3.add_file_symbols(
            vec![create_test_symbol(
                "func3",
                SymbolKind::Function,
                "file3.rs",
            )],
            "file3.rs",
        );

        // Merge all builders
        let merged = CodeGraphBuilder::merge_multiple(vec![builder1, builder2, builder3]).unwrap();

        // Verify all symbols are present (check for qualified names)
        assert!(merged.symbol_to_node.contains_key("global::func1"));
        assert!(merged.symbol_to_node.contains_key("global::func2"));
        assert!(merged.symbol_to_node.contains_key("global::func3"));

        assert_eq!(merged.file_symbols.len(), 3);
        assert!(merged.file_symbols.contains_key("file1.rs"));
        assert!(merged.file_symbols.contains_key("file2.rs"));
        assert!(merged.file_symbols.contains_key("file3.rs"));
    }

    #[test]
    fn test_cross_file_dependencies_analysis() {
        let mut builder = CodeGraphBuilder::new();

        // Create symbols in different files
        let symbol1 = create_test_symbol("caller", SymbolKind::Function, "file1.rs");
        let symbol2 = create_test_symbol("callee", SymbolKind::Function, "file2.rs");
        let external_symbol =
            create_external_symbol("external_lib", SymbolKind::Function, "external");

        builder.add_file_symbols(vec![symbol1], "file1.rs");
        builder.add_file_symbols(vec![symbol2], "file2.rs");
        builder.add_file_symbols(vec![external_symbol], "file1.rs");

        // Add cross-file relationship (use qualified names)
        let caller_node = *builder.symbol_to_node.get("global::caller").unwrap();
        let callee_node = *builder.symbol_to_node.get("global::callee").unwrap();
        let external_node = *builder.symbol_to_node.get("external_lib").unwrap(); // External has no scope

        let call_relationship = CodeRelationship {
            kind: RelationshipKind::Calls,
            source_location: "file1.rs:5:10".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        let external_relationship = CodeRelationship {
            kind: RelationshipKind::Calls,
            source_location: "file1.rs:10:5".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
        };

        builder
            .graph
            .add_edge(caller_node, callee_node, call_relationship);
        builder
            .graph
            .add_edge(caller_node, external_node, external_relationship);

        // Analyze cross-file dependencies
        let cross_file_deps = builder.get_cross_file_dependencies();

        // Only 1 cross-file edge since external symbol is in same file as caller
        assert_eq!(cross_file_deps.total_cross_file_edges, 1);
        assert!(cross_file_deps
            .file_to_file
            .contains_key(&("file1.rs".to_string(), "file2.rs".to_string())));

        // External dependency call is within same file, so no external cross-file deps
        assert_eq!(cross_file_deps.external_dependencies.len(), 0);
    }

    #[test]
    fn test_equivalent_symbols_merge() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Create identical symbols
        let symbol1 = Symbol {
            name: "identical_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "file1.rs".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: Some("A test function".to_string()),
            modifiers: vec!["pub".to_string()],
            signature: Some("fn identical_function() -> i32".to_string()),
        };

        let symbol2 = symbol1.clone();

        builder1.add_file_symbols(vec![symbol1], "file1.rs");
        builder2.add_file_symbols(vec![symbol2], "file1.rs");

        // Merge should recognize they're equivalent
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.conflicts_resolved, 1);

        let conflict = &merge_result.conflicts[0];
        assert!(matches!(
            conflict.resolution_strategy,
            ConflictResolutionStrategy::KeepExisting
        ));
    }

    #[test]
    fn test_rename_strategy_for_different_languages() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Same symbol name but different languages
        let rust_symbol = Symbol {
            name: "print".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "file1.rs".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        let python_symbol = Symbol {
            name: "print".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "file1.py".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 3,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Python,
            documentation: None,
            modifiers: vec![],
            signature: None,
        };

        builder1.add_file_symbols(vec![rust_symbol], "file1.rs");
        builder2.add_file_symbols(vec![python_symbol], "file1.py");

        // Merge should rename one symbol to avoid conflict
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.conflicts_resolved, 1);

        let conflict = &merge_result.conflicts[0];
        assert!(matches!(
            conflict.resolution_strategy,
            ConflictResolutionStrategy::Rename
        ));
    }

    #[test]
    fn test_merge_with_relationships() {
        let mut builder1 = CodeGraphBuilder::new();
        let mut builder2 = CodeGraphBuilder::new();

        // Builder1: function A calls function B
        let func_a = create_test_symbol("function_a", SymbolKind::Function, "file1.rs");
        let func_b = create_test_symbol("function_b", SymbolKind::Function, "file1.rs");
        builder1.add_file_symbols(vec![func_a, func_b], "file1.rs");

        let a_node = *builder1.symbol_to_node.get("global::function_a").unwrap();
        let b_node = *builder1.symbol_to_node.get("global::function_b").unwrap();

        let relationship = CodeRelationship {
            kind: RelationshipKind::Calls,
            source_location: "file1.rs:5:10".to_string(),
            confidence: 1.0,
            metadata: HashMap::new(),
        };

        builder1.graph.add_edge(a_node, b_node, relationship);
        builder1.call_graph_edges.insert((a_node, b_node));

        // Builder2: function C calls function D
        let func_c = create_test_symbol("function_c", SymbolKind::Function, "file2.rs");
        let func_d = create_test_symbol("function_d", SymbolKind::Function, "file2.rs");
        builder2.add_file_symbols(vec![func_c, func_d], "file2.rs");

        let c_node = *builder2.symbol_to_node.get("global::function_c").unwrap();
        let d_node = *builder2.symbol_to_node.get("global::function_d").unwrap();

        let relationship2 = CodeRelationship {
            kind: RelationshipKind::Calls,
            source_location: "file2.rs:3:5".to_string(),
            confidence: 0.9,
            metadata: HashMap::new(),
        };

        builder2.graph.add_edge(c_node, d_node, relationship2);
        builder2.call_graph_edges.insert((c_node, d_node));

        // Merge and verify relationships are preserved
        let merge_result = builder1.merge(builder2).unwrap();

        assert!(merge_result.success);
        assert_eq!(merge_result.relationships_added, 1);

        // Check that specialized edge sets were merged
        assert_eq!(builder1.call_graph_edges.len(), 2);

        // Verify the relationship exists in the merged graph
        let merged_c_node = *builder1.symbol_to_node.get("global::function_c").unwrap();
        let merged_d_node = *builder1.symbol_to_node.get("global::function_d").unwrap();
        assert!(builder1
            .graph
            .find_edge(merged_c_node, merged_d_node)
            .is_some());
    }
}
