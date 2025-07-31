//! # JSON Export
//! 
//! High-performance JSON export with streaming support, compression options,
//! and web-friendly formatting for external consumption.

use super::{ExportData, UniversalExporter, ExportMetadata};
use crate::analysis::AnalysisResult;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// JSON export configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Pretty print JSON (vs compact)
    pub pretty_print: bool,
    
    /// Include detailed symbol information
    pub include_symbol_details: bool,
    
    /// Include relationship information
    pub include_relationships: bool,
    
    /// Include file-level metrics
    pub include_file_metrics: bool,
    
    /// Include analysis metrics
    pub include_analysis_metrics: bool,
    
    /// Maximum number of symbols to export (None = all)
    pub max_symbols: Option<usize>,
    
    /// Maximum number of relationships to export (None = all)
    pub max_relationships: Option<usize>,
    
    /// Filters to apply during export
    pub filters: Option<super::ResultFilter>,
    
    /// Compression settings
    pub compression: CompressionOptions,
    
    /// Streaming options for large exports
    pub streaming: StreamingOptions,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            pretty_print: true,
            include_symbol_details: true,
            include_relationships: true,
            include_file_metrics: true,
            include_analysis_metrics: true,
            max_symbols: None,
            max_relationships: None,
            filters: None,
            compression: CompressionOptions::default(),
            streaming: StreamingOptions::default(),
        }
    }
}

/// Compression options for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOptions {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level (1-9, algorithm dependent)
    pub level: u8,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: CompressionAlgorithm::Gzip,
            level: 6,
        }
    }
}

/// Available compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Zstd,
    Brotli,
}

/// Streaming options for large datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingOptions {
    /// Enable streaming mode
    pub enabled: bool,
    
    /// Chunk size for streaming (number of symbols per chunk)
    pub chunk_size: usize,
    
    /// Include progress information
    pub include_progress: bool,
}

impl Default for StreamingOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            chunk_size: 1000,
            include_progress: false,
        }
    }
}

/// Web-optimized export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebExport {
    /// Basic project information
    pub project: super::ProjectMetadata,
    
    /// Summary statistics
    pub summary: ExportSummary,
    
    /// Symbol data (potentially chunked)
    pub symbols: Vec<WebSymbol>,
    
    /// Relationships (potentially chunked)
    pub relationships: Vec<WebRelationship>,
    
    /// File index for quick lookups
    pub file_index: Vec<WebFileInfo>,
    
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Summary information for web interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    pub total_symbols: usize,
    pub total_relationships: usize,
    pub total_files: usize,
    pub languages: Vec<String>,
    pub complexity_stats: ComplexityStats,
    pub top_files: Vec<TopFileInfo>,
    pub top_symbols: Vec<TopSymbolInfo>,
}

/// Complexity statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityStats {
    pub average: f32,
    pub maximum: u32,
    pub distribution: Vec<ComplexityBucket>,
}

/// Complexity distribution bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBucket {
    pub range: String,
    pub count: usize,
    pub percentage: f32,
}

/// Top file information for summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopFileInfo {
    pub path: String,
    pub symbol_count: usize,
    pub complexity_score: f32,
    pub language: String,
}

/// Top symbol information for summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSymbolInfo {
    pub name: String,
    pub qualified_name: String,
    pub complexity: u32,
    pub file_path: String,
    pub kind: String,
}

/// Web-optimized symbol representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSymbol {
    pub id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub file_path: String,
    pub language: String,
    pub location: super::ExportLocation,
    pub complexity: u32,
    pub dependency_count: usize,
    pub dependent_count: usize,
    pub tags: Vec<String>,
    
    // Optional detailed information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scope_chain: Vec<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<String>,
}

/// Web-optimized relationship representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRelationship {
    pub from_symbol: String,
    pub to_symbol: String,
    pub relationship_type: String,
    pub confidence: f32,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Web-optimized file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFileInfo {
    pub path: String,
    pub language: String,
    pub symbol_count: usize,
    pub complexity_score: f32,
    pub size_bytes: u64,
    pub last_modified: Option<u64>,
}

/// High-performance JSON exporter
pub struct JsonExporter {
    exporter: UniversalExporter,
}

impl JsonExporter {
    /// Create a new JSON exporter
    pub fn new(analysis: AnalysisResult, project_root: String) -> Self {
        Self {
            exporter: UniversalExporter::new(analysis, project_root),
        }
    }
    
    /// Export to JSON string
    pub fn export_to_string(&self, options: &ExportOptions) -> Result<String, JsonExportError> {
        let export_data = self.exporter.create_export_data(options);
        
        let json = if options.pretty_print {
            serde_json::to_string_pretty(&export_data)?
        } else {
            serde_json::to_string(&export_data)?
        };
        
        Ok(json)
    }
    
    /// Export to file
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P, options: &ExportOptions) -> Result<(), JsonExportError> {
        let export_data = self.exporter.create_export_data(options);
        
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        if options.pretty_print {
            serde_json::to_writer_pretty(writer, &export_data)?;
        } else {
            serde_json::to_writer(writer, &export_data)?;
        }
        
        Ok(())
    }
    
    /// Export web-optimized format
    pub fn export_web_format(&self, options: &ExportOptions) -> Result<WebExport, JsonExportError> {
        let export_data = self.exporter.create_export_data(options);
        let web_export = self.convert_to_web_format(export_data)?;
        Ok(web_export)
    }
    
    /// Export web format to string
    pub fn export_web_to_string(&self, options: &ExportOptions) -> Result<String, JsonExportError> {
        let web_export = self.export_web_format(options)?;
        
        let json = if options.pretty_print {
            serde_json::to_string_pretty(&web_export)?
        } else {
            serde_json::to_string(&web_export)?
        };
        
        Ok(json)
    }
    
    /// Export web format to file
    pub fn export_web_to_file<P: AsRef<Path>>(&self, path: P, options: &ExportOptions) -> Result<(), JsonExportError> {
        let web_export = self.export_web_format(options)?;
        
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        if options.pretty_print {
            serde_json::to_writer_pretty(writer, &web_export)?;
        } else {
            serde_json::to_writer(writer, &web_export)?;
        }
        
        Ok(())
    }
    
    /// Stream export for large datasets
    pub fn stream_export<W: Write>(&self, writer: W, options: &ExportOptions) -> Result<(), JsonExportError> {
        if !options.streaming.enabled {
            return Err(JsonExportError::StreamingNotEnabled);
        }
        
        let mut buf_writer = BufWriter::new(writer);
        let export_data = self.exporter.create_export_data(options);
        
        // Write opening
        writeln!(buf_writer, "{{")?;
        writeln!(buf_writer, "  \"project\": {},", serde_json::to_string(&export_data.project)?)?;
        writeln!(buf_writer, "  \"metrics\": {},", serde_json::to_string(&export_data.metrics)?)?;
        writeln!(buf_writer, "  \"symbols\": [")?;
        
        // Stream symbols in chunks
        let chunk_size = options.streaming.chunk_size;
        let total_symbols = export_data.symbols.len();
        
        for (i, chunk) in export_data.symbols.chunks(chunk_size).enumerate() {
            for (j, symbol) in chunk.iter().enumerate() {
                let json = serde_json::to_string(symbol)?;
                write!(buf_writer, "    {json}")?;
                
                // Add comma if not the last symbol
                if i * chunk_size + j < total_symbols - 1 {
                    writeln!(buf_writer, ",")?;
                } else {
                    writeln!(buf_writer)?;
                }
            }
            
            // Progress reporting
            if options.streaming.include_progress {
                // Progress tracking - could be sent to callback instead of printing
                let _progress = ((i + 1) * chunk_size).min(total_symbols) as f32 / total_symbols as f32 * 100.0;
            }
        }
        
        writeln!(buf_writer, "  ],")?;
        
        // Stream relationships
        writeln!(buf_writer, "  \"relationships\": [")?;
        let total_relationships = export_data.relationships.len();
        
        for (i, chunk) in export_data.relationships.chunks(chunk_size).enumerate() {
            for (j, relationship) in chunk.iter().enumerate() {
                let json = serde_json::to_string(relationship)?;
                write!(buf_writer, "    {json}")?;
                
                if i * chunk_size + j < total_relationships - 1 {
                    writeln!(buf_writer, ",")?;
                } else {
                    writeln!(buf_writer)?;
                }
            }
        }
        
        writeln!(buf_writer, "  ],")?;
        writeln!(buf_writer, "  \"files\": {},", serde_json::to_string(&export_data.files)?)?;
        writeln!(buf_writer, "  \"export_info\": {}", serde_json::to_string(&export_data.export_info)?)?;
        writeln!(buf_writer, "}}")?;
        
        buf_writer.flush()?;
        Ok(())
    }
    
    /// Convert full export data to web-optimized format
    fn convert_to_web_format(&self, export_data: ExportData) -> Result<WebExport, JsonExportError> {
        let symbols: Vec<WebSymbol> = export_data.symbols.into_iter()
            .map(|s| WebSymbol {
                id: s.id,
                name: s.name,
                qualified_name: s.qualified_name,
                kind: s.kind,
                file_path: s.file_path,
                language: s.language,
                location: s.location,
                complexity: s.complexity,
                dependency_count: s.dependencies.len(),
                dependent_count: s.dependents.len(),
                tags: s.tags,
                signature: s.signature,
                documentation: s.documentation,
                scope_chain: s.scope_chain,
                modifiers: s.modifiers,
            })
            .collect();
        
        let relationships: Vec<WebRelationship> = export_data.relationships.into_iter()
            .map(|r| WebRelationship {
                from_symbol: r.from_symbol,
                to_symbol: r.to_symbol,
                relationship_type: r.relationship_type,
                confidence: r.confidence,
                context: r.context,
            })
            .collect();
        
        let file_index: Vec<WebFileInfo> = export_data.files.into_iter()
            .map(|f| WebFileInfo {
                path: f.path,
                language: f.language,
                symbol_count: f.symbol_count,
                complexity_score: f.complexity_score,
                size_bytes: f.size_bytes,
                last_modified: f.last_modified,
            })
            .collect();
        
        let summary = self.create_export_summary(&symbols, &relationships, &file_index, &export_data.metrics);
        
        Ok(WebExport {
            project: export_data.project,
            summary,
            symbols,
            relationships,
            file_index,
            metadata: export_data.export_info,
        })
    }
    
    /// Create export summary for web interfaces
    fn create_export_summary(
        &self,
        symbols: &[WebSymbol],
        relationships: &[WebRelationship],
        files: &[WebFileInfo],
        metrics: &super::AnalysisMetrics,
    ) -> ExportSummary {
        // Create complexity distribution
        let total_symbols = symbols.len();
        let complexity_buckets: Vec<ComplexityBucket> = metrics.complexity_distribution.iter()
            .map(|(range, count)| ComplexityBucket {
                range: range.clone(),
                count: *count,
                percentage: if total_symbols > 0 {
                    (*count as f32 / total_symbols as f32) * 100.0
                } else {
                    0.0
                },
            })
            .collect();
        
        let complexity_stats = ComplexityStats {
            average: metrics.average_complexity,
            maximum: metrics.max_complexity,
            distribution: complexity_buckets,
        };
        
        // Top files by symbol count
        let mut top_files: Vec<TopFileInfo> = files.iter()
            .map(|f| TopFileInfo {
                path: f.path.clone(),
                symbol_count: f.symbol_count,
                complexity_score: f.complexity_score,
                language: f.language.clone(),
            })
            .collect();
        top_files.sort_by(|a, b| b.symbol_count.cmp(&a.symbol_count));
        top_files.truncate(10);
        
        // Top symbols by complexity
        let mut top_symbols: Vec<TopSymbolInfo> = symbols.iter()
            .map(|s| TopSymbolInfo {
                name: s.name.clone(),
                qualified_name: s.qualified_name.clone(),
                complexity: s.complexity,
                file_path: s.file_path.clone(),
                kind: s.kind.clone(),
            })
            .collect();
        top_symbols.sort_by(|a, b| b.complexity.cmp(&a.complexity));
        top_symbols.truncate(10);
        
        ExportSummary {
            total_symbols: symbols.len(),
            total_relationships: relationships.len(),
            total_files: files.len(),
            languages: metrics.languages_detected.clone(),
            complexity_stats,
            top_files,
            top_symbols,
        }
    }
}

/// Errors that can occur during JSON export
#[derive(Debug, thiserror::Error)]
pub enum JsonExportError {
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Streaming is not enabled in export options")]
    StreamingNotEnabled,
    
    #[error("Export data too large for non-streaming export")]
    DataTooLarge,
    
    #[error("Compression error: {0}")]
    CompressionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::{Symbol, SymbolKind, Location};
    use crate::parsers::LanguageId;
    use crate::analysis::{CodeNode, CodeMetrics, AnalysisResult};
    use petgraph::Graph;
    use tempfile::NamedTempFile;

    fn create_test_analysis() -> AnalysisResult {
        let mut graph = Graph::new();
        
        let symbol = Symbol {
            name: "test_function".to_string(),
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
            documentation: Some("Test function documentation".to_string()),
            modifiers: vec!["pub".to_string()],
            signature: Some("fn test_function() -> i32".to_string()),
        };
        
        let node_data = CodeNode {
            symbol,
            file_path: "test.rs".to_string(),
            metrics: CodeMetrics {
                cyclomatic_complexity: 5,
                ..Default::default()
            },
        };
        
        graph.add_node(node_data);

        AnalysisResult {
            graph,
            file_count: 1,
            symbol_count: 1,
            relationship_count: 0,
            languages: vec![LanguageId::Rust],
        }
    }

    #[test]
    fn test_json_export() {
        let analysis = create_test_analysis();
        let exporter = JsonExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();
        
        let json = exporter.export_to_string(&options).unwrap();
        assert!(json.contains("test_function"));
        assert!(json.contains("Function"));
        assert!(json.contains("Test function documentation"));
    }

    #[test]
    fn test_web_format_export() {
        let analysis = create_test_analysis();
        let exporter = JsonExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();
        
        let web_export = exporter.export_web_format(&options).unwrap();
        
        assert_eq!(web_export.symbols.len(), 1);
        assert_eq!(web_export.symbols[0].name, "test_function");
        assert_eq!(web_export.summary.total_symbols, 1);
        assert!(web_export.summary.top_symbols.len() <= 1);
    }

    #[test]
    fn test_file_export() {
        let analysis = create_test_analysis();
        let exporter = JsonExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();
        
        let temp_file = NamedTempFile::new().unwrap();
        exporter.export_to_file(temp_file.path(), &options).unwrap();
        
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("test_function"));
    }

    #[test]
    fn test_streaming_export() {
        let analysis = create_test_analysis();
        let exporter = JsonExporter::new(analysis, "/test/project".to_string());
        
        let mut options = ExportOptions::default();
        options.streaming.enabled = true;
        options.streaming.chunk_size = 1;
        
        let mut buffer = Vec::new();
        exporter.stream_export(&mut buffer, &options).unwrap();
        
        let json_str = String::from_utf8(buffer).unwrap();
        assert!(json_str.contains("test_function"));
        assert!(json_str.contains("\"symbols\":"));
    }
}