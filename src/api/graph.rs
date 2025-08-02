//! # Graph API
//!
//! Domain-specific API for graph operations

use crate::domains::graph::GraphEngine;
use napi_derive::napi;
use ts_rs::TS;
use serde::{Deserialize, Serialize};

/// Graph-specific configuration for API
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GraphApiConfig {
    /// Enable parallel algorithms
    pub enable_parallel: Option<bool>,
    
    /// Maximum nodes per graph
    pub max_nodes: Option<u32>,
    
    /// Maximum edges per graph
    pub max_edges: Option<u32>,
}

/// Graph metadata for API responses
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GraphMetadataApi {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_count: u32,
    pub edge_count: u32,
    pub created_at: String,
    pub modified_at: String,
}

/// Graph API wrapper
pub struct GraphApi {
    engine: Option<GraphEngine>,
}

impl GraphApi {
    pub fn new() -> Self {
        Self { engine: None }
    }
    
    pub fn set_engine(&mut self, engine: GraphEngine) {
        self.engine = Some(engine);
    }
    
    pub fn is_available(&self) -> bool {
        self.engine.is_some()
    }
}

impl Default for GraphApi {
    fn default() -> Self {
        Self::new()
    }
}
