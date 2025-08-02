//! # Unified API Layer
//!
//! This module provides a harmonious, consistent API that unifies access to all domains
//! while maintaining clear separation of concerns and architectural integrity.

pub mod graph;
pub mod analysis;
pub mod unified;

use crate::domains::{DomainRegistry, GraphEngine, AnalysisEngine};
use crate::domains::core::{CoreConfig, CoreError, Metrics};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use napi_derive::napi;
use ts_rs::TS;

/// Unified configuration for the entire system
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FastContextConfig {
    /// Core system configuration
    pub core: Option<CoreConfigJs>,
    
    /// Graph domain configuration
    pub graph: Option<GraphConfigJs>,
    
    /// Analysis domain configuration
    pub analysis: Option<AnalysisConfigJs>,
}

/// JavaScript-compatible core configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CoreConfigJs {
    /// Enable graph functionality
    pub enable_graph: Option<bool>,
    
    /// Enable analysis functionality
    pub enable_analysis: Option<bool>,
    
    /// Enable caching
    pub enable_caching: Option<bool>,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u32>,
    
    /// Number of worker threads
    pub worker_threads: Option<u32>,
}

/// JavaScript-compatible graph configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GraphConfigJs {
    /// Enable parallel algorithms
    pub enable_parallel: Option<bool>,
    
    /// Maximum nodes per graph
    pub max_nodes: Option<u32>,
    
    /// Maximum edges per graph
    pub max_edges: Option<u32>,
}

/// JavaScript-compatible analysis configuration
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnalysisConfigJs {
    /// Project root directory
    pub project_root: String,
    
    /// Languages to analyze
    pub languages: Option<Vec<String>>,
    
    /// File patterns to ignore
    pub ignore_patterns: Option<Vec<String>>,
    
    /// Enable file watching
    pub enable_watching: Option<bool>,
    
    /// Maximum file size to analyze (MB)
    pub max_file_size_mb: Option<u32>,
}

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Domain error: {domain} - {message}")]
    Domain { domain: String, message: String },
    
    #[error("Feature disabled: {feature}")]
    FeatureDisabled { feature: String },
    
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Convert API error to NAPI error
impl From<ApiError> for napi::Error {
    fn from(err: ApiError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

/// System health status
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HealthStatus {
    /// Overall system health
    pub healthy: bool,
    
    /// Graph domain status
    pub graph_healthy: bool,
    
    /// Analysis domain status
    pub analysis_healthy: bool,
    
    /// Error messages if any
    pub errors: Vec<String>,
    
    /// System metrics
    pub metrics: SystemMetrics,
}

/// System metrics for monitoring
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SystemMetrics {
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    
    /// Number of active graphs
    pub active_graphs: u32,
    
    /// Number of analysis sessions
    pub analysis_sessions: u32,
    
    /// Cache hit rate
    pub cache_hit_rate: f64,
    
    /// Average operation time in ms
    pub avg_operation_time_ms: f64,
}

/// Feature availability information
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeatureInfo {
    /// Graph algorithms available
    pub graph_available: bool,
    
    /// Analysis features available
    pub analysis_available: bool,
    
    /// Caching available
    pub caching_available: bool,
    
    /// File watching available
    pub watching_available: bool,
    
    /// Supported languages for analysis
    pub supported_languages: Vec<String>,
    
    /// Available export formats
    pub export_formats: Vec<String>,
}

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

/// Validation utilities for API inputs
pub struct ApiValidator;

impl ApiValidator {
    /// Validate configuration
    pub fn validate_config(config: &FastContextConfig) -> ApiResult<()> {
        // Validate core config
        if let Some(core) = &config.core {
            if let Some(memory) = core.max_memory_mb {
                if memory == 0 || memory > 16384 {
                    return Err(ApiError::Configuration {
                        message: "Memory limit must be between 1MB and 16GB".to_string(),
                    });
                }
            }
            
            if let Some(threads) = core.worker_threads {
                if threads == 0 || threads > 64 {
                    return Err(ApiError::Configuration {
                        message: "Worker threads must be between 1 and 64".to_string(),
                    });
                }
            }
        }
        
        // Validate graph config
        if let Some(graph) = &config.graph {
            if let Some(max_nodes) = graph.max_nodes {
                if max_nodes == 0 {
                    return Err(ApiError::Configuration {
                        message: "Maximum nodes must be greater than 0".to_string(),
                    });
                }
            }
        }
        
        // Validate analysis config
        if let Some(analysis) = &config.analysis {
            if analysis.project_root.is_empty() {
                return Err(ApiError::Configuration {
                    message: "Project root cannot be empty".to_string(),
                });
            }
            
            if !std::path::Path::new(&analysis.project_root).exists() {
                return Err(ApiError::Configuration {
                    message: format!("Project root does not exist: {}", analysis.project_root),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate graph operation parameters
    pub fn validate_graph_params(graph_id: &str, node_count: Option<u32>) -> ApiResult<()> {
        if graph_id.is_empty() {
            return Err(ApiError::Configuration {
                message: "Graph ID cannot be empty".to_string(),
            });
        }
        
        if let Some(count) = node_count {
            if count > 1_000_000 {
                return Err(ApiError::Configuration {
                    message: "Node count exceeds maximum limit".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate analysis parameters
    pub fn validate_analysis_params(project_root: &str, languages: &[String]) -> ApiResult<()> {
        if project_root.is_empty() {
            return Err(ApiError::Configuration {
                message: "Project root cannot be empty".to_string(),
            });
        }
        
        // Validate supported languages
        let supported_languages = vec![
            "rust", "javascript", "typescript", "python", "java", "go", "cpp", "csharp"
        ];
        
        for lang in languages {
            if !supported_languages.contains(&lang.as_str()) {
                return Err(ApiError::Configuration {
                    message: format!("Unsupported language: {}", lang),
                });
            }
        }
        
        Ok(())
    }
}
