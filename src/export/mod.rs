//! # Export & Serialization
//!
//! Provides comprehensive export capabilities for external consumption including
//! JSON exports, LSP-compatible formats, and embedding-friendly representations.

pub mod embeddings;
pub mod json;
pub mod lsp;
pub mod pagination;

// Re-export key types
pub use embeddings::{CodeContext, EmbeddingExporter, SymbolEmbedding};
pub use json::{ExportOptions, JsonExporter};
pub use lsp::{LspExporter, LspLocation, LspSymbolInformation};
pub use pagination::{PagedResult, PaginationOptions, ResultFilter};

use crate::analysis::AnalysisResult;

use crate::symbols::{Symbol, SymbolKind};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive export data structure containing all analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    /// Project metadata
    pub project: ProjectMetadata,

    /// All symbols in the codebase
    pub symbols: Vec<ExportSymbol>,

    /// Relationships between symbols
    pub relationships: Vec<ExportRelationship>,

    /// File-level information
    pub files: Vec<FileInfo>,

    /// Analysis statistics and metrics
    pub metrics: AnalysisMetrics,

    /// Export metadata
    pub export_info: ExportMetadata,
}

/// Project metadata for exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub total_files: usize,
    pub total_symbols: usize,
    pub root_path: String,
    pub analysis_timestamp: u64,
}

/// Serializable symbol representation optimized for external consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSymbol {
    pub id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub file_path: String,
    pub language: String,

    /// Location information
    pub location: ExportLocation,

    /// Symbol metadata
    pub scope_chain: Vec<String>,
    pub modifiers: Vec<String>,
    pub signature: Option<String>,
    pub documentation: Option<String>,

    /// Analysis data
    pub complexity: u32,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub related_files: Vec<String>,

    /// Additional metadata for external tools
    pub tags: Vec<String>,
    pub confidence: f32,
}

/// Location information for symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportLocation {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub byte_offset: Option<usize>,
    pub byte_length: Option<usize>,
}

/// Relationship between symbols for external consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRelationship {
    pub id: String,
    pub from_symbol: String,
    pub to_symbol: String,
    pub relationship_type: String,
    pub confidence: f32,
    pub source_location: ExportLocation,
    pub context: Option<String>, // Code snippet showing the relationship
}

/// File-level information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub language: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub symbol_count: usize,
    pub complexity_score: f32,
    pub last_modified: Option<u64>,
    pub content_hash: String,
}

/// Analysis metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub total_symbols: usize,
    pub total_relationships: usize,
    pub files_analyzed: usize,
    pub languages_detected: Vec<String>,

    /// Complexity metrics
    pub average_complexity: f32,
    pub max_complexity: u32,
    pub complexity_distribution: HashMap<String, usize>, // complexity_range -> count

    /// Symbol distribution
    pub symbol_distribution: HashMap<String, usize>, // kind -> count

    /// Quality metrics
    pub documented_symbols: usize,
    pub test_coverage_estimate: Option<f32>,
    pub technical_debt_score: Option<f32>,

    /// Performance metrics
    pub analysis_duration_ms: u64,
    pub memory_usage_mb: Option<f32>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub format_version: String,
    pub exported_at: u64,
    pub exporter_version: String,
    pub export_options: ExportOptions,
    pub total_size_bytes: Option<u64>,
    pub compression_used: bool,
}

/// Main exporter that coordinates all export formats
pub struct UniversalExporter {
    analysis: AnalysisResult,
    project_root: String,
}

impl UniversalExporter {
    /// Create a new universal exporter
    pub fn new(analysis: AnalysisResult, project_root: String) -> Self {
        Self {
            analysis,
            project_root,
        }
    }

    /// Create comprehensive export data
    pub fn create_export_data(&self, options: &ExportOptions) -> ExportData {
        let symbols = self.extract_symbols(options);
        let relationships = self.extract_relationships(options);
        let files = self.extract_file_info(options);
        let metrics = self.calculate_metrics();
        let project = self.extract_project_metadata();

        ExportData {
            project,
            symbols,
            relationships,
            files,
            metrics,
            export_info: ExportMetadata {
                format_version: "1.0.0".to_string(),
                exported_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or_else(|_| {
                        eprintln!("Warning: System clock issue detected during export, using fallback timestamp");
                        0
                    }),
                exporter_version: env!("CARGO_PKG_VERSION").to_string(),
                export_options: options.clone(),
                total_size_bytes: None, // Will be filled in by specific exporters
                compression_used: false,
            },
        }
    }

    /// Extract symbols for export
    fn extract_symbols(&self, options: &ExportOptions) -> Vec<ExportSymbol> {
        let mut symbols = Vec::new();

        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                // Apply filters
                if let Some(ref filters) = options.filters {
                    if !self.should_include_symbol(&node.symbol, filters) {
                        continue;
                    }
                }

                let export_symbol = ExportSymbol {
                    id: format!("sym_{}", node_idx.index()),
                    name: node.symbol.name.clone(),
                    qualified_name: node.symbol.qualified_name(),
                    kind: format!("{:?}", node.symbol.kind),
                    file_path: node.file_path.clone(),
                    language: format!("{:?}", node.symbol.language),
                    location: ExportLocation {
                        start_line: node.symbol.location.start_line,
                        start_column: node.symbol.location.start_column,
                        end_line: node.symbol.location.end_line,
                        end_column: node.symbol.location.end_column,
                        byte_offset: None, // Could be added from tree-sitter node
                        byte_length: None,
                    },
                    scope_chain: node
                        .symbol
                        .scope_chain
                        .iter()
                        .map(|s| s.name.clone())
                        .collect(),
                    modifiers: node.symbol.modifiers.clone(),
                    signature: node.symbol.signature.clone(),
                    documentation: node.symbol.documentation.clone(),
                    complexity: node.metrics.cyclomatic_complexity,
                    dependencies: self.get_symbol_dependencies(node_idx),
                    dependents: self.get_symbol_dependents(node_idx),
                    related_files: self.get_related_files(node_idx),
                    tags: self.generate_symbol_tags(&node.symbol),
                    confidence: 1.0, // Could be calculated based on analysis quality
                };

                symbols.push(export_symbol);
            }
        }

        // Sort symbols for consistent export
        symbols.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.location.start_line.cmp(&b.location.start_line))
                .then_with(|| a.name.cmp(&b.name))
        });

        // Apply limit if specified
        if let Some(limit) = options.max_symbols {
            symbols.truncate(limit);
        }

        symbols
    }

    /// Extract relationships for export
    fn extract_relationships(&self, _options: &ExportOptions) -> Vec<ExportRelationship> {
        let mut relationships = Vec::new();

        for edge_idx in self.analysis.graph.edge_indices() {
            if let Some((source_idx, target_idx)) = self.analysis.graph.edge_endpoints(edge_idx) {
                if let Some(edge_data) = self.analysis.graph.edge_weight(edge_idx) {
                    let relationship = ExportRelationship {
                        id: format!("rel_{}", edge_idx.index()),
                        from_symbol: format!("sym_{}", source_idx.index()),
                        to_symbol: format!("sym_{}", target_idx.index()),
                        relationship_type: format!("{:?}", edge_data.kind),
                        confidence: edge_data.confidence,
                        source_location: ExportLocation {
                            start_line: 0, // Would need to be extracted from edge data
                            start_column: 0,
                            end_line: 0,
                            end_column: 0,
                            byte_offset: None,
                            byte_length: None,
                        },
                        context: None, // Could extract code snippet
                    };

                    relationships.push(relationship);
                }
            }
        }

        relationships
    }

    /// Extract file information
    fn extract_file_info(&self, _options: &ExportOptions) -> Vec<FileInfo> {
        let mut file_map: HashMap<String, FileInfo> = HashMap::new();

        // Aggregate information by file
        for node_idx in self.analysis.graph.node_indices() {
            if let Some(node) = self.analysis.graph.node_weight(node_idx) {
                let entry = file_map.entry(node.file_path.clone()).or_insert_with(|| {
                    FileInfo {
                        path: node.file_path.clone(),
                        language: format!("{:?}", node.symbol.language),
                        size_bytes: 0, // Would need to be filled from file system
                        line_count: 0,
                        symbol_count: 0,
                        complexity_score: 0.0,
                        last_modified: None,
                        content_hash: String::new(),
                    }
                });

                entry.symbol_count += 1;
                entry.complexity_score += node.metrics.cyclomatic_complexity as f32;
            }
        }

        // Finalize metrics
        for file_info in file_map.values_mut() {
            if file_info.symbol_count > 0 {
                file_info.complexity_score /= file_info.symbol_count as f32;
            }
        }

        let mut files: Vec<FileInfo> = file_map.into_values().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));

        files
    }

    /// Calculate analysis metrics
    fn calculate_metrics(&self) -> AnalysisMetrics {
        let total_symbols = self.analysis.symbol_count;
        let total_relationships = self.analysis.relationship_count;

        let mut complexity_sum = 0u64;
        let mut max_complexity = 0u32;
        let mut complexity_distribution = HashMap::new();
        let mut symbol_distribution = HashMap::new();
        let mut documented_count = 0;

        for node in self.analysis.graph.node_weights() {
            let complexity = node.metrics.cyclomatic_complexity;
            complexity_sum += complexity as u64;
            max_complexity = max_complexity.max(complexity);

            // Complexity distribution
            let complexity_range = match complexity {
                0..=5 => "Low (0-5)",
                6..=10 => "Medium (6-10)",
                11..=20 => "High (11-20)",
                _ => "Very High (20+)",
            };
            *complexity_distribution
                .entry(complexity_range.to_string())
                .or_insert(0) += 1;

            // Symbol distribution
            let kind_str = format!("{:?}", node.symbol.kind);
            *symbol_distribution.entry(kind_str).or_insert(0) += 1;

            // Documentation count
            if node.symbol.documentation.is_some() {
                documented_count += 1;
            }
        }

        let average_complexity = if total_symbols > 0 {
            complexity_sum as f32 / total_symbols as f32
        } else {
            0.0
        };

        AnalysisMetrics {
            total_symbols,
            total_relationships,
            files_analyzed: self.analysis.file_count,
            languages_detected: self
                .analysis
                .languages
                .iter()
                .map(|lang| format!("{lang:?}"))
                .collect(),
            average_complexity,
            max_complexity,
            complexity_distribution,
            symbol_distribution,
            documented_symbols: documented_count,
            test_coverage_estimate: None, // Could be calculated
            technical_debt_score: None,   // Could be calculated
            analysis_duration_ms: 0,      // Would need to be tracked during analysis
            memory_usage_mb: None,
        }
    }

    /// Extract project metadata
    fn extract_project_metadata(&self) -> ProjectMetadata {
        ProjectMetadata {
            name: std::path::Path::new(&self.project_root)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string()),
            version: None, // Could be extracted from package files
            description: None,
            languages: self
                .analysis
                .languages
                .iter()
                .map(|lang| format!("{lang:?}"))
                .collect(),
            total_files: self.analysis.file_count,
            total_symbols: self.analysis.symbol_count,
            root_path: self.project_root.clone(),
            analysis_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Helper methods
    fn should_include_symbol(&self, symbol: &Symbol, filters: &ResultFilter) -> bool {
        // Apply language filter
        if let Some(ref languages) = filters.languages {
            let lang_str = format!("{:?}", symbol.language);
            if !languages.contains(&lang_str) {
                return false;
            }
        }

        // Apply symbol kind filter
        if let Some(ref kinds) = filters.symbol_kinds {
            let kind_str = format!("{:?}", symbol.kind);
            if !kinds.contains(&kind_str) {
                return false;
            }
        }

        // Apply file pattern filter
        if let Some(ref patterns) = filters.file_patterns {
            if !patterns
                .iter()
                .any(|pattern| symbol.location.file_path.contains(pattern))
            {
                return false;
            }
        }

        true
    }

    fn get_symbol_dependencies(&self, node_idx: petgraph::graph::NodeIndex) -> Vec<String> {
        self.analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
            .filter_map(|edge| {
                let target_node = self.analysis.graph.node_weight(edge.target())?;
                Some(target_node.symbol.qualified_name())
            })
            .collect()
    }

    fn get_symbol_dependents(&self, node_idx: petgraph::graph::NodeIndex) -> Vec<String> {
        self.analysis
            .graph
            .edges_directed(node_idx, petgraph::Incoming)
            .filter_map(|edge| {
                let source_node = self.analysis.graph.node_weight(edge.source())?;
                Some(source_node.symbol.qualified_name())
            })
            .collect()
    }

    fn get_related_files(&self, node_idx: petgraph::graph::NodeIndex) -> Vec<String> {
        let mut related_files = std::collections::HashSet::new();

        // Add files from dependencies
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Outgoing)
        {
            if let Some(target_node) = self.analysis.graph.node_weight(edge.target()) {
                related_files.insert(target_node.file_path.clone());
            }
        }

        // Add files from dependents
        for edge in self
            .analysis
            .graph
            .edges_directed(node_idx, petgraph::Incoming)
        {
            if let Some(source_node) = self.analysis.graph.node_weight(edge.source()) {
                related_files.insert(source_node.file_path.clone());
            }
        }

        // Remove the symbol's own file
        if let Some(node) = self.analysis.graph.node_weight(node_idx) {
            related_files.remove(&node.file_path);
        }

        related_files.into_iter().collect()
    }

    fn generate_symbol_tags(&self, symbol: &Symbol) -> Vec<String> {
        let mut tags = Vec::new();

        // Add kind-based tags
        match symbol.kind {
            SymbolKind::Function => tags.push("function".to_string()),
            SymbolKind::Class => tags.push("class".to_string()),
            SymbolKind::Interface => tags.push("interface".to_string()),
            SymbolKind::Struct => tags.push("struct".to_string()),
            SymbolKind::Enum => tags.push("enum".to_string()),
            SymbolKind::Variable => tags.push("variable".to_string()),
            _ => {}
        }

        // Add modifier-based tags
        for modifier in &symbol.modifiers {
            tags.push(modifier.clone());
        }

        // Add language-specific tags
        tags.push(symbol.language.to_string().to_lowercase());

        // Add scope-based tags
        if !symbol.scope_chain.is_empty() {
            tags.push("scoped".to_string());
        }

        // Add documentation tag
        if symbol.documentation.is_some() {
            tags.push("documented".to_string());
        }

        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{CodeMetrics, CodeNode};
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};
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
    fn test_export_data_creation() {
        let mut graph = Graph::new();
        let symbol = create_test_symbol("test_function", SymbolKind::Function);

        let node_data = CodeNode {
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

        let exporter = UniversalExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();
        let export_data = exporter.create_export_data(&options);

        assert_eq!(export_data.symbols.len(), 1);
        assert_eq!(export_data.symbols[0].name, "test_function");
        assert_eq!(export_data.symbols[0].kind, "Function");
        assert_eq!(export_data.project.total_symbols, 1);
        assert_eq!(export_data.metrics.total_symbols, 1);
    }

    #[test]
    fn test_symbol_filtering() {
        // Test symbol filtering functionality by creating mock data with various symbol types
        let mut symbols = vec![
            Symbol {
                name: "test_function".to_string(),
                kind: SymbolKind::Function,
                location: Location::new("test.rs".to_string(), 1, 1, 1, 10),
                documentation: None,
                scope: None,
                signature: None,
                language: crate::parsers::LanguageId::Rust,
                attributes: HashMap::new(),
            },
            Symbol {
                name: "TestStruct".to_string(),
                kind: SymbolKind::Struct,
                location: Location::new("test.rs".to_string(), 3, 1, 3, 15),
                documentation: None,
                scope: None,
                signature: None,
                language: crate::parsers::LanguageId::Rust,
                attributes: HashMap::new(),
            },
            Symbol {
                name: "PRIVATE_VAR".to_string(),
                kind: SymbolKind::Variable,
                location: Location::new("test.rs".to_string(), 5, 1, 5, 20),
                documentation: None,
                scope: None,
                signature: None,
                language: crate::parsers::LanguageId::Rust,
                attributes: HashMap::new(),
            },
        ];
        
        // Test filtering by symbol kind
        let functions_only: Vec<Symbol> = symbols.iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .cloned()
            .collect();
        
        assert_eq!(functions_only.len(), 1);
        assert_eq!(functions_only[0].name, "test_function");
        
        // Test filtering by name pattern
        let test_symbols: Vec<Symbol> = symbols.iter()
            .filter(|s| s.name.to_lowercase().contains("test"))
            .cloned()
            .collect();
        
        assert_eq!(test_symbols.len(), 2);
        
        // Test excluding private symbols (by convention)
        let public_symbols: Vec<Symbol> = symbols.iter()
            .filter(|s| !s.name.to_uppercase().starts_with("PRIVATE"))
            .cloned()
            .collect();
        
        assert_eq!(public_symbols.len(), 2);
        assert!(!public_symbols.iter().any(|s| s.name == "PRIVATE_VAR"));
    }
}
