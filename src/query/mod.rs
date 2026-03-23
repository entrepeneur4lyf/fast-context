//! # Query Interface for AI Assistants
//!
//! Provides high-level query interface for AI coding assistants to efficiently
//! retrieve code context, relationships, and insights from the analyzed codebase.

use crate::analysis::{AnalysisResult, CodeRelationship};
use crate::symbols::{Symbol, SymbolKind};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configurable thresholds for architectural analysis
#[derive(Debug, Clone)]
pub struct ArchitecturalThresholds {
    pub high_coupling_threshold: usize,
    pub high_complexity_threshold: u32,
    pub high_dependency_threshold: usize,
    pub critical_impact_threshold: f32,
    pub high_impact_threshold: f32,
    pub medium_impact_threshold: f32,
    pub max_files_for_critical: usize,
    pub max_files_for_high: usize,
    pub large_result_threshold: usize,
}

impl Default for ArchitecturalThresholds {
    fn default() -> Self {
        Self {
            high_coupling_threshold: 10,     // Industry standard for high coupling
            high_complexity_threshold: 20,   // McCabe complexity threshold
            high_dependency_threshold: 15,   // High dependency count
            critical_impact_threshold: 20.0, // Critical cycle impact
            high_impact_threshold: 10.0,     // High cycle impact
            medium_impact_threshold: 5.0,    // Medium cycle impact
            max_files_for_critical: 5,       // Files involved for critical severity
            max_files_for_high: 2,           // Files involved for high severity
            large_result_threshold: 10,      // Large result set threshold
        }
    }
}

impl ArchitecturalThresholds {
    /// Create thresholds optimized for large codebases
    pub fn for_large_codebase() -> Self {
        Self {
            high_coupling_threshold: 15,     // Higher threshold for large codebases
            high_complexity_threshold: 25,   // Higher complexity tolerance
            high_dependency_threshold: 20,   // More dependencies expected
            critical_impact_threshold: 30.0, // Higher impact threshold
            high_impact_threshold: 15.0,     // Higher impact threshold
            large_result_threshold: 20,      // Larger result sets expected
            ..Default::default()
        }
    }

    /// Create thresholds optimized for small/medium codebases
    pub fn for_small_codebase() -> Self {
        Self {
            high_coupling_threshold: 7,      // Lower threshold for small codebases
            high_complexity_threshold: 15,   // Lower complexity tolerance
            high_dependency_threshold: 10,   // Fewer dependencies expected
            critical_impact_threshold: 15.0, // Lower impact threshold
            high_impact_threshold: 8.0,      // Lower impact threshold
            large_result_threshold: 5,       // Smaller result sets expected
            ..Default::default()
        }
    }

    /// Load thresholds from environment variables with fallback to defaults
    pub fn from_environment() -> Self {
        let mut thresholds = Self::default();

        if let Ok(val) = std::env::var("RUSTWORKX_COUPLING_THRESHOLD") {
            if let Ok(threshold) = val.parse::<usize>() {
                thresholds.high_coupling_threshold = threshold.clamp(5, 50);
            }
        }

        if let Ok(val) = std::env::var("RUSTWORKX_COMPLEXITY_THRESHOLD") {
            if let Ok(threshold) = val.parse::<u32>() {
                thresholds.high_complexity_threshold = threshold.clamp(10, 100);
            }
        }

        if let Ok(val) = std::env::var("RUSTWORKX_DEPENDENCY_THRESHOLD") {
            if let Ok(threshold) = val.parse::<usize>() {
                thresholds.high_dependency_threshold = threshold.clamp(5, 100);
            }
        }

        thresholds
    }
}

/// Query result containing requested information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub symbols: Vec<SymbolInfo>,
    pub relationships: Vec<RelationshipInfo>,
    pub context: ContextInfo,
    pub suggestions: Vec<String>,
}

/// Detailed symbol information for AI assistants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub file_path: String,
    pub complexity: u32,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub related_files: Vec<String>,
}

/// Relationship information between symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInfo {
    pub from_symbol: String,
    pub to_symbol: String,
    pub relationship_type: String,
    pub confidence: f32,
    pub source_location: String,
}

/// Context information for better understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub total_symbols: usize,
    pub files_involved: usize,
    pub complexity_score: f32,
    pub architectural_patterns: Vec<String>,
    pub potential_issues: Vec<String>,
}

/// Circular dependency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyInfo {
    /// The symbols involved in the cycle
    pub cycle: Vec<String>,
    /// Severity level of the cycle
    pub severity: CycleSeverity,
    /// File paths involved in the cycle
    pub files_involved: Vec<String>,
    /// Suggestions for breaking the cycle
    pub breaking_suggestions: Vec<String>,
    /// Impact score (higher = more problematic)
    pub impact_score: f32,
}

/// Severity classification for circular dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleSeverity {
    /// Low impact cycles (e.g., utility functions)
    Low,
    /// Medium impact cycles (e.g., related classes)
    Medium,
    /// High impact cycles (e.g., core architecture components)
    High,
    /// Critical cycles (e.g., fundamental system dependencies)
    Critical,
}

impl std::fmt::Display for CycleSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CycleSeverity::Low => write!(f, "LOW"),
            CycleSeverity::Medium => write!(f, "MEDIUM"),
            CycleSeverity::High => write!(f, "HIGH"),
            CycleSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Query interface for code analysis results
pub struct CodeQueryEngine {
    analysis: AnalysisResult,
    symbol_index: HashMap<String, NodeIndex>,
    file_index: HashMap<String, Vec<NodeIndex>>,
    thresholds: ArchitecturalThresholds,
}

impl CodeQueryEngine {
    /// Create a new query engine from analysis results
    pub fn new(analysis: AnalysisResult) -> Self {
        let mut symbol_index = HashMap::new();
        let mut file_index: HashMap<String, Vec<NodeIndex>> = HashMap::new();

        // Build indexes for efficient querying
        for node_idx in analysis.graph.node_indices() {
            if let Some(node) = analysis.graph.node_weight(node_idx) {
                let qualified_name = node.symbol.qualified_name();
                symbol_index.insert(qualified_name, node_idx);

                file_index
                    .entry(node.file_path.clone())
                    .or_insert_with(Vec::new)
                    .push(node_idx);
            }
        }

        Self {
            analysis,
            symbol_index,
            file_index,
            thresholds: ArchitecturalThresholds::from_environment(),
        }
    }

    /// Find symbols by name (fuzzy matching)
    pub fn find_symbols(&self, pattern: &str) -> QueryResult {
        let mut matching_symbols = Vec::new();
        let pattern_lower = pattern.to_lowercase();

        for (symbol_name, &node_idx) in &self.symbol_index {
            if symbol_name.to_lowercase().contains(&pattern_lower) {
                if let Some(symbol_info) = self.get_symbol_info(node_idx) {
                    matching_symbols.push(symbol_info);
                }
            }
        }

        let context = self.build_context(&matching_symbols);
        let suggestions = self.generate_suggestions(&matching_symbols, pattern);

        QueryResult {
            symbols: matching_symbols,
            relationships: Vec::new(),
            context,
            suggestions,
        }
    }

    /// Find symbols by type/kind
    pub fn find_symbols_by_kind(&self, kind: SymbolKind) -> QueryResult {
        let matching_symbols: Vec<SymbolInfo> = self
            .analysis
            .graph
            .node_indices()
            .filter_map(|node_idx| {
                if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                    if node.symbol.kind == kind {
                        return self.get_symbol_info(node_idx);
                    }
                }
                None
            })
            .collect();

        let context = self.build_context(&matching_symbols);
        let suggestions = self.generate_kind_suggestions(&kind);

        QueryResult {
            symbols: matching_symbols,
            relationships: Vec::new(),
            context,
            suggestions,
        }
    }

    /// Find all symbols in a specific file
    pub fn find_symbols_in_file(&self, file_path: &str) -> QueryResult {
        let mut matching_symbols = Vec::new();

        if let Some(node_indices) = self.file_index.get(file_path) {
            for &node_idx in node_indices {
                if let Some(symbol_info) = self.get_symbol_info(node_idx) {
                    matching_symbols.push(symbol_info);
                }
            }
        }

        let context = self.build_context(&matching_symbols);
        let suggestions = self.generate_file_suggestions(file_path);

        QueryResult {
            symbols: matching_symbols,
            relationships: Vec::new(),
            context,
            suggestions,
        }
    }

    /// Find symbols that depend on a given symbol
    pub fn find_dependents(&self, symbol_name: &str) -> QueryResult {
        let mut dependents = Vec::new();
        let mut relationships = Vec::new();

        if let Some(&target_node) = self.symbol_index.get(symbol_name) {
            // Find all nodes that have edges pointing to this symbol
            for edge_idx in self
                .analysis
                .graph
                .edges_directed(target_node, petgraph::Incoming)
            {
                let source_node = edge_idx.source();

                if let Some(symbol_info) = self.get_symbol_info(source_node) {
                    dependents.push(symbol_info);
                }

                if let Some(edge_data) = self.analysis.graph.edge_weight(edge_idx.id()) {
                    relationships.push(self.relationship_to_info(
                        source_node,
                        target_node,
                        edge_data,
                    ));
                }
            }
        }

        let context = self.build_context(&dependents);
        let suggestions = self.generate_dependency_suggestions(symbol_name, true);

        QueryResult {
            symbols: dependents,
            relationships,
            context,
            suggestions,
        }
    }

    /// Find symbols that a given symbol depends on
    pub fn find_dependencies(&self, symbol_name: &str) -> QueryResult {
        let mut dependencies = Vec::new();
        let mut relationships = Vec::new();

        if let Some(&source_node) = self.symbol_index.get(symbol_name) {
            // Find all nodes that this symbol points to
            for edge_idx in self
                .analysis
                .graph
                .edges_directed(source_node, petgraph::Outgoing)
            {
                let target_node = edge_idx.target();

                if let Some(symbol_info) = self.get_symbol_info(target_node) {
                    dependencies.push(symbol_info);
                }

                if let Some(edge_data) = self.analysis.graph.edge_weight(edge_idx.id()) {
                    relationships.push(self.relationship_to_info(
                        source_node,
                        target_node,
                        edge_data,
                    ));
                }
            }
        }

        let context = self.build_context(&dependencies);
        let suggestions = self.generate_dependency_suggestions(symbol_name, false);

        QueryResult {
            symbols: dependencies,
            relationships,
            context,
            suggestions,
        }
    }

    /// Find the most complex symbols in the codebase
    pub fn find_complex_symbols(&self, limit: usize) -> QueryResult {
        let complex_symbols: Vec<SymbolInfo> = self
            .analysis
            .graph
            .node_indices()
            .filter_map(|node_idx| self.get_symbol_info(node_idx))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let mut sorted_symbols = complex_symbols;
        sorted_symbols.sort_by(|a, b| b.complexity.cmp(&a.complexity));
        sorted_symbols.truncate(limit);

        let context = self.build_context(&sorted_symbols);
        let suggestions = vec![
            "Consider refactoring high-complexity functions".to_string(),
            "Break down complex functions into smaller ones".to_string(),
            "Add unit tests for complex logic".to_string(),
        ];

        QueryResult {
            symbols: sorted_symbols,
            relationships: Vec::new(),
            context,
            suggestions,
        }
    }

    /// Find potential architectural issues
    pub fn find_architectural_issues(&self) -> QueryResult {
        let mut issues = Vec::new();
        let mut problem_symbols = Vec::new();

        // Find symbols with high fan-out (too many dependencies)
        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let outgoing_edges = self
                    .analysis
                    .graph
                    .edges_directed(node_idx, petgraph::Outgoing)
                    .count();

                if outgoing_edges > self.thresholds.high_coupling_threshold {
                    issues.push(format!(
                        "Symbol '{}' has high coupling with {} dependencies",
                        node.symbol.name, outgoing_edges
                    ));

                    if let Some(symbol_info) = self.get_symbol_info(node_idx) {
                        problem_symbols.push(symbol_info);
                    }
                }
            }
        }

        // Find circular dependencies with comprehensive analysis
        let cycle_analysis = self.analyze_circular_dependencies();
        for cycle_info in cycle_analysis {
            issues.push(format!(
                "Circular dependency detected ({}): {} -> {}",
                cycle_info.severity,
                cycle_info.cycle.join(" -> "),
                cycle_info.cycle[0]
            ));

            // Add cycle breaking suggestions
            for suggestion in &cycle_info.breaking_suggestions {
                issues.push(format!("  Suggestion: {suggestion}"));
            }
        }

        let context = ContextInfo {
            total_symbols: self.analysis.symbol_count,
            files_involved: problem_symbols
                .iter()
                .map(|s| s.file_path.clone())
                .collect::<HashSet<_>>()
                .len(),
            complexity_score: self.calculate_overall_complexity(),
            architectural_patterns: self.detect_patterns(),
            potential_issues: issues.clone(),
        };

        let suggestions = vec![
            "Consider applying dependency inversion principle".to_string(),
            "Break circular dependencies by introducing interfaces".to_string(),
            "Refactor high-coupling components".to_string(),
        ];

        QueryResult {
            symbols: problem_symbols,
            relationships: Vec::new(),
            context,
            suggestions,
        }
    }

    /// Get detailed information about a specific symbol
    fn get_symbol_info(&self, node_idx: NodeIndex) -> Option<SymbolInfo> {
        let node = self.analysis.graph.node_weight(node_idx)?;

        // Find dependencies (outgoing edges)
        let dependencies: Vec<String> = self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
            .filter_map(|edge| {
                let target_node = self.analysis.graph.node_weight(edge.target())?;
                Some(target_node.symbol.qualified_name())
            })
            .collect();

        // Find dependents (incoming edges)
        let dependents: Vec<String> = self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Incoming)
            .filter_map(|edge| {
                let source_node = self.analysis.graph.node_weight(edge.source())?;
                Some(source_node.symbol.qualified_name())
            })
            .collect();

        // Find related files through dependencies
        let mut related_files: HashSet<String> = dependencies
            .iter()
            .chain(dependents.iter())
            .filter_map(|symbol_name| {
                let node_idx = self.symbol_index.get(symbol_name)?;
                let node = self.analysis.graph.node_weight(*node_idx)?;
                Some(node.file_path.clone())
            })
            .collect();

        related_files.remove(&node.file_path); // Remove the symbol's own file
        let related_files: Vec<String> = related_files.into_iter().collect();

        Some(SymbolInfo {
            symbol: node.symbol.clone(),
            file_path: node.file_path.clone(),
            complexity: node.metrics.cyclomatic_complexity,
            dependencies,
            dependents,
            related_files,
        })
    }

    /// Convert relationship edge to relationship info
    fn relationship_to_info(
        &self,
        source_node: NodeIndex,
        target_node: NodeIndex,
        relationship: &CodeRelationship,
    ) -> RelationshipInfo {
        let source_symbol = self
            .analysis
            .graph
            .node_weight(source_node)
            .map(|n| n.symbol.qualified_name())
            .unwrap_or_else(|| "unknown".to_string());

        let target_symbol = self
            .analysis
            .graph
            .node_weight(target_node)
            .map(|n| n.symbol.qualified_name())
            .unwrap_or_else(|| "unknown".to_string());

        RelationshipInfo {
            from_symbol: source_symbol,
            to_symbol: target_symbol,
            relationship_type: format!("{:?}", relationship.kind),
            confidence: relationship.confidence,
            source_location: relationship.source_location.clone(),
        }
    }

    /// Build context information for query results
    fn build_context(&self, symbols: &[SymbolInfo]) -> ContextInfo {
        let files_involved = symbols
            .iter()
            .map(|s| s.file_path.clone())
            .collect::<HashSet<_>>()
            .len();

        let complexity_score = if symbols.is_empty() {
            0.0
        } else {
            symbols.iter().map(|s| s.complexity as f32).sum::<f32>() / symbols.len() as f32
        };

        ContextInfo {
            total_symbols: symbols.len(),
            files_involved,
            complexity_score,
            architectural_patterns: self.detect_patterns(),
            potential_issues: self.detect_issues(symbols),
        }
    }

    /// Generate suggestions based on query results
    fn generate_suggestions(&self, symbols: &[SymbolInfo], pattern: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if symbols.is_empty() {
            suggestions.push(format!("No symbols found matching '{pattern}'"));
            suggestions.push("Try using partial names or check spelling".to_string());
        } else {
            suggestions.push(format!(
                "Found {} symbols matching '{}'",
                symbols.len(),
                pattern
            ));

            if symbols.len() > self.thresholds.large_result_threshold {
                suggestions.push("Consider narrowing your search".to_string());
            }

            // Suggest related symbols
            let related: HashSet<String> = symbols
                .iter()
                .flat_map(|s| s.dependencies.iter().chain(s.dependents.iter()))
                .cloned()
                .collect();

            if !related.is_empty() {
                let related_list: Vec<String> = related.into_iter().take(3).collect();
                suggestions.push(format!("Related symbols: {}", related_list.join(", ")));
            }
        }

        suggestions
    }

    /// Generate suggestions for symbol kind queries
    fn generate_kind_suggestions(&self, kind: &SymbolKind) -> Vec<String> {
        match kind {
            SymbolKind::Function => vec![
                "Consider analyzing function complexity".to_string(),
                "Look for functions with high coupling".to_string(),
            ],
            SymbolKind::Class | SymbolKind::Struct => vec![
                "Check for inheritance relationships".to_string(),
                "Analyze class responsibilities".to_string(),
            ],
            _ => vec!["Explore relationships with other symbols".to_string()],
        }
    }

    /// Generate suggestions for file-based queries
    fn generate_file_suggestions(&self, _file_path: &str) -> Vec<String> {
        vec![
            "Analyze symbol interactions within this file".to_string(),
            "Check for external dependencies".to_string(),
            "Consider file organization and cohesion".to_string(),
        ]
    }

    /// Generate suggestions for dependency queries
    fn generate_dependency_suggestions(
        &self,
        symbol_name: &str,
        is_dependents: bool,
    ) -> Vec<String> {
        if is_dependents {
            vec![
                format!("Symbols that depend on '{}'", symbol_name),
                "Consider impact of changes to this symbol".to_string(),
            ]
        } else {
            vec![
                format!("Dependencies of '{}'", symbol_name),
                "Consider reducing coupling if dependency count is high".to_string(),
            ]
        }
    }

    /// Comprehensive architectural pattern detection using structural analysis
    fn detect_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Analyze the graph structure for patterns
        patterns.extend(self.detect_factory_pattern());
        patterns.extend(self.detect_singleton_pattern());
        patterns.extend(self.detect_builder_pattern());
        patterns.extend(self.detect_abstract_factory_pattern());
        patterns.extend(self.detect_observer_pattern());
        patterns.extend(self.detect_strategy_pattern());
        patterns.extend(self.detect_decorator_pattern());
        patterns.extend(self.detect_adapter_pattern());
        patterns.extend(self.detect_mvc_pattern());
        patterns.extend(self.detect_repository_pattern());
        patterns.extend(self.detect_dependency_injection_pattern());

        patterns
    }

    /// Detect Factory pattern using structural analysis
    fn detect_factory_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                // Check for factory naming patterns
                if name.contains("factory") || name.ends_with("factory") {
                    // Verify it creates other objects (has outgoing relationships to classes)
                    let creates_objects = self
                        .analysis
                        .graph
                        .edges_directed(node_idx, petgraph::Outgoing)
                        .any(|edge| {
                            if let Some(target) = self.analysis.graph.node_weight(edge.target()) {
                                target.symbol.kind == SymbolKind::Class
                                    && edge.weight().kind
                                        == crate::analysis::RelationshipKind::Calls
                            } else {
                                false
                            }
                        });

                    if creates_objects {
                        patterns.push(format!("Factory Pattern ({})", node.symbol.name));
                    }
                }

                // Check for factory methods (static methods that return instances)
                if node.symbol.kind == SymbolKind::Method {
                    if let Some(signature) = &node.symbol.signature {
                        if signature.contains("static")
                            && (name.contains("create")
                                || name.contains("make")
                                || name.contains("build")
                                || name.contains("factory")
                                || name.contains("instantiate"))
                        {
                            // Enhanced factory method detection with return type analysis
                            if self.is_factory_method(node_idx) {
                                let factory_type = self.classify_factory_method(node_idx);
                                patterns.push(format!(
                                    "Factory Method Pattern ({}) - {}",
                                    node.symbol.name, factory_type
                                ));
                            }
                        }
                    }
                }
            }
        }

        patterns
    }

    /// Detect Singleton pattern using structural analysis
    fn detect_singleton_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                if node.symbol.kind == SymbolKind::Class {
                    let name = &node.symbol.name.to_lowercase();

                    // Check for singleton naming
                    if name.contains("singleton") {
                        patterns.push(format!("Singleton Pattern ({})", node.symbol.name));
                        continue;
                    }

                    // Check for singleton structural patterns
                    let has_private_constructor = self.has_private_constructor(node_idx);
                    let has_static_instance_method = self.has_static_instance_method(node_idx);
                    let has_static_field = self.has_static_instance_field(node_idx);

                    if has_private_constructor && (has_static_instance_method || has_static_field) {
                        patterns.push(format!("Singleton Pattern ({})", node.symbol.name));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Builder pattern using comprehensive structural analysis
    fn detect_builder_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                // Enhanced Builder pattern detection
                if node.symbol.kind == SymbolKind::Class || node.symbol.kind == SymbolKind::Struct {
                    // Primary detection: builder naming
                    let is_named_builder = name.contains("builder");

                    // Structural detection: fluent interface analysis
                    let builder_score = self.calculate_builder_score(node_idx);

                    // Method chaining detection
                    let has_method_chaining = self.has_method_chaining(node_idx);

                    // Configuration pattern detection
                    let has_config_methods = self.has_configuration_methods(node_idx);

                    // Final build/create method
                    let has_build_method = self.has_build_method(node_idx);

                    // Classify builder type based on analysis
                    if is_named_builder && builder_score >= 3 {
                        let builder_type = self.classify_builder_type(node_idx, builder_score);
                        patterns.push(format!(
                            "Builder Pattern ({}) - {}",
                            node.symbol.name, builder_type
                        ));
                    } else if !is_named_builder && builder_score >= 4 {
                        // Structural builder without "builder" in name
                        let builder_type = self.classify_builder_type(node_idx, builder_score);
                        patterns.push(format!(
                            "Builder Pattern ({}) - Structural {}",
                            node.symbol.name, builder_type
                        ));
                    } else if has_method_chaining && has_build_method && has_config_methods {
                        // Fluent interface builder
                        patterns.push(format!(
                            "Builder Pattern ({}) - Fluent Interface",
                            node.symbol.name
                        ));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Abstract Factory pattern using comprehensive family analysis
    fn detect_abstract_factory_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                // Primary detection: abstract factory naming
                let is_abstract_factory = name.contains("abstract") && name.contains("factory")
                    || name.contains("factory")
                        && (name.contains("interface") || name.contains("trait"))
                    || name.contains("abstract") && name.contains("creator");

                // Structural detection: factory family analysis
                let factory_score = self.calculate_abstract_factory_score(node_idx);

                // Interface family detection
                let has_family_interfaces = self.has_factory_family_interfaces(node_idx);

                // Concrete factory implementations
                let has_concrete_factories = self.has_concrete_factory_implementations(node_idx);

                // Product family consistency
                let has_product_family = self.has_consistent_product_family(node_idx);

                // Classify Abstract Factory type
                if is_abstract_factory && factory_score >= 4 {
                    let factory_type = self.classify_abstract_factory_type(node_idx, factory_score);
                    patterns.push(format!(
                        "Abstract Factory Pattern ({}) - {}",
                        node.symbol.name, factory_type
                    ));
                } else if !is_abstract_factory && factory_score >= 5 {
                    // Structural abstract factory without explicit naming
                    let factory_type = self.classify_abstract_factory_type(node_idx, factory_score);
                    patterns.push(format!(
                        "Abstract Factory Pattern ({}) - Structural {}",
                        node.symbol.name, factory_type
                    ));
                } else if has_family_interfaces && has_concrete_factories && has_product_family {
                    // Family-based abstract factory
                    patterns.push(format!(
                        "Abstract Factory Pattern ({}) - Family-Based",
                        node.symbol.name
                    ));
                }
            }
        }

        patterns
    }

    /// Detect Observer pattern
    fn detect_observer_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Look for observer/listener interfaces and implementations
        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if (name.contains("observer")
                    || name.contains("listener")
                    || name.contains("subscriber"))
                    && (node.symbol.kind == SymbolKind::Interface
                        || node.symbol.kind == SymbolKind::Class)
                {
                    // Check for notify/update methods
                    if self.has_notification_methods(node_idx) {
                        patterns.push(format!("Observer Pattern ({})", node.symbol.name));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Strategy pattern
    fn detect_strategy_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if name.contains("strategy") && node.symbol.kind == SymbolKind::Interface {
                    // Check for multiple implementations
                    let implementations = self.count_implementations(node_idx);
                    if implementations > 1 {
                        patterns.push(format!(
                            "Strategy Pattern ({} with {} implementations)",
                            node.symbol.name, implementations
                        ));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Decorator pattern
    fn detect_decorator_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if name.contains("decorator") || name.contains("wrapper") {
                    // Check if it implements the same interface as what it wraps
                    if self.implements_wrapped_interface(node_idx) {
                        patterns.push(format!("Decorator Pattern ({})", node.symbol.name));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Adapter pattern
    fn detect_adapter_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if name.contains("adapter") || name.contains("wrapper") {
                    // Check if it adapts between different interfaces
                    if self.adapts_interfaces(node_idx) {
                        patterns.push(format!("Adapter Pattern ({})", node.symbol.name));
                    }
                }
            }
        }

        patterns
    }

    /// Detect MVC pattern
    fn detect_mvc_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        let has_controllers = self
            .symbol_index
            .keys()
            .any(|name| name.to_lowercase().contains("controller"));
        let has_models = self
            .symbol_index
            .keys()
            .any(|name| name.to_lowercase().contains("model"));
        let has_views = self
            .symbol_index
            .keys()
            .any(|name| name.to_lowercase().contains("view"));

        if has_controllers && has_models && has_views {
            patterns.push("MVC Pattern".to_string());
        }

        patterns
    }

    /// Detect Repository pattern
    fn detect_repository_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if name.contains("repository")
                    && (node.symbol.kind == SymbolKind::Class
                        || node.symbol.kind == SymbolKind::Interface)
                {
                    // Check for CRUD methods
                    if self.has_crud_methods(node_idx) {
                        patterns.push(format!("Repository Pattern ({})", node.symbol.name));
                    }
                }
            }
        }

        patterns
    }

    /// Detect Dependency Injection pattern
    fn detect_dependency_injection_pattern(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Look for constructor injection patterns
        let constructor_injection_count = self.count_constructor_injection();
        if constructor_injection_count > 3 {
            patterns.push(format!(
                "Dependency Injection Pattern ({constructor_injection_count} classes)"
            ));
        }

        // Look for DI containers
        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();

                if (name.contains("container") || name.contains("injector") || name.contains("ioc"))
                    && node.symbol.kind == SymbolKind::Class
                {
                    patterns.push(format!("DI Container Pattern ({})", node.symbol.name));
                }
            }
        }

        patterns
    }

    /// Detect potential issues in symbols
    fn detect_issues(&self, symbols: &[SymbolInfo]) -> Vec<String> {
        let mut issues = Vec::new();

        for symbol in symbols {
            if symbol.complexity > self.thresholds.high_complexity_threshold {
                issues.push(format!("High complexity in '{}'", symbol.symbol.name));
            }

            if symbol.dependencies.len() > self.thresholds.high_dependency_threshold {
                issues.push(format!("High coupling in '{}'", symbol.symbol.name));
            }
        }

        issues
    }

    /// Comprehensive circular dependency analysis with severity classification
    fn analyze_circular_dependencies(&self) -> Vec<CircularDependencyInfo> {
        use petgraph::algo::kosaraju_scc;

        let dependency_graph = &self.analysis.graph;
        let sccs = kosaraju_scc(dependency_graph);

        let mut cycle_infos = Vec::new();

        // Analyze each strongly connected component with more than one node
        for scc in sccs.into_iter().filter(|scc| scc.len() > 1) {
            let cycle_symbols: Vec<String> = scc
                .iter()
                .filter_map(|&node_idx| {
                    dependency_graph
                        .node_weight(node_idx)
                        .map(|node| node.symbol.name.clone())
                })
                .collect();

            if cycle_symbols.is_empty() {
                continue;
            }

            // Get files involved in the cycle
            let files_involved: Vec<String> = scc
                .iter()
                .filter_map(|&node_idx| {
                    dependency_graph
                        .node_weight(node_idx)
                        .map(|node| node.file_path.clone())
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            // Calculate impact score based on various factors
            let impact_score = self.calculate_cycle_impact(&scc);

            // Determine severity based on impact score and cycle characteristics
            let severity = self.classify_cycle_severity(impact_score, &scc, &files_involved);

            // Generate cycle breaking suggestions
            let breaking_suggestions =
                self.generate_cycle_breaking_suggestions(&scc, &cycle_symbols);

            cycle_infos.push(CircularDependencyInfo {
                cycle: cycle_symbols,
                severity,
                files_involved,
                breaking_suggestions,
                impact_score,
            });
        }

        // Sort by impact score (highest first)
        cycle_infos.sort_by(|a, b| {
            b.impact_score
                .partial_cmp(&a.impact_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        cycle_infos
    }

    /// Calculate impact score for a cycle
    fn calculate_cycle_impact(&self, scc: &[NodeIndex]) -> f32 {
        let dependency_graph = &self.analysis.graph;
        let mut impact_score = 0.0;

        for &node_idx in scc {
            if let Some(node) = dependency_graph.node_weight(node_idx) {
                // Factor in complexity
                impact_score += node.metrics.cyclomatic_complexity as f32 * 0.1;

                // Factor in fan-in/fan-out (more connections = higher impact)
                impact_score += (node.metrics.fan_in + node.metrics.fan_out) as f32 * 0.2;

                // Factor in symbol type (classes and modules have higher impact)
                match node.symbol.kind {
                    SymbolKind::Class => impact_score += 2.0,
                    SymbolKind::Module => impact_score += 1.5,
                    SymbolKind::Function => impact_score += 1.0,
                    _ => impact_score += 0.5,
                }
            }
        }

        // Factor in cycle size (larger cycles are more problematic)
        impact_score += scc.len() as f32 * 0.5;

        impact_score
    }

    /// Classify cycle severity based on impact and characteristics
    fn classify_cycle_severity(
        &self,
        impact_score: f32,
        scc: &[NodeIndex],
        files_involved: &[String],
    ) -> CycleSeverity {
        // Critical: High impact, many files, or core system components
        if impact_score > self.thresholds.critical_impact_threshold
            || files_involved.len() > self.thresholds.max_files_for_critical
            || self.involves_core_components(scc)
        {
            CycleSeverity::Critical
        }
        // High: Significant impact or cross-file dependencies
        else if impact_score > self.thresholds.high_impact_threshold
            || files_involved.len() > self.thresholds.max_files_for_high
        {
            CycleSeverity::High
        }
        // Medium: Moderate impact
        else if impact_score > self.thresholds.medium_impact_threshold {
            CycleSeverity::Medium
        }
        // Low: Minor impact, likely utility functions
        else {
            CycleSeverity::Low
        }
    }

    /// Check if cycle involves core system components
    fn involves_core_components(&self, scc: &[NodeIndex]) -> bool {
        let dependency_graph = &self.analysis.graph;

        for &node_idx in scc {
            if let Some(node) = dependency_graph.node_weight(node_idx) {
                let name = &node.symbol.name.to_lowercase();
                // Check for common core component patterns
                if name.contains("main")
                    || name.contains("core")
                    || name.contains("system")
                    || name.contains("manager")
                    || name.contains("controller")
                {
                    return true;
                }
            }
        }
        false
    }

    /// Generate suggestions for breaking circular dependencies
    fn generate_cycle_breaking_suggestions(
        &self,
        scc: &[NodeIndex],
        _cycle_symbols: &[String],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Generic suggestions based on cycle characteristics
        if scc.len() > 3 {
            suggestions.push(
                "Consider breaking this large cycle into smaller, more focused components"
                    .to_string(),
            );
        }

        suggestions.push(
            "Introduce an interface or abstract class to break direct dependencies".to_string(),
        );
        suggestions.push("Use dependency injection to invert control flow".to_string());
        suggestions.push("Extract common functionality into a separate module".to_string());

        // Specific suggestions based on symbol types
        let dependency_graph = &self.analysis.graph;
        let has_classes = scc.iter().any(|&node_idx| {
            dependency_graph
                .node_weight(node_idx)
                .map(|node| node.symbol.kind == SymbolKind::Class)
                .unwrap_or(false)
        });

        if has_classes {
            suggestions.push("Apply the Dependency Inversion Principle - depend on abstractions, not concretions".to_string());
            suggestions
                .push("Consider using the Observer pattern to decouple components".to_string());
        }

        // File-based suggestions
        let files_involved: HashSet<String> = scc
            .iter()
            .filter_map(|&node_idx| {
                dependency_graph
                    .node_weight(node_idx)
                    .map(|node| node.file_path.clone())
            })
            .collect();

        if files_involved.len() > 1 {
            suggestions
                .push("Consider reorganizing code to reduce cross-file dependencies".to_string());
            suggestions
                .push("Move related functionality into the same module or package".to_string());
        }

        suggestions
    }

    /// Calculate overall complexity score for the codebase
    fn calculate_overall_complexity(&self) -> f32 {
        let total_complexity: u32 = self
            .analysis
            .graph
            .node_weights()
            .map(|node| node.metrics.cyclomatic_complexity)
            .sum();

        total_complexity as f32 / self.analysis.symbol_count as f32
    }

    // Helper methods for pattern detection

    /// Check if a class has a private constructor
    fn has_private_constructor(&self, node_idx: NodeIndex) -> bool {
        // Look for constructor methods with private modifiers
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method
                    && (target_node.symbol.name == "constructor"
                        || target_node.symbol.name == "__init__")
                {
                    return target_node
                        .symbol
                        .modifiers
                        .contains(&"private".to_string());
                }
            }
        }
        false
    }

    /// Check if a class has static instance methods
    fn has_static_instance_method(&self, node_idx: NodeIndex) -> bool {
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let name = &target_node.symbol.name.to_lowercase();
                    if (name.contains("instance") || name.contains("getinstance"))
                        && target_node.symbol.modifiers.contains(&"static".to_string())
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a class has static instance fields
    fn has_static_instance_field(&self, node_idx: NodeIndex) -> bool {
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Field {
                    let name = &target_node.symbol.name.to_lowercase();
                    if name.contains("instance")
                        && target_node.symbol.modifiers.contains(&"static".to_string())
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a class has a build method
    fn has_build_method(&self, node_idx: NodeIndex) -> bool {
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let name = &target_node.symbol.name.to_lowercase();
                    if name == "build" || name == "create" || name == "construct" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a class has notification methods (for Observer pattern)
    fn has_notification_methods(&self, node_idx: NodeIndex) -> bool {
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let name = &target_node.symbol.name.to_lowercase();
                    if name.contains("notify")
                        || name.contains("update")
                        || name.contains("onchange")
                        || name.contains("handle")
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Count implementations of an interface
    fn count_implementations(&self, interface_idx: NodeIndex) -> usize {
        let mut count = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(interface_idx, petgraph::Incoming)
        {
            if edge.weight().kind == crate::analysis::RelationshipKind::Implements {
                count += 1;
            }
        }

        count
    }

    /// Check if a decorator implements the same interface as what it wraps
    fn implements_wrapped_interface(&self, node_idx: NodeIndex) -> bool {
        // Find the wrapped component (composition relationship)
        let mut wrapped_components = Vec::new();
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if edge.weight().kind == crate::analysis::RelationshipKind::Composition {
                wrapped_components.push(edge.target());
            }
        }

        if wrapped_components.is_empty() {
            return false; // No wrapped component found
        }

        // Get interfaces implemented by the decorator
        let mut decorator_interfaces = std::collections::HashSet::new();
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if edge.weight().kind == crate::analysis::RelationshipKind::Implements {
                decorator_interfaces.insert(edge.target());
            }
        }

        // Check if any wrapped component implements the same interfaces
        for wrapped_idx in wrapped_components {
            let mut wrapped_interfaces = std::collections::HashSet::new();
            for edge in self
                .analysis
                .graph
                .edges_directed(wrapped_idx, petgraph::Outgoing)
            {
                if edge.weight().kind == crate::analysis::RelationshipKind::Implements {
                    wrapped_interfaces.insert(edge.target());
                }
            }

            // Check for interface overlap
            if !decorator_interfaces.is_disjoint(&wrapped_interfaces) {
                return true;
            }
        }

        false
    }

    /// Check if an adapter adapts between different interfaces
    fn adapts_interfaces(&self, node_idx: NodeIndex) -> bool {
        let mut implements_count = 0;
        let mut depends_count = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            match edge.weight().kind {
                crate::analysis::RelationshipKind::Implements => implements_count += 1,
                crate::analysis::RelationshipKind::DependsOn => depends_count += 1,
                _ => {}
            }
        }

        implements_count >= 1 && depends_count >= 1
    }

    /// Check if a repository has CRUD methods
    fn has_crud_methods(&self, node_idx: NodeIndex) -> bool {
        let mut crud_methods = 0;
        let crud_patterns = [
            "create", "read", "update", "delete", "find", "save", "get", "remove",
        ];

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let name = &target_node.symbol.name.to_lowercase();
                    if crud_patterns.iter().any(|pattern| name.contains(pattern)) {
                        crud_methods += 1;
                    }
                }
            }
        }

        crud_methods >= 2 // At least 2 CRUD operations
    }

    /// Count classes using constructor injection
    fn count_constructor_injection(&self) -> usize {
        let mut count = 0;

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                if node.symbol.kind == SymbolKind::Class {
                    // Check if constructor has dependencies injected
                    for edge in self
                        .analysis
                        .graph
                        .edges_directed(node_idx, petgraph::Outgoing)
                    {
                        if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                            if target_node.symbol.kind == SymbolKind::Method
                                && (target_node.symbol.name == "constructor"
                                    || target_node.symbol.name == "__init__")
                            {
                                // Check if constructor has parameters (simplified check)
                                if let Some(signature) = &target_node.symbol.signature {
                                    if signature.contains("(") && signature.contains(",") {
                                        count += 1;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        count
    }

    /// Enhanced factory method detection with comprehensive signature analysis
    fn is_factory_method(&self, node_idx: NodeIndex) -> bool {
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            if node.symbol.kind != SymbolKind::Method {
                return false;
            }

            // Check method signature for factory characteristics
            if let Some(signature) = &node.symbol.signature {
                // Must be static or class method
                let is_static = signature.contains("static") || signature.contains("classmethod");

                // Should return an object type (not void/unit)
                let returns_object = self.returns_object_type(node_idx);

                // Should have configuration parameters
                let has_config_params = self.has_configuration_parameters(node_idx);

                // Name should indicate creation
                let name = &node.symbol.name.to_lowercase();
                let factory_name_indicators = name.contains("create")
                    || name.contains("make")
                    || name.contains("build")
                    || name.contains("factory")
                    || name.contains("instantiate")
                    || name.contains("new");

                is_static && returns_object && (factory_name_indicators || has_config_params)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Classify factory method type based on signature and usage patterns
    fn classify_factory_method(&self, node_idx: NodeIndex) -> String {
        let mut classification = "Standard".to_string();

        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            let name = &node.symbol.name.to_lowercase();

            // Classify based on naming patterns
            if name.contains("builder") {
                classification = "Builder".to_string();
            } else if name.contains("factory") {
                classification = "Factory".to_string();
            } else if name.contains("create") && name.contains("default") {
                classification = "Default Creator".to_string();
            } else if name.contains("from") {
                classification = "Type Converter".to_string();
            } else if name.contains("parse") {
                classification = "Parser Factory".to_string();
            }

            // Enhance classification based on parameters
            if self.has_configuration_parameters(node_idx) {
                classification = format!("{} (Configurable)", classification);
            }

            // Check if it's part of an Abstract Factory pattern
            if self.is_part_of_abstract_factory(node_idx) {
                classification = format!("{} (Abstract Factory)", classification);
            }
        }

        classification
    }

    /// Check if method returns an object type
    fn returns_object_type(&self, node_idx: NodeIndex) -> bool {
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            if let Some(signature) = &node.symbol.signature {
                // Look for return type annotations in various languages
                let return_indicators = [
                    "->", ":", "returns", "=>", // Common return type syntax
                ];

                for indicator in return_indicators {
                    if signature.contains(indicator) {
                        // Check that it's not returning void/unit
                        let return_part = signature.split(indicator).nth(1).unwrap_or("");
                        let return_type = return_part.trim().to_lowercase();

                        // Exclude void/unit types
                        if !return_type.contains("void")
                            && !return_type.contains("unit")
                            && !return_type.contains("none")
                            && !return_type.is_empty()
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if method has configuration parameters
    fn has_configuration_parameters(&self, node_idx: NodeIndex) -> bool {
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            if let Some(signature) = &node.symbol.signature {
                // Look for parameter patterns that suggest configuration
                let config_indicators = ["config", "options", "settings", "params", "args"];

                for indicator in config_indicators {
                    if signature.to_lowercase().contains(indicator) {
                        return true;
                    }
                }

                // Check for multiple parameters (suggests configuration)
                let param_count = signature.matches(',').count();
                param_count > 1
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if method is part of an Abstract Factory pattern
    fn is_part_of_abstract_factory(&self, node_idx: NodeIndex) -> bool {
        // Look for abstract factory interface relationships
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Incoming)
        {
            if let Some(source_node) = self.analysis.graph.node_weight(edge.source()) {
                let source_name = source_node.symbol.name.to_lowercase();

                // Check if parent is an abstract factory
                if source_name.contains("factory")
                    || source_name.contains("creator")
                    || source_name.contains("builder")
                {
                    // Check if parent has multiple factory methods
                    let mut factory_method_count = 0;
                    for child_edge in self
                        .analysis
                        .graph
                        .edges_directed(edge.source(), petgraph::Outgoing)
                    {
                        if let Some(child_node) =
                            self.analysis.graph.node_weight(child_edge.target())
                        {
                            if child_node.symbol.kind == SymbolKind::Method {
                                let child_name = child_node.symbol.name.to_lowercase();
                                if child_name.contains("create") || child_name.contains("make") {
                                    factory_method_count += 1;
                                }
                            }
                        }
                    }

                    return factory_method_count >= 2;
                }
            }
        }
        false
    }

    /// Calculate comprehensive builder pattern score
    fn calculate_builder_score(&self, node_idx: NodeIndex) -> u32 {
        let mut score = 0;

        // Check for fluent interface methods
        let fluent_count = self.count_fluent_methods(node_idx);
        score += fluent_count * 2;

        // Check for configuration methods (set_*, with_*, add_*)
        let config_count = self.count_configuration_methods(node_idx);
        score += config_count;

        // Check for build/create method
        if self.has_build_method(node_idx) {
            score += 3;
        }

        // Check for method chaining capability
        if self.has_method_chaining(node_idx) {
            score += 2;
        }

        // Check for builder naming patterns
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            let name = node.symbol.name.to_lowercase();
            if name.contains("builder") {
                score += 2;
            }
            if name.ends_with("builder") {
                score += 1;
            }
        }

        score
    }

    /// Count fluent interface methods (returning self/this)
    fn count_fluent_methods(&self, node_idx: NodeIndex) -> u32 {
        let mut count = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    if let Some(signature) = &target_node.symbol.signature {
                        // Enhanced self-return detection across languages
                        if signature.contains("-> Self")
                            || signature.contains("return self")
                            || signature.contains("return this")
                            || signature.contains("return *this")
                            || signature.contains("*this")
                            || signature.contains("&self")
                            || signature.contains("&mut self")
                            || signature.contains("self:")
                            || signature.contains("this:")
                        {
                            count += 1;
                        }
                    }

                    // Check method naming for fluent patterns
                    let method_name = target_node.symbol.name.to_lowercase();
                    if method_name.starts_with("set_")
                        || method_name.starts_with("with_")
                        || method_name.starts_with("add_")
                        || method_name.starts_with("build_")
                        || method_name.starts_with("create_")
                    {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Count configuration methods
    fn count_configuration_methods(&self, node_idx: NodeIndex) -> u32 {
        let mut count = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let method_name = target_node.symbol.name.to_lowercase();

                    // Configuration method patterns
                    if method_name.starts_with("set_")
                        || method_name.starts_with("with_")
                        || method_name.starts_with("add_")
                        || method_name.starts_with("enable_")
                        || method_name.starts_with("disable_")
                        || method_name.starts_with("configure_")
                    {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Check for method chaining capability
    fn has_method_chaining(&self, node_idx: NodeIndex) -> bool {
        let mut chainable_methods = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    if let Some(signature) = &target_node.symbol.signature {
                        // Check for return types that enable chaining
                        if signature.contains("-> Self")
                            || signature.contains("return self")
                            || signature.contains("return this")
                            || signature.contains("&self")
                            || signature.contains("&mut self")
                        {
                            chainable_methods += 1;
                        }
                    }
                }
            }
        }

        chainable_methods >= 2 // Need at least 2 chainable methods
    }

    /// Check for configuration methods presence
    fn has_configuration_methods(&self, node_idx: NodeIndex) -> bool {
        self.count_configuration_methods(node_idx) >= 2
    }

    /// Classify builder type based on analysis
    fn classify_builder_type(&self, node_idx: NodeIndex, score: u32) -> String {
        let fluent_count = self.count_fluent_methods(node_idx);
        let config_count = self.count_configuration_methods(node_idx);

        if score >= 8 && fluent_count >= 3 {
            "Comprehensive Builder".to_string()
        } else if score >= 6 && config_count >= 3 {
            "Configuration Builder".to_string()
        } else if fluent_count >= 2 {
            "Fluent Builder".to_string()
        } else if config_count >= 2 {
            "Step Builder".to_string()
        } else {
            "Basic Builder".to_string()
        }
    }

    /// Calculate comprehensive Abstract Factory pattern score
    fn calculate_abstract_factory_score(&self, node_idx: NodeIndex) -> u32 {
        let mut score = 0;

        // Check for abstract factory naming
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            let name = node.symbol.name.to_lowercase();
            if name.contains("abstract") && name.contains("factory") {
                score += 4;
            } else if name.contains("factory")
                && (name.contains("interface") || name.contains("trait"))
            {
                score += 3;
            } else if name.contains("factory") {
                score += 2;
            }
        }

        // Check for factory family interfaces
        let interface_count = self.count_factory_interfaces(node_idx);
        score += interface_count * 2;

        // Check for concrete factory implementations
        let concrete_count = self.count_concrete_factory_implementations(node_idx);
        score += concrete_count;

        // Check for product family consistency
        if self.has_consistent_product_family(node_idx) {
            score += 3;
        }

        // Check for factory method patterns
        let factory_method_count = self.count_factory_methods(node_idx);
        score += factory_method_count;

        // Check for interface/trait characteristics
        if self.is_interface_or_trait(node_idx) {
            score += 2;
        }

        score
    }

    /// Count factory interfaces in the hierarchy
    fn count_factory_interfaces(&self, node_idx: NodeIndex) -> u32 {
        let mut count = 0;

        // Check if this node itself is an interface
        if self.is_interface_or_trait(node_idx) {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let name = node.symbol.name.to_lowercase();
                if name.contains("factory") || name.contains("creator") {
                    count += 1;
                }
            }
        }

        // Check related interfaces
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if self.is_interface_or_trait(edge.target()) {
                    let target_name = target_node.symbol.name.to_lowercase();
                    if target_name.contains("factory") || target_name.contains("creator") {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Count concrete factory implementations
    fn count_concrete_factory_implementations(&self, node_idx: NodeIndex) -> u32 {
        let mut count = 0;

        // Look for implementations/inheritors
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Incoming)
        {
            if let Some(source_node) = self.analysis.graph.node_weight(edge.source()) {
                let source_name = source_node.symbol.name.to_lowercase();
                if (source_name.contains("factory") || source_name.contains("creator"))
                    && !source_name.contains("abstract")
                {
                    count += 1;
                }
            }
        }

        count
    }

    /// Check if node has factory family interfaces
    fn has_factory_family_interfaces(&self, node_idx: NodeIndex) -> bool {
        self.count_factory_interfaces(node_idx) >= 2
    }

    /// Check if node has concrete factory implementations
    fn has_concrete_factory_implementations(&self, node_idx: NodeIndex) -> bool {
        self.count_concrete_factory_implementations(node_idx) >= 2
    }

    /// Check for consistent product family
    fn has_consistent_product_family(&self, node_idx: NodeIndex) -> bool {
        let mut product_types = std::collections::HashSet::new();

        // Analyze factory methods to identify product types
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let method_name = target_node.symbol.name.to_lowercase();
                    if method_name.starts_with("create")
                        || method_name.starts_with("make")
                        || method_name.starts_with("build")
                    {
                        // Extract product type from method name
                        if let Some(product_type) =
                            self.extract_product_type_from_method(&method_name)
                        {
                            product_types.insert(product_type);
                        }
                    }
                }
            }
        }

        // Check if we have multiple related product types
        product_types.len() >= 2
    }

    /// Extract product type from factory method name
    fn extract_product_type_from_method(&self, method_name: &str) -> Option<String> {
        // Extract from methods like create_button, make_window, build_dialog
        if method_name.starts_with("create") {
            method_name.strip_prefix("create").map(|s| s.to_string())
        } else if method_name.starts_with("make") {
            method_name.strip_prefix("make").map(|s| s.to_string())
        } else if method_name.starts_with("build") {
            method_name.strip_prefix("build").map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Check if node is an interface or trait
    fn is_interface_or_trait(&self, node_idx: NodeIndex) -> bool {
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            match node.symbol.kind {
                SymbolKind::Interface => true,
                SymbolKind::Trait => true,
                _ => {
                    // Check modifiers for interface-like characteristics
                    node.symbol.modifiers.iter().any(|m| {
                        m.contains("abstract") || m.contains("interface") || m.contains("trait")
                    })
                }
            }
        } else {
            false
        }
    }

    /// Count factory methods
    fn count_factory_methods(&self, node_idx: NodeIndex) -> u32 {
        let mut count = 0;

        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                if target_node.symbol.kind == SymbolKind::Method {
                    let method_name = target_node.symbol.name.to_lowercase();
                    if method_name.starts_with("create")
                        || method_name.starts_with("make")
                        || method_name.starts_with("build")
                        || method_name.starts_with("factory")
                    {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Classify Abstract Factory type based on analysis
    fn classify_abstract_factory_type(&self, node_idx: NodeIndex, score: u32) -> String {
        let interface_count = self.count_factory_interfaces(node_idx);
        let concrete_count = self.count_concrete_factory_implementations(node_idx);
        let method_count = self.count_factory_methods(node_idx);

        if score >= 8 && interface_count >= 2 && concrete_count >= 2 {
            "Comprehensive Abstract Factory".to_string()
        } else if score >= 6 && method_count >= 3 {
            "Multi-Product Factory".to_string()
        } else if interface_count >= 2 {
            "Interface Family Factory".to_string()
        } else if concrete_count >= 2 {
            "Concrete Factory Family".to_string()
        } else {
            "Basic Abstract Factory".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::CodeMetrics;
    use crate::parsers::LanguageId;
    use crate::symbols::Location;
    use petgraph::Graph;

    fn create_test_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind,
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
        }
    }

    #[test]
    fn test_query_engine_creation() {
        let graph = Graph::new();
        let analysis = AnalysisResult {
            graph,
            file_count: 1,
            symbol_count: 0,
            relationship_count: 0,
            languages: vec![LanguageId::Rust],
        };

        let engine = CodeQueryEngine::new(analysis);
        assert_eq!(engine.symbol_index.len(), 0);
    }

    #[test]
    fn test_find_symbols_by_kind() {
        let mut graph = Graph::new();
        let symbol = create_test_symbol("test_function", SymbolKind::Function);

        let node_data = crate::analysis::CodeNode {
            symbol,
            file_path: "test.rs".to_string(),
            metrics: CodeMetrics::default(),
        };

        graph.add_node(node_data);

        let analysis = AnalysisResult {
            graph,
            file_count: 1,
            symbol_count: 1,
            relationship_count: 0,
            languages: vec![LanguageId::Rust],
        };

        let engine = CodeQueryEngine::new(analysis);
        let result = engine.find_symbols_by_kind(SymbolKind::Function);

        assert_eq!(result.symbols.len(), 1);
        assert_eq!(result.symbols[0].symbol.name, "test_function");
    }

    #[test]
    fn test_decorator_interface_checking() {
        let mut graph = Graph::new();

        // Create nodes for decorator, wrapped component, and interface
        let decorator_idx = graph.add_node(crate::analysis::CodeNode {
            symbol: Symbol {
                name: "LoggingDecorator".to_string(),
                kind: SymbolKind::Class,
                location: Location {
                    file_path: "/test/decorator.rs".to_string(),
                    start_line: 1,
                    start_column: 0,
                    end_line: 10,
                    end_column: 1,
                },
                signature: Some("class LoggingDecorator".to_string()),
                scope_chain: vec![],
                modifiers: vec![],
                documentation: None,
                language: LanguageId::Rust,
            },
            file_path: "/test/decorator.rs".to_string(),
            metrics: CodeMetrics::default(),
        });

        let wrapped_idx = graph.add_node(crate::analysis::CodeNode {
            symbol: Symbol {
                name: "DatabaseService".to_string(),
                kind: SymbolKind::Class,
                location: Location {
                    file_path: "/test/service.rs".to_string(),
                    start_line: 1,
                    start_column: 0,
                    end_line: 10,
                    end_column: 1,
                },
                signature: Some("class DatabaseService".to_string()),
                scope_chain: vec![],
                modifiers: vec![],
                documentation: None,
                language: LanguageId::Rust,
            },
            file_path: "/test/service.rs".to_string(),
            metrics: CodeMetrics::default(),
        });

        let interface_idx = graph.add_node(crate::analysis::CodeNode {
            symbol: Symbol {
                name: "DataService".to_string(),
                kind: SymbolKind::Interface,
                location: Location {
                    file_path: "/test/interface.rs".to_string(),
                    start_line: 1,
                    start_column: 0,
                    end_line: 5,
                    end_column: 1,
                },
                signature: Some("interface DataService".to_string()),
                scope_chain: vec![],
                modifiers: vec![],
                documentation: None,
                language: LanguageId::Rust,
            },
            file_path: "/test/interface.rs".to_string(),
            metrics: CodeMetrics::default(),
        });

        // Add composition relationship (decorator wraps component)
        graph.add_edge(
            decorator_idx,
            wrapped_idx,
            crate::analysis::CodeRelationship {
                kind: crate::analysis::RelationshipKind::Composition,
                source_location: "test".to_string(),
                confidence: 1.0,
                metadata: std::collections::HashMap::new(),
            },
        );

        // Add implements relationships (both implement same interface)
        graph.add_edge(
            decorator_idx,
            interface_idx,
            crate::analysis::CodeRelationship {
                kind: crate::analysis::RelationshipKind::Implements,
                source_location: "test".to_string(),
                confidence: 1.0,
                metadata: std::collections::HashMap::new(),
            },
        );

        graph.add_edge(
            wrapped_idx,
            interface_idx,
            crate::analysis::CodeRelationship {
                kind: crate::analysis::RelationshipKind::Implements,
                source_location: "test".to_string(),
                confidence: 1.0,
                metadata: std::collections::HashMap::new(),
            },
        );

        let analysis = AnalysisResult {
            graph,
            file_count: 3,
            symbol_count: 3,
            relationship_count: 3,
            languages: vec![LanguageId::Rust],
        };

        let engine = CodeQueryEngine::new(analysis);

        // Test that the decorator implements the same interface as what it wraps
        assert!(engine.implements_wrapped_interface(decorator_idx));

        // Test that a non-decorator doesn't satisfy this condition
        assert!(!engine.implements_wrapped_interface(wrapped_idx));
        assert!(!engine.implements_wrapped_interface(interface_idx));
    }
}
