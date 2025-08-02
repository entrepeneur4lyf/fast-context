//! # Graph Module
//!
//! This module contains all graph-related functionality
//! extracted from the monolithic lib.rs for better organization.

use petgraph::graph::{DiGraph, NodeIndex, UnGraph};
use petgraph::Graph;
use napi_derive::napi;
use ts_rs::TS;
use std::collections::HashMap;

/// Undirected graph implementation
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
impl RustworkxGraph {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: UnGraph::new_undirected(),
        }
    }

    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

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

    #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

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

    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);
        self.inner.find_edge(node_a_idx, node_b_idx).is_some()
    }

    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner.node_weight(node_idx).cloned()
    }

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

    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors(node_idx)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// Directed graph implementation
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
impl RustworkxDiGraph {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: DiGraph::new(),
        }
    }

    #[napi]
    pub fn add_node(&mut self, weight: String) -> u32 {
        self.inner.add_node(weight).index() as u32
    }

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

    #[napi]
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    #[napi]
    pub fn edge_count(&self) -> u32 {
        self.inner.edge_count() as u32
    }

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

    #[napi]
    pub fn has_edge(&self, node_a: u32, node_b: u32) -> bool {
        let node_a_idx = NodeIndex::new(node_a as usize);
        let node_b_idx = NodeIndex::new(node_b as usize);
        self.inner.find_edge(node_a_idx, node_b_idx).is_some()
    }

    #[napi]
    pub fn get_node_data(&self, node: u32) -> Option<String> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner.node_weight(node_idx).cloned()
    }

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

    #[napi]
    pub fn neighbors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors(node_idx)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[napi]
    pub fn predecessors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[napi]
    pub fn successors(&self, node: u32) -> Vec<u32> {
        let node_idx = NodeIndex::new(node as usize);
        self.inner
            .neighbors_directed(node_idx, petgraph::Direction::Outgoing)
            .map(|n| n.index() as u32)
            .collect()
    }

    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
