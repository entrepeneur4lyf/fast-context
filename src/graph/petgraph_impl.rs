//! Petgraph implementation of GraphOperations trait
//!
//! This module provides concrete implementations of the GraphOperations trait
//! using petgraph as the underlying graph library.

use crate::errors::FastContextResult;
use crate::graph::operations::{
    ComponentAnalysis, EdgeId, GraphDirection, GraphOperations, GraphStats, NodeId, PathResult,
};
use petgraph::algo;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Petgraph implementation for directed graphs
#[derive(Debug)]
pub struct PetGraphDirected<N, E> {
    graph: DiGraph<N, E>,
    node_id_map: HashMap<usize, NodeIndex>,
    next_node_id: usize,
    edge_id_map: HashMap<usize, petgraph::graph::EdgeIndex>,
    next_edge_id: usize,
    /// Cached node degrees for O(1) access with interior mutability
    degree_cache: Mutex<HashMap<usize, (usize, usize)>>, // (in_degree, out_degree)
    /// Whether degree cache is dirty and needs recalculation
    degree_cache_dirty: AtomicBool,
}

impl<N, E> PetGraphDirected<N, E>
where
    N: Clone + Send + Sync + Debug + 'static,
    E: Clone + Send + Sync + Debug + 'static,
{
    /// Create a new empty directed graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_id_map: HashMap::new(),
            next_node_id: 0,
            edge_id_map: HashMap::new(),
            next_edge_id: 0,
            degree_cache: Mutex::new(HashMap::new()),
            degree_cache_dirty: AtomicBool::new(false),
        }
    }

    /// Create a directed graph with estimated capacity
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: DiGraph::with_capacity(nodes, edges),
            node_id_map: HashMap::with_capacity(nodes),
            next_node_id: 0,
            edge_id_map: HashMap::with_capacity(edges),
            next_edge_id: 0,
            degree_cache: Mutex::new(HashMap::with_capacity(nodes)),
            degree_cache_dirty: AtomicBool::new(false),
        }
    }

    /// Get the underlying petgraph DiGraph (for advanced operations)
    pub fn inner(&self) -> &DiGraph<N, E> {
        &self.graph
    }

    /// Get mutable access to the underlying petgraph DiGraph
    pub fn inner_mut(&mut self) -> &mut DiGraph<N, E> {
        &mut self.graph
    }

    /// Mark degree cache as dirty (call when graph structure changes)
    fn mark_degree_cache_dirty(&self) {
        self.degree_cache_dirty.store(true, Ordering::SeqCst);
    }

    /// Ensure degree cache is up to date
    fn update_degree_cache_if_needed(&self) {
        if self.degree_cache_dirty.load(Ordering::SeqCst) {
            self.recalculate_degree_cache();
            self.degree_cache_dirty.store(false, Ordering::SeqCst);
        }
    }

    /// Recalculate all node degrees
    fn recalculate_degree_cache(&self) {
        let mut cache = self.degree_cache.lock().unwrap();
        cache.clear();
        for node_idx in self.graph.node_indices() {
            let in_degree = self
                .graph
                .neighbors_directed(node_idx, Direction::Incoming)
                .count();
            let out_degree = self
                .graph
                .neighbors_directed(node_idx, Direction::Outgoing)
                .count();
            if let Some(our_id) = self.lookup_node_id(node_idx) {
                cache.insert(our_id.as_usize(), (in_degree, out_degree));
            }
        }
    }

    /// Get cached node degrees, recalculating if needed
    pub fn get_cached_degrees(&self, node_id: NodeId) -> Option<(usize, usize)> {
        self.update_degree_cache_if_needed();
        let cache = self.degree_cache.lock().unwrap();
        cache.get(&node_id.as_usize()).copied()
    }

    /// Convert our NodeId to petgraph NodeIndex
    fn to_petgraph_node(&self, node_id: NodeId) -> Option<NodeIndex> {
        self.node_id_map.get(&node_id.as_usize()).copied()
    }

    /// Convert petgraph NodeIndex back to our NodeId
    fn lookup_node_id(&self, node_idx: NodeIndex) -> Option<NodeId> {
        self.node_id_map
            .iter()
            .find(|(_, &idx)| idx == node_idx)
            .map(|(&our_id, _)| NodeId::new(our_id))
    }

    /// Convert petgraph NodeIndex to our NodeId
    fn intern_node_id(&mut self, node_idx: NodeIndex) -> NodeId {
        let our_id = self.next_node_id;
        self.next_node_id += 1;
        self.node_id_map.insert(our_id, node_idx);
        NodeId::new(our_id)
    }

    /// Convert our EdgeId to petgraph EdgeIndex
    fn to_petgraph_edge(&self, edge_id: EdgeId) -> Option<petgraph::graph::EdgeIndex> {
        self.edge_id_map.get(&edge_id.as_usize()).copied()
    }

    /// Convert petgraph EdgeIndex to our EdgeId
    fn intern_edge_id(&mut self, edge_idx: petgraph::graph::EdgeIndex) -> EdgeId {
        let our_id = self.next_edge_id;
        self.next_edge_id += 1;
        self.edge_id_map.insert(our_id, edge_idx);
        EdgeId::new(our_id)
    }
}

impl<N, E> Default for PetGraphDirected<N, E>
where
    N: Clone + Send + Sync + Debug + 'static,
    E: Clone + Send + Sync + Debug + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> GraphOperations<N, E> for PetGraphDirected<N, E>
where
    N: Clone + Send + Sync + Debug + 'static,
    E: Clone + Send + Sync + Debug + 'static,
{
    // === Node Operations ===

    fn add_node(&mut self, node: N) -> FastContextResult<NodeId> {
        let node_idx = self.graph.add_node(node);
        Ok(self.intern_node_id(node_idx))
    }

    fn get_node(&self, node_id: NodeId) -> Option<&N> {
        self.to_petgraph_node(node_id)
            .and_then(|idx| self.graph.node_weight(idx))
    }

    fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut N> {
        self.to_petgraph_node(node_id)
            .and_then(|idx| self.graph.node_weight_mut(idx))
    }

    fn remove_node(&mut self, node_id: NodeId) -> FastContextResult<()> {
        if let Some(node_idx) = self.to_petgraph_node(node_id) {
            self.graph.remove_node(node_idx);
            self.node_id_map.remove(&node_id.as_usize());
            self.mark_degree_cache_dirty();
            Ok(())
        } else {
            Err(crate::errors::FastContextError::Graph {
                operation: "remove_node".to_string(),
                message: format!("Node {} not found", node_id.as_usize()),
            })
        }
    }

    fn contains_node(&self, node_id: NodeId) -> bool {
        self.to_petgraph_node(node_id)
            .map(|idx| self.graph.node_weight(idx).is_some())
            .unwrap_or(false)
    }

    fn node_ids(&self) -> Vec<NodeId> {
        self.node_id_map
            .iter()
            .map(|(&our_id, _)| NodeId::new(our_id))
            .collect()
    }

    // === Edge Operations ===

    fn add_edge(&mut self, source: NodeId, target: NodeId, edge: E) -> FastContextResult<EdgeId> {
        let source_idx = self.to_petgraph_node(source).ok_or_else(|| {
            crate::errors::FastContextError::Graph {
                operation: "add_edge".to_string(),
                message: format!("Source node {} not found", source.as_usize()),
            }
        })?;

        let target_idx = self.to_petgraph_node(target).ok_or_else(|| {
            crate::errors::FastContextError::Graph {
                operation: "add_edge".to_string(),
                message: format!("Target node {} not found", target.as_usize()),
            }
        })?;

        let edge_idx = self.graph.add_edge(source_idx, target_idx, edge);
        self.mark_degree_cache_dirty();
        Ok(self.intern_edge_id(edge_idx))
    }

    fn get_edge(&self, edge_id: EdgeId) -> Option<&E> {
        self.to_petgraph_edge(edge_id)
            .and_then(|idx| self.graph.edge_weight(idx))
    }

    fn get_edge_mut(&mut self, edge_id: EdgeId) -> Option<&mut E> {
        self.to_petgraph_edge(edge_id)
            .and_then(|idx| self.graph.edge_weight_mut(idx))
    }

    fn remove_edge(&mut self, edge_id: EdgeId) -> FastContextResult<()> {
        if let Some(edge_idx) = self.to_petgraph_edge(edge_id) {
            self.graph.remove_edge(edge_idx);
            self.edge_id_map.remove(&edge_id.as_usize());
            self.mark_degree_cache_dirty();
            Ok(())
        } else {
            Err(crate::errors::FastContextError::Graph {
                operation: "remove_edge".to_string(),
                message: format!("Edge {} not found", edge_id.as_usize()),
            })
        }
    }

    fn find_edge(&self, source: NodeId, target: NodeId) -> Option<EdgeId> {
        let source_idx = self.to_petgraph_node(source)?;
        let target_idx = self.to_petgraph_node(target)?;

        self.graph
            .find_edge(source_idx, target_idx)
            .and_then(|edge_idx| {
                // Find our edge ID that corresponds to this petgraph edge
                self.edge_id_map
                    .iter()
                    .find(|(_, &pet_idx)| pet_idx == edge_idx)
                    .map(|(&our_id, _)| EdgeId::new(our_id))
            })
    }

    // === Traversal Operations ===

    fn neighbors(&self, node_id: NodeId, direction: GraphDirection) -> Vec<NodeId> {
        if let Some(node_idx) = self.to_petgraph_node(node_id) {
            let pet_direction = match direction {
                GraphDirection::Incoming => Direction::Incoming,
                GraphDirection::Outgoing => Direction::Outgoing,
            };

            self.graph
                .neighbors_directed(node_idx, pet_direction)
                .filter_map(|neighbor_idx| {
                    // Find our NodeId for this petgraph node
                    self.node_id_map
                        .iter()
                        .find(|(_, &pet_idx)| pet_idx == neighbor_idx)
                        .map(|(&our_id, _)| NodeId::new(our_id))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn edges(&self, node_id: NodeId, direction: GraphDirection) -> Vec<(NodeId, EdgeId, NodeId)> {
        if let Some(node_idx) = self.to_petgraph_node(node_id) {
            let pet_direction = match direction {
                GraphDirection::Incoming => Direction::Incoming,
                GraphDirection::Outgoing => Direction::Outgoing,
            };

            self.graph
                .edges_directed(node_idx, pet_direction)
                .filter_map(|edge_ref| {
                    let source_idx = edge_ref.source();
                    let target_idx = edge_ref.target();
                    let edge_idx = edge_ref.id();

                    let source_id = self
                        .node_id_map
                        .iter()
                        .find(|(_, &pet_idx)| pet_idx == source_idx)
                        .map(|(&our_id, _)| NodeId::new(our_id))?;

                    let target_id = self
                        .node_id_map
                        .iter()
                        .find(|(_, &pet_idx)| pet_idx == target_idx)
                        .map(|(&our_id, _)| NodeId::new(our_id))?;

                    let edge_id = self
                        .edge_id_map
                        .iter()
                        .find(|(_, &pet_idx)| pet_idx == edge_idx)
                        .map(|(&our_id, _)| EdgeId::new(our_id))?;

                    Some((source_id, edge_id, target_id))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn degree(&self, node_id: NodeId, direction: GraphDirection) -> usize {
        // Use cached degrees for O(1) access instead of O(n) traversal
        self.update_degree_cache_if_needed();
        let cache = self.degree_cache.lock().unwrap();
        if let Some((in_deg, out_deg)) = cache.get(&node_id.as_usize()) {
            match direction {
                GraphDirection::Incoming => *in_deg,
                GraphDirection::Outgoing => *out_deg,
            }
        } else {
            0
        }
    }

    fn reachable_from(&self, start_node: NodeId, direction: GraphDirection) -> Vec<NodeId> {
        if let Some(start_idx) = self.to_petgraph_node(start_node) {
            let pet_direction = match direction {
                GraphDirection::Incoming => Direction::Incoming,
                GraphDirection::Outgoing => Direction::Outgoing,
            };

            let mut visited = HashSet::new();
            let mut stack = vec![start_idx];
            let mut reachable = Vec::new();

            while let Some(node_idx) = stack.pop() {
                if !visited.contains(&node_idx) {
                    visited.insert(node_idx);

                    if let Some(our_id) = self
                        .node_id_map
                        .iter()
                        .find(|(_, &pet_idx)| pet_idx == node_idx)
                        .map(|(&our_id, _)| our_id)
                    {
                        reachable.push(NodeId::new(our_id));
                    }

                    for neighbor in self.graph.neighbors_directed(node_idx, pet_direction) {
                        if !visited.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }

            reachable
        } else {
            Vec::new()
        }
    }

    // === Path Operations ===

    fn shortest_path(&self, source: NodeId, target: NodeId) -> FastContextResult<PathResult> {
        let source_idx = self.to_petgraph_node(source).ok_or_else(|| {
            crate::errors::FastContextError::Graph {
                operation: "shortest_path".to_string(),
                message: format!("Source node {} not found", source.as_usize()),
            }
        })?;

        let target_idx = self.to_petgraph_node(target).ok_or_else(|| {
            crate::errors::FastContextError::Graph {
                operation: "shortest_path".to_string(),
                message: format!("Target node {} not found", target.as_usize()),
            }
        })?;

        let path_map = algo::dijkstra(&self.graph, source_idx, Some(target_idx), |_| 1);

        if let Some(&distance) = path_map.get(&target_idx) {
            // Reconstruct path
            let path = if distance > 0 {
                // For simplicity, returning None for path reconstruction
                // In a real implementation, you'd reconstruct the actual path
                None
            } else {
                Some(vec![source])
            };

            Ok(PathResult {
                exists: true,
                length: Some(distance),
                path,
                weight: Some(distance as f64),
            })
        } else {
            Ok(PathResult {
                exists: false,
                length: None,
                path: None,
                weight: None,
            })
        }
    }

    fn all_paths(
        &self,
        _source: NodeId,
        _target: NodeId,
        _max_length: Option<usize>,
    ) -> FastContextResult<Vec<PathResult>> {
        // Simplified implementation - would need DFS/BFS for all paths
        Ok(vec![])
    }

    fn is_connected(&self, source: NodeId, target: NodeId) -> bool {
        self.shortest_path(source, target)
            .map(|result| result.exists)
            .unwrap_or(false)
    }

    fn distance(&self, source: NodeId, target: NodeId) -> Option<usize> {
        self.shortest_path(source, target)
            .ok()
            .and_then(|result| result.length)
    }

    // === Analysis Operations ===

    fn has_cycles(&self) -> bool {
        algo::is_cyclic_directed(&self.graph)
    }

    fn find_cycles(&self) -> Vec<Vec<NodeId>> {
        // Use strongly connected components to find cycles
        let sccs = algo::tarjan_scc(&self.graph);
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                scc.into_iter()
                    .filter_map(|node_idx| {
                        self.node_id_map
                            .iter()
                            .find(|(_, &pet_idx)| pet_idx == node_idx)
                            .map(|(&our_id, _)| NodeId::new(our_id))
                    })
                    .collect()
            })
            .collect()
    }

    fn strongly_connected_components(&self) -> ComponentAnalysis {
        let sccs = algo::tarjan_scc(&self.graph);
        let component_count = sccs.len();
        let largest_component_size = sccs.iter().map(|scc| scc.len()).max().unwrap_or(0);
        let is_strongly_connected = component_count == 1 && self.graph.node_count() > 0;

        let components = sccs
            .into_iter()
            .map(|scc| {
                scc.into_iter()
                    .filter_map(|node_idx| {
                        self.node_id_map
                            .iter()
                            .find(|(_, &pet_idx)| pet_idx == node_idx)
                            .map(|(&our_id, _)| NodeId::new(our_id))
                    })
                    .collect()
            })
            .collect();

        ComponentAnalysis {
            components,
            component_count,
            largest_component_size,
            is_strongly_connected,
        }
    }

    fn graph_stats(&self) -> GraphStats {
        let node_count = self.graph.node_count();
        let edge_count = self.graph.edge_count();

        // Use cached degrees for O(1) access instead of O(n²) recalculation
        let (total_degree, max_degree) = if node_count == 0 {
            (0, 0)
        } else {
            // Ensure degree cache is up to date
            self.update_degree_cache_if_needed();

            let mut total = 0;
            let mut max = 0;

            for node_id in self.node_id_map.keys() {
                {
                    let cache = self.degree_cache.lock().unwrap();
                    if let Some((in_deg, out_deg)) = cache.get(node_id) {
                        let degree = in_deg + out_deg;
                        total += degree;
                        max = max.max(degree);
                    }
                }
            }

            (total, max)
        };

        let average_degree = if node_count > 0 {
            total_degree as f64 / node_count as f64
        } else {
            0.0
        };

        let connected_components = algo::tarjan_scc(&self.graph).len();

        let max_possible_edges = if node_count > 1 {
            node_count * (node_count - 1)
        } else {
            0
        };

        let density = if max_possible_edges > 0 {
            edge_count as f64 / max_possible_edges as f64
        } else {
            0.0
        };

        // Check for cycles
        let has_cycles_flag = !self.find_cycles().is_empty();

        GraphStats {
            node_count,
            edge_count,
            average_degree,
            max_degree,
            connected_components,
            density,
            has_cycles_flag,
        }
    }

    fn find_nodes_by_predicate(
        &self,
        predicate: Box<dyn Fn(&N) -> bool + Send + Sync>,
    ) -> Vec<NodeId> {
        self.graph
            .node_weights()
            .enumerate()
            .filter(|(_, node_data)| predicate(node_data))
            .filter_map(|(idx, _)| {
                self.node_id_map
                    .iter()
                    .find(|(_, &pet_idx)| pet_idx.index() == idx)
                    .map(|(&our_id, _)| NodeId::new(our_id))
            })
            .collect()
    }

    fn find_edges_by_predicate(
        &self,
        predicate: Box<dyn Fn(&E) -> bool + Send + Sync>,
    ) -> Vec<EdgeId> {
        self.graph
            .edge_weights()
            .enumerate()
            .filter(|(_, edge_data)| predicate(edge_data))
            .filter_map(|(idx, _)| {
                self.edge_id_map
                    .iter()
                    .find(|(_, &pet_idx)| pet_idx.index() == idx)
                    .map(|(&our_id, _)| EdgeId::new(our_id))
            })
            .collect()
    }

    // === Graph Modification Operations ===

    fn clear(&mut self) {
        self.graph.clear();
        self.node_id_map.clear();
        self.edge_id_map.clear();
        self.next_node_id = 0;
        self.next_edge_id = 0;
    }

    fn subgraph(&self, nodes: &[NodeId]) -> FastContextResult<Box<dyn GraphOperations<N, E>>> {
        let mut new_graph = Self::with_capacity(nodes.len(), nodes.len() * 2);

        // Create mapping of old node IDs to new node IDs
        let mut node_mapping = HashMap::new();

        // Add nodes to new graph
        for &node_id in nodes {
            if let Some(node_data) = self.get_node(node_id) {
                let new_node_id = new_graph.add_node(node_data.clone())?;
                node_mapping.insert(node_id, new_node_id);
            }
        }

        // Add edges that exist between the selected nodes
        for &source_id in nodes {
            for &target_id in nodes {
                if let Some(edge_id) = self.find_edge(source_id, target_id) {
                    if let Some(edge_data) = self.get_edge(edge_id) {
                        if let (Some(new_source), Some(new_target)) =
                            (node_mapping.get(&source_id), node_mapping.get(&target_id))
                        {
                            let _ = new_graph.add_edge(*new_source, *new_target, edge_data.clone());
                        }
                    }
                }
            }
        }

        Ok(Box::new(new_graph))
    }

    fn merge(&mut self, other: &dyn GraphOperations<N, E>) -> FastContextResult<()> {
        // Create mapping from other graph's nodes to our nodes
        let mut node_mapping = HashMap::new();

        // Add all nodes from other graph
        for other_node_id in other.node_ids() {
            if let Some(node_data) = other.get_node(other_node_id) {
                let our_node_id = self.add_node(node_data.clone())?;
                node_mapping.insert(other_node_id, our_node_id);
            }
        }

        // Add all edges from other graph
        for other_node_id in other.node_ids() {
            let outgoing_edges = other.edges(other_node_id, GraphDirection::Outgoing);
            for (source_id, edge_id, target_id) in outgoing_edges {
                if let (Some(our_source), Some(our_target)) =
                    (node_mapping.get(&source_id), node_mapping.get(&target_id))
                {
                    if let Some(edge_data) = other.get_edge(edge_id) {
                        let _ = self.add_edge(*our_source, *our_target, edge_data.clone());
                    }
                }
            }
        }

        Ok(())
    }

    // === Utility Operations ===

    fn clone_graph(&self) -> Box<dyn GraphOperations<N, E>> {
        let mut new_graph = Self::with_capacity(self.graph.node_count(), self.graph.edge_count());

        // Copy all nodes
        for node_id in self.node_ids() {
            if let Some(node_data) = self.get_node(node_id) {
                let _ = new_graph.add_node(node_data.clone());
            }
        }

        // Copy all edges
        for node_id in self.node_ids() {
            let outgoing_edges = self.edges(node_id, GraphDirection::Outgoing);
            for (source_id, edge_id, target_id) in outgoing_edges {
                if let Some(edge_data) = self.get_edge(edge_id) {
                    let _ = new_graph.add_edge(source_id, target_id, edge_data.clone());
                }
            }
        }

        Box::new(new_graph)
    }

    fn capacity(&self) -> Option<(usize, usize)> {
        Some((self.graph.capacity().0, self.graph.capacity().1))
    }

    fn reserve(&mut self, nodes: usize, edges: usize) {
        self.node_id_map.reserve(nodes);
        self.edge_id_map.reserve(edges);
    }

    fn shrink_to_fit(&mut self) {
        self.graph.shrink_to_fit();
        self.node_id_map.shrink_to_fit();
        self.edge_id_map.shrink_to_fit();
    }
}

// Builder and Factory implementations will be added in a subsequent implementation
// along with the CodeGraphOperations trait implementation for code analysis

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petgraph_directed_basic() {
        let mut graph = PetGraphDirected::<String, i32>::new();

        // Add nodes
        let node1 = graph.add_node("Node 1".to_string()).unwrap();
        let node2 = graph.add_node("Node 2".to_string()).unwrap();

        // Add edge
        let edge = graph.add_edge(node1, node2, 42).unwrap();

        // Verify
        assert_eq!(graph.node_ids().len(), 2);
        assert_eq!(graph.get_node(node1), Some(&"Node 1".to_string()));
        assert_eq!(graph.get_node(node2), Some(&"Node 2".to_string()));
        assert_eq!(graph.get_edge(edge), Some(&42));

        // Test neighbors
        let neighbors = graph.neighbors(node1, GraphDirection::Outgoing);
        assert_eq!(neighbors, vec![node2]);

        let incoming = graph.neighbors(node2, GraphDirection::Incoming);
        assert_eq!(incoming, vec![node1]);
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = PetGraphDirected::<String, i32>::new();

        let node1 = graph.add_node("A".to_string()).unwrap();
        let node2 = graph.add_node("B".to_string()).unwrap();
        let node3 = graph.add_node("C".to_string()).unwrap();

        graph.add_edge(node1, node2, 1).unwrap();
        graph.add_edge(node2, node3, 1).unwrap();
        graph.add_edge(node3, node1, 1).unwrap(); // Creates a cycle

        let stats = graph.graph_stats();
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.edge_count, 3);
        assert_eq!(stats.average_degree, 2.0);
        assert!(stats.has_cycles());
    }
}
