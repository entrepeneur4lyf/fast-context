//! Node.js bindings for graph algorithms
//! 
//! This module provides NAPI bindings for the graph structures
//! Only compiled when the "nodejs" feature is enabled

#[cfg(feature = "nodejs")]
use napi_derive::napi;
#[cfg(feature = "nodejs")]
use ts_rs::TS;

/// Node.js bindings for undirected graph
#[cfg(feature = "nodejs")]
#[napi]
#[derive(TS)]
#[ts(export)]
pub struct GraphBindings {
    #[ts(skip)]
    pub graph: crate::graph::RustworkxGraph,
}

#[cfg(feature = "nodejs")]
#[napi]
impl GraphBindings {
    /// Create a new undirected graph
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            graph: crate::graph::RustworkxGraph::new(),
        }
    }

    /// Get node count
    #[napi]
    pub fn node_count(&self) -> u32 {
        self.graph.node_count()
    }

    /// Get edge count
    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.graph.edge_count()
    }

    /// Add a node with weight
    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.graph.add_node(weight)
    }

    /// Add an edge between two nodes
    #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        self.graph.add_edge(node_a, node_b, weight)
    }

    /// Remove a node
    #[napi]
    pub fn remove_node(&mut self, node: u32) -> bool {
        self.graph.remove_node(node)
    }

    /// Remove an edge
    #[napi]
    pub fn remove_edge(&mut self, node_a: u32, node_b: u32) -> bool {
        self.graph.remove_edge(node_a, node_b)
    }

    /// Check if edge exists
    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        self.graph.has_edge(node_a, node_b)
    }

    /// Get node data
    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        self.graph.get_node_data(node)
    }

    /// Get edge data
    #[napi]
    pub fn get_edge_data(&self, node_a: u32, node_b: u32) -> Option<f64> {
        self.graph.get_edge_data(node_a, node_b)
    }

    /// Get neighbors of a node
    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        self.graph.neighbors(node)
    }

    /// Clear the graph
    #[napi]
    pub fn clear(&mut self) {
        self.graph.clear();
    }

    /// Dijkstra's shortest path algorithm
    #[napi]
    pub fn dijkstra_shortest_paths(&self, source: u32, target: Option<u32>) -> Vec<Vec<f64>> {
        self.graph.dijkstra_shortest_paths(source, target)
    }

    /// All-pairs shortest paths
    #[napi]
    pub fn all_pairs_shortest_paths(&self) -> Vec<Vec<Option<f64>>> {
        self.graph.all_pairs_shortest_paths()
    }

    /// Betweenness centrality
    #[napi]
    pub fn betweenness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        self.graph.betweenness_centrality(normalized)
    }

    /// Closeness centrality
    #[napi]
    pub fn closeness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        self.graph.closeness_centrality(normalized)
    }

    /// Check if graph is bipartite
    #[napi]
    pub fn is_bipartite(&self) -> bool {
        self.graph.is_bipartite()
    }

    /// Get number of connected components
    #[napi]
    pub fn number_connected_components(&self) -> u32 {
        self.graph.number_connected_components()
    }

    /// Get connected components
    #[napi]
    pub fn connected_components(&self) -> Vec<u32> {
        self.graph.connected_components()
    }

    /// DFS edges
    #[napi]
    pub fn dfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        self.graph.dfs_edges(start)
    }

    /// BFS edges
    #[napi]
    pub fn bfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        self.graph.bfs_edges(start)
    }

    /// DFS tree
    #[napi]
    pub fn dfs_tree(&self, start: u32) -> Vec<u32> {
        self.graph.dfs_tree(start)
    }

    /// BFS tree
    #[napi]
    pub fn bfs_tree(&self, start: u32) -> Vec<u32> {
        self.graph.bfs_tree(start)
    }
}

/// Node.js bindings for directed graph
#[cfg(feature = "nodejs")]
#[napi]
#[derive(TS)]
#[ts(export)]
pub struct DiGraphBindings {
    #[ts(skip)]
    pub digraph: crate::graph::RustworkxDiGraph,
}

#[cfg(feature = "nodejs")]
#[napi]
impl DiGraphBindings {
    /// Create a new directed graph
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            digraph: crate::graph::RustworkxDiGraph::new(),
        }
    }

    /// Get node count
    #[napi]
    pub fn node_count(&self) -> u32 {
        self.digraph.node_count()
    }

    /// Get edge count
    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.digraph.edge_count()
    }

    /// Add a node with weight
    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.digraph.add_node(weight)
    }

    /// Add an edge between two nodes
    #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        self.digraph.add_edge(node_a, node_b, weight)
    }

    /// Remove a node
    #[napi]
    pub fn remove_node(&mut self, node: u32) -> bool {
        self.digraph.remove_node(node)
    }

    /// Remove an edge
    #[napi]
    pub fn remove_edge(&mut self, node_a: u32, node_b: u32) -> bool {
        self.digraph.remove_edge(node_a, node_b)
    }

    /// Check if edge exists
    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        self.digraph.has_edge(node_a, node_b)
    }

    /// Get node data
    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        self.digraph.get_node_data(node)
    }

    /// Get edge data
    #[napi]
    pub fn get_edge_data(&self, node_a: u32, node_b: u32) -> Option<f64> {
        self.digraph.get_edge_data(node_a, node_b)
    }

    /// Get neighbors of a node
    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        self.digraph.neighbors(node)
    }

    /// Get predecessors of a node
    #[napi]
    pub fn predecessors(&self, node: u32) -> Vec<u32> {
        self.digraph.predecessors(node)
    }

    /// Get successors of a node
    #[napi]
    pub fn successors(&self, node: u32) -> Vec<u32> {
        self.digraph.successors(node)
    }

    /// Clear the graph
    #[napi]
    pub fn clear(&mut self) {
        self.digraph.clear();
    }

    /// Dijkstra's shortest path algorithm
    #[napi]
    pub fn dijkstra_shortest_paths(&self, source: u32, target: Option<u32>) -> Vec<Vec<f64>> {
        self.digraph.dijkstra_shortest_paths(source, target)
    }

    /// All-pairs shortest paths
    #[napi]
    pub fn all_pairs_shortest_paths(&self) -> Vec<Vec<Option<f64>>> {
        self.digraph.all_pairs_shortest_paths()
    }

    /// Betweenness centrality
    #[napi]
    pub fn betweenness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        self.digraph.betweenness_centrality(normalized)
    }

    /// Closeness centrality
    #[napi]
    pub fn closeness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        self.digraph.closeness_centrality(normalized)
    }

    /// Check if graph is a DAG
    #[napi]
    pub fn is_directed_acyclic_graph(&self) -> bool {
        self.digraph.is_directed_acyclic_graph()
    }

    /// Topological sort
    #[napi]
    pub fn topological_sort(&self) -> Vec<u32> {
        self.digraph.topological_sort()
    }

    /// Get strongly connected components
    #[napi]
    pub fn strongly_connected_components(&self) -> Vec<u32> {
        self.digraph.strongly_connected_components()
    }

    /// Get number of strongly connected components
    #[napi]
    pub fn number_strongly_connected_components(&self) -> u32 {
        self.digraph.number_strongly_connected_components()
    }

    /// Get weakly connected components
    #[napi]
    pub fn weakly_connected_components(&self) -> Vec<u32> {
        self.digraph.weakly_connected_components()
    }

    /// DFS edges
    #[napi]
    pub fn dfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        self.digraph.dfs_edges(start)
    }

    /// BFS edges
    #[napi]
    pub fn bfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        self.digraph.bfs_edges(start)
    }

    /// DFS tree
    #[napi]
    pub fn dfs_tree(&self, start: u32) -> Vec<u32> {
        self.digraph.dfs_tree(start)
    }

    /// BFS tree
    #[napi]
    pub fn bfs_tree(&self, start: u32) -> Vec<u32> {
        self.digraph.bfs_tree(start)
    }
}
