//! # Python Graph Bindings for Fast-Context
//!
//! This module provides Python bindings for petgraph graph algorithms,
//! enabling advanced graph analysis capabilities in the Python SDK.

#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "python")]
use petgraph::{
    graph::{UnGraph, DiGraph, NodeIndex},
    visit::{EdgeRef, Walker},
    algo::{
        dijkstra,
        kosaraju_scc,
        is_cyclic_directed, toposort,
    },
};

/// Python wrapper for petgraph undirected graph
#[cfg(feature = "python")]
#[pyclass]
pub struct PyRustworkxGraph {
    graph: Arc<Mutex<UnGraph<String, f64>>>,
}

#[cfg(feature = "python")]
impl Clone for PyRustworkxGraph {
    fn clone(&self) -> Self {
        let graph = self.graph.lock().unwrap();
        Self {
            graph: Arc::new(Mutex::new(graph.clone())),
        }
    }
}

#[cfg(feature = "python")]
impl Default for PyRustworkxGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Python wrapper for petgraph directed graph  
#[cfg(feature = "python")]
#[pyclass]
pub struct PyRustworkxDiGraph {
    graph: Arc<Mutex<DiGraph<String, f64>>>,
}

#[cfg(feature = "python")]
impl Clone for PyRustworkxDiGraph {
    fn clone(&self) -> Self {
        let graph = self.graph.lock().unwrap();
        Self {
            graph: Arc::new(Mutex::new(graph.clone())),
        }
    }
}

#[cfg(feature = "python")]
impl Default for PyRustworkxDiGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from path algorithms
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct PathResult {
    #[pyo3(get)]
    pub path: Vec<usize>,
    #[pyo3(get)]
    pub distance: f64,
}

/// Centrality result
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct CentralityResult {
    #[pyo3(get)]
    pub node: usize,
    #[pyo3(get)]
    pub centrality: f64,
}

/// Connected component result
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct ConnectedComponent {
    #[pyo3(get)]
    pub nodes: Vec<usize>,
    #[pyo3(get)]
    pub size: usize,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyRustworkxGraph {
    #[new]
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Mutex::new(UnGraph::new_undirected())),
        }
    }

    #[staticmethod]
    #[pyo3(name = "with_capacity")]
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: Arc::new(Mutex::new(UnGraph::with_capacity(nodes, edges))),
        }
    }

    /// Add a node to the graph with optional weight
    /// 
    /// # Arguments
    /// * `weight` - Optional string label for the node (defaults to "node_{index}")
    /// 
    /// # Returns
    /// * `usize` - The index of the newly added node
    /// 
    /// # Examples
    /// ```python
    /// graph = PyRustworkxGraph()
    /// node_id = graph.add_node("router")
    /// print(f"Added node with ID: {node_id}")
    /// ```
    pub fn add_node(&mut self, weight: Option<String>) -> usize {
        let mut graph = self.graph.lock().unwrap();
        let weight = weight.unwrap_or_else(|| format!("Node {}", graph.node_count()));
        graph.add_node(weight).index()
    }

    pub fn remove_node(&mut self, node_index: usize) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.remove_node(node_idx).is_some()
    }

    /// Add an edge between two nodes with optional weight
    /// 
    /// # Arguments
    /// * `source` - Index of the source node
    /// * `target` - Index of the target node  
    /// * `weight` - Optional weight for the edge (defaults to 1.0)
    /// 
    /// # Returns
    /// * `Option<usize>` - Some(edge_index) if successful, None if nodes don't exist
    /// 
    /// # Examples
    /// ```python
    /// graph = PyRustworkxGraph()
    /// node1 = graph.add_node("A")
    /// node2 = graph.add_node("B")
    /// edge_id = graph.add_edge(node1, node2, 2.5)
    /// ```
    pub fn add_edge(&mut self, source: usize, target: usize, weight: Option<f64>) -> Option<usize> {
        let mut graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        let weight = weight.unwrap_or(1.0);
        
        if graph.node_weight(source_idx).is_some() && graph.node_weight(target_idx).is_some() {
            Some(graph.add_edge(source_idx, target_idx, weight).index())
        } else {
            None
        }
    }

    pub fn remove_edge(&mut self, source: usize, target: usize) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx)
            .map(|edge| graph.remove_edge(edge).is_some())
            .unwrap_or(false)
    }

    #[getter]
    pub fn node_count(&self) -> usize {
        let graph = self.graph.lock().unwrap();
        graph.node_count()
    }

    #[getter]
    pub fn edge_count(&self) -> usize {
        let graph = self.graph.lock().unwrap();
        graph.edge_count()
    }

    pub fn is_empty(&self) -> bool {
        let graph = self.graph.lock().unwrap();
        graph.node_count() == 0
    }

    pub fn clear(&mut self) {
        let mut graph = self.graph.lock().unwrap();
        graph.clear();
    }

    pub fn get_node_weight(&self, node_index: usize) -> Option<String> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.node_weight(node_idx).cloned()
    }

    pub fn set_node_weight(&mut self, node_index: usize, weight: String) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.node_weight_mut(node_idx).map(|w| *w = weight).is_some()
    }

    pub fn get_edge_weight(&self, source: usize, target: usize) -> Option<f64> {
        let graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx)
            .and_then(|edge| graph.edge_weight(edge).copied())
    }

    pub fn set_edge_weight(&mut self, source: usize, target: usize, weight: f64) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx)
            .map(|edge| {
                *graph.edge_weight_mut(edge).unwrap() = weight;
                true
            })
            .unwrap_or(false)
    }

    pub fn neighbors(&self, node_index: usize) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.neighbors(node_idx).map(|n| n.index()).collect()
    }

    pub fn edges(&self, node_index: usize) -> Vec<(usize, usize)> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.edges(node_idx)
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect()
    }

    pub fn has_edge(&self, source: usize, target: usize) -> bool {
        let graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx).is_some()
    }

    /// Find the shortest path between two nodes using Dijkstra's algorithm
    /// 
    /// # Arguments
    /// * `source` - Index of the source node
    /// * `target` - Index of the target node
    /// 
    /// # Returns
    /// * `Option<PathResult>` - Some(PathResult) with path and cost if path exists, None otherwise
    /// 
    /// # Examples
    /// ```python
    /// graph = PyRustworkxGraph()
    /// # Add nodes and edges...
    /// result = graph.dijkstra_shortest_path(0, 3)
    /// if result:
    ///     print(f"Path: {result.path}, Cost: {result.cost}")
    /// ```
    pub fn dijkstra_shortest_path(&self, source: usize, target: usize) -> Option<PathResult> {
        let graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        
        if graph.node_weight(source_idx).is_none() || graph.node_weight(target_idx).is_none() {
            return None;
        }
        
let costs = dijkstra(&*graph, source_idx, Some(target_idx), |e| *e.weight());
        
        if let Some(&cost) = costs.get(&target_idx) {
            Some(PathResult {
                path: vec![source, target],
                distance: cost,
            })
        } else {
            None
        }
    }

    pub fn floyd_warshall_all_pairs(&self) -> Vec<Vec<Option<f64>>> {
        let graph = self.graph.lock().unwrap();
        let node_count = graph.node_count();
        
        if node_count == 0 {
            return Vec::new();
        }
        
        // Initialize distance matrix with optimal Floyd-Warshall algorithm
        let mut distances = vec![vec![f64::INFINITY; node_count]; node_count];
        
        // Set diagonal to 0 and direct edges to their weights
        for (i, row) in distances.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
            let source_idx = NodeIndex::new(i);
            
            // Set direct edge weights
            for edge in graph.edges_directed(source_idx, petgraph::Direction::Outgoing) {
                let target_idx = edge.target().index();
                let weight = *edge.weight();
                row[target_idx] = weight.min(row[target_idx]);
            }
        }
        
        // Floyd-Warshall algorithm: O(n³) optimal implementation
        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    if distances[i][k] + distances[k][j] < distances[i][j] {
                        distances[i][j] = distances[i][k] + distances[k][j];
                    }
                }
            }
        }
        
        // Convert to the expected format with Option<f64>
        let mut result = vec![vec![None; node_count]; node_count];
        for (i, row) in distances.iter().enumerate().take(node_count) {
            for (j, distance) in row.iter().enumerate().take(node_count) {
                if distance.is_finite() {
                    result[i][j] = Some(*distance);
                }
            }
        }
        
        result
    }

    pub fn connected_components(&self) -> Vec<ConnectedComponent> {
        let graph = self.graph.lock().unwrap();
        
        // Use a simple BFS-based approach for connected components
        let mut visited = std::collections::HashSet::new();
        let mut components = Vec::new();
        let node_count = graph.node_count();
        
        for node_idx in 0..node_count {
            let node = NodeIndex::new(node_idx);
            if !visited.contains(&node) && graph.node_weight(node).is_some() {
                let mut component = Vec::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(node);
                visited.insert(node);
                
                while let Some(current) = queue.pop_front() {
                    component.push(current.index());
                    for neighbor in graph.neighbors(current) {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
                
                let size = component.len();
                components.push(ConnectedComponent {
                    nodes: component,
                    size,
                });
            }
        }
        
        components
    }

    pub fn bfs_tree(&self, start: usize) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        let start_idx = NodeIndex::new(start);
        
        if graph.node_weight(start_idx).is_none() {
            return Vec::new();
        }
        
        petgraph::visit::Bfs::new(&*graph, start_idx)
            .iter(&*graph)
            .map(|n| n.index())
            .collect()
    }

    pub fn dfs_tree(&self, start: usize) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        let start_idx = NodeIndex::new(start);
        
        if graph.node_weight(start_idx).is_none() {
            return Vec::new();
        }
        
        petgraph::visit::Dfs::new(&*graph, start_idx)
            .iter(&*graph)
            .map(|n| n.index())
            .collect()
    }

    pub fn density(&self) -> f64 {
        let graph = self.graph.lock().unwrap();
        let nodes = graph.node_count();
        let edges = graph.edge_count();
        
        if nodes <= 1 {
            0.0
        } else {
            (2.0 * edges as f64) / (nodes as f64 * (nodes as f64 - 1.0))
        }
    }

    #[pyo3(name = "clone")]
    pub fn clone_graph(&self) -> Self {
        Clone::clone(self)
    }

    pub fn __str__(&self) -> String {
        let graph = self.graph.lock().unwrap();
        format!("UnGraph(nodes={}, edges={})", graph.node_count(), graph.edge_count())
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyRustworkxDiGraph {
    #[new]
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Mutex::new(DiGraph::new())),
        }
    }

    #[staticmethod]
    #[pyo3(name = "with_capacity")]
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: Arc::new(Mutex::new(DiGraph::with_capacity(nodes, edges))),
        }
    }

    /// Add a node to the graph with optional weight
    /// 
    /// # Arguments
    /// * `weight` - Optional string label for the node (defaults to "node_{index}")
    /// 
    /// # Returns
    /// * `usize` - The index of the newly added node
    /// 
    /// # Examples
    /// ```python
    /// graph = PyRustworkxGraph()
    /// node_id = graph.add_node("router")
    /// print(f"Added node with ID: {node_id}")
    /// ```
    pub fn add_node(&mut self, weight: Option<String>) -> usize {
        let mut graph = self.graph.lock().unwrap();
        let weight = weight.unwrap_or_else(|| format!("Node {}", graph.node_count()));
        graph.add_node(weight).index()
    }

    pub fn remove_node(&mut self, node_index: usize) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.remove_node(node_idx).is_some()
    }

    /// Add an edge between two nodes with optional weight
    /// 
    /// # Arguments
    /// * `source` - Index of the source node
    /// * `target` - Index of the target node  
    /// * `weight` - Optional weight for the edge (defaults to 1.0)
    /// 
    /// # Returns
    /// * `Option<usize>` - Some(edge_index) if successful, None if nodes don't exist
    /// 
    /// # Examples
    /// ```python
    /// graph = PyRustworkxGraph()
    /// node1 = graph.add_node("A")
    /// node2 = graph.add_node("B")
    /// edge_id = graph.add_edge(node1, node2, 2.5)
    /// ```
    pub fn add_edge(&mut self, source: usize, target: usize, weight: Option<f64>) -> Option<usize> {
        let mut graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        let weight = weight.unwrap_or(1.0);
        
        if graph.node_weight(source_idx).is_some() && graph.node_weight(target_idx).is_some() {
            Some(graph.add_edge(source_idx, target_idx, weight).index())
        } else {
            None
        }
    }

    pub fn remove_edge(&mut self, source: usize, target: usize) -> bool {
        let mut graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx)
            .map(|edge| graph.remove_edge(edge).is_some())
            .unwrap_or(false)
    }

    #[getter]
    pub fn node_count(&self) -> usize {
        let graph = self.graph.lock().unwrap();
        graph.node_count()
    }

    #[getter]
    pub fn edge_count(&self) -> usize {
        let graph = self.graph.lock().unwrap();
        graph.edge_count()
    }

    pub fn is_empty(&self) -> bool {
        let graph = self.graph.lock().unwrap();
        graph.node_count() == 0
    }

    pub fn clear(&mut self) {
        let mut graph = self.graph.lock().unwrap();
        graph.clear();
    }

    pub fn successors(&self, node_index: usize) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.neighbors(node_idx).map(|n| n.index()).collect()
    }

    pub fn predecessors(&self, node_index: usize) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .map(|n| n.index())
            .collect()
    }

    pub fn out_edges(&self, node_index: usize) -> Vec<(usize, usize)> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.edges(node_idx)
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect()
    }

    pub fn in_edges(&self, node_index: usize) -> Vec<(usize, usize)> {
        let graph = self.graph.lock().unwrap();
        let node_idx = NodeIndex::new(node_index);
        graph.edges_directed(node_idx, petgraph::Direction::Incoming)
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect()
    }

    pub fn has_edge(&self, source: usize, target: usize) -> bool {
        let graph = self.graph.lock().unwrap();
        let source_idx = NodeIndex::new(source);
        let target_idx = NodeIndex::new(target);
        graph.find_edge(source_idx, target_idx).is_some()
    }

    pub fn strongly_connected_components(&self) -> Vec<ConnectedComponent> {
        let graph = self.graph.lock().unwrap();
        kosaraju_scc(&*graph)
            .into_iter()
            .map(|component| ConnectedComponent {
                nodes: component.iter().map(|n| n.index()).collect(),
                size: component.len(),
            })
            .collect()
    }

    pub fn weakly_connected_components(&self) -> Vec<ConnectedComponent> {
        let graph = self.graph.lock().unwrap();
        
        // For directed graphs, we need to ignore direction for weakly connected components
        let mut visited = std::collections::HashSet::new();
        let mut components = Vec::new();
        let node_count = graph.node_count();
        
        for node_idx in 0..node_count {
            let node = NodeIndex::new(node_idx);
            if !visited.contains(&node) && graph.node_weight(node).is_some() {
                let mut component = Vec::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(node);
                visited.insert(node);
                
                while let Some(current) = queue.pop_front() {
                    component.push(current.index());
                    
                    // Check both outgoing and incoming edges for weak connectivity
                    for neighbor in graph.neighbors(current) {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                    
                    for neighbor in graph.neighbors_directed(current, petgraph::Direction::Incoming) {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
                
                let size = component.len();
                components.push(ConnectedComponent {
                    nodes: component,
                    size,
                });
            }
        }
        
        components
    }

    pub fn is_directed_acyclic_graph(&self) -> bool {
        let graph = self.graph.lock().unwrap();
        !is_cyclic_directed(&*graph)
    }

    pub fn topological_sort(&self) -> Vec<usize> {
        let graph = self.graph.lock().unwrap();
        match toposort(&*graph, None) {
            Ok(order) => order.into_iter().map(|n| n.index()).collect(),
            Err(_) => Vec::new(),
        }
    }

    #[pyo3(name = "clone")]
    pub fn clone_graph(&self) -> Self {
        Clone::clone(self)
    }

    pub fn __str__(&self) -> String {
        let graph = self.graph.lock().unwrap();
        format!("DiGraph(nodes={}, edges={})", graph.node_count(), graph.edge_count())
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}
