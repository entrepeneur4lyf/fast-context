//! # Error Tracking System
//!
//! Comprehensive error tracking and reporting for the Fast-Context analyzer.
//! Provides real-time error monitoring, categorization, and diagnostic information.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Maximum number of errors to keep in memory
const MAX_ERROR_HISTORY: usize = 100;

/// Error tracking manager
#[derive(Debug, Clone)]
pub struct ErrorTracker {
    inner: Arc<Mutex<ErrorTrackerInner>>,
}

#[derive(Debug)]
struct ErrorTrackerInner {
    errors: VecDeque<TrackedError>,
    error_counts: HashMap<ErrorCategory, u32>,
    last_error_time: Option<SystemTime>,
    session_start: SystemTime,
}

/// Tracked error with context and timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedError {
    pub id: String,
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub message: String,
    pub context: ErrorContext,
    pub timestamp: u64,
    pub stack_trace: Option<String>,
    pub recovery_suggestions: Vec<String>,
}

/// Error categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// File system operations
    FileSystem,
    /// Parsing and syntax analysis
    Parsing,
    /// Symbol extraction and analysis
    SymbolExtraction,
    /// Graph construction and manipulation
    GraphConstruction,
    /// Query execution and processing
    QueryExecution,
    /// Cache operations
    Cache,
    /// File watching and monitoring
    FileWatching,
    /// Configuration and validation
    Configuration,
    /// Memory and resource management
    Resource,
    /// Network and external dependencies
    Network,
    /// Unknown or uncategorized
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::FileSystem => write!(f, "File System"),
            ErrorCategory::Parsing => write!(f, "Parsing"),
            ErrorCategory::SymbolExtraction => write!(f, "Symbol Extraction"),
            ErrorCategory::GraphConstruction => write!(f, "Graph Construction"),
            ErrorCategory::QueryExecution => write!(f, "Query Execution"),
            ErrorCategory::Cache => write!(f, "Cache"),
            ErrorCategory::FileWatching => write!(f, "File Watching"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational messages
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions that don't stop execution
    Error,
    /// Critical errors that may cause system failure
    Critical,
    /// Fatal errors that require immediate attention
    Fatal,
}

/// Context information for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub file_path: Option<String>,
    pub symbol_name: Option<String>,
    pub language: Option<String>,
    pub additional_data: HashMap<String, String>,
}

/// Error statistics and summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub total_errors: u32,
    pub errors_by_category: HashMap<ErrorCategory, u32>,
    pub errors_by_severity: HashMap<ErrorSeverity, u32>,
    pub last_error_time: Option<u64>,
    pub session_duration_seconds: u64,
    pub error_rate_per_minute: f64,
    pub most_common_category: Option<ErrorCategory>,
    pub recent_errors: Vec<TrackedError>,
}

/// Comprehensive error types for the analyzer
#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        path: Option<String>,
    },

    #[error("Parsing error in {language}: {message}")]
    Parsing {
        language: String,
        message: String,
        file_path: Option<String>,
    },

    #[error("Symbol extraction failed: {message}")]
    SymbolExtraction {
        message: String,
        symbol: Option<String>,
    },

    #[error("Graph construction error: {message}")]
    GraphConstruction { message: String },

    #[error("Query execution failed: {message}")]
    QueryExecution {
        message: String,
        query: Option<String>,
    },

    #[error("Cache operation failed: {message}")]
    Cache {
        message: String,
        operation: Option<String>,
    },

    #[error("File watching error: {message}")]
    FileWatching {
        message: String,
        path: Option<String>,
    },

    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
    },

    #[error("Resource error: {message}")]
    Resource {
        message: String,
        resource_type: Option<String>,
    },

    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
    },
}

impl ErrorTracker {
    /// Create a new error tracker
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ErrorTrackerInner {
                errors: VecDeque::new(),
                error_counts: HashMap::new(),
                last_error_time: None,
                session_start: SystemTime::now(),
            })),
        }
    }

    /// Track a new error
    pub fn track_error(&self, error: AnalyzerError, context: ErrorContext) {
        let tracked_error = self.create_tracked_error(error, context);

        if let Ok(mut inner) = self.inner.lock() {
            // Add to error history
            inner.errors.push_back(tracked_error.clone());

            // Maintain size limit
            while inner.errors.len() > MAX_ERROR_HISTORY {
                inner.errors.pop_front();
            }

            // Update counts
            *inner
                .error_counts
                .entry(tracked_error.category.clone())
                .or_insert(0) += 1;
            inner.last_error_time = Some(SystemTime::now());
        }
    }

    /// Get error summary and statistics
    pub fn get_summary(&self) -> ErrorSummary {
        if let Ok(inner) = self.inner.lock() {
            let session_duration = inner
                .session_start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs();

            let total_errors = inner.errors.len() as u32;
            let error_rate = if session_duration > 0 {
                (total_errors as f64) / (session_duration as f64 / 60.0)
            } else {
                0.0
            };

            let most_common_category = inner
                .error_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(category, _)| category.clone());

            let mut errors_by_severity = HashMap::new();
            for error in &inner.errors {
                *errors_by_severity
                    .entry(error.severity.clone())
                    .or_insert(0) += 1;
            }

            ErrorSummary {
                total_errors,
                errors_by_category: inner.error_counts.clone(),
                errors_by_severity,
                last_error_time: inner
                    .last_error_time
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
                session_duration_seconds: session_duration,
                error_rate_per_minute: error_rate,
                most_common_category,
                recent_errors: inner.errors.iter().rev().take(10).cloned().collect(),
            }
        } else {
            ErrorSummary {
                total_errors: 0,
                errors_by_category: HashMap::new(),
                errors_by_severity: HashMap::new(),
                last_error_time: None,
                session_duration_seconds: 0,
                error_rate_per_minute: 0.0,
                most_common_category: None,
                recent_errors: Vec::new(),
            }
        }
    }

    /// Get recent errors with optional filtering
    pub fn get_recent_errors(
        &self,
        limit: Option<usize>,
        category: Option<ErrorCategory>,
    ) -> Vec<TrackedError> {
        if let Ok(inner) = self.inner.lock() {
            let limit = limit.unwrap_or(10);
            inner
                .errors
                .iter()
                .rev()
                .filter(|error| category.as_ref().is_none_or(|cat| &error.category == cat))
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear error history
    pub fn clear_errors(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.errors.clear();
            inner.error_counts.clear();
            inner.last_error_time = None;
        }
    }

    /// Check if there are any critical or fatal errors
    pub fn has_critical_errors(&self) -> bool {
        if let Ok(inner) = self.inner.lock() {
            inner.errors.iter().any(|error| {
                matches!(
                    error.severity,
                    ErrorSeverity::Critical | ErrorSeverity::Fatal
                )
            })
        } else {
            false
        }
    }

    fn create_tracked_error(&self, error: AnalyzerError, context: ErrorContext) -> TrackedError {
        let (category, severity, recovery_suggestions) = self.categorize_error(&error);

        TrackedError {
            id: self.generate_error_id(),
            category,
            severity,
            message: error.to_string(),
            context,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
            stack_trace: None, // Could be enhanced with backtrace
            recovery_suggestions,
        }
    }

    fn categorize_error(
        &self,
        error: &AnalyzerError,
    ) -> (ErrorCategory, ErrorSeverity, Vec<String>) {
        match error {
            AnalyzerError::FileSystem { .. } => (
                ErrorCategory::FileSystem,
                ErrorSeverity::Error,
                vec![
                    "Check file permissions and disk space".to_string(),
                    "Verify the file path exists and is accessible".to_string(),
                ],
            ),
            AnalyzerError::Parsing { .. } => (
                ErrorCategory::Parsing,
                ErrorSeverity::Warning,
                vec![
                    "Check file syntax and encoding".to_string(),
                    "Ensure the file is valid for the detected language".to_string(),
                ],
            ),
            AnalyzerError::SymbolExtraction { .. } => (
                ErrorCategory::SymbolExtraction,
                ErrorSeverity::Warning,
                vec![
                    "Review symbol definitions and syntax".to_string(),
                    "Check for unsupported language features".to_string(),
                ],
            ),
            AnalyzerError::GraphConstruction { .. } => (
                ErrorCategory::GraphConstruction,
                ErrorSeverity::Error,
                vec![
                    "Check for circular dependencies".to_string(),
                    "Verify symbol relationships are valid".to_string(),
                ],
            ),
            AnalyzerError::QueryExecution { .. } => (
                ErrorCategory::QueryExecution,
                ErrorSeverity::Error,
                vec![
                    "Simplify the query or add more specific filters".to_string(),
                    "Ensure the analyzer has been initialized".to_string(),
                ],
            ),
            AnalyzerError::Cache { .. } => (
                ErrorCategory::Cache,
                ErrorSeverity::Warning,
                vec![
                    "Clear cache and retry operation".to_string(),
                    "Check available disk space".to_string(),
                ],
            ),
            AnalyzerError::FileWatching { .. } => (
                ErrorCategory::FileWatching,
                ErrorSeverity::Warning,
                vec![
                    "Restart file watching if needed".to_string(),
                    "Check system file descriptor limits".to_string(),
                ],
            ),
            AnalyzerError::Configuration { .. } => (
                ErrorCategory::Configuration,
                ErrorSeverity::Error,
                vec![
                    "Review configuration settings".to_string(),
                    "Check configuration file syntax".to_string(),
                ],
            ),
            AnalyzerError::Resource { .. } => (
                ErrorCategory::Resource,
                ErrorSeverity::Critical,
                vec![
                    "Free up system resources".to_string(),
                    "Reduce analysis scope or enable streaming".to_string(),
                ],
            ),
            AnalyzerError::Network { .. } => (
                ErrorCategory::Network,
                ErrorSeverity::Warning,
                vec![
                    "Check network connectivity".to_string(),
                    "Retry operation after network issues are resolved".to_string(),
                ],
            ),
        }
    }

    fn generate_error_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        format!("ERR_{:x}", hasher.finish())
    }
}

impl Default for ErrorTracker {
    fn default() -> Self {
        Self::new()
    }
}
