//! # Graph Domain
//!
//! Pure graph algorithms and data structures with no external dependencies.
//! This domain provides the computational foundation for graph-based analysis.

use super::core::{CoreError, Metrics};
use super::Domain;
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Graph domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Enable parallel algorithms
    pub enable_parallel: bool,

    /// Maximum nodes per graph
    pub max_nodes: usize,

    /// Maximum edges per graph
    pub max_edges: usize,

    /// Enable graph caching
    pub enable_caching: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            enable_parallel: true,
            max_nodes: 1_000_000,
            max_edges: 10_000_000,
            enable_caching: true,
        }
    }
}

/// Graph domain errors
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Graph limit exceeded: {limit_type} - {message}")]
    LimitExceeded { limit_type: String, message: String },

    #[error("Invalid graph operation: {operation} - {message}")]
    InvalidOperation { operation: String, message: String },

    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: String },

    #[error("Edge not found: {edge_id}")]
    EdgeNotFound { edge_id: String },

    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Graph types supported by the engine
#[derive(Debug, Clone)]
pub enum GraphType {
    /// Undirected graph
    Undirected(UnGraph<String, f64>),
    /// Directed graph
    Directed(DiGraph<String, f64>),
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Graph container with metadata
pub struct ManagedGraph {
    pub metadata: GraphMetadata,
    pub graph: GraphType,
}

/// Graph engine - pure graph algorithms and operations
pub struct GraphEngine {
    config: GraphConfig,
    graphs: HashMap<String, ManagedGraph>,
    metrics: Arc<Metrics>,
}

impl GraphEngine {
    /// Create a new graph engine
    pub fn new(config: GraphConfig, metrics: Arc<Metrics>) -> Self {
        Self {
            config,
            graphs: HashMap::new(),
            metrics,
        }
    }

    /// Create a new undirected graph
    pub async fn create_undirected_graph(
        &mut self,
        id: String,
        name: String,
    ) -> Result<(), GraphError> {
        self.validate_graph_limits(0, 0)?;

        let graph = GraphType::Undirected(UnGraph::new_undirected());
        let metadata = GraphMetadata {
            id: id.clone(),
            name,
            description: None,
            node_count: 0,
            edge_count: 0,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };

        self.graphs
            .insert(id.clone(), ManagedGraph { metadata, graph });
        self.metrics.increment_counter("graphs_created").await;

        Ok(())
    }

    /// Create a new directed graph
    pub async fn create_directed_graph(
        &mut self,
        id: String,
        name: String,
    ) -> Result<(), GraphError> {
        self.validate_graph_limits(0, 0)?;

        let graph = GraphType::Directed(DiGraph::new());
        let metadata = GraphMetadata {
            id: id.clone(),
            name,
            description: None,
            node_count: 0,
            edge_count: 0,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
        };

        self.graphs
            .insert(id.clone(), ManagedGraph { metadata, graph });
        self.metrics.increment_counter("graphs_created").await;

        Ok(())
    }

    /// Add node to graph
    pub async fn add_node(&mut self, graph_id: &str, node_data: String) -> Result<u32, GraphError> {
        // Validate limits first
        if let Some(managed_graph) = self.graphs.get(graph_id) {
            self.validate_graph_limits(
                managed_graph.metadata.node_count + 1,
                managed_graph.metadata.edge_count,
            )?;
        } else {
            return Err(GraphError::NodeNotFound {
                node_id: graph_id.to_string(),
            });
        }

        let managed_graph =
            self.graphs
                .get_mut(graph_id)
                .ok_or_else(|| GraphError::NodeNotFound {
                    node_id: graph_id.to_string(),
                })?;

        let node_index = match &mut managed_graph.graph {
            GraphType::Undirected(graph) => graph.add_node(node_data),
            GraphType::Directed(graph) => graph.add_node(node_data),
        };

        managed_graph.metadata.node_count += 1;
        managed_graph.metadata.modified_at = chrono::Utc::now();

        self.metrics.increment_counter("nodes_added").await;

        Ok(node_index.index() as u32)
    }

    /// Add edge to graph
    pub async fn add_edge(
        &mut self,
        graph_id: &str,
        source: u32,
        target: u32,
        weight: f64,
    ) -> Result<Option<u32>, GraphError> {
        // Validate limits first
        if let Some(managed_graph) = self.graphs.get(graph_id) {
            self.validate_graph_limits(
                managed_graph.metadata.node_count,
                managed_graph.metadata.edge_count + 1,
            )?;
        } else {
            return Err(GraphError::NodeNotFound {
                node_id: graph_id.to_string(),
            });
        }

        let managed_graph =
            self.graphs
                .get_mut(graph_id)
                .ok_or_else(|| GraphError::NodeNotFound {
                    node_id: graph_id.to_string(),
                })?;

        let source_idx = NodeIndex::new(source as usize);
        let target_idx = NodeIndex::new(target as usize);

        let edge_index = match &mut managed_graph.graph {
            GraphType::Undirected(graph) => {
                if graph.node_weight(source_idx).is_some()
                    && graph.node_weight(target_idx).is_some()
                {
                    Some(graph.add_edge(source_idx, target_idx, weight))
                } else {
                    return Err(GraphError::InvalidOperation {
                        operation: "add_edge".to_string(),
                        message: "Source or target node does not exist".to_string(),
                    });
                }
            }
            GraphType::Directed(graph) => {
                if graph.node_weight(source_idx).is_some()
                    && graph.node_weight(target_idx).is_some()
                {
                    Some(graph.add_edge(source_idx, target_idx, weight))
                } else {
                    return Err(GraphError::InvalidOperation {
                        operation: "add_edge".to_string(),
                        message: "Source or target node does not exist".to_string(),
                    });
                }
            }
        };

        if edge_index.is_some() {
            managed_graph.metadata.edge_count += 1;
            managed_graph.metadata.modified_at = chrono::Utc::now();
            self.metrics.increment_counter("edges_added").await;
        }

        Ok(edge_index.map(|idx| idx.index() as u32))
    }

    /// Get graph metadata
    pub fn get_graph_metadata(&self, graph_id: &str) -> Option<&GraphMetadata> {
        self.graphs.get(graph_id).map(|g| &g.metadata)
    }

    /// List all graphs
    pub fn list_graphs(&self) -> Vec<&GraphMetadata> {
        self.graphs.values().map(|g| &g.metadata).collect()
    }

    /// Delete graph
    pub async fn delete_graph(&mut self, graph_id: &str) -> Result<(), GraphError> {
        if self.graphs.remove(graph_id).is_some() {
            self.metrics.increment_counter("graphs_deleted").await;
            Ok(())
        } else {
            Err(GraphError::NodeNotFound {
                node_id: graph_id.to_string(),
            })
        }
    }

    /// Validate graph limits
    fn validate_graph_limits(&self, nodes: usize, edges: usize) -> Result<(), GraphError> {
        if nodes > self.config.max_nodes {
            return Err(GraphError::LimitExceeded {
                limit_type: "nodes".to_string(),
                message: format!(
                    "Exceeds maximum nodes: {} > {}",
                    nodes, self.config.max_nodes
                ),
            });
        }

        if edges > self.config.max_edges {
            return Err(GraphError::LimitExceeded {
                limit_type: "edges".to_string(),
                message: format!(
                    "Exceeds maximum edges: {} > {}",
                    edges, self.config.max_edges
                ),
            });
        }

        Ok(())
    }

    /// Get the count of managed graphs
    pub fn get_graph_count(&self) -> u32 {
        self.graphs.len() as u32
    }
}

impl Domain for GraphEngine {
    type Config = GraphConfig;
    type Error = GraphError;

    fn initialize(config: Self::Config) -> Result<Self, Self::Error> {
        let metrics = Arc::new(Metrics::new());
        Ok(Self::new(config, metrics))
    }

    fn domain_name(&self) -> &'static str {
        "graph"
    }

    fn health_check(&self) -> Result<(), Self::Error> {
        // Basic health checks
        if self.graphs.len() > 1000 {
            return Err(GraphError::LimitExceeded {
                limit_type: "graph_count".to_string(),
                message: "Too many graphs in memory".to_string(),
            });
        }

        Ok(())
    }
}
