//! # Analysis Domain
//!
//! Codebase analysis and intelligence features with optional graph integration.
//! This domain provides semantic understanding of code structures.

use super::core::{CoreError, Metrics};
use super::Domain;
use crate::analysis::AnalysisResult;
use crate::cache::AdaptiveCacheManager;
use crate::export::ExportOptions;
use crate::parsers::ParserFactory;
use crate::query::{CodeQueryEngine, QueryResult};
use crate::symbols::SymbolExtractorFactory;
use crate::watcher::CodebaseWatcher;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};

/// Analysis domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Project root directory
    pub project_root: String,
    
    /// Languages to analyze
    pub languages: Vec<String>,
    
    /// File patterns to ignore
    pub ignore_patterns: Vec<String>,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache policy
    pub cache_policy: String,
    
    /// Enable file watching
    pub enable_watching: bool,
    
    /// Maximum file size to analyze (MB)
    pub max_file_size_mb: usize,
    
    /// Enable graph integration
    pub enable_graph_integration: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            project_root: ".".to_string(),
            languages: vec![],
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
            enable_caching: true,
            cache_policy: "adaptive".to_string(),
            enable_watching: false,
            max_file_size_mb: 10,
            enable_graph_integration: false,
        }
    }
}

/// Analysis domain errors
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Project error: {message}")]
    Project { message: String },
    
    #[error("Parser error: {language} - {message}")]
    Parser { language: String, message: String },
    
    #[error("Cache error: {message}")]
    Cache { message: String },
    
    #[error("Query error: {message}")]
    Query { message: String },
    
    #[error("Export error: {format} - {message}")]
    Export { format: String, message: String },
    
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Analysis session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSession {
    pub id: String,
    pub project_root: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub files_analyzed: usize,
    pub symbols_extracted: usize,
    pub errors_encountered: usize,
}

/// Analysis engine - codebase analysis and intelligence
pub struct AnalysisEngine {
    config: AnalysisConfig,
    #[allow(dead_code)]
    runtime: Runtime,
    current_session: Option<AnalysisSession>,
    analysis_result: Option<AnalysisResult>,
    query_engine: Option<CodeQueryEngine>,
    cache_manager: Option<Arc<AdaptiveCacheManager<String>>>,
    #[allow(dead_code)]
    watcher: Option<CodebaseWatcher>,
    metrics: Arc<Metrics>,
}

impl AnalysisEngine {
    /// Create a new analysis engine
    pub fn new(config: AnalysisConfig, metrics: Arc<Metrics>) -> Result<Self, AnalysisError> {
        let runtime = Runtime::new().map_err(|e| AnalysisError::Project {
            message: format!("Failed to create async runtime: {e}"),
        })?;
        
        Ok(Self {
            config,
            runtime,
            current_session: None,
            analysis_result: None,
            query_engine: None,
            cache_manager: None,
            watcher: None,
            metrics,
        })
    }
    
    /// Start analysis session
    pub async fn start_analysis(&mut self) -> Result<String, AnalysisError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        self.current_session = Some(AnalysisSession {
            id: session_id.clone(),
            project_root: self.config.project_root.clone(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            files_analyzed: 0,
            symbols_extracted: 0,
            errors_encountered: 0,
        });
        
        self.metrics.increment_counter("analysis_sessions_started").await;
        
        Ok(session_id)
    }
    
    /// Analyze project
    pub async fn analyze_project(&mut self) -> Result<AnalysisResult, AnalysisError> {
        let start_time = std::time::Instant::now();
        
        // Validate project root
        if !std::path::Path::new(&self.config.project_root).exists() {
            return Err(AnalysisError::Project {
                message: format!("Project root does not exist: {}", self.config.project_root),
            });
        }
        
        // Initialize cache if enabled
        if self.config.enable_caching && self.cache_manager.is_none() {
            // Initialize cache manager (simplified for example)
            // self.cache_manager = Some(Arc::new(AdaptiveCacheManager::new()));
        }
        
        // Initialize parser factory
        let parser_factory = ParserFactory::new();
        
        // Initialize symbol extractor
        let symbol_extractor = SymbolExtractorFactory::new();
        
        // Perform analysis (simplified for example)
        let analysis_result = self.perform_analysis(&parser_factory, &symbol_extractor).await?;
        
        // Update session
        if let Some(session) = &mut self.current_session {
            session.completed_at = Some(chrono::Utc::now());
            session.files_analyzed = analysis_result.file_count;
            session.symbols_extracted = analysis_result.symbol_count;
        }
        
        let duration = start_time.elapsed();
        self.metrics.record_timing("analysis_duration_ms", duration.as_millis() as u64).await;
        self.metrics.increment_counter("analyses_completed").await;
        
        self.analysis_result = Some(analysis_result.clone());
        
        Ok(analysis_result)
    }
    
    /// Query analyzed codebase
    pub async fn query(&mut self, query: &str) -> Result<QueryResult, AnalysisError> {
        if self.analysis_result.is_none() {
            return Err(AnalysisError::Query {
                message: "No analysis result available. Run analyze_project first.".to_string(),
            });
        }
        
        // Initialize query engine if needed (simplified for architectural example)
        if self.query_engine.is_none() && self.analysis_result.is_some() {
            let analysis_result = self.analysis_result.as_ref().unwrap().clone();
            self.query_engine = Some(CodeQueryEngine::new(analysis_result));
        }

        // Perform query (simplified for example)
        use crate::query::ContextInfo;
        let result = QueryResult {
            symbols: vec![], // Simplified
            relationships: vec![], // Simplified
            context: ContextInfo {
                total_symbols: 0,
                files_involved: 0,
                complexity_score: 0.0,
                architectural_patterns: vec![],
                potential_issues: vec![],
            },
            suggestions: vec![format!("Query: {}", query)],
        };
        
        self.metrics.increment_counter("queries_executed").await;
        
        Ok(result)
    }
    
    /// Export analysis results
    pub async fn export(&self, _options: ExportOptions) -> Result<String, AnalysisError> {
        let _analysis_result = self.analysis_result.as_ref()
            .ok_or_else(|| AnalysisError::Export {
                format: "json".to_string(),
                message: "No analysis result to export".to_string(),
            })?;

        // Simplified export implementation for architectural example
        // In a real implementation, this would use the JsonExporter
        Ok("{}".to_string())
    }
    
    /// Get current session
    pub fn get_current_session(&self) -> Option<&AnalysisSession> {
        self.current_session.as_ref()
    }
    
    /// Get analysis result
    pub fn get_analysis_result(&self) -> Option<&AnalysisResult> {
        self.analysis_result.as_ref()
    }
    
    /// Enable file watching
    pub async fn enable_watching(&mut self) -> Result<(), AnalysisError> {
        if !self.config.enable_watching {
            return Err(AnalysisError::Project {
                message: "File watching is disabled in configuration".to_string(),
            });
        }
        
        // Initialize watcher (simplified)
        // self.watcher = Some(CodebaseWatcher::new(&self.config.project_root)?);
        
        Ok(())
    }
    
    /// Simplified analysis implementation
    async fn perform_analysis(
        &self,
        _parser_factory: &ParserFactory,
        _symbol_extractor: &SymbolExtractorFactory,
    ) -> Result<AnalysisResult, AnalysisError> {
        use crate::analysis::CodeGraph;

        // This is a simplified implementation for the architectural example
        Ok(AnalysisResult {
            graph: CodeGraph::new(),
            file_count: 0,
            symbol_count: 0,
            relationship_count: 0,
            languages: vec![],
        })
    }

    /// Get the count of active analysis sessions
    pub fn get_session_count(&self) -> u32 {
        if self.current_session.is_some() {
            1
        } else {
            0
        }
    }
}

impl Domain for AnalysisEngine {
    type Config = AnalysisConfig;
    type Error = AnalysisError;
    
    fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        let metrics = Arc::new(Metrics::new());
        Self::new(config, metrics)
    }
    
    fn domain_name(&self) -> &'static str {
        "analysis"
    }
    
    fn health_check(&self) -> Result<(), Self::Error> {
        // Check if project root is accessible
        if !std::path::Path::new(&self.config.project_root).exists() {
            return Err(AnalysisError::Project {
                message: "Project root is not accessible".to_string(),
            });
        }
        
        Ok(())
    }
}
