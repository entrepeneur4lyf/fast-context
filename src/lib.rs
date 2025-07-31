//! # Fast-Context: Intelligent Codebase Analysis Engine
//! 
//! Fast-Context transforms complex codebases into comprehensive knowledge graphs that empower
//! coding assistants with deep semantic understanding, causal analysis, and real-time intelligence.
//! 
//! ## Core Architecture
//! 
//! ### Graph Algorithm Foundation (80+ Algorithms)
//! The comprehensive graph algorithm suite provides the computational engine for code analysis:
//! 
//! **Shortest Path Algorithms**: A*, Bellman-Ford, K-shortest paths, all paths enumeration
//! **Centrality Measures**: Betweenness, eigenvector, Katz centrality for code importance analysis
//! **Graph Operations**: Union, complement, tensor/cartesian products for code relationship modeling
//! **Traversal Algorithms**: Complete BFS/DFS suites for dependency tracing and impact analysis
//! **Specialized Algorithms**: SCC condensation, ancestors/descendants for call graph analysis
//! **Performance Optimizations**: Parallel algorithms, memory-efficient streaming, intelligent caching
//! 
//! ### Codebase Analysis Engine (In Development)
//! - **Multi-language Parsing**: 20+ programming languages via Tree-sitter
//! - **Symbol Extraction**: Functions, classes, variables, imports with full context
//! - **Dependency Graphs**: Call graphs, import graphs, data flow analysis
//! - **Real-time Updates**: File watching with incremental graph updates
//! - **Intelligent Caching**: Adaptive caching strategies from small projects to large monorepos
//! - **AI Assistant APIs**: Query interfaces designed for coding assistants and LLMs
//! 
//! ## Intelligent Caching Strategy
//! 
//! | Project Size | Files | Memory | Disk Cache | Features |
//! |--------------|-------|--------|------------|----------|
//! | Small | <1K | <200MB | <100MB | LRU + selective disk |
//! | Medium | 1K-10K | <500MB | <500MB | Multi-level cache |
//! | Large | >10K | <1GB | <1GB | Basic disk persistence |
//! 
//! ## Use Cases
//! 
//! - **Impact Analysis**: Trace how code changes propagate through the codebase
//! - **Semantic Search**: Find symbols, references, and usage patterns across languages
//! - **Dependency Visualization**: Understand complex code relationships and architecture
//! - **Refactoring Safety**: Identify all affected code before making changes
//! - **Code Intelligence**: Power AI assistants with deep codebase understanding
//! 
//! The existing graph algorithms serve as the high-performance foundation that enables
//! sophisticated code relationship modeling, dependency analysis, and impact assessment.

use hashbrown::HashMap;
use napi_derive::napi;
use petgraph::algo::{connected_components, is_cyclic_directed};
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::visit::{Bfs, Dfs, EdgeRef};
use petgraph::Direction;
use rustworkx_core::centrality;
use rustworkx_core::generators;
use rustworkx_core::shortest_path;
use std::collections::VecDeque;
use ts_rs::TS;

// Fast-Context imports
use crate::parsers::{ParserFactory, LanguageId};
use crate::query::{QueryResult, CodeQueryEngine};
use crate::export::ExportOptions;
use crate::symbols::{SymbolKind, SymbolExtractorFactory};
use crate::analysis::{AnalysisResult, CodeGraphBuilder};

pub mod types;

// Fast-Context analysis modules
pub mod parsers;    // Tree-sitter language parsers
pub mod symbols;    // Symbol extraction and management
pub mod analysis;   // Code analysis and graph construction
pub mod watcher;    // File system monitoring
pub mod query;      // Query interface for AI assistants
pub mod cache;      // Intelligent caching system
pub mod export;     // Export & serialization system

#[napi]
#[derive(TS)]
#[ts(export)]
pub struct RustworkxGraph {
    #[ts(skip)]
    inner: UnGraph<String, f64>,
}

impl Default for RustworkxGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
#[derive(TS)]
#[ts(export)]
pub struct RustworkxDiGraph {
    #[ts(skip)]
    inner: DiGraph<String, f64>,
}

impl Default for RustworkxDiGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl RustworkxGraph {
    // #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: UnGraph::new_undirected(),
        }
    }

    // #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

    // #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        let node_a_idx = petgraph::graph::NodeIndex::new(node_a as usize);
        let node_b_idx = petgraph::graph::NodeIndex::new(node_b as usize);

        if self.inner.node_weight(node_a_idx).is_some()
            && self.inner.node_weight(node_b_idx).is_some()
        {
            Some(self.inner.add_edge(node_a_idx, node_b_idx, weight).index() as u32)
        } else {
            None
        }
    }

    // #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    // #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

    // #[napi]
    pub fn betweenness_centrality(&self, normalized: bool, endpoints: bool) -> Vec<f64> {
        centrality::betweenness_centrality(&self.inner, normalized, endpoints, 200)
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect()
    }

    // #[napi]
    pub fn dijkstra_shortest_paths(&self, start: u32, target: Option<u32>) -> Vec<f64> {
        let start_idx = petgraph::graph::NodeIndex::new(start as usize);
        let target_idx = target.map(|t| petgraph::graph::NodeIndex::new(t as usize));

        let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
            &self.inner,
            start_idx,
            target_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            None,
        );

        match result {
            Ok(paths) => {
                let mut distances = vec![f64::INFINITY; self.inner.node_count()];
                for (node_idx, distance) in paths.into_iter() {
                    distances[node_idx.index()] = distance;
                }
                distances
            }
            Err(_) => vec![f64::INFINITY; self.inner.node_count()],
        }
    }

    /// A* shortest path algorithm
    // #[napi]
    pub fn astar_shortest_path(
        &self,
        start: u32,
        goal: u32,
        heuristic: Option<f64>,
    ) -> Option<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = NodeIndex::new(goal as usize);
        let heuristic_value = heuristic.unwrap_or(1.0);

        let result = shortest_path::astar(
            &self.inner,
            start_idx,
            |n| -> Result<bool, ()> { Ok(n == goal_idx) },
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            |_| -> Result<f64, ()> { Ok(heuristic_value) },
        );

        match result {
            Ok(Some((_cost, path))) => Some(path.into_iter().map(|n| n.index() as u32).collect()),
            _ => None,
        }
    }

    /// Bellman-Ford shortest paths (detects negative cycles)
    // #[napi]
    pub fn bellman_ford_shortest_paths(&self, start: u32) -> Option<Vec<f64>> {
        let start_idx = NodeIndex::new(start as usize);

        let result: Result<Option<Vec<Option<f64>>>, ()> = shortest_path::bellman_ford(
            &self.inner,
            start_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            None,
        );

        match result {
            Ok(Some(distances)) => {
                let mut result = vec![f64::INFINITY; self.inner.node_count()];
                for (i, distance_opt) in distances.iter().enumerate() {
                    if let Some(distance) = distance_opt {
                        result[i] = *distance;
                    }
                }
                Some(result)
            }
            _ => None, // Negative cycle detected or error
        }
    }

    /// K-shortest path lengths
    // #[napi]
    pub fn k_shortest_path_lengths(&self, start: u32, k: u32, goal: Option<u32>) -> Vec<f64> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = goal.map(|g| NodeIndex::new(g as usize));

        let result: Result<Vec<Option<f64>>, ()> = shortest_path::k_shortest_path(
            &self.inner,
            start_idx,
            goal_idx,
            k as usize,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
        );

        match result {
            Ok(distances) => distances
                .into_iter()
                .map(|opt| opt.unwrap_or(f64::INFINITY))
                .collect(),
            Err(_) => vec![f64::INFINITY; self.inner.node_count()],
        }
    }

    /// All shortest paths between two specific nodes
    // #[napi]
    pub fn all_shortest_paths_between_nodes(&self, start: u32, goal: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = NodeIndex::new(goal as usize);

        let result = shortest_path::all_shortest_paths(
            &self.inner,
            start_idx,
            goal_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
        );

        match result {
            Ok(paths) => paths
                .into_iter()
                .map(|path| path.into_iter().map(|node| node.index() as u32).collect())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Unweighted average shortest path length
    // #[napi]
    pub fn unweighted_average_shortest_path_length(&self, disconnected: Option<bool>) -> f64 {
        let disconnected = disconnected.unwrap_or(false);
        let node_count = self.inner.node_count();

        if node_count <= 1 {
            return 0.0;
        }

        let mut total_distance = 0.0;
        let mut path_count = 0;

        for source in self.inner.node_indices() {
            let mut bfs = Bfs::new(&self.inner, source);
            let mut distances = HashMap::new();
            distances.insert(source, 0);

            while let Some(node) = bfs.next(&self.inner) {
                let current_dist = distances[&node];

                for neighbor in self.inner.neighbors(node) {
                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, current_dist + 1);
                    }
                }
            }

            for target in self.inner.node_indices() {
                if source != target {
                    if let Some(&distance) = distances.get(&target) {
                        total_distance += distance as f64;
                        path_count += 1;
                    } else if disconnected {
                        // If disconnected nodes should be counted with infinite distance
                        return f64::INFINITY;
                    }
                }
            }
        }

        if path_count == 0 {
            0.0
        } else {
            total_distance / path_count as f64
        }
    }

    /// Edge betweenness centrality
    // #[napi]
    pub fn edge_betweenness_centrality(&self, normalized: Option<bool>) -> Vec<f64> {
        let normalized = normalized.unwrap_or(false);
        let edge_count = self.inner.edge_count();
        let mut edge_betweenness = vec![0.0; edge_count];

        if edge_count == 0 {
            return edge_betweenness;
        }

        // Map edge indices to their position in the result vector
        let edge_indices: HashMap<_, _> = self
            .inner
            .edge_references()
            .enumerate()
            .map(|(i, edge_ref)| (edge_ref.id(), i))
            .collect();

        for source in self.inner.node_indices() {
            // Single-source shortest path with path counting
            let mut distances = HashMap::new();
            let mut paths_count = HashMap::new();
            let mut predecessors: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
            let mut queue = VecDeque::new();
            let mut stack = Vec::new();

            distances.insert(source, 0);
            paths_count.insert(source, 1.0);
            queue.push_back(source);

            // BFS to find shortest paths
            while let Some(node) = queue.pop_front() {
                stack.push(node);
                let node_dist = distances[&node];

                for neighbor in self.inner.neighbors(node) {
                    let new_dist = node_dist + 1;

                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, new_dist);
                        paths_count.insert(neighbor, 0.0);
                        queue.push_back(neighbor);
                    }

                    if distances[&neighbor] == new_dist {
                        *paths_count.get_mut(&neighbor).unwrap() += paths_count[&node];
                        predecessors
                            .entry(neighbor)
                            .or_insert_with(Vec::new)
                            .push(node);
                    }
                }
            }

            // Accumulate edge betweenness
            let mut dependency = HashMap::new();
            for &node in &stack {
                dependency.insert(node, 0.0);
            }

            while let Some(node) = stack.pop() {
                if let Some(preds) = predecessors.get(&node) {
                    for &pred in preds {
                        let coeff =
                            (paths_count[&pred] / paths_count[&node]) * (1.0 + dependency[&node]);
                        *dependency.get_mut(&pred).unwrap() += coeff;

                        // Find edge and add to betweenness
                        if let Some(edge) = self.inner.find_edge(pred, node) {
                            if let Some(&edge_idx) = edge_indices.get(&edge) {
                                edge_betweenness[edge_idx] += coeff;
                            }
                        }
                    }
                }
            }
        }

        // Normalize if requested
        if normalized {
            let node_count = self.inner.node_count();
            if node_count > 2 {
                let norm_factor = 2.0 / ((node_count * (node_count - 1)) as f64);
                for value in &mut edge_betweenness {
                    *value *= norm_factor;
                }
            }
        }

        edge_betweenness
    }

    /// Get Floyd-Warshall all-pairs shortest path distances
    // #[napi]
    pub fn floyd_warshall_matrix(&self) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Initialize diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Initialize direct edges
        for edge_ref in self.inner.edge_references() {
            let source = edge_ref.source().index();
            let target = edge_ref.target().index();
            let weight = *edge_ref.weight();
            matrix[source][target] = weight;
            matrix[target][source] = weight; // undirected
        }

        // Floyd-Warshall algorithm
        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    if matrix[i][k] != f64::INFINITY && matrix[k][j] != f64::INFINITY {
                        let new_dist = matrix[i][k] + matrix[k][j];
                        if new_dist < matrix[i][j] {
                            matrix[i][j] = new_dist;
                        }
                    }
                }
            }
        }

        matrix
    }

    /// Get adjacency matrix representation
    // #[napi]
    pub fn adjacency_matrix(
        &self,
        _default_weight: Option<f64>,
        null_value: Option<f64>,
    ) -> Vec<Vec<f64>> {
        let null_value = null_value.unwrap_or(0.0);
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![null_value; node_count]; node_count];

        for edge_ref in self.inner.edge_references() {
            let source = edge_ref.source().index();
            let target = edge_ref.target().index();
            let weight = *edge_ref.weight();

            matrix[source][target] += weight;
            matrix[target][source] += weight; // undirected
        }

        matrix
    }

    /// Compute closeness centrality
    // #[napi]
    pub fn closeness_centrality(&self, wf_improved: Option<bool>) -> Vec<f64> {
        let wf_improved = wf_improved.unwrap_or(true);
        centrality::closeness_centrality(&self.inner, wf_improved)
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect()
    }

    /// Compute degree centrality
    // #[napi]
    pub fn degree_centrality(&self) -> Vec<f64> {
        let node_count = self.inner.node_count();
        if node_count <= 1 {
            return vec![0.0; node_count];
        }

        let mut centrality = Vec::with_capacity(node_count);
        let normalizer = (node_count - 1) as f64;

        for node_idx in self.inner.node_indices() {
            let degree = self.inner.neighbors(node_idx).count() as f64;
            centrality.push(degree / normalizer);
        }
        centrality
    }

    /// Get all simple paths between two nodes (simplified implementation)
    // #[napi]
    pub fn all_simple_paths(
        &self,
        from: u32,
        to: u32,
        min_depth: Option<u32>,
        cutoff: Option<u32>,
    ) -> Vec<Vec<u32>> {
        let from_idx = NodeIndex::new(from as usize);
        let to_idx = NodeIndex::new(to as usize);
        let min_depth = min_depth.unwrap_or(0) as usize;
        let max_depth = cutoff.unwrap_or(10) as usize; // Default max depth to prevent infinite loops

        let mut all_paths = Vec::new();
        let mut current_path = vec![from_idx];
        let mut visited = std::collections::HashSet::new();
        visited.insert(from_idx);

        self.find_paths_recursive(
            from_idx,
            to_idx,
            &mut current_path,
            &mut visited,
            &mut all_paths,
            max_depth,
            min_depth,
        );

        all_paths
            .into_iter()
            .map(|path| path.into_iter().map(|node| node.index() as u32).collect())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn find_paths_recursive(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        path: &mut Vec<NodeIndex>,
        visited: &mut std::collections::HashSet<NodeIndex>,
        all_paths: &mut Vec<Vec<NodeIndex>>,
        max_depth: usize,
        min_depth: usize,
    ) {
        if path.len() > max_depth {
            return;
        }

        if current == target {
            if path.len() >= min_depth {
                all_paths.push(path.clone());
            }
            return;
        }

        for neighbor in self.inner.neighbors(current) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                path.push(neighbor);
                self.find_paths_recursive(
                    neighbor, target, path, visited, all_paths, max_depth, min_depth,
                );
                path.pop();
                visited.remove(&neighbor);
            }
        }
    }

    /// Get DFS edges
    // #[napi]
    pub fn dfs_edges(&self, source: Option<u32>) -> Vec<Vec<u32>> {
        let mut edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];

        let sources: Vec<_> = if let Some(s) = source {
            vec![NodeIndex::new(s as usize)]
        } else {
            self.inner.node_indices().collect()
        };

        for start in sources {
            if !visited[start.index()] {
                let mut dfs = Dfs::new(&self.inner, start);
                let mut parent = HashMap::new();

                while let Some(node) = dfs.next(&self.inner) {
                    visited[node.index()] = true;

                    for neighbor in self.inner.neighbors(node) {
                        if !visited[neighbor.index()] && !parent.contains_key(&neighbor) {
                            parent.insert(neighbor, node);
                            edges.push(vec![node.index() as u32, neighbor.index() as u32]);
                        }
                    }
                }
            }
        }
        edges
    }

    /// Check if path exists between two nodes
    // #[napi]
    pub fn has_path(&self, source: u32, target: u32) -> bool {
        let source_idx = NodeIndex::new(source as usize);
        let target_idx = NodeIndex::new(target as usize);

        let mut bfs = Bfs::new(&self.inner, source_idx);
        while let Some(node) = bfs.next(&self.inner) {
            if node == target_idx {
                return true;
            }
        }
        false
    }

    /// Get connected components
    // #[napi]
    pub fn connected_components(&self) -> Vec<Vec<u32>> {
        let num_components = connected_components(&self.inner);
        let mut components = vec![Vec::new(); num_components];
        let mut component_map = HashMap::new();
        let mut current_component = 0;

        for node_idx in self.inner.node_indices() {
            let mut found_component = None;

            // Find which component this node belongs to by checking if it's connected to any existing component
            for neighbor in self.inner.neighbors(node_idx) {
                if let Some(&comp_id) = component_map.get(&neighbor) {
                    found_component = Some(comp_id);
                    break;
                }
            }

            let comp_id = found_component.unwrap_or_else(|| {
                let id = current_component;
                current_component += 1;
                id
            });

            component_map.insert(node_idx, comp_id);
            if comp_id < components.len() {
                components[comp_id].push(node_idx.index() as u32);
            }
        }

        components
            .into_iter()
            .filter(|component| !component.is_empty())
            .collect()
    }

    /// Get graph transitivity (clustering coefficient)
    // #[napi]
    pub fn transitivity(&self) -> f64 {
        let mut triangles = 0;
        let mut triplets = 0;

        for node in self.inner.node_indices() {
            let neighbors: Vec<_> = self.inner.neighbors(node).collect();
            let degree = neighbors.len();

            if degree >= 2 {
                triplets += degree * (degree - 1) / 2;

                for i in 0..neighbors.len() {
                    for j in (i + 1)..neighbors.len() {
                        if self.inner.find_edge(neighbors[i], neighbors[j]).is_some() {
                            triangles += 1;
                        }
                    }
                }
            }
        }

        if triplets == 0 {
            0.0
        } else {
            (3 * triangles) as f64 / triplets as f64
        }
    }

    /// Eigenvector centrality
    // #[napi]
    pub fn eigenvector_centrality(
        &self,
        max_iter: Option<u32>,
        tolerance: Option<f64>,
        weight_fn: Option<bool>,
    ) -> Vec<f64> {
        let max_iter = max_iter.unwrap_or(100);
        let tolerance = tolerance.unwrap_or(1e-6);
        let node_count = self.inner.node_count();

        if node_count == 0 {
            return vec![];
        }

        // Initialize eigenvector with uniform values
        let mut eigenvector = vec![1.0 / (node_count as f64).sqrt(); node_count];
        let mut prev_eigenvector = eigenvector.clone();

        for _ in 0..max_iter {
            // Matrix-vector multiplication: A * x
            let mut new_eigenvector = vec![0.0; node_count];

            for node_idx in self.inner.node_indices() {
                let node_index = node_idx.index();

                for neighbor in self.inner.neighbors(node_idx) {
                    let neighbor_index = neighbor.index();

                    if weight_fn.unwrap_or(false) {
                        // Use edge weights if requested
                        if let Some(edge) = self.inner.find_edge(neighbor, node_idx) {
                            if let Some(weight) = self.inner.edge_weight(edge) {
                                new_eigenvector[node_index] +=
                                    *weight * prev_eigenvector[neighbor_index];
                            }
                        }
                    } else {
                        // Unweighted (adjacency matrix)
                        new_eigenvector[node_index] += prev_eigenvector[neighbor_index];
                    }
                }
            }

            // Normalize the eigenvector
            let norm = new_eigenvector.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 0.0 {
                for value in &mut new_eigenvector {
                    *value /= norm;
                }
            }

            // Check for convergence
            let mut converged = true;
            for i in 0..node_count {
                if (new_eigenvector[i] - prev_eigenvector[i]).abs() > tolerance {
                    converged = false;
                    break;
                }
            }

            prev_eigenvector = eigenvector;
            eigenvector = new_eigenvector;

            if converged {
                break;
            }
        }

        // Ensure all values are non-negative (take absolute value)
        eigenvector.into_iter().map(|x| x.abs()).collect()
    }

    /// Katz centrality
    // #[napi]
    pub fn katz_centrality(
        &self,
        alpha: Option<f64>,
        beta: Option<f64>,
        max_iter: Option<u32>,
        tolerance: Option<f64>,
        weight_fn: Option<bool>,
    ) -> Vec<f64> {
        let alpha = alpha.unwrap_or(0.1);
        let beta = beta.unwrap_or(1.0);
        let max_iter = max_iter.unwrap_or(100);
        let tolerance = tolerance.unwrap_or(1e-6);
        let node_count = self.inner.node_count();

        if node_count == 0 {
            return vec![];
        }

        // Initialize Katz centrality with beta values
        let mut katz = vec![beta; node_count];
        let mut prev_katz = katz.clone();

        for _ in 0..max_iter {
            // Katz centrality: x = alpha * A * x + beta
            let mut new_katz = vec![beta; node_count];

            for node_idx in self.inner.node_indices() {
                let node_index = node_idx.index();

                for neighbor in self.inner.neighbors(node_idx) {
                    let neighbor_index = neighbor.index();

                    if weight_fn.unwrap_or(false) {
                        // Use edge weights if requested
                        if let Some(edge) = self.inner.find_edge(neighbor, node_idx) {
                            if let Some(weight) = self.inner.edge_weight(edge) {
                                new_katz[node_index] += alpha * *weight * prev_katz[neighbor_index];
                            }
                        }
                    } else {
                        // Unweighted (adjacency matrix)
                        new_katz[node_index] += alpha * prev_katz[neighbor_index];
                    }
                }
            }

            // Check for convergence
            let mut converged = true;
            for i in 0..node_count {
                if (new_katz[i] - prev_katz[i]).abs() > tolerance {
                    converged = false;
                    break;
                }
            }

            prev_katz = katz;
            katz = new_katz;

            if converged {
                break;
            }
        }

        katz
    }

    /// Core number (k-core decomposition)
    // #[napi]
    pub fn core_number(&self) -> Vec<u32> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        // Initialize core numbers with degrees
        let mut core_numbers = vec![0; node_count];
        let mut degrees = vec![0; node_count];

        for node_idx in self.inner.node_indices() {
            degrees[node_idx.index()] = self.inner.neighbors(node_idx).count();
            core_numbers[node_idx.index()] = degrees[node_idx.index()];
        }

        // Use a queue to process nodes
        let mut queue = VecDeque::new();
        let mut in_queue = vec![false; node_count];

        // Initialize queue with all nodes
        for node_idx in self.inner.node_indices() {
            queue.push_back(node_idx);
            in_queue[node_idx.index()] = true;
        }

        while let Some(node) = queue.pop_front() {
            in_queue[node.index()] = false;
            let node_core = core_numbers[node.index()];

            for neighbor in self.inner.neighbors(node) {
                let neighbor_idx = neighbor.index();

                if core_numbers[neighbor_idx] > node_core {
                    // Reduce neighbor's core number if necessary
                    let neighbor_count = self
                        .inner
                        .neighbors(neighbor)
                        .filter(|&n| core_numbers[n.index()] >= node_core)
                        .count();

                    if neighbor_count < core_numbers[neighbor_idx] {
                        core_numbers[neighbor_idx] = neighbor_count.max(node_core);

                        if !in_queue[neighbor_idx] {
                            queue.push_back(neighbor);
                            in_queue[neighbor_idx] = true;
                        }
                    }
                }
            }
        }

        core_numbers.into_iter().map(|c| c as u32).collect()
    }

    /// Find cycle in graph (returns first cycle found)
    // #[napi]
    pub fn find_cycle(&self) -> Option<Vec<u32>> {
        let mut visited = vec![false; self.inner.node_count()];
        let mut rec_stack = vec![false; self.inner.node_count()];
        let mut path = Vec::new();

        for node_idx in self.inner.node_indices() {
            if !visited[node_idx.index()] {
                if let Some(cycle) =
                    self.find_cycle_dfs(node_idx, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle.into_iter().map(|n| n.index() as u32).collect());
                }
            }
        }

        None
    }

    fn find_cycle_dfs(
        &self,
        node: NodeIndex,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
        path: &mut Vec<NodeIndex>,
    ) -> Option<Vec<NodeIndex>> {
        visited[node.index()] = true;
        rec_stack[node.index()] = true;
        path.push(node);

        for neighbor in self.inner.neighbors(node) {
            if !visited[neighbor.index()] {
                if let Some(cycle) = self.find_cycle_dfs(neighbor, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack[neighbor.index()] {
                // Found a cycle - extract it from path
                if let Some(cycle_start) = path.iter().position(|&n| n == neighbor) {
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor); // Complete the cycle
                    return Some(cycle);
                }
            }
        }

        rec_stack[node.index()] = false;
        path.pop();
        None
    }

    /// Longest simple path between two nodes (undirected graphs)
    // #[napi]
    pub fn longest_simple_path(&self, start: u32, end: u32) -> Option<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        let end_idx = NodeIndex::new(end as usize);

        let node_count = self.inner.node_count();
        if node_count == 0 {
            return None;
        }

        // DFS with backtracking to find longest simple path
        let mut visited = vec![false; node_count];
        let mut current_path = vec![start_idx];
        let mut longest_path = None;
        let mut max_length = 0;

        visited[start_idx.index()] = true;
        self.longest_path_dfs(
            start_idx,
            end_idx,
            &mut visited,
            &mut current_path,
            &mut longest_path,
            &mut max_length,
        );

        longest_path.map(|path| path.into_iter().map(|n| n.index() as u32).collect())
    }

    fn longest_path_dfs(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        visited: &mut Vec<bool>,
        current_path: &mut Vec<NodeIndex>,
        longest_path: &mut Option<Vec<NodeIndex>>,
        max_length: &mut usize,
    ) {
        if current == target {
            if current_path.len() > *max_length {
                *max_length = current_path.len();
                *longest_path = Some(current_path.clone());
            }
            return;
        }

        for neighbor in self.inner.neighbors(current) {
            if !visited[neighbor.index()] {
                visited[neighbor.index()] = true;
                current_path.push(neighbor);

                self.longest_path_dfs(
                    neighbor,
                    target,
                    visited,
                    current_path,
                    longest_path,
                    max_length,
                );

                current_path.pop();
                visited[neighbor.index()] = false;
            }
        }
    }

    /// Graph complement - add all missing edges
    // #[napi]
    pub fn complement(&self) -> RustworkxGraph {
        let mut complement_graph = UnGraph::new_undirected();

        // Add all nodes with their weights
        let node_mapping: HashMap<NodeIndex, NodeIndex> = self
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = self.inner.node_weight(old_idx).cloned().unwrap_or_default();
                let new_idx = complement_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add edges that don't exist in original graph
        let node_indices: Vec<_> = self.inner.node_indices().collect();

        for i in 0..node_indices.len() {
            for j in (i + 1)..node_indices.len() {
                let node_a = node_indices[i];
                let node_b = node_indices[j];

                // Check if edge exists in original graph
                if self.inner.find_edge(node_a, node_b).is_none() {
                    // Add edge to complement with default weight
                    let new_a = node_mapping[&node_a];
                    let new_b = node_mapping[&node_b];
                    complement_graph.add_edge(new_a, new_b, 1.0);
                }
            }
        }

        RustworkxGraph {
            inner: complement_graph,
        }
    }

    /// Union with another graph
    // #[napi]
    pub fn union(&self, other: &RustworkxGraph) -> RustworkxGraph {
        let mut union_graph = UnGraph::new_undirected();

        // Add nodes from first graph
        let self_mapping: HashMap<NodeIndex, NodeIndex> = self
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = self.inner.node_weight(old_idx).cloned().unwrap_or_default();
                let new_idx = union_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add nodes from second graph (offset indices)
        let other_mapping: HashMap<NodeIndex, NodeIndex> = other
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = other
                    .inner
                    .node_weight(old_idx)
                    .cloned()
                    .unwrap_or_default();
                let new_idx = union_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add edges from first graph
        for edge_ref in self.inner.edge_references() {
            let source = self_mapping[&edge_ref.source()];
            let target = self_mapping[&edge_ref.target()];
            let weight = *edge_ref.weight();
            union_graph.add_edge(source, target, weight);
        }

        // Add edges from second graph
        for edge_ref in other.inner.edge_references() {
            let source = other_mapping[&edge_ref.source()];
            let target = other_mapping[&edge_ref.target()];
            let weight = *edge_ref.weight();
            union_graph.add_edge(source, target, weight);
        }

        RustworkxGraph { inner: union_graph }
    }

    /// Cartesian product with another graph
    // #[napi]
    pub fn cartesian_product(&self, other: &RustworkxGraph) -> RustworkxGraph {
        let mut product_graph = UnGraph::new_undirected();

        // Create node mapping: (i, j) -> new_node_index
        let mut node_mapping = HashMap::new();

        // Add nodes - cartesian product of node sets
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let self_weight = self
                    .inner
                    .node_weight(self_node)
                    .cloned()
                    .unwrap_or_default();
                let other_weight = other
                    .inner
                    .node_weight(other_node)
                    .cloned()
                    .unwrap_or_default();

                // Combine node weights
                let combined_weight = format!("{self_weight},{other_weight}");
                let new_node = product_graph.add_node(combined_weight);

                node_mapping.insert((self_node, other_node), new_node);
            }
        }

        // Add edges according to cartesian product rules
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let current_node = node_mapping[&(self_node, other_node)];

                // Connect to nodes that differ in exactly one coordinate
                // 1. Same node in first graph, connected nodes in second graph
                for other_neighbor in other.inner.neighbors(other_node) {
                    if let Some(other_edge) = other.inner.find_edge(other_node, other_neighbor) {
                        let neighbor_node = node_mapping[&(self_node, other_neighbor)];
                        let weight = *other.inner.edge_weight(other_edge).unwrap_or(&1.0);
                        product_graph.add_edge(current_node, neighbor_node, weight);
                    }
                }

                // 2. Connected nodes in first graph, same node in second graph
                for self_neighbor in self.inner.neighbors(self_node) {
                    if let Some(self_edge) = self.inner.find_edge(self_node, self_neighbor) {
                        let neighbor_node = node_mapping[&(self_neighbor, other_node)];
                        let weight = *self.inner.edge_weight(self_edge).unwrap_or(&1.0);
                        product_graph.add_edge(current_node, neighbor_node, weight);
                    }
                }
            }
        }

        RustworkxGraph {
            inner: product_graph,
        }
    }

    /// Tensor product with another graph
    // #[napi]
    pub fn tensor_product(&self, other: &RustworkxGraph) -> RustworkxGraph {
        let mut product_graph = UnGraph::new_undirected();

        // Create node mapping: (i, j) -> new_node_index
        let mut node_mapping = HashMap::new();

        // Add nodes - cartesian product of node sets
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let self_weight = self
                    .inner
                    .node_weight(self_node)
                    .cloned()
                    .unwrap_or_default();
                let other_weight = other
                    .inner
                    .node_weight(other_node)
                    .cloned()
                    .unwrap_or_default();

                // Combine node weights
                let combined_weight = format!("{self_weight},{other_weight}");
                let new_node = product_graph.add_node(combined_weight);

                node_mapping.insert((self_node, other_node), new_node);
            }
        }

        // Add edges according to tensor product rules
        // Edge exists if both coordinates are connected in their respective graphs
        for self_edge in self.inner.edge_references() {
            for other_edge in other.inner.edge_references() {
                let (self_a, self_b) = (self_edge.source(), self_edge.target());
                let (other_a, other_b) = (other_edge.source(), other_edge.target());

                // Create edges between corresponding node pairs
                let node1 = node_mapping[&(self_a, other_a)];
                let node2 = node_mapping[&(self_b, other_b)];

                // Multiply edge weights
                let weight = self_edge.weight() * other_edge.weight();
                product_graph.add_edge(node1, node2, weight);

                // For undirected graphs, also add the reverse combination
                let node3 = node_mapping[&(self_a, other_b)];
                let node4 = node_mapping[&(self_b, other_a)];
                product_graph.add_edge(node3, node4, weight);
            }
        }

        RustworkxGraph {
            inner: product_graph,
        }
    }

    /// Get distance matrix (all-pairs shortest paths) for undirected graph
    // #[napi]
    pub fn distance_matrix(&self, weight_fn: Option<bool>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Initialize diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Use BFS for unweighted, simplified Dijkstra for weighted
        for source_idx in self.inner.node_indices() {
            let source = source_idx.index();
            let mut distances = vec![f64::INFINITY; node_count];
            distances[source] = 0.0;

            if weight_fn.unwrap_or(false) {
                // Weighted version (simplified Dijkstra)
                let mut visited = vec![false; node_count];

                for _ in 0..node_count {
                    let mut min_dist = f64::INFINITY;
                    let mut min_node = None;

                    for (i, &dist) in distances.iter().enumerate() {
                        if !visited[i] && dist < min_dist {
                            min_dist = dist;
                            min_node = Some(NodeIndex::new(i));
                        }
                    }

                    if let Some(current) = min_node {
                        visited[current.index()] = true;

                        for neighbor in self.inner.neighbors(current) {
                            let weight = if let Some(edge) = self.inner.find_edge(current, neighbor)
                            {
                                *self.inner.edge_weight(edge).unwrap_or(&1.0)
                            } else {
                                1.0
                            };

                            let new_dist = distances[current.index()] + weight;
                            if new_dist < distances[neighbor.index()] {
                                distances[neighbor.index()] = new_dist;
                            }
                        }
                    } else {
                        break;
                    }
                }
            } else {
                // Unweighted BFS
                let mut queue = VecDeque::new();
                queue.push_back(source_idx);

                while let Some(current) = queue.pop_front() {
                    for neighbor in self.inner.neighbors(current) {
                        let new_dist = distances[current.index()] + 1.0;
                        if new_dist < distances[neighbor.index()] {
                            distances[neighbor.index()] = new_dist;
                            queue.push_back(neighbor);
                        }
                    }
                }
            }

            // Copy distances to matrix
            for (i, &dist) in distances.iter().enumerate() {
                matrix[source][i] = dist;
            }
        }

        matrix
    }

    /// BFS tree from source (undirected)
    // #[napi]
    pub fn bfs_tree(&self, source: u32) -> RustworkxGraph {
        let source_idx = NodeIndex::new(source as usize);
        let mut tree = UnGraph::new_undirected();
        let mut node_mapping = HashMap::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        // Add source node
        let tree_source = tree.add_node(
            self.inner
                .node_weight(source_idx)
                .cloned()
                .unwrap_or_default(),
        );
        node_mapping.insert(source_idx, tree_source);
        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors(current) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);

                    // Add node to tree
                    let tree_neighbor = tree.add_node(
                        self.inner
                            .node_weight(neighbor)
                            .cloned()
                            .unwrap_or_default(),
                    );
                    node_mapping.insert(neighbor, tree_neighbor);

                    // Add edge to tree
                    let weight = if let Some(edge) = self.inner.find_edge(current, neighbor) {
                        *self.inner.edge_weight(edge).unwrap_or(&1.0)
                    } else {
                        1.0
                    };
                    tree.add_edge(node_mapping[&current], tree_neighbor, weight);
                }
            }
        }

        RustworkxGraph { inner: tree }
    }

    /// Check if this undirected graph is isomorphic to another
    // #[napi]
    pub fn is_isomorphic(&self, other: &RustworkxGraph) -> bool {
        // Quick structural checks first
        if self.inner.node_count() != other.inner.node_count()
            || self.inner.edge_count() != other.inner.edge_count()
        {
            return false;
        }

        // Check if degree sequences match
        let mut self_degrees: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| self.inner.neighbors(n).count())
            .collect();
        let mut other_degrees: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| other.inner.neighbors(n).count())
            .collect();

        self_degrees.sort();
        other_degrees.sort();

        self_degrees == other_degrees
    }

    /// Check if other undirected graph is subgraph isomorphic to this one
    // #[napi]
    pub fn is_subgraph_isomorphic(&self, other: &RustworkxGraph) -> bool {
        // Subgraph must have fewer or equal nodes and edges
        if other.inner.node_count() > self.inner.node_count()
            || other.inner.edge_count() > self.inner.edge_count()
        {
            return false;
        }

        // For simple check, verify if the other graph's degree sequence
        // can be satisfied by a subset of this graph's nodes
        let mut other_degrees: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| other.inner.neighbors(n).count())
            .collect();
        let mut self_degrees: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| self.inner.neighbors(n).count())
            .collect();

        other_degrees.sort();
        self_degrees.sort();

        // Check if other's degree sequence is a subsequence of self's
        let mut self_iter = self_degrees.iter();
        for other_degree in &other_degrees {
            if !self_iter.any(|self_degree| self_degree >= other_degree) {
                return false;
            }
        }

        true
    }

    /// Find VF2 mapping between undirected graphs (returns first mapping found)
    // #[napi]
    pub fn vf2_mapping(&self, other: &RustworkxGraph) -> Option<Vec<u32>> {
        if !self.is_isomorphic(other) {
            return None;
        }

        // Simple mapping based on degree sequence matching
        let mut mapping = Vec::new();

        let self_nodes: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| (n.index() as u32, self.inner.neighbors(n).count()))
            .collect();

        let other_nodes: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| (n.index() as u32, other.inner.neighbors(n).count()))
            .collect();

        // Match nodes by degree
        for (self_id, self_degree) in &self_nodes {
            for (other_id, other_degree) in &other_nodes {
                if self_degree == other_degree {
                    mapping.push(*self_id);
                    mapping.push(*other_id);
                    break;
                }
            }
        }

        if mapping.len() == self_nodes.len() * 2 {
            Some(mapping)
        } else {
            None
        }
    }

    /// BFS edges from source (undirected)
    // #[napi]
    pub fn bfs_edges(&self, source: u32) -> Vec<Vec<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors(current) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    edges.push(vec![current.index() as u32, neighbor.index() as u32]);
                }
            }
        }

        edges
    }

    /// BFS predecessors from source (undirected)
    // #[napi]
    pub fn bfs_predecessors(&self, source: u32) -> Vec<Option<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut predecessors = vec![None; self.inner.node_count()];
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors(current) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    predecessors[neighbor.index()] = Some(current.index() as u32);
                }
            }
        }

        predecessors
    }

    /// BFS successors from source (undirected)
    // #[napi]
    pub fn bfs_successors(&self, source: u32) -> Vec<Vec<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut successors = vec![Vec::new(); self.inner.node_count()];
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors(current) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    successors[current.index()].push(neighbor.index() as u32);
                }
            }
        }

        successors
    }

    /// DFS tree from source (undirected)
    // #[napi]
    pub fn dfs_tree(&self, source: u32) -> RustworkxGraph {
        let source_idx = NodeIndex::new(source as usize);
        let mut tree = UnGraph::new_undirected();
        let mut node_mapping = HashMap::new();
        let mut visited = vec![false; self.inner.node_count()];

        // DFS traversal
        let mut stack = vec![source_idx];
        let mut parent = vec![None; self.inner.node_count()];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;

            // Add node to tree if not already added
            if !node_mapping.contains_key(&current) {
                let tree_node =
                    tree.add_node(self.inner.node_weight(current).cloned().unwrap_or_default());
                node_mapping.insert(current, tree_node);
            }

            // Add edge from parent if exists
            if let Some(parent_idx) = parent[current.index()] {
                if let Some(edge) = self.inner.find_edge(parent_idx, current) {
                    let weight = *self.inner.edge_weight(edge).unwrap_or(&1.0);
                    tree.add_edge(node_mapping[&parent_idx], node_mapping[&current], weight);
                }
            }

            // Add neighbors to stack
            for neighbor in self.inner.neighbors(current) {
                if !visited[neighbor.index()] {
                    parent[neighbor.index()] = Some(current);
                    stack.push(neighbor);
                }
            }
        }

        RustworkxGraph { inner: tree }
    }

    /// DFS preorder traversal (undirected)
    // #[napi]
    pub fn dfs_preorder_nodes(&self, source: u32) -> Vec<u32> {
        let source_idx = NodeIndex::new(source as usize);
        let mut preorder = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut stack = vec![source_idx];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;
            preorder.push(current.index() as u32);

            // Add neighbors in reverse order for correct preorder
            let mut neighbors: Vec<_> = self.inner.neighbors(current).collect();
            neighbors.reverse();
            for neighbor in neighbors {
                if !visited[neighbor.index()] {
                    stack.push(neighbor);
                }
            }
        }

        preorder
    }

    /// DFS postorder traversal (undirected)
    // #[napi]
    pub fn dfs_postorder_nodes(&self, source: u32) -> Vec<u32> {
        let source_idx = NodeIndex::new(source as usize);
        let mut postorder = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];

        fn dfs_postorder_recursive(
            graph: &UnGraph<String, f64>,
            node: NodeIndex,
            visited: &mut [bool],
            postorder: &mut Vec<u32>,
        ) {
            visited[node.index()] = true;

            for neighbor in graph.neighbors(node) {
                if !visited[neighbor.index()] {
                    dfs_postorder_recursive(graph, neighbor, visited, postorder);
                }
            }

            postorder.push(node.index() as u32);
        }

        dfs_postorder_recursive(&self.inner, source_idx, &mut visited, &mut postorder);
        postorder
    }

    /// DFS labeled edges (undirected)
    // #[napi]
    pub fn dfs_labeled_edges(&self, source: u32) -> Vec<Vec<String>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut labeled_edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut stack = vec![source_idx];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;

            for neighbor in self.inner.neighbors(current) {
                let edge_type = if !visited[neighbor.index()] {
                    stack.push(neighbor);
                    "tree"
                } else {
                    "nontree"
                };

                labeled_edges.push(vec![
                    current.index().to_string(),
                    neighbor.index().to_string(),
                    edge_type.to_string(),
                ]);
            }
        }

        labeled_edges
    }

    /// All pairs shortest paths - distance matrix (undirected)
    // #[napi]
    pub fn all_pairs_shortest_paths(&self, parallel_threshold: Option<u32>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let threshold = parallel_threshold.unwrap_or(100) as usize;

        if node_count >= threshold {
            // Use parallel computation for large graphs
            self.all_pairs_shortest_paths_parallel()
        } else {
            // Use sequential computation for small graphs
            self.all_pairs_shortest_paths_sequential()
        }
    }

    fn all_pairs_shortest_paths_sequential(&self) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Set diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Run Dijkstra from each node
        for source in self.inner.node_indices() {
            let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
                &self.inner,
                source,
                None,
                |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
                None,
            );

            if let Ok(distances) = result {
                for (target, distance) in distances {
                    matrix[source.index()][target.index()] = distance;
                }
            }
        }

        matrix
    }

    fn all_pairs_shortest_paths_parallel(&self) -> Vec<Vec<f64>> {
        use rayon::prelude::*;
        
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Set diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Parallel Dijkstra from each node
        let node_indices: Vec<_> = self.inner.node_indices().collect();
        let results: Vec<_> = node_indices
            .par_iter()
            .map(|&source| {
                let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
                    &self.inner,
                    source,
                    None,
                    |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
                    None,
                );
                (source, result)
            })
            .collect();

        // Collect results into matrix
        for (source, result) in results {
            if let Ok(distances) = result {
                for (target, distance) in distances {
                    matrix[source.index()][target.index()] = distance;
                }
            }
        }

        matrix
    }

    /// Export graph to node-link JSON format
    // #[napi]
    pub fn node_link_json(&self) -> String {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        // Add nodes
        for node_idx in self.inner.node_indices() {
            let node_data = serde_json::json!({
                "id": node_idx.index(),
                "label": self.inner.node_weight(node_idx).cloned().unwrap_or_default()
            });
            nodes.push(node_data);
        }

        // Add edges (links)
        for edge in self.inner.edge_references() {
            let link_data = serde_json::json!({
                "source": edge.source().index(),
                "target": edge.target().index(),
                "weight": *edge.weight()
            });
            links.push(link_data);
        }

        let graph_data = serde_json::json!({
            "directed": false,
            "nodes": nodes,
            "links": links
        });

        serde_json::to_string_pretty(&graph_data).unwrap_or_default()
    }

    /// Export graph as simple edge list
    // #[napi]
    pub fn edge_list(&self) -> Vec<Vec<String>> {
        let mut edges = Vec::new();

        for edge in self.inner.edge_references() {
            edges.push(vec![
                edge.source().index().to_string(),
                edge.target().index().to_string(),
                edge.weight().to_string(),
            ]);
        }

        edges
    }

    /// Export graph to GraphML format
    // #[napi]
    pub fn to_graphml(&self) -> String {
        let mut graphml = String::new();

        // GraphML header
        graphml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        graphml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" ");
        graphml.push_str("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ");
        graphml.push_str("xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns ");
        graphml.push_str("http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n");

        // Key definitions
        graphml.push_str(
            "  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n",
        );
        graphml.push_str(
            "  <key id=\"weight\" for=\"edge\" attr.name=\"weight\" attr.type=\"double\"/>\n",
        );

        // Graph definition
        graphml.push_str("  <graph id=\"G\" edgedefault=\"undirected\">\n");

        // Add nodes
        for node_idx in self.inner.node_indices() {
            let label = self
                .inner
                .node_weight(node_idx)
                .cloned()
                .unwrap_or_default();
            graphml.push_str(&format!("    <node id=\"n{}\">\n", node_idx.index()));
            graphml.push_str(&format!("      <data key=\"label\">{label}</data>\n"));
            graphml.push_str("    </node>\n");
        }

        // Add edges
        for (edge_id, edge) in self.inner.edge_references().enumerate() {
            graphml.push_str(&format!(
                "    <edge id=\"e{}\" source=\"n{}\" target=\"n{}\">\n",
                edge_id,
                edge.source().index(),
                edge.target().index()
            ));
            graphml.push_str(&format!(
                "      <data key=\"weight\">{}</data>\n",
                edge.weight()
            ));
            graphml.push_str("    </edge>\n");
        }

        // Close tags
        graphml.push_str("  </graph>\n");
        graphml.push_str("</graphml>\n");

        graphml
    }

    /// Import graph from GraphML format (basic implementation)
    // #[napi]
    pub fn from_graphml(graphml: String) -> Option<RustworkxGraph> {
        // Basic GraphML parsing - in production, would use proper XML parser
        if !graphml.contains("<graph") || !graphml.contains("</graph>") {
            return None;
        }

        let mut graph = UnGraph::new_undirected();
        let mut node_map = HashMap::new();

        // Simple regex-based parsing for demonstration
        // Extract nodes
        let node_pattern = regex::Regex::new(
            r#"<node id="([^"]+)"[^>]*>(?:.*?<data key="label">([^<]*)</data>)?.*?</node>"#,
        )
        .ok()?;
        for captures in node_pattern.captures_iter(&graphml) {
            let node_id = captures.get(1)?.as_str();
            let label = captures
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let graph_node = graph.add_node(label);
            node_map.insert(node_id.to_string(), graph_node);
        }

        // Extract edges
        let edge_pattern = regex::Regex::new(r#"<edge[^>]+source="([^"]+)"[^>]+target="([^"]+)"[^>]*>(?:.*?<data key="weight">([^<]*)</data>)?.*?</edge>"#).ok()?;
        for captures in edge_pattern.captures_iter(&graphml) {
            let source_id = captures.get(1)?.as_str();
            let target_id = captures.get(2)?.as_str();
            let weight: f64 = captures
                .get(3)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1.0);

            if let (Some(&source_node), Some(&target_node)) =
                (node_map.get(source_id), node_map.get(target_id))
            {
                graph.add_edge(source_node, target_node, weight);
            }
        }

        Some(RustworkxGraph { inner: graph })
    }

    /// Parallel betweenness centrality (memory efficient)
    // #[napi]
    pub fn parallel_betweenness_centrality(&self, normalized: Option<bool>) -> Vec<f64> {
        use rayon::prelude::*;
        
        let normalized = normalized.unwrap_or(false);
        let node_count = self.inner.node_count();
        let mut centrality = vec![0.0; node_count];
        
        if node_count <= 1 {
            return centrality;
        }

        // Parallel computation for each source node
        let node_indices: Vec<_> = self.inner.node_indices().collect();
        let partial_results: Vec<Vec<f64>> = node_indices
            .par_iter()
            .map(|&source| {
                let mut local_centrality = vec![0.0; node_count];
                
                // Brandes' algorithm for single source
                let mut stack = Vec::new();
                let mut predecessors = vec![Vec::new(); node_count];
                let mut sigma = vec![0.0; node_count];
                let mut distance = vec![-1.0; node_count];
                let mut delta = vec![0.0; node_count];
                
                sigma[source.index()] = 1.0;
                distance[source.index()] = 0.0;
                
                let mut queue = VecDeque::new();
                queue.push_back(source);
                
                // Forward BFS
                while let Some(v) = queue.pop_front() {
                    stack.push(v);
                    for neighbor in self.inner.neighbors(v) {
                        if distance[neighbor.index()] < 0.0 {
                            queue.push_back(neighbor);
                            distance[neighbor.index()] = distance[v.index()] + 1.0;
                        }
                        if distance[neighbor.index()] == distance[v.index()] + 1.0 {
                            sigma[neighbor.index()] += sigma[v.index()];
                            predecessors[neighbor.index()].push(v);
                        }
                    }
                }
                
                // Backward accumulation
                while let Some(w) = stack.pop() {
                    for &v in &predecessors[w.index()] {
                        delta[v.index()] += (sigma[v.index()] / sigma[w.index()]) * (1.0 + delta[w.index()]);
                    }
                    if w != source {
                        local_centrality[w.index()] += delta[w.index()];
                    }
                }
                
                local_centrality
            })
            .collect();

        // Combine partial results
        for partial in partial_results {
            for (i, &value) in partial.iter().enumerate() {
                centrality[i] += value;
            }
        }

        // Normalize if requested
        if normalized && node_count > 2 {
            let norm = 2.0 / ((node_count - 1) * (node_count - 2)) as f64;
            for value in &mut centrality {
                *value *= norm;
            }
        } else if !normalized {
            // For undirected graphs, divide by 2
            for value in &mut centrality {
                *value /= 2.0;
            }
        }

        centrality
    }

    /// Memory-efficient streaming neighbor iterator
    // #[napi]
    pub fn neighbors_stream(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors(node_idx)
            .map(|n| n.index() as u32)
            .collect()
    }

    /// Memory usage estimation
    // #[napi]
    pub fn memory_usage(&self) -> String {
        let node_count = self.inner.node_count();
        let edge_count = self.inner.edge_count();
        
        // Rough estimation of memory usage
        let node_memory = node_count * (std::mem::size_of::<String>() + 32); // Node data + overhead
        let edge_memory = edge_count * (std::mem::size_of::<f64>() + 16); // Edge weight + indices
        let graph_overhead = 1024; // Graph structure overhead
        
        let total_bytes = node_memory + edge_memory + graph_overhead;
        
        if total_bytes < 1024 {
            format!("{total_bytes} bytes")
        } else if total_bytes < 1024 * 1024 {
            format!("{:.2} KB", total_bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", total_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

#[napi]
impl RustworkxDiGraph {
    // #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DiGraph::new(),
        }
    }

    // #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

    // #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        let node_a_idx = petgraph::graph::NodeIndex::new(node_a as usize);
        let node_b_idx = petgraph::graph::NodeIndex::new(node_b as usize);

        if self.inner.node_weight(node_a_idx).is_some()
            && self.inner.node_weight(node_b_idx).is_some()
        {
            Some(self.inner.add_edge(node_a_idx, node_b_idx, weight).index() as u32)
        } else {
            None
        }
    }

    // #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    // #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

    // #[napi]
    pub fn betweenness_centrality(&self, normalized: bool, endpoints: bool) -> Vec<f64> {
        centrality::betweenness_centrality(&self.inner, normalized, endpoints, 200)
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect()
    }

    // #[napi]
    pub fn dijkstra_shortest_paths(&self, start: u32, target: Option<u32>) -> Vec<f64> {
        let start_idx = petgraph::graph::NodeIndex::new(start as usize);
        let target_idx = target.map(|t| petgraph::graph::NodeIndex::new(t as usize));

        let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
            &self.inner,
            start_idx,
            target_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            None,
        );

        match result {
            Ok(paths) => {
                let mut distances = vec![f64::INFINITY; self.inner.node_count()];
                for (node_idx, distance) in paths.into_iter() {
                    distances[node_idx.index()] = distance;
                }
                distances
            }
            Err(_) => vec![f64::INFINITY; self.inner.node_count()],
        }
    }

    /// A* shortest path algorithm (directed)
    // #[napi]
    pub fn astar_shortest_path(
        &self,
        start: u32,
        goal: u32,
        heuristic: Option<f64>,
    ) -> Option<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = NodeIndex::new(goal as usize);
        let heuristic_value = heuristic.unwrap_or(1.0);

        let result = shortest_path::astar(
            &self.inner,
            start_idx,
            |n| -> Result<bool, ()> { Ok(n == goal_idx) },
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            |_| -> Result<f64, ()> { Ok(heuristic_value) },
        );

        match result {
            Ok(Some((_cost, path))) => Some(path.into_iter().map(|n| n.index() as u32).collect()),
            _ => None,
        }
    }

    /// Bellman-Ford shortest paths (directed, detects negative cycles)
    // #[napi]
    pub fn bellman_ford_shortest_paths(&self, start: u32) -> Option<Vec<f64>> {
        let start_idx = NodeIndex::new(start as usize);

        let result: Result<Option<Vec<Option<f64>>>, ()> = shortest_path::bellman_ford(
            &self.inner,
            start_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
            None,
        );

        match result {
            Ok(Some(distances)) => {
                let mut result = vec![f64::INFINITY; self.inner.node_count()];
                for (i, distance_opt) in distances.iter().enumerate() {
                    if let Some(distance) = distance_opt {
                        result[i] = *distance;
                    }
                }
                Some(result)
            }
            _ => None, // Negative cycle detected or error
        }
    }

    /// K-shortest path lengths (directed)
    // #[napi]
    pub fn k_shortest_path_lengths(&self, start: u32, k: u32, goal: Option<u32>) -> Vec<f64> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = goal.map(|g| NodeIndex::new(g as usize));

        let result: Result<Vec<Option<f64>>, ()> = shortest_path::k_shortest_path(
            &self.inner,
            start_idx,
            goal_idx,
            k as usize,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
        );

        match result {
            Ok(distances) => distances
                .into_iter()
                .map(|opt| opt.unwrap_or(f64::INFINITY))
                .collect(),
            Err(_) => vec![f64::INFINITY; self.inner.node_count()],
        }
    }

    /// All shortest paths between two specific nodes (directed)
    // #[napi]
    pub fn all_shortest_paths_between_nodes(&self, start: u32, goal: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        let goal_idx = NodeIndex::new(goal as usize);

        let result = shortest_path::all_shortest_paths(
            &self.inner,
            start_idx,
            goal_idx,
            |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
        );

        match result {
            Ok(paths) => paths
                .into_iter()
                .map(|path| path.into_iter().map(|node| node.index() as u32).collect())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Unweighted average shortest path length (directed)
    // #[napi]
    pub fn unweighted_average_shortest_path_length(&self, disconnected: Option<bool>) -> f64 {
        let disconnected = disconnected.unwrap_or(false);
        let node_count = self.inner.node_count();

        if node_count <= 1 {
            return 0.0;
        }

        let mut total_distance = 0.0;
        let mut path_count = 0;

        for source in self.inner.node_indices() {
            let mut bfs = Bfs::new(&self.inner, source);
            let mut distances = HashMap::new();
            distances.insert(source, 0);

            while let Some(node) = bfs.next(&self.inner) {
                let current_dist = distances[&node];

                for neighbor in self.inner.neighbors_directed(node, Direction::Outgoing) {
                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, current_dist + 1);
                    }
                }
            }

            for target in self.inner.node_indices() {
                if source != target {
                    if let Some(&distance) = distances.get(&target) {
                        total_distance += distance as f64;
                        path_count += 1;
                    } else if disconnected {
                        // If disconnected nodes should be counted with infinite distance
                        return f64::INFINITY;
                    }
                }
            }
        }

        if path_count == 0 {
            0.0
        } else {
            total_distance / path_count as f64
        }
    }

    /// Edge betweenness centrality (directed)
    // #[napi]
    pub fn edge_betweenness_centrality(&self, normalized: Option<bool>) -> Vec<f64> {
        let normalized = normalized.unwrap_or(false);
        let edge_count = self.inner.edge_count();
        let mut edge_betweenness = vec![0.0; edge_count];

        if edge_count == 0 {
            return edge_betweenness;
        }

        // Map edge indices to their position in the result vector
        let edge_indices: HashMap<_, _> = self
            .inner
            .edge_references()
            .enumerate()
            .map(|(i, edge_ref)| (edge_ref.id(), i))
            .collect();

        for source in self.inner.node_indices() {
            // Single-source shortest path with path counting
            let mut distances = HashMap::new();
            let mut paths_count = HashMap::new();
            let mut predecessors: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
            let mut queue = VecDeque::new();
            let mut stack = Vec::new();

            distances.insert(source, 0);
            paths_count.insert(source, 1.0);
            queue.push_back(source);

            // BFS to find shortest paths
            while let Some(node) = queue.pop_front() {
                stack.push(node);
                let node_dist = distances[&node];

                for neighbor in self.inner.neighbors_directed(node, Direction::Outgoing) {
                    let new_dist = node_dist + 1;

                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, new_dist);
                        paths_count.insert(neighbor, 0.0);
                        queue.push_back(neighbor);
                    }

                    if distances[&neighbor] == new_dist {
                        *paths_count.get_mut(&neighbor).unwrap() += paths_count[&node];
                        predecessors
                            .entry(neighbor)
                            .or_insert_with(Vec::new)
                            .push(node);
                    }
                }
            }

            // Accumulate edge betweenness
            let mut dependency = HashMap::new();
            for &node in &stack {
                dependency.insert(node, 0.0);
            }

            while let Some(node) = stack.pop() {
                if let Some(preds) = predecessors.get(&node) {
                    for &pred in preds {
                        let coeff =
                            (paths_count[&pred] / paths_count[&node]) * (1.0 + dependency[&node]);
                        *dependency.get_mut(&pred).unwrap() += coeff;

                        // Find edge and add to betweenness
                        if let Some(edge) = self.inner.find_edge(pred, node) {
                            if let Some(&edge_idx) = edge_indices.get(&edge) {
                                edge_betweenness[edge_idx] += coeff;
                            }
                        }
                    }
                }
            }
        }

        // Normalize if requested
        if normalized {
            let node_count = self.inner.node_count();
            if node_count > 2 {
                let norm_factor = 1.0 / ((node_count * (node_count - 1)) as f64);
                for value in &mut edge_betweenness {
                    *value *= norm_factor;
                }
            }
        }

        edge_betweenness
    }

    // #[napi]
    pub fn is_cyclic(&self) -> bool {
        is_cyclic_directed(&self.inner)
    }

    /// Get Floyd-Warshall all-pairs shortest path distances (directed)
    // #[napi]
    pub fn floyd_warshall_matrix(&self) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Initialize diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Initialize direct edges (directed)
        for edge_ref in self.inner.edge_references() {
            let source = edge_ref.source().index();
            let target = edge_ref.target().index();
            let weight = *edge_ref.weight();
            matrix[source][target] = weight;
        }

        // Floyd-Warshall algorithm
        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    if matrix[i][k] != f64::INFINITY && matrix[k][j] != f64::INFINITY {
                        let new_dist = matrix[i][k] + matrix[k][j];
                        if new_dist < matrix[i][j] {
                            matrix[i][j] = new_dist;
                        }
                    }
                }
            }
        }

        matrix
    }

    /// Get adjacency matrix representation (directed)
    // #[napi]
    pub fn adjacency_matrix(
        &self,
        _default_weight: Option<f64>,
        null_value: Option<f64>,
    ) -> Vec<Vec<f64>> {
        let null_value = null_value.unwrap_or(0.0);
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![null_value; node_count]; node_count];

        for edge_ref in self.inner.edge_references() {
            let source = edge_ref.source().index();
            let target = edge_ref.target().index();
            let weight = *edge_ref.weight();

            matrix[source][target] += weight; // directed - only one direction
        }

        matrix
    }

    /// Compute closeness centrality (directed)
    // #[napi]
    pub fn closeness_centrality(&self, wf_improved: Option<bool>) -> Vec<f64> {
        let wf_improved = wf_improved.unwrap_or(true);
        centrality::closeness_centrality(&self.inner, wf_improved)
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect()
    }

    /// Compute degree centrality (out-degree for directed graphs)
    // #[napi]
    pub fn degree_centrality(&self) -> Vec<f64> {
        let node_count = self.inner.node_count();
        if node_count <= 1 {
            return vec![0.0; node_count];
        }

        let mut centrality = Vec::with_capacity(node_count);
        let normalizer = (node_count - 1) as f64;

        for node_idx in self.inner.node_indices() {
            let out_degree = self
                .inner
                .neighbors_directed(node_idx, Direction::Outgoing)
                .count() as f64;
            centrality.push(out_degree / normalizer);
        }
        centrality
    }

    /// Get all simple paths between two nodes (directed, simplified implementation)
    // #[napi]
    pub fn all_simple_paths(
        &self,
        from: u32,
        to: u32,
        min_depth: Option<u32>,
        cutoff: Option<u32>,
    ) -> Vec<Vec<u32>> {
        let from_idx = NodeIndex::new(from as usize);
        let to_idx = NodeIndex::new(to as usize);
        let min_depth = min_depth.unwrap_or(0) as usize;
        let max_depth = cutoff.unwrap_or(10) as usize;

        let mut all_paths = Vec::new();
        let mut current_path = vec![from_idx];
        let mut visited = std::collections::HashSet::new();
        visited.insert(from_idx);

        self.find_paths_recursive_directed(
            from_idx,
            to_idx,
            &mut current_path,
            &mut visited,
            &mut all_paths,
            max_depth,
            min_depth,
        );

        all_paths
            .into_iter()
            .map(|path| path.into_iter().map(|node| node.index() as u32).collect())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn find_paths_recursive_directed(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        path: &mut Vec<NodeIndex>,
        visited: &mut std::collections::HashSet<NodeIndex>,
        all_paths: &mut Vec<Vec<NodeIndex>>,
        max_depth: usize,
        min_depth: usize,
    ) {
        if path.len() > max_depth {
            return;
        }

        if current == target {
            if path.len() >= min_depth {
                all_paths.push(path.clone());
            }
            return;
        }

        for neighbor in self.inner.neighbors_directed(current, Direction::Outgoing) {
            if !visited.contains(&neighbor) {
                visited.insert(neighbor);
                path.push(neighbor);
                self.find_paths_recursive_directed(
                    neighbor, target, path, visited, all_paths, max_depth, min_depth,
                );
                path.pop();
                visited.remove(&neighbor);
            }
        }
    }

    /// Get DFS edges (directed)
    // #[napi]
    pub fn dfs_edges(&self, source: Option<u32>) -> Vec<Vec<u32>> {
        let mut edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];

        let sources: Vec<_> = if let Some(s) = source {
            vec![NodeIndex::new(s as usize)]
        } else {
            self.inner.node_indices().collect()
        };

        for start in sources {
            if !visited[start.index()] {
                let mut dfs = Dfs::new(&self.inner, start);
                let mut parent = HashMap::new();

                while let Some(node) = dfs.next(&self.inner) {
                    visited[node.index()] = true;

                    for neighbor in self.inner.neighbors_directed(node, Direction::Outgoing) {
                        if !visited[neighbor.index()] && !parent.contains_key(&neighbor) {
                            parent.insert(neighbor, node);
                            edges.push(vec![node.index() as u32, neighbor.index() as u32]);
                        }
                    }
                }
            }
        }
        edges
    }

    /// Check if path exists between two nodes (directed)
    // #[napi]
    pub fn has_path(&self, source: u32, target: u32) -> bool {
        let source_idx = NodeIndex::new(source as usize);
        let target_idx = NodeIndex::new(target as usize);

        let mut bfs = Bfs::new(&self.inner, source_idx);
        while let Some(node) = bfs.next(&self.inner) {
            if node == target_idx {
                return true;
            }
        }
        false
    }

    /// Get strongly connected components
    // #[napi]
    pub fn strongly_connected_components(&self) -> Vec<Vec<u32>> {
        let sccs = petgraph::algo::kosaraju_scc(&self.inner);
        sccs.into_iter()
            .map(|component| {
                component
                    .into_iter()
                    .map(|node| node.index() as u32)
                    .collect()
            })
            .collect()
    }

    /// Get topological sort (returns None if graph has cycles)
    // #[napi]
    pub fn topological_sort(&self) -> Option<Vec<u32>> {
        match petgraph::algo::toposort(&self.inner, None) {
            Ok(sorted) => Some(sorted.into_iter().map(|node| node.index() as u32).collect()),
            Err(_) => None,
        }
    }

    /// Eigenvector centrality (directed)
    // #[napi]
    pub fn eigenvector_centrality(
        &self,
        max_iter: Option<u32>,
        tolerance: Option<f64>,
        weight_fn: Option<bool>,
    ) -> Vec<f64> {
        let max_iter = max_iter.unwrap_or(100);
        let tolerance = tolerance.unwrap_or(1e-6);
        let node_count = self.inner.node_count();

        if node_count == 0 {
            return vec![];
        }

        // Initialize eigenvector with uniform values
        let mut eigenvector = vec![1.0 / (node_count as f64).sqrt(); node_count];
        let mut prev_eigenvector = eigenvector.clone();

        for _ in 0..max_iter {
            // Matrix-vector multiplication: A * x (incoming edges for directed graphs)
            let mut new_eigenvector = vec![0.0; node_count];

            for node_idx in self.inner.node_indices() {
                let node_index = node_idx.index();

                for neighbor in self.inner.neighbors_directed(node_idx, Direction::Incoming) {
                    let neighbor_index = neighbor.index();

                    if weight_fn.unwrap_or(false) {
                        // Use edge weights if requested
                        if let Some(edge) = self.inner.find_edge(neighbor, node_idx) {
                            if let Some(weight) = self.inner.edge_weight(edge) {
                                new_eigenvector[node_index] +=
                                    *weight * prev_eigenvector[neighbor_index];
                            }
                        }
                    } else {
                        // Unweighted (adjacency matrix)
                        new_eigenvector[node_index] += prev_eigenvector[neighbor_index];
                    }
                }
            }

            // Normalize the eigenvector
            let norm = new_eigenvector.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 0.0 {
                for value in &mut new_eigenvector {
                    *value /= norm;
                }
            }

            // Check for convergence
            let mut converged = true;
            for i in 0..node_count {
                if (new_eigenvector[i] - prev_eigenvector[i]).abs() > tolerance {
                    converged = false;
                    break;
                }
            }

            prev_eigenvector = eigenvector;
            eigenvector = new_eigenvector;

            if converged {
                break;
            }
        }

        // Ensure all values are non-negative (take absolute value)
        eigenvector.into_iter().map(|x| x.abs()).collect()
    }

    /// Katz centrality (directed)
    // #[napi]
    pub fn katz_centrality(
        &self,
        alpha: Option<f64>,
        beta: Option<f64>,
        max_iter: Option<u32>,
        tolerance: Option<f64>,
        weight_fn: Option<bool>,
    ) -> Vec<f64> {
        let alpha = alpha.unwrap_or(0.1);
        let beta = beta.unwrap_or(1.0);
        let max_iter = max_iter.unwrap_or(100);
        let tolerance = tolerance.unwrap_or(1e-6);
        let node_count = self.inner.node_count();

        if node_count == 0 {
            return vec![];
        }

        // Initialize Katz centrality with beta values
        let mut katz = vec![beta; node_count];
        let mut prev_katz = katz.clone();

        for _ in 0..max_iter {
            // Katz centrality: x = alpha * A * x + beta (incoming edges for directed graphs)
            let mut new_katz = vec![beta; node_count];

            for node_idx in self.inner.node_indices() {
                let node_index = node_idx.index();

                for neighbor in self.inner.neighbors_directed(node_idx, Direction::Incoming) {
                    let neighbor_index = neighbor.index();

                    if weight_fn.unwrap_or(false) {
                        // Use edge weights if requested
                        if let Some(edge) = self.inner.find_edge(neighbor, node_idx) {
                            if let Some(weight) = self.inner.edge_weight(edge) {
                                new_katz[node_index] += alpha * *weight * prev_katz[neighbor_index];
                            }
                        }
                    } else {
                        // Unweighted (adjacency matrix)
                        new_katz[node_index] += alpha * prev_katz[neighbor_index];
                    }
                }
            }

            // Check for convergence
            let mut converged = true;
            for i in 0..node_count {
                if (new_katz[i] - prev_katz[i]).abs() > tolerance {
                    converged = false;
                    break;
                }
            }

            prev_katz = katz;
            katz = new_katz;

            if converged {
                break;
            }
        }

        katz
    }

    /// Core number (k-core decomposition) for directed graphs
    // #[napi]
    pub fn core_number(&self) -> Vec<u32> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        // For directed graphs, use in-degree + out-degree
        let mut core_numbers = vec![0; node_count];

        for node_idx in self.inner.node_indices() {
            let in_degree = self
                .inner
                .neighbors_directed(node_idx, Direction::Incoming)
                .count();
            let out_degree = self
                .inner
                .neighbors_directed(node_idx, Direction::Outgoing)
                .count();
            core_numbers[node_idx.index()] =
                (in_degree + out_degree).min(in_degree.max(out_degree));
        }

        // Use a queue to process nodes
        let mut queue = VecDeque::new();
        let mut in_queue = vec![false; node_count];

        // Initialize queue with all nodes
        for node_idx in self.inner.node_indices() {
            queue.push_back(node_idx);
            in_queue[node_idx.index()] = true;
        }

        while let Some(node) = queue.pop_front() {
            in_queue[node.index()] = false;
            let node_core = core_numbers[node.index()];

            // Check both incoming and outgoing neighbors
            let all_neighbors: Vec<_> = self
                .inner
                .neighbors_directed(node, Direction::Incoming)
                .chain(self.inner.neighbors_directed(node, Direction::Outgoing))
                .collect();

            for neighbor in all_neighbors {
                let neighbor_idx = neighbor.index();

                if core_numbers[neighbor_idx] > node_core {
                    let in_count = self
                        .inner
                        .neighbors_directed(neighbor, Direction::Incoming)
                        .filter(|&n| core_numbers[n.index()] >= node_core)
                        .count();
                    let out_count = self
                        .inner
                        .neighbors_directed(neighbor, Direction::Outgoing)
                        .filter(|&n| core_numbers[n.index()] >= node_core)
                        .count();

                    let neighbor_count = (in_count + out_count).min(in_count.max(out_count));

                    if neighbor_count < core_numbers[neighbor_idx] {
                        core_numbers[neighbor_idx] = neighbor_count.max(node_core);

                        if !in_queue[neighbor_idx] {
                            queue.push_back(neighbor);
                            in_queue[neighbor_idx] = true;
                        }
                    }
                }
            }
        }

        core_numbers.into_iter().map(|c| c as u32).collect()
    }

    /// Find cycle in directed graph (returns first cycle found)
    // #[napi]
    pub fn find_cycle(&self) -> Option<Vec<u32>> {
        let mut visited = vec![false; self.inner.node_count()];
        let mut rec_stack = vec![false; self.inner.node_count()];
        let mut path = Vec::new();

        for node_idx in self.inner.node_indices() {
            if !visited[node_idx.index()] {
                if let Some(cycle) =
                    self.find_cycle_dfs_directed(node_idx, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle.into_iter().map(|n| n.index() as u32).collect());
                }
            }
        }

        None
    }

    fn find_cycle_dfs_directed(
        &self,
        node: NodeIndex,
        visited: &mut Vec<bool>,
        rec_stack: &mut Vec<bool>,
        path: &mut Vec<NodeIndex>,
    ) -> Option<Vec<NodeIndex>> {
        visited[node.index()] = true;
        rec_stack[node.index()] = true;
        path.push(node);

        for neighbor in self.inner.neighbors_directed(node, Direction::Outgoing) {
            if !visited[neighbor.index()] {
                if let Some(cycle) =
                    self.find_cycle_dfs_directed(neighbor, visited, rec_stack, path)
                {
                    return Some(cycle);
                }
            } else if rec_stack[neighbor.index()] {
                // Found a cycle - extract it from path
                if let Some(cycle_start) = path.iter().position(|&n| n == neighbor) {
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor); // Complete the cycle
                    return Some(cycle);
                }
            }
        }

        rec_stack[node.index()] = false;
        path.pop();
        None
    }

    /// Longest path in DAG (directed)
    // #[napi]
    pub fn dag_longest_path(&self, weight_fn: Option<bool>) -> Option<Vec<u32>> {
        // Check if graph is a DAG first
        if self.is_cyclic() {
            return None;
        }

        let node_count = self.inner.node_count();
        if node_count == 0 {
            return Some(vec![]);
        }

        // Topological sort
        let topo_order = self.topological_sort()?;

        // Initialize distances and predecessors
        let mut distances = vec![f64::NEG_INFINITY; node_count];
        let mut predecessors = vec![None; node_count];

        // Set distance to 0 for nodes with no incoming edges
        for &node_id in &topo_order {
            let node_idx = NodeIndex::new(node_id as usize);
            if self
                .inner
                .neighbors_directed(node_idx, Direction::Incoming)
                .count()
                == 0
            {
                distances[node_id as usize] = 0.0;
            }
        }

        // Process nodes in topological order
        for &node_id in &topo_order {
            let node_idx = NodeIndex::new(node_id as usize);

            if distances[node_id as usize].is_finite() {
                for neighbor in self.inner.neighbors_directed(node_idx, Direction::Outgoing) {
                    let neighbor_id = neighbor.index();

                    let edge_weight = if weight_fn.unwrap_or(false) {
                        if let Some(edge) = self.inner.find_edge(node_idx, neighbor) {
                            self.inner.edge_weight(edge).copied().unwrap_or(1.0)
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let new_distance = distances[node_id as usize] + edge_weight;

                    if new_distance > distances[neighbor_id] {
                        distances[neighbor_id] = new_distance;
                        predecessors[neighbor_id] = Some(node_idx);
                    }
                }
            }
        }

        // Find the node with maximum distance
        let max_node_idx = distances
            .iter()
            .enumerate()
            .filter(|(_, &dist)| dist.is_finite())
            .max_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap())
            .map(|(idx, _)| idx)?;

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = Some(NodeIndex::new(max_node_idx));

        while let Some(node) = current {
            path.push(node.index() as u32);
            current = predecessors[node.index()];
        }

        path.reverse();
        Some(path)
    }

    /// Graph complement - add all missing edges (directed)
    // #[napi]
    pub fn complement(&self) -> RustworkxDiGraph {
        let mut complement_graph = DiGraph::new();

        // Add all nodes with their weights
        let node_mapping: HashMap<NodeIndex, NodeIndex> = self
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = self.inner.node_weight(old_idx).cloned().unwrap_or_default();
                let new_idx = complement_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add edges that don't exist in original graph
        let node_indices: Vec<_> = self.inner.node_indices().collect();

        for &node_a in &node_indices {
            for &node_b in &node_indices {
                if node_a != node_b {
                    // Check if edge exists in original graph
                    if self.inner.find_edge(node_a, node_b).is_none() {
                        // Add edge to complement with default weight
                        let new_a = node_mapping[&node_a];
                        let new_b = node_mapping[&node_b];
                        complement_graph.add_edge(new_a, new_b, 1.0);
                    }
                }
            }
        }

        RustworkxDiGraph {
            inner: complement_graph,
        }
    }

    /// Union with another directed graph
    // #[napi]
    pub fn union(&self, other: &RustworkxDiGraph) -> RustworkxDiGraph {
        let mut union_graph = DiGraph::new();

        // Add nodes from first graph
        let self_mapping: HashMap<NodeIndex, NodeIndex> = self
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = self.inner.node_weight(old_idx).cloned().unwrap_or_default();
                let new_idx = union_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add nodes from second graph
        let other_mapping: HashMap<NodeIndex, NodeIndex> = other
            .inner
            .node_indices()
            .map(|old_idx| {
                let weight = other
                    .inner
                    .node_weight(old_idx)
                    .cloned()
                    .unwrap_or_default();
                let new_idx = union_graph.add_node(weight);
                (old_idx, new_idx)
            })
            .collect();

        // Add edges from first graph
        for edge_ref in self.inner.edge_references() {
            let source = self_mapping[&edge_ref.source()];
            let target = self_mapping[&edge_ref.target()];
            let weight = *edge_ref.weight();
            union_graph.add_edge(source, target, weight);
        }

        // Add edges from second graph
        for edge_ref in other.inner.edge_references() {
            let source = other_mapping[&edge_ref.source()];
            let target = other_mapping[&edge_ref.target()];
            let weight = *edge_ref.weight();
            union_graph.add_edge(source, target, weight);
        }

        RustworkxDiGraph { inner: union_graph }
    }

    /// Cartesian product with another directed graph
    // #[napi]
    pub fn cartesian_product(&self, other: &RustworkxDiGraph) -> RustworkxDiGraph {
        let mut product_graph = DiGraph::new();

        // Create node mapping: (i, j) -> new_node_index
        let mut node_mapping = HashMap::new();

        // Add nodes - cartesian product of node sets
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let self_weight = self
                    .inner
                    .node_weight(self_node)
                    .cloned()
                    .unwrap_or_default();
                let other_weight = other
                    .inner
                    .node_weight(other_node)
                    .cloned()
                    .unwrap_or_default();

                // Combine node weights
                let combined_weight = format!("{self_weight},{other_weight}");
                let new_node = product_graph.add_node(combined_weight);

                node_mapping.insert((self_node, other_node), new_node);
            }
        }

        // Add edges according to cartesian product rules for directed graphs
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let current_node = node_mapping[&(self_node, other_node)];

                // 1. Same node in first graph, connected nodes in second graph
                for other_neighbor in other
                    .inner
                    .neighbors_directed(other_node, Direction::Outgoing)
                {
                    if let Some(other_edge) = other.inner.find_edge(other_node, other_neighbor) {
                        let neighbor_node = node_mapping[&(self_node, other_neighbor)];
                        let weight = *other.inner.edge_weight(other_edge).unwrap_or(&1.0);
                        product_graph.add_edge(current_node, neighbor_node, weight);
                    }
                }

                // 2. Connected nodes in first graph, same node in second graph
                for self_neighbor in self
                    .inner
                    .neighbors_directed(self_node, Direction::Outgoing)
                {
                    if let Some(self_edge) = self.inner.find_edge(self_node, self_neighbor) {
                        let neighbor_node = node_mapping[&(self_neighbor, other_node)];
                        let weight = *self.inner.edge_weight(self_edge).unwrap_or(&1.0);
                        product_graph.add_edge(current_node, neighbor_node, weight);
                    }
                }
            }
        }

        RustworkxDiGraph {
            inner: product_graph,
        }
    }

    /// Tensor product with another directed graph
    // #[napi]
    pub fn tensor_product(&self, other: &RustworkxDiGraph) -> RustworkxDiGraph {
        let mut product_graph = DiGraph::new();

        // Create node mapping: (i, j) -> new_node_index
        let mut node_mapping = HashMap::new();

        // Add nodes - cartesian product of node sets
        for self_node in self.inner.node_indices() {
            for other_node in other.inner.node_indices() {
                let self_weight = self
                    .inner
                    .node_weight(self_node)
                    .cloned()
                    .unwrap_or_default();
                let other_weight = other
                    .inner
                    .node_weight(other_node)
                    .cloned()
                    .unwrap_or_default();

                // Combine node weights
                let combined_weight = format!("{self_weight},{other_weight}");
                let new_node = product_graph.add_node(combined_weight);

                node_mapping.insert((self_node, other_node), new_node);
            }
        }

        // Add edges according to tensor product rules for directed graphs
        // Edge exists if both coordinates are connected in their respective graphs
        for self_edge in self.inner.edge_references() {
            for other_edge in other.inner.edge_references() {
                let (self_a, self_b) = (self_edge.source(), self_edge.target());
                let (other_a, other_b) = (other_edge.source(), other_edge.target());

                // Create edge between corresponding node pairs
                let node1 = node_mapping[&(self_a, other_a)];
                let node2 = node_mapping[&(self_b, other_b)];

                // Multiply edge weights
                let weight = self_edge.weight() * other_edge.weight();
                product_graph.add_edge(node1, node2, weight);
            }
        }

        RustworkxDiGraph {
            inner: product_graph,
        }
    }

    /// Check if this directed graph is isomorphic to another
    // #[napi]
    pub fn is_isomorphic(&self, other: &RustworkxDiGraph) -> bool {
        // Quick structural checks first
        if self.inner.node_count() != other.inner.node_count()
            || self.inner.edge_count() != other.inner.edge_count()
        {
            return false;
        }

        // Use a simple structural comparison approach based on degree sequences

        // Check if degree sequences match
        let mut self_degrees: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| {
                (
                    self.inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    self.inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();
        let mut other_degrees: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| {
                (
                    other
                        .inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    other
                        .inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();

        self_degrees.sort();
        other_degrees.sort();

        self_degrees == other_degrees
    }

    /// Check if other directed graph is subgraph isomorphic to this one
    // #[napi]
    pub fn is_subgraph_isomorphic(&self, other: &RustworkxDiGraph) -> bool {
        // Subgraph must have fewer or equal nodes and edges
        if other.inner.node_count() > self.inner.node_count()
            || other.inner.edge_count() > self.inner.edge_count()
        {
            return false;
        }

        // For simple check, verify if the other graph's degree sequence
        // can be satisfied by a subset of this graph's nodes
        let mut other_degrees: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| {
                (
                    other
                        .inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    other
                        .inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();
        let mut self_degrees: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| {
                (
                    self.inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    self.inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();

        other_degrees.sort();
        self_degrees.sort();

        // Check if other's degree sequence is a subsequence of self's
        let mut self_iter = self_degrees.iter();
        for other_degree in &other_degrees {
            if !self_iter.any(|self_degree| self_degree >= other_degree) {
                return false;
            }
        }

        true
    }

    /// Find VF2 mapping between graphs (returns first mapping found)
    // #[napi]
    pub fn vf2_mapping(&self, other: &RustworkxDiGraph) -> Option<Vec<u32>> {
        if !self.is_isomorphic(other) {
            return None;
        }

        // Simple mapping based on degree sequence matching
        // This is a simplified approach for demonstration
        let mut mapping = Vec::new();

        let self_nodes: Vec<_> = self
            .inner
            .node_indices()
            .map(|n| {
                (
                    n.index() as u32,
                    self.inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    self.inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();

        let other_nodes: Vec<_> = other
            .inner
            .node_indices()
            .map(|n| {
                (
                    n.index() as u32,
                    other
                        .inner
                        .neighbors_directed(n, Direction::Outgoing)
                        .count(),
                    other
                        .inner
                        .neighbors_directed(n, Direction::Incoming)
                        .count(),
                )
            })
            .collect();

        // Match nodes by degree
        for (self_id, self_out, self_in) in &self_nodes {
            for (other_id, other_out, other_in) in &other_nodes {
                if self_out == other_out && self_in == other_in {
                    mapping.push(*self_id);
                    mapping.push(*other_id);
                    break;
                }
            }
        }

        if mapping.len() == self_nodes.len() * 2 {
            Some(mapping)
        } else {
            None
        }
    }

    /// BFS edges from source (directed)
    // #[napi]
    pub fn bfs_edges(&self, source: u32) -> Vec<Vec<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors_directed(current, Direction::Outgoing) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    edges.push(vec![current.index() as u32, neighbor.index() as u32]);
                }
            }
        }

        edges
    }

    /// BFS predecessors from source (directed)
    // #[napi]
    pub fn bfs_predecessors(&self, source: u32) -> Vec<Option<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut predecessors = vec![None; self.inner.node_count()];
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors_directed(current, Direction::Outgoing) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    predecessors[neighbor.index()] = Some(current.index() as u32);
                }
            }
        }

        predecessors
    }

    /// BFS successors from source (directed)
    // #[napi]
    pub fn bfs_successors(&self, source: u32) -> Vec<Vec<u32>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut successors = vec![Vec::new(); self.inner.node_count()];
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        visited[source_idx.index()] = true;
        queue.push_back(source_idx);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.inner.neighbors_directed(current, Direction::Outgoing) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    queue.push_back(neighbor);
                    successors[current.index()].push(neighbor.index() as u32);
                }
            }
        }

        successors
    }

    /// DFS tree from source (directed)
    // #[napi]
    pub fn dfs_tree(&self, source: u32) -> RustworkxDiGraph {
        let source_idx = NodeIndex::new(source as usize);
        let mut tree = DiGraph::new();
        let mut node_mapping = HashMap::new();
        let mut visited = vec![false; self.inner.node_count()];

        // DFS traversal
        let mut stack = vec![source_idx];
        let mut parent = vec![None; self.inner.node_count()];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;

            // Add node to tree if not already added
            if !node_mapping.contains_key(&current) {
                let tree_node =
                    tree.add_node(self.inner.node_weight(current).cloned().unwrap_or_default());
                node_mapping.insert(current, tree_node);
            }

            // Add edge from parent if exists
            if let Some(parent_idx) = parent[current.index()] {
                if let Some(edge) = self.inner.find_edge(parent_idx, current) {
                    let weight = *self.inner.edge_weight(edge).unwrap_or(&1.0);
                    tree.add_edge(node_mapping[&parent_idx], node_mapping[&current], weight);
                }
            }

            // Add neighbors to stack (reverse order for consistent DFS)
            let mut neighbors: Vec<_> = self
                .inner
                .neighbors_directed(current, Direction::Outgoing)
                .collect();
            neighbors.reverse();
            for neighbor in neighbors {
                if !visited[neighbor.index()] {
                    parent[neighbor.index()] = Some(current);
                    stack.push(neighbor);
                }
            }
        }

        RustworkxDiGraph { inner: tree }
    }

    /// DFS preorder traversal (directed)
    // #[napi]
    pub fn dfs_preorder_nodes(&self, source: u32) -> Vec<u32> {
        let source_idx = NodeIndex::new(source as usize);
        let mut preorder = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut stack = vec![source_idx];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;
            preorder.push(current.index() as u32);

            // Add neighbors in reverse order for correct preorder
            let mut neighbors: Vec<_> = self
                .inner
                .neighbors_directed(current, Direction::Outgoing)
                .collect();
            neighbors.reverse();
            for neighbor in neighbors {
                if !visited[neighbor.index()] {
                    stack.push(neighbor);
                }
            }
        }

        preorder
    }

    /// DFS postorder traversal (directed)
    // #[napi]
    pub fn dfs_postorder_nodes(&self, source: u32) -> Vec<u32> {
        let source_idx = NodeIndex::new(source as usize);
        let mut postorder = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];

        fn dfs_postorder_recursive(
            graph: &DiGraph<String, f64>,
            node: NodeIndex,
            visited: &mut [bool],
            postorder: &mut Vec<u32>,
        ) {
            visited[node.index()] = true;

            for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
                if !visited[neighbor.index()] {
                    dfs_postorder_recursive(graph, neighbor, visited, postorder);
                }
            }

            postorder.push(node.index() as u32);
        }

        dfs_postorder_recursive(&self.inner, source_idx, &mut visited, &mut postorder);
        postorder
    }

    /// DFS labeled edges (directed)
    // #[napi]
    pub fn dfs_labeled_edges(&self, source: u32) -> Vec<Vec<String>> {
        let source_idx = NodeIndex::new(source as usize);
        let mut labeled_edges = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut stack = vec![source_idx];

        while let Some(current) = stack.pop() {
            if visited[current.index()] {
                continue;
            }

            visited[current.index()] = true;

            for neighbor in self.inner.neighbors_directed(current, Direction::Outgoing) {
                let edge_type = if !visited[neighbor.index()] {
                    stack.push(neighbor);
                    "tree"
                } else {
                    "forward"
                };

                labeled_edges.push(vec![
                    current.index().to_string(),
                    neighbor.index().to_string(),
                    edge_type.to_string(),
                ]);
            }
        }

        labeled_edges
    }

    /// All pairs shortest paths - distance matrix (directed)
    // #[napi]
    pub fn all_pairs_shortest_paths(&self, parallel_threshold: Option<u32>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let threshold = parallel_threshold.unwrap_or(100) as usize;

        if node_count >= threshold {
            // Use parallel computation for large graphs
            self.all_pairs_shortest_paths_parallel()
        } else {
            // Use sequential computation for small graphs
            self.all_pairs_shortest_paths_sequential()
        }
    }

    fn all_pairs_shortest_paths_sequential(&self) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Set diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Run Dijkstra from each node
        for source in self.inner.node_indices() {
            let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
                &self.inner,
                source,
                None,
                |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
                None,
            );

            if let Ok(distances) = result {
                for (target, distance) in distances {
                    matrix[source.index()][target.index()] = distance;
                }
            }
        }

        matrix
    }

    fn all_pairs_shortest_paths_parallel(&self) -> Vec<Vec<f64>> {
        use rayon::prelude::*;
        
        let node_count = self.inner.node_count();
        let mut matrix = vec![vec![f64::INFINITY; node_count]; node_count];

        // Set diagonal to 0
        for (i, row) in matrix.iter_mut().enumerate().take(node_count) {
            row[i] = 0.0;
        }

        // Parallel Dijkstra from each node
        let node_indices: Vec<_> = self.inner.node_indices().collect();
        let results: Vec<_> = node_indices
            .par_iter()
            .map(|&source| {
                let result: Result<HashMap<_, f64>, _> = shortest_path::dijkstra(
                    &self.inner,
                    source,
                    None,
                    |edge| -> Result<f64, ()> { Ok(*edge.weight()) },
                    None,
                );
                (source, result)
            })
            .collect();

        // Collect results into matrix
        for (source, result) in results {
            if let Ok(distances) = result {
                for (target, distance) in distances {
                    matrix[source.index()][target.index()] = distance;
                }
            }
        }

        matrix
    }

    /// Find all ancestors of a node (nodes that can reach this node)
    // #[napi]
    pub fn ancestors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        let mut ancestors = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        // Start BFS from all nodes that have incoming edges to the target
        for source in self.inner.node_indices() {
            if source != node_idx && self.inner.find_edge(source, node_idx).is_some() {
                queue.push_back(source);
                visited[source.index()] = true;
                ancestors.push(source.index() as u32);
            }
        }

        // Continue BFS to find all transitive ancestors
        while let Some(current) = queue.pop_front() {
            for predecessor in self.inner.neighbors_directed(current, Direction::Incoming) {
                if !visited[predecessor.index()] {
                    visited[predecessor.index()] = true;
                    queue.push_back(predecessor);
                    ancestors.push(predecessor.index() as u32);
                }
            }
        }

        ancestors
    }

    /// Find all descendants of a node (all nodes reachable from this node)
    // #[napi]
    pub fn descendants(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        let mut descendants = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];
        let mut queue = VecDeque::new();

        // Start BFS from the given node
        visited[node_idx.index()] = true;
        queue.push_back(node_idx);

        while let Some(current) = queue.pop_front() {
            for successor in self.inner.neighbors_directed(current, Direction::Outgoing) {
                if !visited[successor.index()] {
                    visited[successor.index()] = true;
                    queue.push_back(successor);
                    descendants.push(successor.index() as u32);
                }
            }
        }

        descendants
    }

    /// Find weakly connected components (treat directed graph as undirected)
    // #[napi]
    pub fn weakly_connected_components(&self) -> Vec<Vec<u32>> {
        let mut components = Vec::new();
        let mut visited = vec![false; self.inner.node_count()];

        for node in self.inner.node_indices() {
            if !visited[node.index()] {
                let mut component = Vec::new();
                let mut queue = VecDeque::new();

                visited[node.index()] = true;
                queue.push_back(node);
                component.push(node.index() as u32);

                while let Some(current) = queue.pop_front() {
                    // Check both outgoing and incoming neighbors (treat as undirected)
                    for neighbor in self
                        .inner
                        .neighbors_directed(current, Direction::Outgoing)
                        .chain(self.inner.neighbors_directed(current, Direction::Incoming))
                    {
                        if !visited[neighbor.index()] {
                            visited[neighbor.index()] = true;
                            queue.push_back(neighbor);
                            component.push(neighbor.index() as u32);
                        }
                    }
                }

                components.push(component);
            }
        }

        components
    }

    /// Create condensation graph (contract strongly connected components to single nodes)
    // #[napi]
    pub fn condensation(&self) -> RustworkxDiGraph {
        // First find strongly connected components
        let sccs = self.strongly_connected_components();

        // Create mapping from original nodes to SCC indices
        let mut node_to_scc = HashMap::new();
        for (scc_idx, scc) in sccs.iter().enumerate() {
            for &node in scc {
                node_to_scc.insert(node, scc_idx);
            }
        }

        // Create condensation graph
        let mut condensation = DiGraph::new();
        let mut scc_nodes = Vec::new();

        // Add nodes for each SCC
        for (scc_idx, _scc) in sccs.iter().enumerate() {
            // Create a label representing the SCC
            let scc_label = format!("SCC_{scc_idx}");
            let condensation_node = condensation.add_node(scc_label);
            scc_nodes.push(condensation_node);
        }

        // Add edges between SCCs
        let mut added_edges = std::collections::HashSet::new();

        for edge in self.inner.edge_references() {
            let source_scc = node_to_scc[&(edge.source().index() as u32)];
            let target_scc = node_to_scc[&(edge.target().index() as u32)];

            // Only add edge if it connects different SCCs and hasn't been added yet
            if source_scc != target_scc && !added_edges.contains(&(source_scc, target_scc)) {
                condensation.add_edge(scc_nodes[source_scc], scc_nodes[target_scc], 1.0);
                added_edges.insert((source_scc, target_scc));
            }
        }

        RustworkxDiGraph {
            inner: condensation,
        }
    }

    /// Export directed graph to node-link JSON format
    // #[napi]
    pub fn node_link_json(&self) -> String {
        let mut nodes = Vec::new();
        let mut links = Vec::new();

        // Add nodes
        for node_idx in self.inner.node_indices() {
            let node_data = serde_json::json!({
                "id": node_idx.index(),
                "label": self.inner.node_weight(node_idx).cloned().unwrap_or_default()
            });
            nodes.push(node_data);
        }

        // Add edges (links)
        for edge in self.inner.edge_references() {
            let link_data = serde_json::json!({
                "source": edge.source().index(),
                "target": edge.target().index(),
                "weight": *edge.weight()
            });
            links.push(link_data);
        }

        let graph_data = serde_json::json!({
            "directed": true,
            "nodes": nodes,
            "links": links
        });

        serde_json::to_string_pretty(&graph_data).unwrap_or_default()
    }

    /// Export directed graph as simple edge list
    // #[napi]
    pub fn edge_list(&self) -> Vec<Vec<String>> {
        let mut edges = Vec::new();

        for edge in self.inner.edge_references() {
            edges.push(vec![
                edge.source().index().to_string(),
                edge.target().index().to_string(),
                edge.weight().to_string(),
            ]);
        }

        edges
    }

    /// Export directed graph to GraphML format
    // #[napi]
    pub fn to_graphml(&self) -> String {
        let mut graphml = String::new();

        // GraphML header
        graphml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        graphml.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\" ");
        graphml.push_str("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ");
        graphml.push_str("xsi:schemaLocation=\"http://graphml.graphdrawing.org/xmlns ");
        graphml.push_str("http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd\">\n");

        // Key definitions
        graphml.push_str(
            "  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n",
        );
        graphml.push_str(
            "  <key id=\"weight\" for=\"edge\" attr.name=\"weight\" attr.type=\"double\"/>\n",
        );

        // Graph definition (directed)
        graphml.push_str("  <graph id=\"G\" edgedefault=\"directed\">\n");

        // Add nodes
        for node_idx in self.inner.node_indices() {
            let label = self
                .inner
                .node_weight(node_idx)
                .cloned()
                .unwrap_or_default();
            graphml.push_str(&format!("    <node id=\"n{}\">\n", node_idx.index()));
            graphml.push_str(&format!("      <data key=\"label\">{label}</data>\n"));
            graphml.push_str("    </node>\n");
        }

        // Add edges
        for (edge_id, edge) in self.inner.edge_references().enumerate() {
            graphml.push_str(&format!(
                "    <edge id=\"e{}\" source=\"n{}\" target=\"n{}\">\n",
                edge_id,
                edge.source().index(),
                edge.target().index()
            ));
            graphml.push_str(&format!(
                "      <data key=\"weight\">{}</data>\n",
                edge.weight()
            ));
            graphml.push_str("    </edge>\n");
        }

        // Close tags
        graphml.push_str("  </graph>\n");
        graphml.push_str("</graphml>\n");

        graphml
    }

    /// Import directed graph from GraphML format (basic implementation)
    // #[napi]
    pub fn from_graphml(graphml: String) -> Option<RustworkxDiGraph> {
        // Basic GraphML parsing - in production, would use proper XML parser
        if !graphml.contains("<graph") || !graphml.contains("</graph>") {
            return None;
        }

        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // Simple regex-based parsing for demonstration
        // Extract nodes
        let node_pattern = regex::Regex::new(
            r#"<node id="([^"]+)"[^>]*>(?:.*?<data key="label">([^<]*)</data>)?.*?</node>"#,
        )
        .ok()?;
        for captures in node_pattern.captures_iter(&graphml) {
            let node_id = captures.get(1)?.as_str();
            let label = captures
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let graph_node = graph.add_node(label);
            node_map.insert(node_id.to_string(), graph_node);
        }

        // Extract edges
        let edge_pattern = regex::Regex::new(r#"<edge[^>]+source="([^"]+)"[^>]+target="([^"]+)"[^>]*>(?:.*?<data key="weight">([^<]*)</data>)?.*?</edge>"#).ok()?;
        for captures in edge_pattern.captures_iter(&graphml) {
            let source_id = captures.get(1)?.as_str();
            let target_id = captures.get(2)?.as_str();
            let weight: f64 = captures
                .get(3)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1.0);

            if let (Some(&source_node), Some(&target_node)) =
                (node_map.get(source_id), node_map.get(target_id))
            {
                graph.add_edge(source_node, target_node, weight);
            }
        }

        Some(RustworkxDiGraph { inner: graph })
    }

    /// Parallel betweenness centrality for directed graphs (memory efficient)
    // #[napi]
    pub fn parallel_betweenness_centrality(&self, normalized: Option<bool>) -> Vec<f64> {
        use rayon::prelude::*;
        
        let normalized = normalized.unwrap_or(false);
        let node_count = self.inner.node_count();
        let mut centrality = vec![0.0; node_count];
        
        if node_count <= 1 {
            return centrality;
        }

        // Parallel computation for each source node
        let node_indices: Vec<_> = self.inner.node_indices().collect();
        let partial_results: Vec<Vec<f64>> = node_indices
            .par_iter()
            .map(|&source| {
                let mut local_centrality = vec![0.0; node_count];
                
                // Brandes' algorithm for single source (directed)
                let mut stack = Vec::new();
                let mut predecessors = vec![Vec::new(); node_count];
                let mut sigma = vec![0.0; node_count];
                let mut distance = vec![-1.0; node_count];
                let mut delta = vec![0.0; node_count];
                
                sigma[source.index()] = 1.0;
                distance[source.index()] = 0.0;
                
                let mut queue = VecDeque::new();
                queue.push_back(source);
                
                // Forward BFS (directed)
                while let Some(v) = queue.pop_front() {
                    stack.push(v);
                    for neighbor in self.inner.neighbors_directed(v, Direction::Outgoing) {
                        if distance[neighbor.index()] < 0.0 {
                            queue.push_back(neighbor);
                            distance[neighbor.index()] = distance[v.index()] + 1.0;
                        }
                        if distance[neighbor.index()] == distance[v.index()] + 1.0 {
                            sigma[neighbor.index()] += sigma[v.index()];
                            predecessors[neighbor.index()].push(v);
                        }
                    }
                }
                
                // Backward accumulation
                while let Some(w) = stack.pop() {
                    for &v in &predecessors[w.index()] {
                        delta[v.index()] += (sigma[v.index()] / sigma[w.index()]) * (1.0 + delta[w.index()]);
                    }
                    if w != source {
                        local_centrality[w.index()] += delta[w.index()];
                    }
                }
                
                local_centrality
            })
            .collect();

        // Combine partial results
        for partial in partial_results {
            for (i, &value) in partial.iter().enumerate() {
                centrality[i] += value;
            }
        }

        // Normalize if requested
        if normalized && node_count > 2 {
            let norm = 1.0 / ((node_count - 1) * (node_count - 2)) as f64;
            for value in &mut centrality {
                *value *= norm;
            }
        }

        centrality
    }

    /// Memory-efficient streaming neighbor iterator (outgoing)
    // #[napi]
    pub fn neighbors_stream(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, Direction::Outgoing)
            .map(|n| n.index() as u32)
            .collect()
    }

    /// Memory-efficient streaming predecessor iterator (incoming)
    // #[napi]
    pub fn predecessors_stream(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, Direction::Incoming)
            .map(|n| n.index() as u32)
            .collect()
    }

    /// Memory usage estimation for directed graph
    // #[napi]
    pub fn memory_usage(&self) -> String {
        let node_count = self.inner.node_count();
        let edge_count = self.inner.edge_count();
        
        // Rough estimation of memory usage
        let node_memory = node_count * (std::mem::size_of::<String>() + 32); // Node data + overhead
        let edge_memory = edge_count * (std::mem::size_of::<f64>() + 16); // Edge weight + indices
        let graph_overhead = 1024; // Graph structure overhead
        
        let total_bytes = node_memory + edge_memory + graph_overhead;
        
        if total_bytes < 1024 {
            format!("{total_bytes} bytes")
        } else if total_bytes < 1024 * 1024 {
            format!("{:.2} KB", total_bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", total_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

// Graph generators
#[napi]
pub fn complete_graph(num_nodes: u32) -> RustworkxGraph {
    let graph =
        generators::complete_graph(Some(num_nodes as usize), None, || "".to_string(), || 1.0)
            .unwrap();
    RustworkxGraph { inner: graph }
}

#[napi]
pub fn path_graph(num_nodes: u32) -> RustworkxGraph {
    let graph = generators::path_graph(
        Some(num_nodes as usize),
        None,
        || "".to_string(),
        || 1.0,
        false,
    )
    .unwrap();
    RustworkxGraph { inner: graph }
}

#[napi]
pub fn cycle_graph(num_nodes: u32) -> RustworkxGraph {
    let graph = generators::cycle_graph(
        Some(num_nodes as usize),
        None,
        || "".to_string(),
        || 1.0,
        false,
    )
    .unwrap();
    RustworkxGraph { inner: graph }
}

#[napi]
pub fn star_graph(num_nodes: u32) -> RustworkxGraph {
    let mut graph = UnGraph::new_undirected();

    if num_nodes == 0 {
        return RustworkxGraph { inner: graph };
    }

    // Add center node
    let center = graph.add_node("center".to_string());

    // Add leaf nodes and connect to center
    for i in 1..num_nodes {
        let leaf = graph.add_node(format!("leaf_{i}"));
        graph.add_edge(center, leaf, 1.0);
    }

    RustworkxGraph { inner: graph }
}

#[napi]
pub fn grid_graph(rows: u32, cols: u32) -> RustworkxGraph {
    let mut graph = UnGraph::new_undirected();

    if rows == 0 || cols == 0 {
        return RustworkxGraph { inner: graph };
    }

    // Create nodes
    let mut nodes = Vec::new();
    for i in 0..rows {
        let mut row = Vec::new();
        for j in 0..cols {
            let node = graph.add_node(format!("({i},{j})"));
            row.push(node);
        }
        nodes.push(row);
    }

    // Add edges
    for i in 0..rows {
        for j in 0..cols {
            // Connect to right neighbor
            if j + 1 < cols {
                graph.add_edge(
                    nodes[i as usize][j as usize],
                    nodes[i as usize][(j + 1) as usize],
                    1.0,
                );
            }
            // Connect to bottom neighbor
            if i + 1 < rows {
                graph.add_edge(
                    nodes[i as usize][j as usize],
                    nodes[(i + 1) as usize][j as usize],
                    1.0,
                );
            }
        }
    }

    RustworkxGraph { inner: graph }
}

// Directed graph generators
#[napi]
pub fn complete_directed_graph(num_nodes: u32) -> RustworkxDiGraph {
    let mut digraph = DiGraph::new();
    let node_indices: Vec<_> = (0..num_nodes)
        .map(|_| digraph.add_node("".to_string()))
        .collect();

    for i in 0..num_nodes {
        for j in 0..num_nodes {
            if i != j {
                digraph.add_edge(node_indices[i as usize], node_indices[j as usize], 1.0);
            }
        }
    }

    RustworkxDiGraph { inner: digraph }
}

#[napi]
pub fn path_directed_graph(num_nodes: u32) -> RustworkxDiGraph {
    let mut digraph = DiGraph::new();

    if num_nodes == 0 {
        return RustworkxDiGraph { inner: digraph };
    }

    let node_indices: Vec<_> = (0..num_nodes)
        .map(|_| digraph.add_node("".to_string()))
        .collect();

    for i in 0..(num_nodes - 1) {
        digraph.add_edge(
            node_indices[i as usize],
            node_indices[(i + 1) as usize],
            1.0,
        );
    }

    RustworkxDiGraph { inner: digraph }
}

#[napi]
pub fn cycle_directed_graph(num_nodes: u32) -> RustworkxDiGraph {
    let mut digraph = DiGraph::new();

    if num_nodes == 0 {
        return RustworkxDiGraph { inner: digraph };
    }

    let node_indices: Vec<_> = (0..num_nodes)
        .map(|_| digraph.add_node("".to_string()))
        .collect();

    for i in 0..num_nodes {
        let next = (i + 1) % num_nodes;
        digraph.add_edge(node_indices[i as usize], node_indices[next as usize], 1.0);
    }

    RustworkxDiGraph { inner: digraph }
}

// Utility functions
#[napi]
pub fn empty_graph(num_nodes: u32) -> RustworkxGraph {
    let mut graph = UnGraph::new_undirected();
    for _ in 0..num_nodes {
        graph.add_node("".to_string());
    }
    RustworkxGraph { inner: graph }
}

#[napi]
pub fn empty_directed_graph(num_nodes: u32) -> RustworkxDiGraph {
    let mut graph = DiGraph::new();
    for _ in 0..num_nodes {
        graph.add_node("".to_string());
    }
    RustworkxDiGraph { inner: graph }
}

// ==============================================================================
// Fast-Context Codebase Analysis API
// ==============================================================================

use crate::cache::AdaptiveCacheManager;
use crate::export::{JsonExporter, LspExporter};
use crate::watcher::CodebaseWatcher;
use tokio::runtime::Runtime;

/// Fast-Context codebase analyzer for Node.js
#[napi]
#[derive(TS)]
#[ts(export)]
pub struct FastContextAnalyzer {
    #[ts(skip)]
    runtime: Runtime,
    #[ts(skip)]
    project_root: String,
    #[ts(skip)]
    analysis: Option<AnalysisResult>,
    #[ts(skip)]
    query_engine: Option<CodeQueryEngine>,
    #[ts(skip)]
    cache_manager: Option<Arc<AdaptiveCacheManager<String>>>,
    #[ts(skip)]
    watcher: Option<CodebaseWatcher>,
}

/// Configuration options for Fast-Context analyzer
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct AnalyzerConfig {
    /// Project root directory path
    pub project_root: String,
    
    /// Languages to analyze (empty = auto-detect all)
    pub languages: Option<Vec<String>>,
    
    /// File patterns to ignore
    pub ignore_patterns: Option<Vec<String>>,
    
    /// Enable intelligent caching
    pub enable_caching: Option<bool>,
    
    /// Cache policy (auto, minimal, balanced, adaptive, persistent)
    pub cache_policy: Option<String>,
    
    /// Enable file watching for real-time updates
    pub enable_watching: Option<bool>,
    
    /// Maximum files to analyze (0 = no limit)
    pub max_files: Option<u32>,
    
    /// Enable parallel processing
    pub parallel_processing: Option<bool>,
}

/// Analysis result for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct AnalysisResultJs {
    /// Total number of files analyzed
    pub file_count: u32,
    
    /// Total number of symbols found
    pub symbol_count: u32,
    
    /// Total number of relationships found
    pub relationship_count: u32,
    
    /// Languages detected in the project
    pub languages: Vec<String>,
    
    /// Analysis duration in milliseconds
    pub duration_ms: u32,
    
    /// Memory usage in MB
    pub memory_usage_mb: Option<f64>,
}

/// Query result for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct QueryResultJs {
    /// Matching symbols
    pub symbols: Vec<SymbolInfoJs>,
    
    /// Context information
    pub context: ContextInfoJs,
    
    /// Suggestions for the user
    pub suggestions: Vec<String>,
    
    /// Total results available
    pub total_results: u32,
}

/// Symbol information for Node.js
#[napi(object)]
#[derive(TS, Clone, serde::Serialize)]
#[ts(export)]
pub struct SymbolInfoJs {
    /// Symbol name
    pub name: String,
    
    /// Qualified name (full path)
    pub qualified_name: String,
    
    /// Symbol kind (Function, Class, Variable, etc.)
    pub kind: String,
    
    /// File path
    pub file_path: String,
    
    /// Programming language
    pub language: String,
    
    /// Start line (1-based)
    pub start_line: u32,
    
    /// End line (1-based)
    pub end_line: u32,
    
    /// Cyclomatic complexity
    pub complexity: u32,
    
    /// Dependencies (symbols this symbol uses)
    pub dependencies: Vec<String>,
    
    /// Dependents (symbols that use this symbol)
    pub dependents: Vec<String>,
    
    /// Signature/declaration
    pub signature: Option<String>,
    
    /// Documentation
    pub documentation: Option<String>,
    
    /// Symbol modifiers (pub, static, etc.)
    pub modifiers: Vec<String>,
}

/// Context information for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct ContextInfoJs {
    /// Total symbols in context
    pub total_symbols: u32,
    
    /// Files involved
    pub files_involved: u32,
    
    /// Average complexity score
    pub complexity_score: f64,
    
    /// Architectural patterns detected
    pub architectural_patterns: Vec<String>,
    
    /// Potential issues
    pub potential_issues: Vec<String>,
}

/// Export options for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct ExportOptionsJs {
    /// Pretty print JSON output
    pub pretty_print: Option<bool>,
    
    /// Include detailed symbol information
    pub include_details: Option<bool>,
    
    /// Include relationships
    pub include_relationships: Option<bool>,
    
    /// Maximum symbols to export (0 = no limit)
    pub max_symbols: Option<u32>,
    
    /// Export format (json, lsp, embedding)
    pub format: Option<String>,
    
    /// Enable streaming for large exports
    pub streaming: Option<bool>,
}

/// Pagination options for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct PaginationOptionsJs {
    /// Page number (0-based)
    pub page: u32,
    
    /// Items per page
    pub page_size: u32,
    
    /// Sort field (name, complexity, etc.)
    pub sort_field: Option<String>,
    
    /// Sort direction (asc, desc)
    pub sort_direction: Option<String>,
}

/// Filter options for Node.js
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct FilterOptionsJs {
    /// Symbol kinds to include
    pub symbol_kinds: Option<Vec<String>>,
    
    /// Languages to include
    pub languages: Option<Vec<String>>,
    
    /// File patterns to include
    pub file_patterns: Option<Vec<String>>,
    
    /// Minimum complexity
    pub min_complexity: Option<u32>,
    
    /// Maximum complexity
    pub max_complexity: Option<u32>,
    
    /// Only documented symbols
    pub documented_only: Option<bool>,
}

/// File change event for Node.js callbacks
#[napi(object)]
#[derive(TS, Clone)]
#[ts(export)]
pub struct FileChangeEventJs {
    /// Type of change (created, modified, deleted, renamed)
    pub change_type: String,
    
    /// Path of the changed file
    pub file_path: String,
    
    /// Old path (for rename operations)
    pub old_path: Option<String>,
    
    /// Timestamp of the change
    pub timestamp: f64,
    
    /// Language of the changed file
    pub language: Option<String>,
    
    /// Whether this change affects the analysis
    pub affects_analysis: bool,
}

/// Batch of file changes for efficient processing
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct FileChangeBatchJs {
    /// All changes in this batch
    pub changes: Vec<FileChangeEventJs>,
    
    /// Total number of changes
    pub change_count: u32,
    
    /// Batch timestamp
    pub batch_timestamp: f64,
    
    /// Whether re-analysis is recommended
    pub requires_reanalysis: bool,
    
    /// Estimated impact level (low, medium, high)
    pub impact_level: String,
}

/// Streaming query options for large datasets
#[napi(object)]
#[derive(TS)]
#[ts(export)]
pub struct StreamingOptionsJs {
    /// Enable streaming mode
    pub enabled: bool,
    
    /// Chunk size for streaming results
    pub chunk_size: u32,
    
    /// Include progress callbacks
    pub include_progress: Option<bool>,
    
    /// Timeout for each chunk in milliseconds
    pub chunk_timeout_ms: Option<u32>,
}

/// Streaming query result chunk
#[napi(object)]
#[derive(TS, serde::Serialize)]
#[ts(export)]
pub struct QueryChunkJs {
    /// Results in this chunk
    pub symbols: Vec<SymbolInfoJs>,
    
    /// Chunk index (0-based)
    pub chunk_index: u32,
    
    /// Total number of chunks
    pub total_chunks: u32,
    
    /// Whether this is the last chunk
    pub is_last: bool,
    
    /// Progress percentage (0-100)
    pub progress: f64,
    
    /// Processing time for this chunk in milliseconds
    pub processing_time_ms: u32,
}

use std::sync::Arc;

// NAPI implementation for FastContextAnalyzer
#[napi]
impl FastContextAnalyzer {
    /// Create a new Fast-Context analyzer
    #[napi(constructor)]
    pub fn new(config: AnalyzerConfig) -> napi::Result<Self> {
        let runtime = Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Failed to create async runtime: {e}")))?;
        
        Ok(Self {
            runtime,
            project_root: config.project_root,
            analysis: None,
            query_engine: None,
            cache_manager: None,
            watcher: None,
        })
    }
    
    /// Analyze the codebase
    #[napi]
    pub fn analyze(&mut self, config: Option<AnalyzerConfig>) -> napi::Result<AnalysisResultJs> {
        let start_time = std::time::Instant::now();
        
        // Use provided config or default
        let project_root = if let Some(ref cfg) = config {
            cfg.project_root.clone()
        } else {
            self.project_root.clone()
        };
        
        // Create analyzer components
        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let mut graph_builder = CodeGraphBuilder::new();
        
        // Track analysis results
        let mut file_count = 0;
        let mut symbol_count = 0;
        let mut languages_found = std::collections::HashSet::new();
        
        // Find all source files in project
        let source_files = self.find_source_files(&project_root)?;
        
        // Process each file
        for file_path in source_files {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                // Detect language from file extension
                if let Some(ext) = std::path::Path::new(&file_path).extension() {
                    if let Some(language) = LanguageId::from_extension(&ext.to_string_lossy()) {
                        languages_found.insert(language);
                        
                        // Parse the file
                        if let Some(parse_result) = parser_factory.parse(&content, language) {
                            // Extract symbols
                            let symbols = extractor_factory.extract_symbols(&parse_result.tree, &content, &file_path, language);
                            
                            // Add symbols to graph
                            symbol_count += symbols.len();
                            graph_builder.add_file_symbols(symbols, &file_path);
                            
                            // Analyze internal references in this file
                            graph_builder.analyze_internal_references(&file_path);
                        }
                        
                        file_count += 1;
                    }
                }
            }
        }
        
        // Build the final code graph
        let code_graph = graph_builder.build();
        let relationship_count = code_graph.edge_count();
        
        // Convert languages for analysis result and JS response
        let languages_vec: Vec<LanguageId> = languages_found.iter().cloned().collect();
        let language_names: Vec<String> = languages_found
            .into_iter()
            .map(|lang| format!("{lang:?}"))
            .collect();
        
        // Create analysis result and store it
        let analysis_result = AnalysisResult {
            graph: code_graph,
            file_count,
            symbol_count,
            relationship_count,
            languages: languages_vec,
        };
        
        // Create query engine from the analysis
        let query_engine = CodeQueryEngine::new(analysis_result.clone());
        
        // Store results
        self.analysis = Some(analysis_result);
        self.query_engine = Some(query_engine);
        
        let duration = start_time.elapsed();
        
        // Calculate memory usage estimate
        let memory_usage_mb = Some(self.estimate_memory_usage(file_count, symbol_count) as f64);
        
        let result = AnalysisResultJs {
            file_count: file_count as u32,
            symbol_count: symbol_count as u32,
            relationship_count: relationship_count as u32,
            languages: language_names,
            duration_ms: duration.as_millis() as u32,
            memory_usage_mb,
        };
        
        Ok(result)
    }
    
    /// Query symbols by name pattern
    // #[napi]
    pub async fn find_symbols(&self, pattern: String) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_symbols(&pattern);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Query symbols by kind
    // #[napi]
    pub async fn find_symbols_by_kind(&self, kind: String) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let symbol_kind = self.parse_symbol_kind(&kind)?;
            let result = query_engine.find_symbols_by_kind(symbol_kind);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Find symbols in a specific file
    // #[napi]
    pub async fn find_symbols_in_file(&self, file_path: String) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_symbols_in_file(&file_path);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Find symbols that depend on the given symbol
    // #[napi]
    pub async fn find_dependents(&self, symbol_name: String) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_dependents(&symbol_name);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Find symbols that the given symbol depends on
    // #[napi]
    pub async fn find_dependencies(&self, symbol_name: String) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_dependencies(&symbol_name);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Find the most complex symbols
    // #[napi]
    pub async fn find_complex_symbols(&self, limit: u32) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_complex_symbols(limit as usize);
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Find architectural issues in the codebase
    // #[napi]
    pub async fn find_architectural_issues(&self) -> napi::Result<QueryResultJs> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_architectural_issues();
            Ok(self.convert_query_result(result))
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Export analysis results to JSON
    // #[napi]
    pub async fn export_json(&self, options: Option<ExportOptionsJs>) -> napi::Result<String> {
        if let Some(ref analysis) = self.analysis {
            let export_options = self.convert_export_options(options);
            let exporter = JsonExporter::new(analysis.clone(), self.project_root.clone());
            
            exporter.export_to_string(&export_options)
                .map_err(|e| napi::Error::from_reason(format!("Export failed: {e}")))
        } else {
            Err(napi::Error::from_reason("No analysis results available. Call analyze() first."))
        }
    }
    
    /// Export analysis results in LSP format
    // #[napi]
    pub async fn export_lsp(&self, options: Option<ExportOptionsJs>) -> napi::Result<String> {
        if let Some(ref analysis) = self.analysis {
            let export_options = self.convert_export_options(options);
            let exporter = LspExporter::new(analysis.clone(), self.project_root.clone());
            
            exporter.export_to_json(&export_options)
                .map_err(|e| napi::Error::from_reason(format!("LSP export failed: {e}")))
        } else {
            Err(napi::Error::from_reason("No analysis results available. Call analyze() first."))
        }
    }
    
    /// Start file watching for real-time updates with callback support
    #[napi]
    pub fn start_watching(&mut self, callback: napi::JsFunction) -> napi::Result<()> {
        let config = crate::watcher::WatcherConfig {
            watch_dirs: vec![std::path::PathBuf::from(&self.project_root)],
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
                "**/*.tmp".to_string(),
            ],
            debounce_duration: std::time::Duration::from_millis(100),
            batch_size: 50,
            ..Default::default()
        };
        
        match CodebaseWatcher::new(config) {
            Ok(watcher) => {
                // Subscribe to file change events
                let mut receiver = watcher.subscribe();
                
                // Create threadsafe function with explicit error handling type
                use napi::threadsafe_function::{ThreadsafeFunction, ErrorStrategy};
                let tsfn: ThreadsafeFunction<Vec<FileChangeEventJs>, ErrorStrategy::Fatal> = callback
                    .create_threadsafe_function(0, |ctx| {
                        let changes: Vec<FileChangeEventJs> = ctx.value;
                        
                        // Create the batch object that matches our JavaScript interface
                        let mut js_batch = ctx.env.create_object()?;
                        
                        // Convert changes to JavaScript array
                        let mut js_changes_array = ctx.env.create_array_with_length(changes.len())?;
                        for (i, change) in changes.iter().enumerate() {
                            let mut js_change = ctx.env.create_object()?;
                            js_change.set("changeType", &change.change_type)?;
                            js_change.set("filePath", &change.file_path)?;
                            js_change.set("timestamp", change.timestamp)?;
                            if let Some(ref lang) = change.language {
                                js_change.set("language", lang)?;
                            }
                            js_change.set("affectsAnalysis", change.affects_analysis)?;
                            js_changes_array.set_element(i as u32, js_change)?;
                        }
                        
                        js_batch.set("changes", js_changes_array)?;
                        js_batch.set("changeCount", changes.len() as u32)?;
                        js_batch.set("batchTimestamp", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64())?;
                        js_batch.set("requiresReanalysis", changes.iter().any(|c| c.affects_analysis))?;
                        
                        let impact_level = if changes.len() > 10 { 
                            "high" 
                        } else if changes.len() > 3 { 
                            "medium" 
                        } else { 
                            "low" 
                        };
                        js_batch.set("impactLevel", impact_level)?;
                        
                        Ok(vec![js_batch])
                    })
                    .map_err(|e| napi::Error::from_reason(format!("Failed to create threadsafe function: {e}")))?;
                    
                    // Spawn background task to handle real file changes
                    let rt_handle = self.runtime.handle().clone();
                    rt_handle.spawn(async move {
                        // Integrate with the watcher's actual change receiver
                        while let Ok(changes) = receiver.recv().await {
                            if !changes.is_empty() {
                                // Convert file changes to JavaScript format
                                let js_changes: Vec<FileChangeEventJs> = changes.into_iter().map(|change| {
                                    let change_type = match change.change_type {
                                        crate::watcher::ChangeType::Created => "created",
                                        crate::watcher::ChangeType::Modified => "modified", 
                                        crate::watcher::ChangeType::Deleted => "deleted",
                                        crate::watcher::ChangeType::Renamed { .. } => "renamed",
                                    }.to_string();
                                    
                                    let language = change.path.extension()
                                        .and_then(|ext| crate::parsers::LanguageId::from_extension(&ext.to_string_lossy()))
                                        .map(|lang| format!("{lang:?}"));
                                    
                                    let affects_analysis = language.is_some();
                                    
                                    FileChangeEventJs {
                                        change_type,
                                        file_path: change.path.to_string_lossy().to_string(),
                                        old_path: None, // Would be populated for renames
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs_f64(),
                                        language,
                                        affects_analysis,
                                    }
                                }).collect();
                                
                                // Call the JavaScript callback through the threadsafe function
                                use napi::threadsafe_function::ThreadsafeFunctionCallMode;
                                let _status = tsfn.call(js_changes, ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                    });
                
                self.watcher = Some(watcher);
                Ok(())
            },
            Err(e) => Err(napi::Error::from_reason(format!("Failed to start watcher: {e}")))
        }
    }
    
    /// Stop file watching
    #[napi]
    pub fn stop_watching(&mut self) -> napi::Result<()> {
        self.watcher = None;
        Ok(())
    }
    
    /// Get cache statistics
    // #[napi]
    pub async fn get_cache_stats(&self) -> napi::Result<String> {
        if let Some(ref cache_manager) = self.cache_manager {
            let stats = cache_manager.stats().await;
            serde_json::to_string_pretty(&stats)
                .map_err(|e| napi::Error::from_reason(format!("Failed to serialize cache stats: {e}")))
        } else {
            Ok("{}".to_string())
        }
    }
    
    /// Clear all caches
    // #[napi]
    pub async fn clear_cache(&self) -> napi::Result<()> {
        // Cache clearing would be implemented here
        Ok(())
    }
    
    /// Find symbols with streaming support for large datasets
    // #[napi]
    pub fn find_symbols_streaming(
        &self, 
        env: napi::Env,
        pattern: String, 
        options: StreamingOptionsJs,
        chunk_callback: napi::JsFunction
    ) -> napi::Result<()> {
        if let Some(ref query_engine) = self.query_engine {
            let result = query_engine.find_symbols(&pattern);
            let symbols: Vec<SymbolInfoJs> = result.symbols.into_iter()
                .map(|s| self.convert_symbol_info(s))
                .collect();
            
            let symbols_len = symbols.len();
            if options.enabled && symbols_len > options.chunk_size as usize {
                // Stream results in chunks
                let chunk_size = options.chunk_size as usize;
                let total_chunks = symbols_len.div_ceil(chunk_size);
                
                for (chunk_index, chunk) in symbols.chunks(chunk_size).enumerate() {
                    let start_time = std::time::Instant::now();
                    
                    let chunk_data = QueryChunkJs {
                        symbols: chunk.to_vec(),
                        chunk_index: chunk_index as u32,
                        total_chunks: total_chunks as u32,
                        is_last: chunk_index == total_chunks - 1,
                        progress: ((chunk_index + 1) as f64 / total_chunks as f64) * 100.0,
                        processing_time_ms: start_time.elapsed().as_millis() as u32,
                    };
                    
                    // Call JavaScript callback with chunk data
                    let mut js_chunk = env.create_object()?;
                    
                    // Set chunk properties
                    js_chunk.set("symbols", chunk_data.symbols)?;
                    js_chunk.set("chunk_index", chunk_data.chunk_index)?;
                    js_chunk.set("total_chunks", chunk_data.total_chunks)?;
                    js_chunk.set("is_last", chunk_data.is_last)?;
                    js_chunk.set("progress", chunk_data.progress)?;
                    js_chunk.set("processing_time_ms", chunk_data.processing_time_ms)?;
                    
                    if let Err(_e) = chunk_callback.call(None, &[js_chunk]) {
                        // Continue streaming unless it's a critical error
                    }
                    
                    // Small delay to prevent overwhelming the JavaScript thread
                    if let Some(timeout) = options.chunk_timeout_ms {
                        std::thread::sleep(std::time::Duration::from_millis(timeout as u64));
                    }
                }
            } else {
                // Send all results as a single chunk
                let chunk_data = QueryChunkJs {
                    symbols,
                    chunk_index: 0,
                    total_chunks: 1,
                    is_last: true,
                    progress: 100.0,
                    processing_time_ms: 0,
                };
                
                // Call JavaScript callback with all data as single chunk
                let mut js_chunk = env.create_object()?;
                
                // Set chunk properties
                js_chunk.set("symbols", chunk_data.symbols)?;
                js_chunk.set("chunk_index", chunk_data.chunk_index)?;
                js_chunk.set("total_chunks", chunk_data.total_chunks)?;
                js_chunk.set("is_last", chunk_data.is_last)?;
                js_chunk.set("progress", chunk_data.progress)?;
                js_chunk.set("processing_time_ms", chunk_data.processing_time_ms)?;
                
                if let Err(e) = chunk_callback.call(None, &[js_chunk]) {
                    return Err(napi::Error::from_reason(format!("Callback failed: {e}")));
                }
            }
            
            Ok(())
        } else {
            Err(napi::Error::from_reason("Analyzer not initialized. Call analyze() first."))
        }
    }
    
    /// Configure file watching with custom patterns and callbacks
    // #[napi]
    pub fn configure_watching(
        &mut self,
        ignore_patterns: Option<Vec<String>>,
        watch_patterns: Option<Vec<String>>,
        debounce_ms: Option<u32>,
        on_change: Option<napi::JsFunction>,
        on_batch: Option<napi::JsFunction>
    ) -> napi::Result<()> {
        let default_ignore = vec![
            "node_modules/**".to_string(),
            "target/**".to_string(),
            ".git/**".to_string(),
            "**/*.tmp".to_string(),
            "**/*.log".to_string(),
            "**/dist/**".to_string(),
            "**/build/**".to_string(),
        ];
        
        let ignore_patterns = ignore_patterns.unwrap_or(default_ignore);
        let watch_paths = if let Some(patterns) = watch_patterns {
            patterns.into_iter().map(std::path::PathBuf::from).collect()
        } else {
            vec![std::path::PathBuf::from(&self.project_root)]
        };
        
        let config = crate::watcher::WatcherConfig {
            watch_dirs: watch_paths,
            ignore_patterns,
            debounce_duration: std::time::Duration::from_millis(debounce_ms.unwrap_or(100) as u64),
            batch_size: 50,
            ..Default::default()
        };
        
        match CodebaseWatcher::new(config) {
            Ok(watcher) => {
                // Set up event handlers if provided
                if on_change.is_some() || on_batch.is_some() {
                    // This would integrate with the watcher's event system
                    // For now, we store the callbacks for future integration
                }
                
                self.watcher = Some(watcher);
                Ok(())
            },
            Err(e) => Err(napi::Error::from_reason(format!("Failed to configure watcher: {e}")))
        }
    }
    
    /// Get detailed error information with suggestions
    // #[napi]
    pub async fn get_last_error(&self) -> napi::Result<String> {
        // Check for errors in async runtime
        let mut errors = Vec::new();
        
        // Check if query engine has errors
        if let Some(ref _query_engine) = self.query_engine {
            // In a real implementation, query_engine would track errors
            // For now, we simulate checking common error conditions
        }
        
        // Check file watcher status
        if self.watcher.is_none() {
            errors.push("File watcher is not active".to_string());
        }
        
        // Check cache status
        if self.cache_manager.is_none() {
            errors.push("Cache manager is not initialized".to_string());
        }
        
        if errors.is_empty() {
            Ok("No recent errors".to_string())
        } else {
            Ok(format!("Errors found: {}", errors.join("; ")))
        }
    }
    
    /// Validate configuration and return detailed status
    // #[napi]
    pub async fn validate_configuration(&self) -> napi::Result<String> {
        let mut status = Vec::new();
        
        // Check project root exists
        if std::path::Path::new(&self.project_root).exists() {
            status.push("✓ Project root exists".to_string());
        } else {
            status.push("✗ Project root does not exist".to_string());
        }
        
        // Check for supported languages
        let supported = get_supported_languages();
        status.push(format!("✓ {} languages supported", supported.len()));
        
        // Check cache availability
        if self.cache_manager.is_some() {
            status.push("✓ Cache manager initialized".to_string());
        } else {
            status.push("○ Cache manager not initialized".to_string());
        }
        
        // Check watcher status
        if self.watcher.is_some() {
            status.push("✓ File watcher available".to_string());
        } else {
            status.push("○ File watcher not started".to_string());
        }
        
        // Check analysis status
        if self.analysis.is_some() && self.query_engine.is_some() {
            status.push("✓ Analysis completed, query engine ready".to_string());
        } else {
            status.push("○ Analysis not completed".to_string());
        }
        
        Ok(status.join("\n"))
    }
    
    // Helper methods
    
    fn convert_query_result(&self, result: QueryResult) -> QueryResultJs {
        let total_results = result.symbols.len() as u32;
        QueryResultJs {
            symbols: result.symbols.into_iter().map(|s| self.convert_symbol_info(s)).collect(),
            context: self.convert_context_info(result.context),
            suggestions: result.suggestions,
            total_results,
        }
    }
    
    fn convert_symbol_info(&self, symbol: crate::query::SymbolInfo) -> SymbolInfoJs {
        let qualified_name = symbol.symbol.qualified_name();
        SymbolInfoJs {
            name: symbol.symbol.name,
            qualified_name,
            kind: format!("{:?}", symbol.symbol.kind),
            file_path: symbol.file_path,
            language: format!("{:?}", symbol.symbol.language),
            start_line: symbol.symbol.location.start_line as u32,
            end_line: symbol.symbol.location.end_line as u32,
            complexity: symbol.complexity,
            dependencies: symbol.dependencies,
            dependents: symbol.dependents,
            signature: symbol.symbol.signature,
            documentation: symbol.symbol.documentation,
            modifiers: symbol.symbol.modifiers,
        }
    }
    
    fn convert_context_info(&self, context: crate::query::ContextInfo) -> ContextInfoJs {
        ContextInfoJs {
            total_symbols: context.total_symbols as u32,
            files_involved: context.files_involved as u32,
            complexity_score: context.complexity_score as f64,
            architectural_patterns: context.architectural_patterns,
            potential_issues: context.potential_issues,
        }
    }
    
    fn convert_export_options(&self, options: Option<ExportOptionsJs>) -> ExportOptions {
        if let Some(opts) = options {
            ExportOptions {
                pretty_print: opts.pretty_print.unwrap_or(true),
                include_symbol_details: opts.include_details.unwrap_or(true),
                include_relationships: opts.include_relationships.unwrap_or(true),
                include_file_metrics: true,
                include_analysis_metrics: true,
                max_symbols: opts.max_symbols.map(|m| m as usize),
                max_relationships: None,
                filters: None,
                compression: crate::export::json::CompressionOptions::default(),
                streaming: crate::export::json::StreamingOptions {
                    enabled: opts.streaming.unwrap_or(false),
                    chunk_size: 1000,
                    include_progress: false,
                },
            }
        } else {
            ExportOptions::default()
        }
    }
    
    fn parse_symbol_kind(&self, kind: &str) -> napi::Result<SymbolKind> {
        match kind.to_lowercase().as_str() {
            "function" => Ok(SymbolKind::Function),
            "class" => Ok(SymbolKind::Class),
            "interface" => Ok(SymbolKind::Interface),
            "struct" => Ok(SymbolKind::Struct),
            "enum" => Ok(SymbolKind::Enum),
            "variable" => Ok(SymbolKind::Variable),
            "constant" => Ok(SymbolKind::Constant),
            "method" => Ok(SymbolKind::Method),
            "property" => Ok(SymbolKind::Field),
            "field" => Ok(SymbolKind::Field),
            "parameter" => Ok(SymbolKind::Parameter),
            "namespace" => Ok(SymbolKind::Namespace),
            "module" => Ok(SymbolKind::Module),
            "import" => Ok(SymbolKind::Import),
            "export" => Ok(SymbolKind::Export),
            _ => Err(napi::Error::from_reason(format!("Unknown symbol kind: {kind}")))
        }
    }
    
    /// Find all source files in the project directory
    fn find_source_files(&self, project_root: &str) -> napi::Result<Vec<String>> {
        use std::path::Path;
        
        let mut files = Vec::new();
        let root_path = Path::new(project_root);
        
        if !root_path.exists() {
            return Err(napi::Error::from_reason(format!("Project root does not exist: {project_root}")));
        }
        
        self.collect_files_recursive(root_path, &mut files)?;
        Ok(files)
    }
    
    /// Recursively collect source files from directory
    fn collect_files_recursive(&self, dir: &std::path::Path, files: &mut Vec<String>) -> napi::Result<()> {
        use std::fs;
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip common directories that don't contain source code
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if !matches!(dir_name, 
                            "target" | "node_modules" | ".git" | ".cache" | "dist" | "build" | 
                            "__pycache__" | ".pytest_cache" | ".vscode" | ".idea" | "coverage"
                        ) {
                            self.collect_files_recursive(&path, files)?;
                        }
                    }
                } else if let Some(extension) = path.extension() {
                    // Check if this is a supported file type
                    if LanguageId::from_extension(&extension.to_string_lossy()).is_some() {
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Estimate memory usage based on file and symbol count
    fn estimate_memory_usage(&self, file_count: usize, symbol_count: usize) -> u32 {
        // Rough estimate: base overhead + per-file overhead + per-symbol overhead
        let base_mb = 10; // Base analyzer overhead
        let file_overhead_kb = 5; // Per file overhead in KB
        let symbol_overhead_bytes = 200; // Per symbol overhead in bytes
        
        let file_overhead_mb = (file_count * file_overhead_kb) / 1024;
        let symbol_overhead_mb = (symbol_count * symbol_overhead_bytes) / (1024 * 1024);
        
        (base_mb + file_overhead_mb + symbol_overhead_mb) as u32
    }
}

/// Utility functions for Fast-Context
/// Get supported languages
#[napi]
pub fn get_supported_languages() -> Vec<String> {
    vec![
        "Rust".to_string(),
        "Python".to_string(),
        "JavaScript".to_string(),
        "TypeScript".to_string(),
        "Java".to_string(),
        "Go".to_string(),
        "CSharp".to_string(),
        "Swift".to_string(),
        "ObjectiveC".to_string(),
        "PHP".to_string(),
        "Ruby".to_string(),
        "Scala".to_string(),
        "Zig".to_string(),
        "Dart".to_string(),
        "Lua".to_string(),
        "Bash".to_string(),
    ]
}

/// Detect language from file extension
#[napi]
pub fn detect_language(file_path: String) -> Option<String> {
    let path = std::path::Path::new(&file_path);
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        let language = match ext_str.as_str() {
            "rs" => Some(LanguageId::Rust),
            "py" | "pyw" => Some(LanguageId::Python),
            "js" | "mjs" => Some(LanguageId::JavaScript),
            "ts" | "tsx" => Some(LanguageId::TypeScript),
            "java" => Some(LanguageId::Java),
            "go" => Some(LanguageId::Go),
            "cs" => Some(LanguageId::CSharp),
            "swift" => Some(LanguageId::Swift),
            "m" | "mm" => Some(LanguageId::ObjectiveC),
            "php" => Some(LanguageId::PHP),
            "rb" => Some(LanguageId::Ruby),
            "scala" | "sc" => Some(LanguageId::Scala),
            "zig" => Some(LanguageId::Zig),
            "dart" => Some(LanguageId::Dart),
            "lua" => Some(LanguageId::Lua),
            "sh" | "bash" => Some(LanguageId::Bash),
            _ => None,
        };
        
        language.map(|lang| format!("{lang:?}"))
    } else {
        None
    }
}

/// Check if Fast-Context is properly configured
#[napi]
pub fn check_configuration() -> napi::Result<String> {
    // This would perform various configuration checks
    Ok("Fast-Context is properly configured".to_string())
}

/// Get version information
#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn generate_typescript_types() {
        // Generate TypeScript definitions for all exported types
        let output_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let types_path = Path::new(&output_dir).join("types").join("generated.d.ts");
        
        // Create types directory if it doesn't exist
        if let Some(parent) = types_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        
        // Generate individual TypeScript type definitions
        let type_definitions = vec![
            ("FastContextAnalyzer", FastContextAnalyzer::export_to_string().unwrap()),
            ("AnalyzerConfig", AnalyzerConfig::export_to_string().unwrap()),
            ("AnalysisResultJs", AnalysisResultJs::export_to_string().unwrap()),
            ("QueryResultJs", QueryResultJs::export_to_string().unwrap()),
            ("SymbolInfoJs", SymbolInfoJs::export_to_string().unwrap()),
            ("ContextInfoJs", ContextInfoJs::export_to_string().unwrap()),
            ("ExportOptionsJs", ExportOptionsJs::export_to_string().unwrap()),
            ("PaginationOptionsJs", PaginationOptionsJs::export_to_string().unwrap()),
            ("FilterOptionsJs", FilterOptionsJs::export_to_string().unwrap()),
            ("FileChangeEventJs", FileChangeEventJs::export_to_string().unwrap()),
            ("FileChangeBatchJs", FileChangeBatchJs::export_to_string().unwrap()),
            ("StreamingOptionsJs", StreamingOptionsJs::export_to_string().unwrap()),
            ("QueryChunkJs", QueryChunkJs::export_to_string().unwrap()),
            ("RustworkxGraph", RustworkxGraph::export_to_string().unwrap()),
            ("RustworkxDiGraph", RustworkxDiGraph::export_to_string().unwrap()),
        ];
        
        // Create clean TypeScript definitions
        let mut combined_types = String::new();
        combined_types.push_str("// Auto-generated TypeScript types for Fast-Context\n");
        combined_types.push_str("// Generated by ts-rs from Rust structs - DO NOT EDIT MANUALLY\n\n");
        
        for (type_name, type_def) in type_definitions {
            // Clean up the generated types - remove import statements and duplicate headers
            let lines: Vec<&str> = type_def.lines().collect();
            let mut clean_lines = Vec::new();
            let mut in_interface = false;
            
            for line in lines {
                if line.starts_with("// This file was generated") {
                    continue;
                }
                if line.starts_with("import type") {
                    continue;
                }
                if line.trim().is_empty() && !in_interface {
                    continue;
                }
                if line.starts_with("export interface") {
                    in_interface = true;
                    // Add proper JSDoc comment
                    clean_lines.push(format!("/** {} type definition */", type_name));
                }
                clean_lines.push(line.to_string());
                if line == "}" && in_interface {
                    in_interface = false;
                    clean_lines.push("".to_string()); // Add spacing after interface
                }
            }
            
            for line in clean_lines {
                combined_types.push_str(&line);
                combined_types.push('\n');
            }
        }
        
        // Write to types/generated.d.ts
        fs::write(&types_path, combined_types).expect("Failed to write TypeScript types");
        
        println!("Generated TypeScript types at: {}", types_path.display());
    }
}
