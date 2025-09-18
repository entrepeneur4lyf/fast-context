//! # Core Domain
//!
//! Shared utilities, abstractions, and common functionality used across all domains.
//! This module provides the foundation for architectural harmony.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core error types used across all domains
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Resource error: {resource} - {message}")]
    Resource { resource: String, message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type for core operations
pub type CoreResult<T> = Result<T, CoreError>;

/// Configuration management for all domains
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreConfig {
    /// Global feature flags
    pub features: FeatureFlags,

    /// Performance settings
    pub performance: PerformanceConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Plugin configuration
    pub plugins: HashMap<String, serde_json::Value>,
}

/// Feature flags for enabling/disabling functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable graph algorithms
    pub enable_graph: bool,

    /// Enable codebase analysis
    pub enable_analysis: bool,

    /// Enable caching
    pub enable_caching: bool,

    /// Enable file watching
    pub enable_watching: bool,

    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_graph: true,
            enable_analysis: true,
            enable_caching: true,
            enable_watching: true,
            enable_monitoring: false,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Number of worker threads
    pub worker_threads: usize,

    /// Enable parallel processing
    pub enable_parallel: bool,

    /// Batch size for processing
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            worker_threads: num_cpus::get(),
            enable_parallel: true,
            batch_size: 100,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,

    /// Enable structured logging
    pub structured: bool,

    /// Log file path (optional)
    pub file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            file_path: None,
        }
    }
}

/// Metrics collection for performance monitoring
#[derive(Debug, Clone)]
pub struct Metrics {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    timers: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Increment a counter
    pub async fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Record a timing measurement
    pub async fn record_timing(&self, name: &str, duration_ms: u64) {
        let mut timers = self.timers.write().await;
        timers
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration_ms);
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> u64 {
        let counters = self.counters.read().await;
        counters.get(name).copied().unwrap_or(0)
    }

    /// Get average timing
    pub async fn get_average_timing(&self, name: &str) -> Option<f64> {
        let timers = self.timers.read().await;
        if let Some(times) = timers.get(name) {
            if !times.is_empty() {
                let sum: u64 = times.iter().sum();
                Some(sum as f64 / times.len() as f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().await;
        gauges.get(name).copied()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate file path
    pub fn validate_path(path: &str) -> CoreResult<()> {
        if path.is_empty() {
            return Err(CoreError::Validation {
                field: "path".to_string(),
                message: "Path cannot be empty".to_string(),
            });
        }

        if !std::path::Path::new(path).exists() {
            return Err(CoreError::Validation {
                field: "path".to_string(),
                message: format!("Path does not exist: {path}"),
            });
        }

        Ok(())
    }

    /// Validate memory limit
    pub fn validate_memory_limit(limit_mb: usize) -> CoreResult<()> {
        if limit_mb == 0 {
            return Err(CoreError::Validation {
                field: "memory_limit".to_string(),
                message: "Memory limit must be greater than 0".to_string(),
            });
        }

        if limit_mb > 16384 {
            // 16GB limit
            return Err(CoreError::Validation {
                field: "memory_limit".to_string(),
                message: "Memory limit too high (max 16GB)".to_string(),
            });
        }

        Ok(())
    }
}
