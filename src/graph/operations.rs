//! Graph Operations Abstraction
//!
//! This module provides a trait-based abstraction for graph operations,
//! decoupling the codebase from specific graph implementations like petgraph.
//! This enables easier testing, implementation switching, and maintainability.

use crate::errors::FastContextResult;
use std::fmt::Debug;

/// Graph direction for traversal operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphDirection {
    /// Incoming edges (dependencies, callers, etc.)
    Incoming,
    /// Outgoing edges (dependents, callees, etc.)
    Outgoing,
}

/// Node identifier abstraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Create a new NodeId from a usize
    pub fn new(id: usize) -> Self {
        NodeId(id)
    }

    /// Get the underlying usize value
    pub fn as_usize(&self) -> usize {
        self.0
    }

    /// Convert from petgraph NodeIndex
    pub fn from_petgraph(index: petgraph::graph::NodeIndex) -> Self {
        NodeId(index.index())
    }

    /// Convert to petgraph NodeIndex
    pub fn to_petgraph(&self) -> petgraph::graph::NodeIndex {
        petgraph::graph::NodeIndex::new(self.0)
    }
}

/// Edge identifier abstraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(usize);

impl EdgeId {
    /// Create a new EdgeId from a usize
    pub fn new(id: usize) -> Self {
        EdgeId(id)
    }

    /// Get the underlying usize value
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// Graph metadata and statistics
#[derive(Debug, Clone)]
pub struct GraphStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Average degree (average connections per node)
    pub average_degree: f64,
    /// Maximum degree (most connected node)
    pub max_degree: usize,
    /// Number of connected components
    pub connected_components: usize,
    /// Graph density (ratio of actual edges to possible edges)
    pub density: f64,
    /// Whether the graph contains cycles
    pub has_cycles_flag: bool,
}

impl GraphStats {
    /// Check if the graph contains cycles
    pub fn has_cycles(&self) -> bool {
        self.has_cycles_flag
    }
}

/// Path analysis result
#[derive(Debug, Clone)]
pub struct PathResult {
    /// Path exists between source and target
    pub exists: bool,
    /// Length of shortest path (if exists)
    pub length: Option<usize>,
    /// Actual path nodes (if requested and exists)
    pub path: Option<Vec<NodeId>>,
    /// Total path weight/cost
    pub weight: Option<f64>,
}

/// Strongly connected component analysis
#[derive(Debug, Clone)]
pub struct ComponentAnalysis {
    /// List of strongly connected components
    pub components: Vec<Vec<NodeId>>,
    /// Number of components
    pub component_count: usize,
    /// Largest component size
    pub largest_component_size: usize,
    /// Whether the graph is strongly connected
    pub is_strongly_connected: bool,
}

/// Graph operations trait providing abstraction over graph implementations
pub trait GraphOperations<N, E>: Send + Sync + Debug
where
    N: Clone + Send + Sync + Debug,
    E: Clone + Send + Sync + Debug,
{
    // === Node Operations ===

    /// Add a node to the graph
    fn add_node(&mut self, node: N) -> FastContextResult<NodeId>;

    /// Get node data by ID
    fn get_node(&self, node_id: NodeId) -> Option<&N>;

    /// Get mutable node data by ID
    fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut N>;

    /// Remove a node from the graph
    fn remove_node(&mut self, node_id: NodeId) -> FastContextResult<()>;

    /// Check if a node exists
    fn contains_node(&self, node_id: NodeId) -> bool;

    /// Get all node IDs in the graph
    fn node_ids(&self) -> Vec<NodeId>;

    // === Edge Operations ===

    /// Add an edge between two nodes
    fn add_edge(&mut self, source: NodeId, target: NodeId, edge: E) -> FastContextResult<EdgeId>;

    /// Get edge data by ID
    fn get_edge(&self, edge_id: EdgeId) -> Option<&E>;

    /// Get mutable edge data by ID
    fn get_edge_mut(&mut self, edge_id: EdgeId) -> Option<&mut E>;

    /// Remove an edge from the graph
    fn remove_edge(&mut self, edge_id: EdgeId) -> FastContextResult<()>;

    /// Get edge between two specific nodes (if any)
    fn find_edge(&self, source: NodeId, target: NodeId) -> Option<EdgeId>;

    // === Traversal Operations ===

    /// Get neighbors of a node in the specified direction
    fn neighbors(&self, node_id: NodeId, direction: GraphDirection) -> Vec<NodeId>;

    /// Get all edges connected to a node in the specified direction
    fn edges(&self, node_id: NodeId, direction: GraphDirection) -> Vec<(NodeId, EdgeId, NodeId)>;

    /// Get degree of a node (number of connections) in specified direction
    fn degree(&self, node_id: NodeId, direction: GraphDirection) -> usize;

    /// Get all nodes reachable from a starting node
    fn reachable_from(&self, start_node: NodeId, direction: GraphDirection) -> Vec<NodeId>;

    // === Path Operations ===

    /// Find shortest path between two nodes
    fn shortest_path(&self, source: NodeId, target: NodeId) -> FastContextResult<PathResult>;

    /// Find all paths between two nodes (up to a maximum length)
    fn all_paths(&self, source: NodeId, target: NodeId, max_length: Option<usize>) -> FastContextResult<Vec<PathResult>>;

    /// Check if two nodes are connected
    fn is_connected(&self, source: NodeId, target: NodeId) -> bool;

    /// Get distance (shortest path length) between two nodes
    fn distance(&self, source: NodeId, target: NodeId) -> Option<usize>;

    // === Analysis Operations ===

    /// Detect cycles in the graph
    fn has_cycles(&self) -> bool;

    /// Find all cycles in the graph
    fn find_cycles(&self) -> Vec<Vec<NodeId>>;

    /// Perform strongly connected component analysis
    fn strongly_connected_components(&self) -> ComponentAnalysis;

    /// Get graph statistics and metrics
    fn graph_stats(&self) -> GraphStats;

    /// Find nodes with specific properties (using a predicate)
    fn find_nodes_by_predicate(&self, predicate: Box<dyn Fn(&N) -> bool + Send + Sync>) -> Vec<NodeId>;

    /// Find edges with specific properties (using a predicate)
    fn find_edges_by_predicate(&self, predicate: Box<dyn Fn(&E) -> bool + Send + Sync>) -> Vec<EdgeId>;

    // === Graph Modification Operations ===

    /// Clear all nodes and edges from the graph
    fn clear(&mut self);

    /// Create a subgraph containing only specified nodes
    fn subgraph(&self, nodes: &[NodeId]) -> FastContextResult<Box<dyn GraphOperations<N, E>>>;

    /// Merge another graph into this one
    fn merge(&mut self, other: &dyn GraphOperations<N, E>) -> FastContextResult<()>;

    // === Utility Operations ===

    /// Clone the graph
    fn clone_graph(&self) -> Box<dyn GraphOperations<N, E>>;

    /// Get graph capacity information (if applicable)
    fn capacity(&self) -> Option<(usize, usize)> {
        None
    }

    /// Reserve capacity for nodes and edges (if applicable)
    fn reserve(&mut self, _nodes: usize, _edges: usize) {}

    /// Shrink capacity to fit actual size (if applicable)
    fn shrink_to_fit(&mut self) {}
}

/// Graph builder trait for constructing graphs
pub trait GraphBuilder<N, E>: Send + Sync {
    /// Create a new empty graph
    fn new() -> Self;

    /// Create a graph with estimated capacity
    fn with_capacity(nodes: usize, edges: usize) -> Self;

    /// Add a node and return its ID
    fn add_node(&mut self, node: N) -> NodeId;

    /// Add an edge between nodes
    fn add_edge(&mut self, source: NodeId, target: NodeId, edge: E) -> EdgeId;

    /// Build the final graph
    fn build(self) -> Box<dyn GraphOperations<N, E>>;
}

/// Factory trait for creating graph implementations
pub trait GraphFactory: Send + Sync {
    type Node: Send + Sync + Debug + Clone;
    type Edge: Send + Sync + Debug + Clone;
    type Graph: GraphOperations<Self::Node, Self::Edge>;
    type Builder: GraphBuilder<Self::Node, Self::Edge>;

    /// Create a new empty graph
    fn create_graph() -> Self::Graph;

    /// Create a graph with initial capacity
    fn create_graph_with_capacity(nodes: usize, edges: usize) -> Self::Graph;

    /// Create a graph builder
    fn create_builder() -> Self::Builder;

    /// Create a builder with initial capacity
    fn create_builder_with_capacity(nodes: usize, edges: usize) -> Self::Builder;
}

/// Specialized graph operations for code analysis
pub trait CodeGraphOperations: GraphOperations<crate::analysis::CodeNode, crate::analysis::CodeRelationship> {
    /// Find all functions that call a given function
    fn find_callers(&self, function_id: NodeId) -> Vec<NodeId>;

    /// Find all functions called by a given function
    fn find_callees(&self, function_id: NodeId) -> Vec<NodeId>;

    /// Find all modules that import a given module
    fn find_importers(&self, module_id: NodeId) -> Vec<NodeId>;

    /// Find all modules imported by a given module
    fn find_imports(&self, module_id: NodeId) -> Vec<NodeId>;

    /// Calculate call depth for a function
    fn calculate_call_depth(&self, function_id: NodeId) -> usize;

    /// Calculate import depth for a module
    fn calculate_import_depth(&self, module_id: NodeId) -> usize;

    /// Find clusters of strongly connected functions
    fn find_call_clusters(&self) -> Vec<Vec<NodeId>>;

    /// Find clusters of strongly connected modules
    fn find_import_clusters(&self) -> Vec<Vec<NodeId>>;

    /// Analyze control flow complexity
    fn analyze_control_flow(&self, function_id: NodeId) -> FastContextResult<crate::analysis::code_graph::ControlFlowAnalysis>;
}

/// Helper trait for converting between different graph node types
pub trait NodeConverter<From, To>: Send + Sync {
    fn convert(&self, node: &From) -> To;
}

/// Helper trait for converting between different graph edge types
pub trait EdgeConverter<From, To>: Send + Sync {
    fn convert(&self, edge: &From) -> To;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_conversions() {
        let id = NodeId::new(42);
        assert_eq!(id.as_usize(), 42);

        let petgraph_id = petgraph::graph::NodeIndex::new(42);
        let converted = NodeId::from_petgraph(petgraph_id);
        assert_eq!(converted, id);

        let back_to_petgraph = converted.to_petgraph();
        assert_eq!(back_to_petgraph, petgraph_id);
    }

    #[test]
    fn test_edge_id_conversions() {
        let id = EdgeId::new(123);
        assert_eq!(id.as_usize(), 123);
    }

    #[test]
    fn test_graph_direction_variants() {
        let incoming = GraphDirection::Incoming;
        let outgoing = GraphDirection::Outgoing;
        
        assert_ne!(incoming, outgoing);
        assert_eq!(incoming, GraphDirection::Incoming);
        assert_eq!(outgoing, GraphDirection::Outgoing);
    }

    #[test]
    fn test_path_result_creation() {
        let path_exists = PathResult {
            exists: true,
            length: Some(3),
            path: Some(vec![NodeId::new(0), NodeId::new(1), NodeId::new(2)]),
            weight: Some(1.5),
        };

        assert!(path_exists.exists);
        assert_eq!(path_exists.length, Some(3));
        assert_eq!(path_exists.path.as_ref().unwrap().len(), 3);
        assert_eq!(path_exists.weight, Some(1.5));
    }
}