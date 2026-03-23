//! # FastContextAnalyzer Module
//!
//! This module contains the main FastContextAnalyzer implementation
//! extracted from the monolithic lib.rs for better organization.

use crate::analysis::AnalysisResult;
use crate::cache::AdaptiveCacheManager;
use crate::core::{CoreAnalyzer, CoreAnalyzerOptions};
use crate::domains;
use crate::parsers::LanguageId;
use crate::query::{CodeQueryEngine, QueryResult};
use crate::watcher::CodebaseWatcher;

use napi_derive::napi;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use tokio::runtime::Runtime;
use ts_rs::TS;

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
/// THREAD-SAFE ARCHITECTURE: This analyzer now uses proper synchronization
/// for all shared state while maintaining 100% backward compatibility.
#[napi]
pub struct FastContextAnalyzer {
    // Core runtime and state (thread-safe)
    #[allow(dead_code)]
    runtime: Arc<Runtime>,
    project_root: String,
    languages: Vec<String>,
    ignore_patterns: Vec<String>,
    max_files: Option<u32>,
    parallel_processing: bool,

    // Thread-safe shared state using Arc<RwLock<T>>
    analysis: Arc<RwLock<Option<AnalysisResult>>>,
    #[allow(dead_code)]
    query_engine: Arc<RwLock<Option<CodeQueryEngine>>>,
    #[allow(dead_code)]
    cache_manager: Arc<RwLock<Option<Arc<AdaptiveCacheManager<String>>>>>,
    watcher: Arc<RwLock<Option<CodebaseWatcher>>>,

    // Thread-safe caches with proper synchronization
    #[allow(dead_code)]
    file_query_cache: Arc<Mutex<HashMap<String, (QueryResult, std::time::Instant)>>>,
    #[allow(dead_code)]
    cache_access_order: Arc<Mutex<VecDeque<String>>>,

    // Domain architecture components (thread-safe)
    #[allow(dead_code)]
    domain_metrics: Arc<domains::core::Metrics>,
    #[allow(dead_code)]
    architectural_mode: ArchitecturalMode,
}

#[napi]
impl FastContextAnalyzer {
    fn core_analyzer(&self) -> CoreAnalyzer {
        CoreAnalyzer::with_options(
            self.project_root.clone(),
            Some(self.languages.clone()),
            Some(self.ignore_patterns.clone()),
            CoreAnalyzerOptions {
                max_files: self.max_files.map(|max_files| max_files as usize),
                parallel_processing: self.parallel_processing,
            },
        )
    }

    #[napi(constructor)]
    pub fn new(config: AnalyzerConfig) -> napi::Result<Self> {
        // Create Tokio runtime for async operations
        let runtime = Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create runtime: {e}")))?;

        // Validate project root exists and is accessible
        crate::validation::validate_directory_path(&config.project_root).map_err(|e| {
            napi::Error::from_reason(format!(
                "Invalid project root '{}': {}",
                config.project_root, e
            ))
        })?;

        // Validate configuration parameters
        Self::validate_config(&config)?;

        // Initialize domain architecture components
        let domain_metrics = Arc::new(domains::core::Metrics::new());
        let architectural_mode = if config.enable_experimental_architecture.unwrap_or(false) {
            ArchitecturalMode::Harmonious
        } else {
            ArchitecturalMode::Legacy
        };

        Ok(Self {
            runtime: Arc::new(runtime),
            project_root: config.project_root,
            languages: config.languages.unwrap_or_default(),
            ignore_patterns: config.ignore_patterns.unwrap_or_default(),
            max_files: config.max_files,
            parallel_processing: config.parallel_processing.unwrap_or(true),
            analysis: Arc::new(RwLock::new(None)),
            query_engine: Arc::new(RwLock::new(None)),
            cache_manager: Arc::new(RwLock::new(None)),
            watcher: Arc::new(RwLock::new(None)),
            file_query_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_access_order: Arc::new(Mutex::new(VecDeque::new())),
            domain_metrics,
            architectural_mode,
        })
    }

    /// Analyze the codebase and return analysis results
    #[napi]
    pub fn analyze(&self) -> napi::Result<AnalysisResultJs> {
        let core = self.core_analyzer();
        let start_time = std::time::Instant::now();
        let summary = core
            .analyze()
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        let duration = start_time.elapsed();

        let js_result = AnalysisResultJs {
            file_count: summary.file_count,
            symbol_count: summary.symbol_count,
            relationship_count: summary.relationships.len() as u32,
            languages: summary.languages.clone(),
            duration_ms: duration.as_millis() as u32,
            memory_usage_mb: None,
        };

        // Store analysis result in thread-safe manner
        if let Ok(mut analysis) = self.analysis.write() {
            // Convert string languages to LanguageId
            let languages: Vec<LanguageId> = summary
                .languages
                .iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "rust" => Some(LanguageId::Rust),
                    "javascript" => Some(LanguageId::JavaScript),
                    "typescript" => Some(LanguageId::TypeScript),
                    "python" => Some(LanguageId::Python),
                    "java" => Some(LanguageId::Java),
                    "go" => Some(LanguageId::Go),
                    "cpp" | "c++" | "c" => Some(LanguageId::Cpp),
                    "csharp" | "c#" => Some(LanguageId::CSharp),
                    "swift" => Some(LanguageId::Swift),
                    "php" => Some(LanguageId::PHP),
                    "ruby" => Some(LanguageId::Ruby),
                    "bash" => Some(LanguageId::Bash),
                    _ => None,
                })
                .collect();

            *analysis = Some(AnalysisResult {
                file_count: summary.file_count as usize,
                symbol_count: summary.symbol_count as usize,
                relationship_count: summary.relationships.len(),
                languages,
                graph: petgraph::Graph::new(),
            });
        }

        Ok(js_result)
    }

    /// Start watching the codebase for changes
    #[napi]
    pub fn start_watching(&self) -> napi::Result<()> {
        use crate::watcher::{CodebaseWatcher, WatcherConfig};
        use std::collections::HashSet;
        use std::path::PathBuf;
        use std::time::Duration;

        // Check if watcher is already running (thread-safe)
        if let Ok(watcher_guard) = self.watcher.read() {
            if watcher_guard.is_some() {
                return Err(napi::Error::from_reason("File watcher is already running"));
            }
        } else {
            return Err(napi::Error::from_reason("Failed to acquire watcher lock"));
        }

        // Create watcher configuration
        let mut watched_extensions = HashSet::new();
        watched_extensions.insert("rs".to_string());
        watched_extensions.insert("js".to_string());
        watched_extensions.insert("ts".to_string());
        watched_extensions.insert("py".to_string());
        watched_extensions.insert("java".to_string());
        watched_extensions.insert("go".to_string());
        watched_extensions.insert("cpp".to_string());
        watched_extensions.insert("c".to_string());
        watched_extensions.insert("cs".to_string());

        let config = WatcherConfig {
            watch_dirs: vec![PathBuf::from(&self.project_root)],
            watched_extensions,
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
                "*.tmp".to_string(),
            ],
            debounce_duration: Duration::from_millis(500),
            batch_size: 100,
        };

        // Create the watcher
        let watcher = CodebaseWatcher::new(config)
            .map_err(|e| napi::Error::from_reason(format!("Failed to create watcher: {e}")))?;

        // Store watcher in thread-safe manner
        if let Ok(mut watcher_guard) = self.watcher.write() {
            *watcher_guard = Some(watcher);
        } else {
            return Err(napi::Error::from_reason(
                "Failed to acquire watcher write lock",
            ));
        }

        Ok(())
    }

    /// Stop watching the codebase
    #[napi]
    pub fn stop_watching(&self) -> napi::Result<()> {
        // Check and stop watcher in thread-safe manner
        if let Ok(mut watcher_guard) = self.watcher.write() {
            if watcher_guard.is_none() {
                return Err(napi::Error::from_reason("File watcher is not running"));
            }
            *watcher_guard = None;
        } else {
            return Err(napi::Error::from_reason(
                "Failed to acquire watcher write lock",
            ));
        }

        Ok(())
    }

    /// Get the current analysis results
    #[napi]
    pub fn get_analysis(&self) -> Option<AnalysisResultJs> {
        // Access analysis result in thread-safe manner
        if let Ok(analysis_guard) = self.analysis.read() {
            analysis_guard.as_ref().map(|result| AnalysisResultJs {
                file_count: result.file_count as u32,
                symbol_count: result.symbol_count as u32,
                relationship_count: result.relationship_count as u32,
                languages: result.languages.iter().map(|l| format!("{l:?}")).collect(),
                duration_ms: 0,
                memory_usage_mb: None,
            })
        } else {
            None
        }
    }

    /// Find symbols by kind (function, class, variable, etc.)
    #[napi]
    pub fn find_symbols_by_kind(&self, kind: String) -> napi::Result<Vec<String>> {
        self.core_analyzer()
            .find_symbols_by_kind(kind)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Find symbols in a specific file
    #[napi]
    pub fn find_symbols_in_file(&self, file_path: String) -> napi::Result<Vec<String>> {
        // Use secure path resolution within project boundaries
        let full_path = crate::validation::resolve_project_path(
            std::path::Path::new(&self.project_root),
            &file_path,
        )
        .map_err(|e| {
            napi::Error::from_reason(format!("Invalid file path '{}': {}", file_path, e))
        })?;

        self.core_analyzer()
            .find_symbols_in_file(full_path.to_string_lossy().into_owned())
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Find dependencies of a symbol
    #[napi]
    pub fn find_dependencies(&self, symbol_name: String) -> napi::Result<Vec<String>> {
        self.core_analyzer()
            .find_dependencies(symbol_name)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Find complex symbols (high complexity)
    #[napi]
    pub fn find_complex_symbols(&self, complexity_threshold: u32) -> napi::Result<Vec<String>> {
        self.core_analyzer()
            .find_complex_symbols(complexity_threshold)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Check if a file should be ignored based on common patterns
    #[cfg(any())]
    fn should_ignore_file(&self, path: &str) -> bool {
        let ignore_patterns = [
            "node_modules/",
            "target/",
            ".git/",
            "dist/",
            "build/",
            ".cache/",
            "coverage/",
            ".nyc_output/",
            "*.tmp",
            "*.log",
            "*.lock",
            ".DS_Store",
            "Thumbs.db",
        ];

        for pattern in &ignore_patterns {
            if pattern.ends_with('/') {
                // Directory pattern
                if path.contains(pattern) {
                    return true;
                }
            } else if let Some(ext) = pattern.strip_prefix("*.") {
                // Extension pattern
                if path.ends_with(ext) {
                    return true;
                }
            } else {
                // Exact match
                if path.contains(pattern) {
                    return true;
                }
            }
        }

        false
    }

    /// Validate analyzer configuration for security
    fn validate_config(config: &AnalyzerConfig) -> napi::Result<()> {
        use crate::validation::*;

        // Validate project root path (already done above, but double-check for security)
        validate_config_key("project_root")
            .map_err(|e| napi::Error::from_reason(format!("Invalid config key: {}", e)))?;
        if config.project_root.trim().is_empty() {
            return Err(napi::Error::from_reason("Project root cannot be empty"));
        }

        // Validate languages if provided
        if let Some(ref languages) = config.languages {
            validate_languages(languages)
                .map_err(|e| napi::Error::from_reason(format!("Invalid languages: {}", e)))?;
        }

        // Validate ignore patterns if provided
        if let Some(ref patterns) = config.ignore_patterns {
            validate_ignore_patterns(patterns)
                .map_err(|e| napi::Error::from_reason(format!("Invalid ignore patterns: {}", e)))?;
        }

        // Validate cache policy if provided
        if let Some(ref policy) = config.cache_policy {
            validate_string(policy, 50, "cache_policy")
                .map_err(|e| napi::Error::from_reason(format!("Invalid cache policy: {}", e)))?;
            let allowed_policies = ["auto", "minimal", "balanced", "adaptive", "persistent"];
            if !allowed_policies.contains(&policy.to_lowercase().as_str()) {
                return Err(napi::Error::from_reason(format!(
                    "Invalid cache policy: {}. Allowed: {}",
                    policy,
                    allowed_policies.join(", ")
                )));
            }
        }

        // Validate numeric parameters
        if let Some(max_files) = config.max_files {
            if max_files > 1000000 {
                return Err(napi::Error::from_reason(
                    "max_files too large: must be <= 1,000,000",
                ));
            }
        }

        Ok(())
    }
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

    /// Analysis duration in milliseconds
    pub duration_ms: u32,

    /// Memory usage in MB (optional)
    pub memory_usage_mb: Option<f64>,
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
