//! # Unified API Implementation
//!
//! The main API class that provides a harmonious interface to all functionality
//! while maintaining clear separation between graph and analysis domains.

use super::{ApiError, ApiValidator, FastContextConfig, HealthStatus, FeatureInfo, SystemMetrics};
use crate::domains::{DomainRegistry, GraphEngine, AnalysisEngine, Domain};
use crate::domains::core::{CoreConfig, FeatureFlags, PerformanceConfig, Metrics};
use crate::domains::graph::{GraphConfig, GraphError};
use crate::domains::analysis::{AnalysisConfig, AnalysisError};
use std::sync::Arc;
use napi_derive::napi;
use ts_rs::TS;
use tokio::sync::RwLock;

/// The main FastContext API - unified interface to all functionality
#[napi]
pub struct FastContext {
    registry: Arc<RwLock<DomainRegistry>>,
    config: FastContextConfig,
    metrics: Arc<Metrics>,
    initialized: bool,
}

/// TypeScript type definition for FastContext
#[derive(TS)]
#[ts(export)]
pub struct FastContextType {}

#[napi]
impl FastContext {
    /// Create a new FastContext instance
    #[napi(constructor)]
    pub fn new(config: FastContextConfig) -> napi::Result<Self> {
        // Validate configuration
        ApiValidator::validate_config(&config).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        let metrics = Arc::new(Metrics::new());
        let registry = Arc::new(RwLock::new(DomainRegistry::new()));
        
        Ok(Self {
            registry,
            config,
            metrics,
            initialized: false,
        })
    }
    
    /// Initialize the FastContext system
    #[napi]
    pub async unsafe fn initialize(&mut self) -> napi::Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        let mut registry = self.registry.write().await;
        
        // Initialize graph engine if enabled
        if self.is_graph_enabled() {
            let graph_config = self.build_graph_config();
            let graph_engine = GraphEngine::initialize(graph_config)
                .map_err(|e| napi::Error::from_reason(format!("Graph domain error: {}", e)))?;
            
            registry.register_graph_engine(graph_engine);
            self.metrics.increment_counter("graph_engine_initialized").await;
        }
        
        // Initialize analysis engine if enabled
        if self.is_analysis_enabled() {
            let analysis_config = self.build_analysis_config();
            let analysis_engine = AnalysisEngine::initialize(analysis_config)
                .map_err(|e| napi::Error::from_reason(format!("Analysis domain error: {}", e)))?;
            
            registry.register_analysis_engine(analysis_engine);
            self.metrics.increment_counter("analysis_engine_initialized").await;
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check system health
    #[napi]
    pub async fn health_check(&self) -> napi::Result<HealthStatus> {
        if !self.initialized {
            return Ok(HealthStatus {
                healthy: false,
                graph_healthy: false,
                analysis_healthy: false,
                errors: vec!["System not initialized".to_string()],
                metrics: self.get_system_metrics().await,
            });
        }
        
        let registry = self.registry.read().await;
        let mut errors = Vec::new();
        let mut graph_healthy = true;
        let mut analysis_healthy = true;
        
        // Check graph engine health
        if let Some(graph_engine) = registry.graph_engine() {
            if let Err(e) = graph_engine.health_check() {
                graph_healthy = false;
                errors.push(format!("Graph engine: {}", e));
            }
        }
        
        // Check analysis engine health
        if let Some(analysis_engine) = registry.analysis_engine() {
            if let Err(e) = analysis_engine.health_check() {
                analysis_healthy = false;
                errors.push(format!("Analysis engine: {}", e));
            }
        }
        
        let healthy = graph_healthy && analysis_healthy && errors.is_empty();
        
        Ok(HealthStatus {
            healthy,
            graph_healthy,
            analysis_healthy,
            errors,
            metrics: self.get_system_metrics().await,
        })
    }
    
    /// Get feature information
    #[napi]
    pub fn get_feature_info(&self) -> FeatureInfo {
        FeatureInfo {
            graph_available: self.is_graph_enabled(),
            analysis_available: self.is_analysis_enabled(),
            caching_available: self.is_caching_enabled(),
            watching_available: self.is_watching_enabled(),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "python".to_string(),
                "java".to_string(),
                "go".to_string(),
                "cpp".to_string(),
                "csharp".to_string(),
            ],
            export_formats: vec![
                "json".to_string(),
                "lsp".to_string(),
                "embeddings".to_string(),
            ],
        }
    }
    
    /// Create a new graph (requires graph domain)
    #[napi]
    pub async fn create_graph(&self, graph_id: String, name: String, directed: bool) -> napi::Result<()> {
        self.ensure_initialized()?;
        self.ensure_graph_enabled()?;
        
        ApiValidator::validate_graph_params(&graph_id, None)?;
        
        let mut registry = self.registry.write().await;
        if let Some(graph_engine) = registry.graph_engine() {
            // Note: This is a simplified example - in reality we'd need mutable access
            // The actual implementation would use interior mutability or different patterns
            return Err(napi::Error::from_reason("Graph creation requires mutable access - architectural example"));
        }
        
        Err(napi::Error::from_reason("Graph engine not available"))
    }
    
    /// Start codebase analysis (requires analysis domain)
    #[napi]
    pub async fn analyze_project(&self, project_root: String) -> napi::Result<String> {
        self.ensure_initialized()?;
        self.ensure_analysis_enabled()?;
        
        ApiValidator::validate_analysis_params(&project_root, &[])?;
        
        let mut registry = self.registry.write().await;
        if let Some(analysis_engine) = registry.analysis_engine() {
            // Note: This is a simplified example - in reality we'd need mutable access
            return Err(napi::Error::from_reason("Analysis requires mutable access - architectural example"));
        }
        
        Err(napi::Error::from_reason("Analysis engine not available"))
    }
    
    /// Query analyzed codebase
    #[napi]
    pub async fn query(&self, query: String) -> napi::Result<String> {
        self.ensure_initialized()?;
        self.ensure_analysis_enabled()?;
        
        // Simplified query implementation
        Ok(format!("Query result for: {}", query))
    }
    
    /// Get system metrics
    async fn get_system_metrics(&self) -> SystemMetrics {
        SystemMetrics {
            memory_usage_mb: 0.0, // Would be calculated from actual usage
            active_graphs: 0,
            analysis_sessions: 0,
            cache_hit_rate: self.metrics.get_gauge("cache_hit_rate").await.unwrap_or(0.0),
            avg_operation_time_ms: self.metrics.get_average_timing("operation_time").await.unwrap_or(0.0),
        }
    }
    
    /// Check if graph domain is enabled
    fn is_graph_enabled(&self) -> bool {
        self.config.core.as_ref()
            .and_then(|c| c.enable_graph)
            .unwrap_or(true)
    }
    
    /// Check if analysis domain is enabled
    fn is_analysis_enabled(&self) -> bool {
        self.config.core.as_ref()
            .and_then(|c| c.enable_analysis)
            .unwrap_or(true)
    }
    
    /// Check if caching is enabled
    fn is_caching_enabled(&self) -> bool {
        self.config.core.as_ref()
            .and_then(|c| c.enable_caching)
            .unwrap_or(true)
    }
    
    /// Check if file watching is enabled
    fn is_watching_enabled(&self) -> bool {
        self.config.analysis.as_ref()
            .and_then(|c| c.enable_watching)
            .unwrap_or(false)
    }
    
    /// Build graph configuration from API config
    fn build_graph_config(&self) -> GraphConfig {
        let graph_config = self.config.graph.as_ref();
        
        GraphConfig {
            enable_parallel: graph_config
                .and_then(|c| c.enable_parallel)
                .unwrap_or(true),
            max_nodes: graph_config
                .and_then(|c| c.max_nodes)
                .unwrap_or(1_000_000) as usize,
            max_edges: graph_config
                .and_then(|c| c.max_edges)
                .unwrap_or(10_000_000) as usize,
            enable_caching: self.is_caching_enabled(),
        }
    }
    
    /// Build analysis configuration from API config
    fn build_analysis_config(&self) -> AnalysisConfig {
        let analysis_config = self.config.analysis.as_ref();
        
        AnalysisConfig {
            project_root: analysis_config
                .map(|c| c.project_root.clone())
                .unwrap_or_else(|| ".".to_string()),
            languages: analysis_config
                .and_then(|c| c.languages.clone())
                .unwrap_or_default(),
            ignore_patterns: analysis_config
                .and_then(|c| c.ignore_patterns.clone())
                .unwrap_or_else(|| vec![
                    "node_modules/**".to_string(),
                    ".git/**".to_string(),
                ]),
            enable_caching: self.is_caching_enabled(),
            cache_policy: "adaptive".to_string(),
            enable_watching: self.is_watching_enabled(),
            max_file_size_mb: analysis_config
                .and_then(|c| c.max_file_size_mb)
                .unwrap_or(10) as usize,
            enable_graph_integration: self.is_graph_enabled(),
        }
    }
    
    /// Ensure system is initialized
    fn ensure_initialized(&self) -> napi::Result<()> {
        if !self.initialized {
            return Err(napi::Error::from_reason("FastContext not initialized. Call initialize() first."));
        }
        Ok(())
    }
    
    /// Ensure graph domain is enabled
    fn ensure_graph_enabled(&self) -> napi::Result<()> {
        if !self.is_graph_enabled() {
            return Err(napi::Error::from_reason("Graph functionality is disabled"));
        }
        Ok(())
    }
    
    /// Ensure analysis domain is enabled
    fn ensure_analysis_enabled(&self) -> napi::Result<()> {
        if !self.is_analysis_enabled() {
            return Err(napi::Error::from_reason("Analysis functionality is disabled"));
        }
        Ok(())
    }
}
