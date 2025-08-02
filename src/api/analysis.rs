//! # Analysis API
//!
//! Domain-specific API for codebase analysis operations

use crate::domains::analysis::AnalysisEngine;
use napi_derive::napi;
use ts_rs::TS;
use serde::{Deserialize, Serialize};

/// Analysis-specific configuration for API
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnalysisApiConfig {
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

/// Analysis session info for API responses
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnalysisSessionApi {
    pub id: String,
    pub project_root: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub files_analyzed: u32,
    pub symbols_extracted: u32,
    pub errors_encountered: u32,
}

/// Analysis API wrapper
pub struct AnalysisApi {
    engine: Option<AnalysisEngine>,
}

impl AnalysisApi {
    pub fn new() -> Self {
        Self { engine: None }
    }
    
    pub fn set_engine(&mut self, engine: AnalysisEngine) {
        self.engine = Some(engine);
    }
    
    pub fn is_available(&self) -> bool {
        self.engine.is_some()
    }
}

impl Default for AnalysisApi {
    fn default() -> Self {
        Self::new()
    }
}
