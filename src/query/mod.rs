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

/// Query interface for code analysis results
pub struct CodeQueryEngine {
    analysis: AnalysisResult,
    symbol_index: HashMap<String, NodeIndex>,
    file_index: HashMap<String, Vec<NodeIndex>>,
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
        let matching_symbols: Vec<SymbolInfo> = self.analysis.graph
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
            for edge_idx in self.analysis.graph.edges_directed(target_node, petgraph::Incoming) {
                let source_node = edge_idx.source();
                
                if let Some(symbol_info) = self.get_symbol_info(source_node) {
                    dependents.push(symbol_info);
                }

                if let Some(edge_data) = self.analysis.graph.edge_weight(edge_idx.id()) {
                    relationships.push(self.relationship_to_info(source_node, target_node, edge_data));
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
            for edge_idx in self.analysis.graph.edges_directed(source_node, petgraph::Outgoing) {
                let target_node = edge_idx.target();
                
                if let Some(symbol_info) = self.get_symbol_info(target_node) {
                    dependencies.push(symbol_info);
                }

                if let Some(edge_data) = self.analysis.graph.edge_weight(edge_idx.id()) {
                    relationships.push(self.relationship_to_info(source_node, target_node, edge_data));
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
        let complex_symbols: Vec<SymbolInfo> = self.analysis.graph
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
                let outgoing_edges = self.analysis.graph
                    .edges_directed(node_idx, petgraph::Outgoing)
                    .count();

                if outgoing_edges > 10 {
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

        // Find circular dependencies (simplified check)
        if let Some(cycles) = self.find_cycles() {
            for cycle in cycles {
                issues.push(format!("Circular dependency detected: {}", cycle.join(" -> ")));
            }
        }

        let context = ContextInfo {
            total_symbols: self.analysis.symbol_count,
            files_involved: problem_symbols.iter()
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
        let dependencies: Vec<String> = self.analysis.graph
            .edges_directed(node_idx, petgraph::Outgoing)
            .filter_map(|edge| {
                let target_node = self.analysis.graph.node_weight(edge.target())?;
                Some(target_node.symbol.qualified_name())
            })
            .collect();

        // Find dependents (incoming edges)
        let dependents: Vec<String> = self.analysis.graph
            .edges_directed(node_idx, petgraph::Incoming)
            .filter_map(|edge| {
                let source_node = self.analysis.graph.node_weight(edge.source())?;
                Some(source_node.symbol.qualified_name())
            })
            .collect();

        // Find related files through dependencies
        let mut related_files: HashSet<String> = dependencies.iter()
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
        let source_symbol = self.analysis.graph
            .node_weight(source_node)
            .map(|n| n.symbol.qualified_name())
            .unwrap_or_else(|| "unknown".to_string());

        let target_symbol = self.analysis.graph
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
        let files_involved = symbols.iter()
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
            suggestions.push(format!("Found {} symbols matching '{}'", symbols.len(), pattern));
            
            if symbols.len() > 10 {
                suggestions.push("Consider narrowing your search".to_string());
            }

            // Suggest related symbols
            let related: HashSet<String> = symbols.iter()
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
    fn generate_dependency_suggestions(&self, symbol_name: &str, is_dependents: bool) -> Vec<String> {
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

    /// Detect architectural patterns in the codebase
    fn detect_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Simple pattern detection based on symbol names and relationships
        let symbol_names: Vec<&str> = self.symbol_index.keys()
            .map(|s| s.as_str())
            .collect();

        // Factory pattern detection
        if symbol_names.iter().any(|name| name.contains("Factory")) {
            patterns.push("Factory Pattern".to_string());
        }

        // Singleton pattern detection
        if symbol_names.iter().any(|name| name.contains("Singleton")) {
            patterns.push("Singleton Pattern".to_string());
        }

        // Builder pattern detection
        if symbol_names.iter().any(|name| name.contains("Builder")) {
            patterns.push("Builder Pattern".to_string());
        }

        patterns
    }

    /// Detect potential issues in symbols
    fn detect_issues(&self, symbols: &[SymbolInfo]) -> Vec<String> {
        let mut issues = Vec::new();

        for symbol in symbols {
            if symbol.complexity > 20 {
                issues.push(format!("High complexity in '{}'", symbol.symbol.name));
            }
            
            if symbol.dependencies.len() > 15 {
                issues.push(format!("High coupling in '{}'", symbol.symbol.name));
            }
        }

        issues
    }

    /// Find circular dependencies using strongly connected components
    fn find_cycles(&self) -> Option<Vec<Vec<String>>> {
        use petgraph::algo::kosaraju_scc;
        
        // Get strongly connected components from the dependency graph
        let dependency_graph = &self.analysis.graph;
        let sccs = kosaraju_scc(dependency_graph);
        
        // Filter out single-node SCCs (not cycles) and collect cycles with more than one node
        let cycles: Vec<Vec<String>> = sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                scc.into_iter()
                    .map(|node_idx| {
                        dependency_graph.node_weight(node_idx)
                            .map(|node| node.symbol.name.clone())
                            .unwrap_or_else(|| format!("node_{}", node_idx.index()))
                    })
                    .collect()
            })
            .collect();
        
        if cycles.is_empty() {
            None
        } else {
            Some(cycles)
        }
    }

    /// Calculate overall complexity score for the codebase
    fn calculate_overall_complexity(&self) -> f32 {
        let total_complexity: u32 = self.analysis.graph
            .node_weights()
            .map(|node| node.metrics.cyclomatic_complexity)
            .sum();

        total_complexity as f32 / self.analysis.symbol_count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::CodeMetrics;
    use crate::symbols::Location;
    use crate::parsers::LanguageId;
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
}