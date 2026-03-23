//! # Graph Module
//!
//! This module contains all graph-related functionality
//! extracted from the monolithic lib.rs for better organization.

pub mod operations;
pub mod petgraph_impl;

#[cfg(feature = "nodejs")]
use napi_derive::napi;
#[cfg(feature = "nodejs")]
use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
#[cfg(feature = "nodejs")]
use petgraph::visit::EdgeRef;
#[cfg(feature = "nodejs")]
use ts_rs::TS;

/// Undirected graph implementation
#[cfg(feature = "nodejs")]
#[napi]
#[derive(Clone, TS)]
#[ts(export)]
pub struct RustworkxGraph {
    #[ts(skip)]
    inner: UnGraph<String, f64>,
}

#[cfg(feature = "nodejs")]
impl Default for RustworkxGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "nodejs")]
#[napi]
impl RustworkxGraph {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: UnGraph::new_undirected(),
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if self.inner.node_weight(node_a_idx).is_some()
            && self.inner.node_weight(node_b_idx).is_some()
        {
            Some(self.inner.add_edge(node_a_idx, node_b_idx, weight).index() as u32)
        } else {
            None
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn remove_node(&mut self, node: u32) -> bool {
        let node_idx = NodeIndex::new(node as usize);
        if self.inner.node_weight(node_idx).is_some() {
            self.inner.remove_node(node_idx);
            true
        } else {
            false
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn remove_edge(&mut self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if let Some(edge) = self.inner.find_edge(node_a_idx, node_b_idx) {
            self.inner.remove_edge(edge);
            true
        } else {
            false
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);
        self.inner.find_edge(node_a_idx, node_b_idx).is_some()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner.node_weight(node_idx).cloned()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn get_edge_data(&self, node_a: u32, node_b: u32) -> Option<f64> {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if let Some(edge) = self.inner.find_edge(node_a_idx, node_b_idx) {
            self.inner.edge_weight(edge).copied()
        } else {
            None
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors(node_idx)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Dijkstra's shortest path algorithm
    /// Returns a list of [nodeId, distance] pairs for JavaScript compatibility
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dijkstra_shortest_paths(&self, source: u32, target: Option<u32>) -> Vec<Vec<f64>> {
        use petgraph::algo::dijkstra;

        let source_idx = NodeIndex::new(source as usize);

        // Validate source node exists
        if self.inner.node_weight(source_idx).is_none() {
            return vec![];
        }

        let target_idx = target.map(|t| NodeIndex::new(t as usize));

        // Run Dijkstra's algorithm
        let distances = dijkstra(&self.inner, source_idx, target_idx, |edge| *edge.weight());

        // Convert to Vec<Vec<f64>> format: [[nodeId, distance], ...]
        distances
            .into_iter()
            .map(|(node_idx, distance)| vec![node_idx.index() as f64, distance])
            .collect()
    }

    /// All-pairs shortest paths using a simple implementation
    /// Returns a 2D matrix of shortest distances between all pairs of nodes
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn all_pairs_shortest_paths(&self) -> Vec<Vec<Option<f64>>> {
        let node_count = self.inner.node_count();
        let mut result = vec![vec![None; node_count]; node_count];

        // Initialize with direct edges
        #[allow(clippy::needless_range_loop)]
        for i in 0..node_count {
            for j in 0..node_count {
                if i == j {
                    result[i][j] = Some(0.0);
                } else {
                    let node_i = NodeIndex::new(i);
                    let node_j = NodeIndex::new(j);
                    if let Some(edge) = self.inner.find_edge(node_i, node_j) {
                        if let Some(weight) = self.inner.edge_weight(edge) {
                            result[i][j] = Some(*weight);
                        }
                    }
                }
            }
        }

        // Floyd-Warshall algorithm
        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    if let (Some(ik), Some(kj)) = (result[i][k], result[k][j]) {
                        let new_dist = ik + kj;
                        match result[i][j] {
                            None => result[i][j] = Some(new_dist),
                            Some(current) => {
                                if new_dist < current {
                                    result[i][j] = Some(new_dist);
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Betweenness centrality for undirected graphs
    /// Returns a list of [nodeId, centrality] pairs
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn betweenness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        let normalize = normalized.unwrap_or(true);
        let mut centrality = vec![0.0; node_count];

        // Brandes' algorithm for betweenness centrality
        for s in 0..node_count {
            let source_idx = NodeIndex::new(s);
            if self.inner.node_weight(source_idx).is_none() {
                continue;
            }

            // BFS to find shortest paths
            let mut stack = Vec::new();
            let mut paths = vec![Vec::new(); node_count];
            let mut sigma = vec![0.0; node_count];
            let mut dist = vec![-1.0; node_count];
            let mut queue = std::collections::VecDeque::new();

            sigma[s] = 1.0;
            dist[s] = 0.0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                let v_idx = NodeIndex::new(v);

                for edge in self.inner.edges(v_idx) {
                    let w = edge.target().index();

                    // First time we find shortest path to w?
                    if dist[w] < 0.0 {
                        queue.push_back(w);
                        dist[w] = dist[v] + 1.0;
                    }

                    // Shortest path to w via v?
                    if dist[w] == dist[v] + 1.0 {
                        sigma[w] += sigma[v];
                        paths[w].push(v);
                    }
                }
            }

            // Accumulation
            let mut delta = vec![0.0; node_count];
            while let Some(w) = stack.pop() {
                for &v in &paths[w] {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
                if w != s {
                    centrality[w] += delta[w];
                }
            }
        }

        // Normalization
        if normalize && node_count > 2 {
            let norm_factor = 2.0 / ((node_count - 1) * (node_count - 2)) as f64;
            for c in &mut centrality {
                *c *= norm_factor;
            }
        }

        // Convert to JavaScript-compatible format
        centrality
            .into_iter()
            .enumerate()
            .map(|(i, c)| vec![i as f64, c])
            .collect()
    }

    /// Closeness centrality for undirected graphs
    /// Returns a list of [nodeId, centrality] pairs
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn closeness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        let normalize = normalized.unwrap_or(true);
        let mut centrality = Vec::new();

        for i in 0..node_count {
            let source_idx = NodeIndex::new(i);
            if self.inner.node_weight(source_idx).is_none() {
                centrality.push(vec![i as f64, 0.0]);
                continue;
            }

            // Use Dijkstra to find distances to all other nodes
            use petgraph::algo::dijkstra;
            let distances = dijkstra(&self.inner, source_idx, None, |edge| *edge.weight());

            let mut total_distance = 0.0;
            let mut reachable_count = 0;

            for (_, distance) in distances {
                if distance > 0.0 && distance != f64::INFINITY {
                    total_distance += distance;
                    reachable_count += 1;
                }
            }

            let closeness = if total_distance > 0.0 {
                if normalize && reachable_count > 0 {
                    (reachable_count as f64) / total_distance
                } else {
                    1.0 / total_distance
                }
            } else {
                0.0
            };

            centrality.push(vec![i as f64, closeness]);
        }

        centrality
    }

    /// Check if the undirected graph is bipartite
    /// Returns true if the graph can be colored with two colors
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn is_bipartite(&self) -> bool {
        use petgraph::algo::is_bipartite_undirected;
        is_bipartite_undirected(&self.inner, NodeIndex::new(0))
    }

    /// Get the number of connected components
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn number_connected_components(&self) -> u32 {
        use petgraph::algo::connected_components;
        connected_components(&self.inner) as u32
    }

    /// Get connected components using a simple DFS approach
    /// Returns a list of component IDs for each node
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn connected_components(&self) -> Vec<u32> {
        let node_count = self.inner.node_count();
        let mut visited = vec![false; node_count];
        let mut component_id = vec![0u32; node_count];
        let mut current_component = 0u32;

        for i in 0..node_count {
            let node_idx = NodeIndex::new(i);
            if !visited[i] && self.inner.node_weight(node_idx).is_some() {
                // DFS to mark all nodes in this component
                let mut stack = vec![i];
                while let Some(node) = stack.pop() {
                    if !visited[node] {
                        visited[node] = true;
                        component_id[node] = current_component;

                        let node_idx = NodeIndex::new(node);
                        for neighbor in self.inner.neighbors(node_idx) {
                            let neighbor_idx = neighbor.index();
                            if !visited[neighbor_idx] {
                                stack.push(neighbor_idx);
                            }
                        }
                    }
                }
                current_component += 1;
            }
        }

        component_id
    }

    /// Depth-first search edges from a starting node
    /// Returns a list of [source, target] edge pairs in DFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut edges = Vec::new();
        let mut stack = vec![start_idx];

        while let Some(node) = stack.pop() {
            if !visited[node.index()] {
                visited[node.index()] = true;

                for edge in self.inner.edges(node) {
                    let target = edge.target();
                    if !visited[target.index()] {
                        edges.push(vec![node.index() as u32, target.index() as u32]);
                        stack.push(target);
                    }
                }
            }
        }

        edges
    }

    /// Breadth-first search edges from a starting node
    /// Returns a list of [source, target] edge pairs in BFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn bfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut edges = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        visited[start_idx.index()] = true;
        queue.push_back(start_idx);

        while let Some(node) = queue.pop_front() {
            for edge in self.inner.edges(node) {
                let target = edge.target();
                if !visited[target.index()] {
                    visited[target.index()] = true;
                    edges.push(vec![node.index() as u32, target.index() as u32]);
                    queue.push_back(target);
                }
            }
        }

        edges
    }

    /// Depth-first search tree from a starting node
    /// Returns a list of nodes in DFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dfs_tree(&self, start: u32) -> Vec<u32> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut tree = Vec::new();
        let mut stack = vec![start_idx];

        while let Some(node) = stack.pop() {
            if !visited[node.index()] {
                visited[node.index()] = true;
                tree.push(node.index() as u32);

                // Add neighbors to stack in reverse order to maintain left-to-right traversal
                let mut neighbors: Vec<_> = self.inner.neighbors(node).collect();
                neighbors.reverse();
                for neighbor in neighbors {
                    if !visited[neighbor.index()] {
                        stack.push(neighbor);
                    }
                }
            }
        }

        tree
    }

    /// Breadth-first search tree from a starting node
    /// Returns a list of nodes in BFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn bfs_tree(&self, start: u32) -> Vec<u32> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut tree = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        visited[start_idx.index()] = true;
        tree.push(start_idx.index() as u32);
        queue.push_back(start_idx);

        while let Some(node) = queue.pop_front() {
            for neighbor in self.inner.neighbors(node) {
                if !visited[neighbor.index()] {
                    visited[neighbor.index()] = true;
                    tree.push(neighbor.index() as u32);
                    queue.push_back(neighbor);
                }
            }
        }

        tree
    }
}

/// Directed graph implementation
#[cfg(feature = "nodejs")]
#[napi]
#[derive(Clone, TS)]
#[ts(export)]
pub struct RustworkxDiGraph {
    #[ts(skip)]
    inner: DiGraph<String, f64>,
}

#[cfg(feature = "nodejs")]
impl Default for RustworkxDiGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "nodejs")]
#[napi]
impl RustworkxDiGraph {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DiGraph::new(),
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn add_edge(&mut self, node_a: u32, node_b: u32, weight: f64) -> Option<u32> {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if self.inner.node_weight(node_a_idx).is_some()
            && self.inner.node_weight(node_b_idx).is_some()
        {
            Some(self.inner.add_edge(node_a_idx, node_b_idx, weight).index() as u32)
        } else {
            None
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn remove_node(&mut self, node: u32) -> bool {
        let node_idx = NodeIndex::new(node as usize);
        if self.inner.node_weight(node_idx).is_some() {
            self.inner.remove_node(node_idx);
            true
        } else {
            false
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn remove_edge(&mut self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if let Some(edge) = self.inner.find_edge(node_a_idx, node_b_idx) {
            self.inner.remove_edge(edge);
            true
        } else {
            false
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);
        self.inner.find_edge(node_a_idx, node_b_idx).is_some()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner.node_weight(node_idx).cloned()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn get_edge_data(&self, node_a: u32, node_b: u32) -> Option<f64> {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);

        if let Some(edge) = self.inner.find_edge(node_a_idx, node_b_idx) {
            self.inner.edge_weight(edge).copied()
        } else {
            None
        }
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors(node_idx)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn predecessors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn successors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Dijkstra's shortest path algorithm for directed graphs
    /// Returns a list of [nodeId, distance] pairs for JavaScript compatibility
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dijkstra_shortest_paths(&self, source: u32, target: Option<u32>) -> Vec<Vec<f64>> {
        use petgraph::algo::dijkstra;

        let source_idx = NodeIndex::new(source as usize);

        // Validate source node exists
        if self.inner.node_weight(source_idx).is_none() {
            return vec![];
        }

        let target_idx = target.map(|t| NodeIndex::new(t as usize));

        // Run Dijkstra's algorithm
        let distances = dijkstra(&self.inner, source_idx, target_idx, |edge| *edge.weight());

        // Convert to Vec<Vec<f64>> format: [[nodeId, distance], ...]
        distances
            .into_iter()
            .map(|(node_idx, distance)| vec![node_idx.index() as f64, distance])
            .collect()
    }

    /// All-pairs shortest paths for directed graphs
    /// Returns a 2D matrix of shortest distances between all pairs of nodes
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn all_pairs_shortest_paths(&self) -> Vec<Vec<Option<f64>>> {
        let node_count = self.inner.node_count();
        let mut result = vec![vec![None; node_count]; node_count];

        // Initialize with direct edges
        #[allow(clippy::needless_range_loop)]
        for i in 0..node_count {
            for j in 0..node_count {
                if i == j {
                    result[i][j] = Some(0.0);
                } else {
                    let node_i = NodeIndex::new(i);
                    let node_j = NodeIndex::new(j);
                    if let Some(edge) = self.inner.find_edge(node_i, node_j) {
                        if let Some(weight) = self.inner.edge_weight(edge) {
                            result[i][j] = Some(*weight);
                        }
                    }
                }
            }
        }

        // Floyd-Warshall algorithm
        for k in 0..node_count {
            for i in 0..node_count {
                for j in 0..node_count {
                    if let (Some(ik), Some(kj)) = (result[i][k], result[k][j]) {
                        let new_dist = ik + kj;
                        match result[i][j] {
                            None => result[i][j] = Some(new_dist),
                            Some(current) => {
                                if new_dist < current {
                                    result[i][j] = Some(new_dist);
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Betweenness centrality for directed graphs
    /// Returns a list of [nodeId, centrality] pairs
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn betweenness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        let normalize = normalized.unwrap_or(true);
        let mut centrality = vec![0.0; node_count];

        // Brandes' algorithm for betweenness centrality
        for s in 0..node_count {
            let source_idx = NodeIndex::new(s);
            if self.inner.node_weight(source_idx).is_none() {
                continue;
            }

            // BFS to find shortest paths
            let mut stack = Vec::new();
            let mut paths = vec![Vec::new(); node_count];
            let mut sigma = vec![0.0; node_count];
            let mut dist = vec![-1.0; node_count];
            let mut queue = std::collections::VecDeque::new();

            sigma[s] = 1.0;
            dist[s] = 0.0;
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                let v_idx = NodeIndex::new(v);

                for edge in self.inner.edges(v_idx) {
                    let w = edge.target().index();

                    // First time we find shortest path to w?
                    if dist[w] < 0.0 {
                        queue.push_back(w);
                        dist[w] = dist[v] + 1.0;
                    }

                    // Shortest path to w via v?
                    if dist[w] == dist[v] + 1.0 {
                        sigma[w] += sigma[v];
                        paths[w].push(v);
                    }
                }
            }

            // Accumulation
            let mut delta = vec![0.0; node_count];
            while let Some(w) = stack.pop() {
                for &v in &paths[w] {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
                if w != s {
                    centrality[w] += delta[w];
                }
            }
        }

        // Normalization
        if normalize && node_count > 2 {
            let norm_factor = 2.0 / ((node_count - 1) * (node_count - 2)) as f64;
            for c in &mut centrality {
                *c *= norm_factor;
            }
        }

        // Convert to JavaScript-compatible format
        centrality
            .into_iter()
            .enumerate()
            .map(|(i, c)| vec![i as f64, c])
            .collect()
    }

    /// Closeness centrality for directed graphs
    /// Returns a list of [nodeId, centrality] pairs
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn closeness_centrality(&self, normalized: Option<bool>) -> Vec<Vec<f64>> {
        let node_count = self.inner.node_count();
        if node_count == 0 {
            return vec![];
        }

        let normalize = normalized.unwrap_or(true);
        let mut centrality = Vec::new();

        for i in 0..node_count {
            let source_idx = NodeIndex::new(i);
            if self.inner.node_weight(source_idx).is_none() {
                centrality.push(vec![i as f64, 0.0]);
                continue;
            }

            // Use Dijkstra to find distances to all other nodes
            use petgraph::algo::dijkstra;
            let distances = dijkstra(&self.inner, source_idx, None, |edge| *edge.weight());

            let mut total_distance = 0.0;
            let mut reachable_count = 0;

            for (_, distance) in distances {
                if distance > 0.0 && distance != f64::INFINITY {
                    total_distance += distance;
                    reachable_count += 1;
                }
            }

            let closeness = if total_distance > 0.0 {
                if normalize && reachable_count > 0 {
                    (reachable_count as f64) / total_distance
                } else {
                    1.0 / total_distance
                }
            } else {
                0.0
            };

            centrality.push(vec![i as f64, closeness]);
        }

        centrality
    }

    /// Check if the directed graph is acyclic (DAG)
    /// Returns true if the graph contains no cycles
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn is_directed_acyclic_graph(&self) -> bool {
        use petgraph::algo::is_cyclic_directed;
        !is_cyclic_directed(&self.inner)
    }

    /// Topological sort of the directed graph
    /// Returns nodes in topological order, or empty if graph has cycles
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn topological_sort(&self) -> Vec<u32> {
        use petgraph::algo::toposort;

        match toposort(&self.inner, None) {
            Ok(sorted) => sorted.into_iter().map(|idx| idx.index() as u32).collect(),
            Err(_) => vec![], // Graph has cycles
        }
    }

    /// Get strongly connected components using Tarjan's algorithm
    /// Returns a list of component IDs for each node
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn strongly_connected_components(&self) -> Vec<u32> {
        use petgraph::algo::tarjan_scc;

        let sccs = tarjan_scc(&self.inner);
        let mut result = vec![0u32; self.inner.node_count()];

        for (component_id, component) in sccs.iter().enumerate() {
            for &node_idx in component {
                result[node_idx.index()] = component_id as u32;
            }
        }

        result
    }

    /// Get the number of strongly connected components
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn number_strongly_connected_components(&self) -> u32 {
        use petgraph::algo::tarjan_scc;
        tarjan_scc(&self.inner).len() as u32
    }

    /// Get weakly connected components (treating edges as undirected)
    /// Returns a list of component IDs for each node
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn weakly_connected_components(&self) -> Vec<u32> {
        let node_count = self.inner.node_count();
        let mut visited = vec![false; node_count];
        let mut component_id = vec![0u32; node_count];
        let mut current_component = 0u32;

        for i in 0..node_count {
            let node_idx = NodeIndex::new(i);
            if !visited[i] && self.inner.node_weight(node_idx).is_some() {
                // DFS treating edges as undirected
                let mut stack = vec![i];
                while let Some(node) = stack.pop() {
                    if !visited[node] {
                        visited[node] = true;
                        component_id[node] = current_component;

                        let node_idx = NodeIndex::new(node);
                        // Check both outgoing and incoming edges
                        for neighbor in self.inner.neighbors(node_idx) {
                            let neighbor_idx = neighbor.index();
                            if !visited[neighbor_idx] {
                                stack.push(neighbor_idx);
                            }
                        }

                        // Also check incoming edges (predecessors)
                        for edge in self
                            .inner
                            .edges_directed(node_idx, petgraph::Direction::Incoming)
                        {
                            let neighbor_idx = edge.source().index();
                            if !visited[neighbor_idx] {
                                stack.push(neighbor_idx);
                            }
                        }
                    }
                }
                current_component += 1;
            }
        }

        component_id
    }

    /// Depth-first search edges from a starting node (directed)
    /// Returns a list of [source, target] edge pairs in DFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut edges = Vec::new();
        let mut stack = vec![start_idx];

        while let Some(node) = stack.pop() {
            if !visited[node.index()] {
                visited[node.index()] = true;

                for edge in self.inner.edges(node) {
                    let target = edge.target();
                    if !visited[target.index()] {
                        edges.push(vec![node.index() as u32, target.index() as u32]);
                        stack.push(target);
                    }
                }
            }
        }

        edges
    }

    /// Breadth-first search edges from a starting node (directed)
    /// Returns a list of [source, target] edge pairs in BFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn bfs_edges(&self, start: u32) -> Vec<Vec<u32>> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut edges = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        visited[start_idx.index()] = true;
        queue.push_back(start_idx);

        while let Some(node) = queue.pop_front() {
            for edge in self.inner.edges(node) {
                let target = edge.target();
                if !visited[target.index()] {
                    visited[target.index()] = true;
                    edges.push(vec![node.index() as u32, target.index() as u32]);
                    queue.push_back(target);
                }
            }
        }

        edges
    }

    /// Depth-first search tree from a starting node (directed)
    /// Returns a list of nodes in DFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn dfs_tree(&self, start: u32) -> Vec<u32> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut tree = Vec::new();
        let mut stack = vec![start_idx];

        while let Some(node) = stack.pop() {
            if !visited[node.index()] {
                visited[node.index()] = true;
                tree.push(node.index() as u32);

                // Add successors to stack in reverse order
                let mut successors: Vec<_> = self.inner.neighbors(node).collect();
                successors.reverse();
                for successor in successors {
                    if !visited[successor.index()] {
                        stack.push(successor);
                    }
                }
            }
        }

        tree
    }

    /// Breadth-first search tree from a starting node (directed)
    /// Returns a list of nodes in BFS order
    #[cfg(feature = "nodejs")]
    #[napi]
    pub fn bfs_tree(&self, start: u32) -> Vec<u32> {
        let start_idx = NodeIndex::new(start as usize);
        if self.inner.node_weight(start_idx).is_none() {
            return vec![];
        }

        let mut visited = vec![false; self.inner.node_count()];
        let mut tree = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        visited[start_idx.index()] = true;
        tree.push(start_idx.index() as u32);
        queue.push_back(start_idx);

        while let Some(node) = queue.pop_front() {
            for successor in self.inner.neighbors(node) {
                if !visited[successor.index()] {
                    visited[successor.index()] = true;
                    tree.push(successor.index() as u32);
                    queue.push_back(successor);
                }
            }
        }

        tree
    }
}
