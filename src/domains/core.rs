//! # Core Domain
//!
//! Shared utilities, abstractions, and common functionality used across all domains.
//! This module provides the foundation for architectural harmony.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::System;

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

/// Performance histogram for statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    values: VecDeque<f64>,
    max_size: usize,
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl Histogram {
    pub fn new(max_size: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(max_size),
            max_size,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            count: 0,
        }
    }

    pub fn record(&mut self, value: f64) {
        if self.values.len() >= self.max_size {
            if let Some(removed) = self.values.pop_front() {
                self.sum -= removed;
                self.count -= 1;
            }
        }
        
        self.values.push_back(value);
        self.sum += value;
        self.count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.sum / self.count as f64 }
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.values.is_empty() { return None; }
        
        let mut sorted: Vec<f64> = self.values.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (p / 100.0 * (sorted.len() - 1) as f64) as usize;
        Some(sorted[index.min(sorted.len() - 1)])
    }
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
    pub process_cpu_usage_percent: f64,
    pub process_memory_mb: u64,
    pub thread_count: usize,
}

/// Enhanced metrics collection for comprehensive performance monitoring
#[derive(Debug, Clone)]
pub struct Metrics {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    timers: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
    system_metrics: Arc<RwLock<Option<SystemMetrics>>>,
    performance_thresholds: Arc<RwLock<HashMap<String, (f64, f64)>>>, // (warning, critical)
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(None)),
            performance_thresholds: Arc::new(RwLock::new(HashMap::new())),
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

    /// Record a value in a histogram for statistical analysis
    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Histogram::new(1000))
            .record(value);
    }

    /// Get histogram statistics
    pub async fn get_histogram_stats(&self, name: &str) -> Option<Histogram> {
        let histograms = self.histograms.read().await;
        histograms.get(name).cloned()
    }

    /// Update system metrics
    pub async fn update_system_metrics(&self) {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let current_pid = std::process::id();
        let process = sys.process(sysinfo::Pid::from_u32(current_pid));
        
        let metrics = SystemMetrics {
            cpu_usage_percent: sys.global_cpu_info().cpu_usage().into(),
            memory_usage_mb: sys.used_memory(),
            memory_total_mb: sys.total_memory(),
            memory_usage_percent: (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
            process_cpu_usage_percent: process.map_or(0.0, |p| p.cpu_usage().into()),
            process_memory_mb: process.map_or(0, |p| p.memory()),
            thread_count: 0, // Thread count not available in current sysinfo version
        };
        
        let mut system_metrics = self.system_metrics.write().await;
        *system_metrics = Some(metrics);
    }

    /// Get current system metrics
    pub async fn get_system_metrics(&self) -> Option<SystemMetrics> {
        let system_metrics = self.system_metrics.read().await;
        system_metrics.clone()
    }

    /// Set performance threshold (warning, critical)
    pub async fn set_threshold(&self, metric_name: &str, warning: f64, critical: f64) {
        let mut thresholds = self.performance_thresholds.write().await;
        thresholds.insert(metric_name.to_string(), (warning, critical));
    }

    /// Check if metric exceeds thresholds, returns severity level
    pub async fn check_threshold(&self, metric_name: &str, value: f64) -> Option<&'static str> {
        let thresholds = self.performance_thresholds.read().await;
        thresholds.get(metric_name).and_then(|&(warning, critical)| {
            if value >= critical {
                Some("critical")
            } else if value >= warning {
                Some("warning")
            } else {
                None
            }
        })
    }

    /// Get comprehensive performance report
    pub async fn get_performance_report(&self) -> HashMap<String, serde_json::Value> {
        let mut report = HashMap::new();
        
        // Counter metrics
        let counters = self.counters.read().await;
        report.insert("counters".to_string(), serde_json::json!(counters.clone()));
        
        // Timer statistics
        let timers = self.timers.read().await;
        let mut timer_stats = HashMap::new();
        for (name, times) in timers.iter() {
            if !times.is_empty() {
                let sum: u64 = times.iter().sum();
                let avg = sum as f64 / times.len() as f64;
                let min = times.iter().min().copied().unwrap_or(0);
                let max = times.iter().max().copied().unwrap_or(0);
                timer_stats.insert(name.clone(), serde_json::json!({
                    "count": times.len(),
                    "average_ms": avg,
                    "min_ms": min,
                    "max_ms": max,
                    "total_ms": sum
                }));
            }
        }
        report.insert("timers".to_string(), serde_json::json!(timer_stats));
        
        // System metrics
        if let Some(sys_metrics) = self.get_system_metrics().await {
            report.insert("system".to_string(), serde_json::json!(sys_metrics));
        }
        
        report
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
