//! # FastContextAnalyzer Module
//!
//! This module contains the main FastContextAnalyzer implementation
//! extracted from the monolithic lib.rs for better organization.

use crate::analysis::{AnalysisResult, CodeGraph, CodeGraphBuilder};
use crate::cache::AdaptiveCacheManager;
use crate::error_tracking::ErrorTracker;
use crate::export::{ExportOptions, JsonExporter, LspExporter, EmbeddingExporter};
use crate::parsers::{LanguageId, ParserFactory};
use crate::query::{CodeQueryEngine, QueryResult, SymbolInfo, RelationshipInfo, ContextInfo};
use crate::symbols::{SymbolExtractorFactory, SymbolKind};
use crate::watcher::CodebaseWatcher;
use crate::domains;

use napi_derive::napi;
use ts_rs::TS;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Configuration options for Fast-Context analyzer
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct AnalyzerConfig {
    /// Project root directory path
    pub project_root: String,

    /// Languages to analyze (empty = auto-detect all)
    pub languages: Option<Vec<String>>,

    /// File patterns to ignore
    pub ignore_patterns: Option<Vec<String>>,

    /// Enable intelligent caching
    pub enable_caching: Option<bool>,

    /// Cache policy (auto, minimal, balanced, adaptive, persistent)
    pub cache_policy: Option<String>,

    /// Enable file watching for real-time updates
    pub enable_watching: Option<bool>,

    /// Maximum files to analyze (0 = no limit)
    pub max_files: Option<u32>,

    /// Enable parallel processing
    pub parallel_processing: Option<bool>,
    
    /// Enable experimental harmonious architecture (default: false for compatibility)
    pub enable_experimental_architecture: Option<bool>,
}

/// Architectural mode for the analyzer
#[derive(Debug, Clone)]
pub enum ArchitecturalMode {
    /// Legacy monolithic mode (backward compatibility)
    Legacy,
    /// New harmonious domain-based architecture
    Harmonious,
    /// Hybrid mode (gradual migration)
    Hybrid,
}

/// Fast-Context codebase analyzer for Node.js
/// 
/// ARCHITECTURAL REMEDIATION: This analyzer now uses harmonious domain architecture
/// internally while maintaining 100% backward compatibility.
#[napi]
pub struct FastContextAnalyzer {
    // Core runtime and state (preserved for compatibility)
    runtime: Runtime,
    project_root: String,
    analysis: Option<AnalysisResult>,
    query_engine: Option<CodeQueryEngine>,
    cache_manager: Option<Arc<AdaptiveCacheManager<String>>>,
    watcher: Option<CodebaseWatcher>,
    file_query_cache: HashMap<String, (QueryResult, std::time::Instant)>,
    cache_access_order: VecDeque<String>,
    
    // NEW: Harmonious domain architecture (internal)
    domain_metrics: Arc<domains::core::Metrics>,
    architectural_mode: ArchitecturalMode,
}

/// TypeScript type definition for FastContextAnalyzer
#[derive(TS)]
#[ts(export)]
pub struct FastContextAnalyzerType {}

/// Analysis result for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct AnalysisResultJs {
    /// Total number of files analyzed
    pub file_count: u32,

    /// Total number of symbols found
    pub symbol_count: u32,

    /// Total number of relationships found
    pub relationship_count: u32,

    /// Languages detected in the project
    pub languages: Vec<String>,
}

/// Query result for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct QueryResultJs {
    /// Matching symbols
    pub symbols: Vec<SymbolInfoJs>,

    /// Related relationships
    pub relationships: Vec<RelationshipInfoJs>,

    /// Context information
    pub context: ContextInfoJs,

    /// AI assistant suggestions
    pub suggestions: Vec<String>,
}

/// Symbol information for JavaScript
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct SymbolInfoJs {
    /// Symbol name
    pub name: String,

    /// Symbol kind (function, class, variable, etc.)
    pub kind: String,

    /// File path where symbol is defined
    pub file_path: String,

    /// Line number in the file
    pub line: u32,

    /// Column number in the file
    pub column: u32,

    /// Symbol documentation/comments
    pub documentation: Option<String>,

    /// Symbol signature (for functions/methods)
    pub signature: Option<String>,

    /// Symbol scope (global, local, etc.)
    pub scope: String,

    /// Language of the file
    pub language: String,
}

/// Relationship information for JavaScript
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct RelationshipInfoJs {
    /// Source symbol name
    pub from_symbol: String,

    /// Target symbol name
    pub to_symbol: String,

    /// Relationship type (calls, imports, extends, etc.)
    pub relationship_type: String,

    /// Source file path
    pub from_file: String,

    /// Target file path
    pub to_file: String,

    /// Line number where relationship occurs
    pub line: u32,

    /// Relationship strength/weight
    pub weight: f64,
}

/// Context information for JavaScript
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct ContextInfoJs {
    /// Total symbols in the codebase
    pub total_symbols: u32,

    /// Number of files involved in the query
    pub files_involved: u32,

    /// Complexity score of the query result
    pub complexity_score: f64,

    /// Architectural patterns detected
    pub architectural_patterns: Vec<String>,

    /// Potential issues or improvements
    pub potential_issues: Vec<String>,
}

/// Export format options
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct ExportOptionsJs {
    /// Export format (json, lsp, embeddings)
    pub format: String,

    /// Output file path (optional)
    pub output_path: Option<String>,

    /// Include source code in export
    pub include_source: Option<bool>,

    /// Include documentation in export
    pub include_docs: Option<bool>,

    /// Minify the output
    pub minify: Option<bool>,
}

/// File watching event
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct FileWatchEvent {
    /// Event type (created, modified, deleted)
    pub event_type: String,

    /// File path that changed
    pub file_path: String,

    /// Timestamp of the event
    pub timestamp: String,

    /// Whether this affects the analysis
    pub affects_analysis: bool,
}

/// Progress information for chunked analysis
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct AnalysisProgress {
    /// Current chunk being processed
    pub current_chunk: u32,

    /// Total number of chunks
    pub total_chunks: u32,

    /// Whether this is the last chunk
    pub is_last: bool,

    /// Progress percentage (0-100)
    pub progress: f64,

    /// Processing time for this chunk in milliseconds
    pub processing_time_ms: u32,
}
