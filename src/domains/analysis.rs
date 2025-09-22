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
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::cell::RefCell;
use std::thread_local;

thread_local! {
    static ANALYSIS_DEPTH: RefCell<u32> = const { RefCell::new(0) };
}

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: u32 = 10;

/// Guard against circular dependencies and infinite recursion
pub struct RecursionGuard {
    depth: u32,
}

impl RecursionGuard {
    /// Create a new recursion guard, incrementing depth
    pub fn new() -> Result<Self, AnalysisError> {
        let depth = ANALYSIS_DEPTH.with(|d| {
            let current = *d.borrow();
            *d.borrow_mut() = current + 1;
            current + 1
        });
        
        if depth > MAX_RECURSION_DEPTH {
            ANALYSIS_DEPTH.with(|d| *d.borrow_mut() = 0); // Reset on error
            return Err(AnalysisError::Project {
                message: format!("Maximum recursion depth exceeded: {}", depth),
            });
        }
        
        Ok(Self { depth })
    }
    
    /// Get current depth
    pub fn depth(&self) -> u32 {
        self.depth
    }
}

impl Drop for RecursionGuard {
    fn drop(&mut self) {
        ANALYSIS_DEPTH.with(|d| {
            let current = *d.borrow();
            if current > 0 {
                *d.borrow_mut() = current - 1;
            }
        });
    }
}

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

        self.metrics
            .increment_counter("analysis_sessions_started")
            .await;

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
            match AdaptiveCacheManager::new(&self.config.project_root).await {
                Ok(cache_manager) => {
                    self.cache_manager = Some(Arc::new(cache_manager));
                    println!("✅ Cache manager initialized successfully");
                }
                Err(e) => {
                    eprintln!("⚠️ Warning: Failed to initialize cache manager: {}", e);
                    // Continue without caching - don't fail the entire analysis
                }
            }
        }

        // Initialize parser factory
        let parser_factory = ParserFactory::new();

        // Initialize symbol extractor
        let symbol_extractor = SymbolExtractorFactory::new();

        // Perform analysis (simplified for example)
        let analysis_result = self
            .perform_analysis(&parser_factory, &symbol_extractor)
            .await?;

        // Update session
        if let Some(session) = &mut self.current_session {
            session.completed_at = Some(chrono::Utc::now());
            session.files_analyzed = analysis_result.file_count;
            session.symbols_extracted = analysis_result.symbol_count;
        }

        let duration = start_time.elapsed();
        self.metrics
            .record_timing("analysis_duration_ms", duration.as_millis() as u64)
            .await;
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
            symbols: vec![],       // Simplified
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
    pub async fn export(&self, options: ExportOptions) -> Result<String, AnalysisError> {
        let analysis_result =
            self.analysis_result
                .as_ref()
                .ok_or_else(|| AnalysisError::Export {
                    format: "json".to_string(),
                    message: "No analysis result to export".to_string(),
                })?;

                use crate::export::JsonExporter;

        let exporter = JsonExporter::new(analysis_result.clone(), self.config.project_root.clone());

        exporter.export_to_string(&options)
            .map_err(|e| AnalysisError::Export {
                format: "json".to_string(),
                message: format!("Export failed: {}", e),
            })
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

    /// Real analysis implementation using the working CoreAnalyzer
    async fn perform_analysis(
        &self,
        _parser_factory: &ParserFactory,
        _symbol_extractor: &SymbolExtractorFactory,
    ) -> Result<AnalysisResult, AnalysisError> {
        use crate::core::CoreAnalyzer;
        use crate::parsers::LanguageId;

        // Guard against circular dependencies and infinite recursion
        let _guard = RecursionGuard::new()?;
        
                let core_analyzer = CoreAnalyzer::new(
            self.config.project_root.clone(),
            Some(self.config.languages.clone()),
            Some(self.config.ignore_patterns.clone()),
        );

        // Unified analysis result structure
        struct UnifiedAnalysisResult {
            file_count: u32,
            symbol_count: u32,
            languages: Vec<String>,
            #[allow(dead_code)]
        duration_ms: u32,
            relationships: Vec<crate::symbols::Dependency>,
        }

        // Run the actual analysis that works
        let unified_result = {
            #[cfg(not(feature = "python"))]
            {
                let core_result = core_analyzer.analyze()
                    .map_err(|e| AnalysisError::Project {
                        message: format!("Core analysis failed: {}", e)
                    })?;
                
                UnifiedAnalysisResult {
                    file_count: core_result.file_count,
                    symbol_count: core_result.symbol_count,
                    languages: core_result.languages,
                    duration_ms: core_result.duration_ms,
                    relationships: core_result.relationships,
                }
            }

            #[cfg(feature = "python")]
            {
                // When Python feature is enabled, we get AnalysisResult with PyDependency
                let py_result = core_analyzer.analyze()
                    .map_err(|e| AnalysisError::Project {
                        message: format!("Core analysis failed: {}", e)
                    })?;
                
                // Convert PyDependency relationships to Dependency relationships for graph building
                use crate::symbols::{Dependency, DependencyType, Location};
                let dependencies: Vec<Dependency> = py_result.relationships.into_iter().map(|py_dep| {
                    Dependency {
                        from_symbol: py_dep.from_symbol,
                        to_symbol: py_dep.to_symbol,
                        relationship_type: match py_dep.relationship_type.as_str() {
                            "Calls" => DependencyType::Calls,
                            "References" => DependencyType::References,
                            "Imports" => DependencyType::Imports,
                            "Inherits" => DependencyType::Inherits,
                            "Implements" => DependencyType::Implements,
                            "Assigns" => DependencyType::Assigns,
                            "ControlFlow" => DependencyType::ControlFlow,
                            "Uses" => DependencyType::Uses,
                            "TypeOf" => DependencyType::TypeOf,
                            "Declares" => DependencyType::Declares,
                            "ModuleDependency" => DependencyType::ModuleDependency,
                            _ => DependencyType::References, // Fallback
                        },
                        location: Location {
                            file_path: "unknown".to_string(), // PyDependency doesn't have file_path
                            start_line: 1, // Default location since PyDependency doesn't have it
                            end_line: 1,
                            start_column: 1,
                            end_column: 1,
                        },
                        file_path: "unknown".to_string(), // PyDependency doesn't have file_path
                        language: crate::parsers::LanguageId::TypeScript, // Default fallback
                        context: None,
                        strength: 1.0,
                        is_conditional: false,
                    }
                }).collect();
                
                UnifiedAnalysisResult {
                    file_count: py_result.file_count,
                    symbol_count: py_result.symbol_count,
                    languages: py_result.languages,
                    duration_ms: py_result.duration_ms,
                    relationships: dependencies,
                }
            }
        };

        // Convert core result to domain result format
        let languages: Vec<LanguageId> = unified_result.languages.iter()
            .filter_map(|s| LanguageId::from_string(s))
            .collect();

        // Build actual relationship graph from extracted dependencies
        let mut graph_builder = crate::analysis::CodeGraphBuilder::new();
        
        // Add all symbols to the graph (need to extract them again since core_result doesn't have them)
        use crate::parsers::ParserFactory;
        use crate::symbols::SymbolExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;
        
        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        
        for entry in WalkDir::new(&self.config.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy();
                if crate::utils::should_ignore_file(&path_str, &self.config.ignore_patterns) {
                    continue;
                }
                
                if let Ok(content) = fs::read_to_string(&*path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        let symbols = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                        graph_builder.add_file_symbols(symbols, &path_str);
                    }
                }
            }
        }
        
        // Add relationships from the extracted dependencies
        for dep in &unified_result.relationships {
            use crate::analysis::{RelationshipKind, CodeRelationship};
            
            let relationship_kind = match dep.relationship_type {
                crate::symbols::DependencyType::Calls => RelationshipKind::Calls,
                crate::symbols::DependencyType::Imports => RelationshipKind::Imports,
                crate::symbols::DependencyType::Inherits => RelationshipKind::Inherits,
                crate::symbols::DependencyType::Implements => RelationshipKind::Implements,
                crate::symbols::DependencyType::References => RelationshipKind::References,
                crate::symbols::DependencyType::Assigns => RelationshipKind::References,
                crate::symbols::DependencyType::ControlFlow => RelationshipKind::DependsOn,
                crate::symbols::DependencyType::Uses => RelationshipKind::DependsOn,
                crate::symbols::DependencyType::TypeOf => RelationshipKind::DependsOn,
                crate::symbols::DependencyType::Declares => RelationshipKind::DefinedIn,
                crate::symbols::DependencyType::ModuleDependency => RelationshipKind::Imports,
                _ => RelationshipKind::DependsOn, // Fallback for other types
            };
            
            let relationship = CodeRelationship {
                kind: relationship_kind,
                source_location: format!("{}:{}", dep.file_path, dep.location.start_line),
                confidence: 0.8, // Default confidence
                metadata: std::collections::HashMap::new(),
            };
            
            // Try to add the relationship to the graph
            let _ = graph_builder.add_relationship(&dep.from_symbol, &dep.to_symbol, relationship);
        }
        
        let graph = graph_builder.build();

        Ok(AnalysisResult {
            graph,
            file_count: unified_result.file_count as usize,
            symbol_count: unified_result.symbol_count as usize,
            relationship_count: unified_result.relationships.len(),
            languages,
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
