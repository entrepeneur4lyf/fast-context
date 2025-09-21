//! # Comprehensive Error Management System
//!
//! This module provides unified error handling across all components of the Fast-Context analyzer,
//! including both error type definitions and advanced error tracking capabilities.
//!
//! ## Modules
//!
//! - `types`: Core error type definitions and standardized error handling
//! - `tracking`: Advanced error tracking, monitoring, and diagnostic capabilities

pub mod tracking;

use thiserror::Error;
use std::path::PathBuf;

#[cfg(feature = "python")]
use pyo3::IntoPy;

/// Main error type for the Fast-Context analyzer
#[derive(Debug, Error)]
pub enum FastContextError {
    // === File System Errors ===
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    // === Parsing Errors ===
    #[error("Parsing error in {language}: {message}")]
    Parsing {
        language: String,
        message: String,
        file_path: Option<PathBuf>,
        line: Option<usize>,
        column: Option<usize>,
    },

    #[error("Unsupported language: {language}")]
    UnsupportedLanguage { language: String },

    #[error("Invalid syntax in {file_path}: {message}")]
    InvalidSyntax {
        file_path: PathBuf,
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    // === Symbol Extraction Errors ===
    #[error("Symbol extraction failed: {message}")]
    SymbolExtraction {
        message: String,
        file_path: Option<PathBuf>,
        symbol_name: Option<String>,
    },

    #[error("Dependency extraction failed: {message}")]
    DependencyExtraction {
        message: String,
        file_path: Option<PathBuf>,
        from_symbol: Option<String>,
        to_symbol: Option<String>,
    },

    // === Analysis Errors ===
    #[error("Analysis failed: {message}")]
    Analysis {
        message: String,
        phase: Option<String>,
        file_count: Option<usize>,
    },

    #[error("Graph construction error: {message}")]
    GraphConstruction {
        message: String,
        node_count: Option<usize>,
        edge_count: Option<usize>,
    },

    #[error("Graph operation failed: {operation}: {message}")]
    Graph {
        operation: String,
        message: String,
    },

    #[error("Query execution failed: {message}")]
    QueryExecution {
        message: String,
        query: Option<String>,
        result_count: Option<usize>,
    },

    // === Cache Errors ===
    #[error("Cache operation failed: {operation}: {message}")]
    Cache {
        operation: String,
        message: String,
        cache_type: Option<String>,
    },

    #[error("Cache corruption detected: {message}")]
    CacheCorruption {
        message: String,
        cache_path: Option<PathBuf>,
    },

    // === Configuration Errors ===
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
        value: Option<String>,
    },

    #[error("Invalid project root: {path}")]
    InvalidProjectRoot { path: PathBuf },

    #[error("Missing required configuration: {field}")]
    MissingConfiguration { field: String },

    // === Resource Errors ===
    #[error("Resource limit exceeded: {resource}: {message}")]
    ResourceLimit {
        resource: String,
        message: String,
        current: Option<usize>,
        limit: Option<usize>,
    },

    #[error("Out of memory: {message}")]
    OutOfMemory {
        message: String,
        requested: Option<usize>,
        available: Option<usize>,
    },

    // === Thread Safety Errors ===
    #[error("Thread synchronization error: {message}")]
    ThreadSync {
        message: String,
        operation: Option<String>,
    },

    #[error("Lock acquisition failed: {message}")]
    LockFailed {
        message: String,
        lock_type: Option<String>,
    },

    // === Export Errors ===
    #[error("Export failed: {format}: {message}")]
    Export {
        format: String,
        message: String,
        output_path: Option<PathBuf>,
    },

    // === File Watching Errors ===
    #[error("File watching error: {message}")]
    FileWatching {
        message: String,
        path: Option<PathBuf>,
    },

    // === Network Errors ===
    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
    },

    // === Internal Errors ===
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        component: Option<String>,
    },

    // === Validation Errors ===
    #[error("Validation error: {field}: {message}")]
    Validation {
        field: String,
        message: String,
        value: Option<String>,
    },
}

/// Result type using the standardized error
pub type FastContextResult<T> = Result<T, FastContextError>;

/// Convenience constructors for common error patterns
impl FastContextError {
    /// Create a file system error with context
    pub fn file_system(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::FileSystem {
            message: message.into(),
            path,
            source: None,
        }
    }

    /// Create a file system error from an IO error
    pub fn from_io_error(error: std::io::Error, path: Option<PathBuf>) -> Self {
        Self::FileSystem {
            message: error.to_string(),
            path,
            source: Some(error),
        }
    }

    /// Create a parsing error with location
    pub fn parsing_error(
        language: impl Into<String>,
        message: impl Into<String>,
        file_path: Option<PathBuf>,
        line: Option<usize>,
        column: Option<usize>,
    ) -> Self {
        Self::Parsing {
            language: language.into(),
            message: message.into(),
            file_path,
            line,
            column,
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>, field: Option<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field,
            value: None,
        }
    }

    /// Create an analysis error
    pub fn analysis_error(message: impl Into<String>, phase: Option<String>) -> Self {
        Self::Analysis {
            message: message.into(),
            phase,
            file_count: None,
        }
    }

    /// Create a thread synchronization error
    pub fn thread_sync_error(message: impl Into<String>, operation: Option<String>) -> Self {
        Self::ThreadSync {
            message: message.into(),
            operation,
        }
    }

    /// Create a cache error
    pub fn cache_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
            message: message.into(),
            cache_type: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>, component: Option<String>) -> Self {
        Self::Internal {
            message: message.into(),
            component,
        }
    }
}

/// Convert from domain-specific errors
impl From<crate::domains::core::CoreError> for FastContextError {
    fn from(err: crate::domains::core::CoreError) -> Self {
        match err {
            crate::domains::core::CoreError::Configuration { message } => {
                Self::config_error(message, None)
            }
            crate::domains::core::CoreError::Validation { field, message } => Self::Validation {
                field,
                message,
                value: None,
            },
            crate::domains::core::CoreError::Resource { resource, message } => {
                Self::ResourceLimit {
                    resource,
                    message,
                    current: None,
                    limit: None,
                }
            }
            crate::domains::core::CoreError::Internal { message } => {
                Self::internal_error(message, Some("core".to_string()))
            }
        }
    }
}

/// Convert from String errors (for backward compatibility)
impl From<String> for FastContextError {
    fn from(message: String) -> Self {
        Self::internal_error(message, None)
    }
}

/// Convert from &str errors (for convenience)
impl From<&str> for FastContextError {
    fn from(message: &str) -> Self {
        Self::internal_error(message.to_string(), None)
    }
}

/// Convert from IO errors
impl From<std::io::Error> for FastContextError {
    fn from(err: std::io::Error) -> Self {
        Self::from_io_error(err, None)
    }
}

/// Convert to NAPI errors for Node.js bindings
#[cfg(feature = "nodejs")]
impl From<FastContextError> for napi::Error {
    fn from(err: FastContextError) -> Self {
        napi::Error::from_reason(err.to_string())
    }
}

/// Convert to PyO3 errors for Python bindings
#[cfg(feature = "python")]
impl From<FastContextError> for pyo3::PyErr {
    fn from(err: FastContextError) -> Self {
        match err {
            FastContextError::FileNotFound { .. } => {
                pyo3::exceptions::PyFileNotFoundError::new_err(err.to_string())
            }
            FastContextError::PermissionDenied { .. } => {
                pyo3::exceptions::PyPermissionError::new_err(err.to_string())
            }
            FastContextError::Configuration { .. } | FastContextError::Validation { .. } => {
                pyo3::exceptions::PyValueError::new_err(err.to_string())
            }
            FastContextError::OutOfMemory { .. } => {
                pyo3::exceptions::PyMemoryError::new_err(err.to_string())
            }
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}

/// Implement PyO3 conversion traits for error handling
#[cfg(feature = "python")]
impl pyo3::IntoPy<pyo3::PyObject> for FastContextError {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        let pyerr: pyo3::PyErr = self.into();
        pyerr.into_py(py)
    }
}

#[cfg(feature = "python")]
impl pyo3::ToPyObject for FastContextError {
    fn to_object(&self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        // Convert the error to a Python string representation
        self.to_string().into_py(py)
    }
}
