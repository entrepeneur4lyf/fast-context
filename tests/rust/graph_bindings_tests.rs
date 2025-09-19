//! Comprehensive test suite for Python graph bindings
//! 
//! Tests all graph algorithms, data structures, and edge cases
//! Only compiled when the "python" feature is enabled

#[cfg(test)]
#[cfg(feature = "python")]
mod tests {
    use crate::python_bindings_graph::*;
    use pyo3::prelude::*;

    /// Test basic graph creation and manipulation
    #[test]
    fn test_graph_creation() {
        Python::with_gil(|py| {
            // Test undirected graph creation
            let graph = PyRustworkxGraph::new();
            assert_eq!(graph.node_count(py).unwrap(), 0);
            assert_eq!(graph.edge_count(py).unwrap(), 0);
            assert!(graph.is_empty(py).unwrap());

            // Test directed graph creation
            let digraph = PyRustworkxDiGraph::new();
            assert_eq!(digraph.node_count(py).unwrap(), 0);
            assert_eq!(digraph.edge_count(py).unwrap(), 0);
            assert!(digraph.is_empty(py).unwrap());
        });
    }

    /// Test node addition and removal
    #[test]
    fn test_node_operations() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Add nodes
            let node1 = graph.add_node(py, "node1".to_string()).unwrap();
            let node2 = graph.add_node(py, "node2".to_string()).unwrap();
            let node3 = graph.add_node(py, "node3".to_string()).unwrap();
            
            assert_eq!(graph.node_count(py).unwrap(), 3);
            assert!(!graph.is_empty(py).unwrap());
            
            // Remove node
            let removed = graph.remove_node(py, node1).unwrap();
            assert!(removed);
            assert_eq!(graph.node_count(py).unwrap(), 2);
            
            // Remove non-existent node
            let removed = graph.remove_node(py, 999).unwrap();
            assert!(!removed);
        });
    }

    /// Test edge addition and removal
    #[test]
    fn test_edge_operations() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Add nodes
            let node1 = graph.add_node(py, "node1".to_string()).unwrap();
            let node2 = graph.add_node(py, "node2".to_string()).unwrap();
            
            // Add edge
            let edge_added = graph.add_edge(py, node1, node2, 1.0).unwrap();
            assert!(edge_added);
            assert_eq!(graph.edge_count(py).unwrap(), 1);
            
            // Add duplicate edge (should fail)
            let edge_added = graph.add_edge(py, node1, node2, 1.0).unwrap();
            assert!(!edge_added);
            
            // Remove edge
            let edge_removed = graph.remove_edge(py, node1, node2).unwrap();
            assert!(edge_removed);
            assert_eq!(graph.edge_count(py).unwrap(), 0);
            
            // Remove non-existent edge
            let edge_removed = graph.remove_edge(py, node1, node2).unwrap();
            assert!(!edge_removed);
        });
    }

    /// Test Dijkstra's shortest path algorithm
    #[test]
    fn test_dijkstra_shortest_paths() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Create a simple graph: 0--1--2--3
            let node0 = graph.add_node(py, "0".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            let node3 = graph.add_node(py, "3".to_string()).unwrap();
            
            graph.add_edge(py, node0, node1, 1.0).unwrap();
            graph.add_edge(py, node1, node2, 1.0).unwrap();
            graph.add_edge(py, node2, node3, 1.0).unwrap();
            
            // Test shortest path from node0
            let distances = graph.dijkstra_shortest_paths(py, node0).unwrap();
            let dist_dict: &PyDict = distances.extract(py).unwrap();
            
            assert_eq!(dist_dict.len(), 4);
            assert_eq!(dist_dict.get_item("0").unwrap().extract::<f64>().unwrap(), 0.0);
            assert_eq!(dist_dict.get_item("1").unwrap().extract::<f64>().unwrap(), 1.0);
            assert_eq!(dist_dict.get_item("2").unwrap().extract::<f64>().unwrap(), 2.0);
            assert_eq!(dist_dict.get_item("3").unwrap().extract::<f64>().unwrap(), 3.0);
        });
    }

    /// Test graph with disconnected components
    #[test]
    fn test_disconnected_graph() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Create two disconnected components
            let node0 = graph.add_node(py, "0".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            let node3 = graph.add_node(py, "3".to_string()).unwrap();
            
            // Connect only first component
            graph.add_edge(py, node0, node1, 1.0).unwrap();
            
            // Second component remains disconnected
            let components = graph.connected_components(py).unwrap();
            let comp_list: Vec<Vec<usize>> = components.extract(py).unwrap();
            
            assert_eq!(comp_list.len(), 2); // Two connected components
        });
    }

    /// Test directed graph strongly connected components
    #[test]
    fn test_strongly_connected_components() {
        Python::with_gil(|py| {
            let graph = PyRustworkxDiGraph::new();
            
            // Create a directed cycle: 0->1->2->0
            let node0 = graph.add_node(py, "0".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            
            graph.add_edge(py, node0, node1, 1.0).unwrap();
            graph.add_edge(py, node1, node2, 1.0).unwrap();
            graph.add_edge(py, node2, node0, 1.0).unwrap();
            
            let components = graph.strongly_connected_components(py).unwrap();
            let comp_list: Vec<Vec<usize>> = components.extract(py).unwrap();
            
            assert_eq!(comp_list.len(), 1); // One strongly connected component
        });
    }

    /// Test betweenness centrality calculation
    #[test]
    fn test_betweenness_centrality() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Create a star graph: center connected to all others
            let center = graph.add_node(py, "center".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            let node3 = graph.add_node(py, "3".to_string()).unwrap();
            
            graph.add_edge(py, center, node1, 1.0).unwrap();
            graph.add_edge(py, center, node2, 1.0).unwrap();
            graph.add_edge(py, center, node3, 1.0).unwrap();
            
            let centrality = graph.betweenness_centrality(py, true).unwrap();
            let cent_dict: &PyDict = centrality.extract(py).unwrap();
            
            // Center node should have highest betweenness centrality
            let center_cent = cent_dict.get_item("center").unwrap().extract::<f64>().unwrap();
            let node1_cent = cent_dict.get_item("1").unwrap().extract::<f64>().unwrap();
            
            assert!(center_cent > node1_cent);
        });
    }

    /// Test topological sort on DAG
    #[test]
    fn test_topological_sort() {
        Python::with_gil(|py| {
            let graph = PyRustworkxDiGraph::new();
            
            // Create a DAG: 0->1->2, 0->3->2
            let node0 = graph.add_node(py, "0".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            let node3 = graph.add_node(py, "3".to_string()).unwrap();
            
            graph.add_edge(py, node0, node1, 1.0).unwrap();
            graph.add_edge(py, node1, node2, 1.0).unwrap();
            graph.add_edge(py, node0, node3, 1.0).unwrap();
            graph.add_edge(py, node3, node2, 1.0).unwrap();
            
            // Check if it's a DAG
            let is_dag = graph.is_directed_acyclic_graph(py).unwrap();
            assert!(is_dag);
            
            // Get topological order
            let topo_order = graph.topological_sort(py).unwrap();
            let order_list: Vec<usize> = topo_order.extract(py).unwrap();
            
            assert_eq!(order_list.len(), 4);
            
            // Verify ordering constraints (node0 should come before node2)
            let pos0 = order_list.iter().position(|&x| x == node0).unwrap();
            let pos2 = order_list.iter().position(|&x| x == node2).unwrap();
            assert!(pos0 < pos2);
        });
    }

    /// Test cycle detection
    #[test]
    fn test_cycle_detection() {
        Python::with_gil(|py| {
            let graph = PyRustworkxDiGraph::new();
            
            // Create a graph without cycle
            let node0 = graph.add_node(py, "0".to_string()).unwrap();
            let node1 = graph.add_node(py, "1".to_string()).unwrap();
            let node2 = graph.add_node(py, "2".to_string()).unwrap();
            
            graph.add_edge(py, node0, node1, 1.0).unwrap();
            graph.add_edge(py, node1, node2, 1.0).unwrap();
            
            // Should be acyclic
            assert!(graph.is_directed_acyclic_graph(py).unwrap());
            
            // Add cycle
            graph.add_edge(py, node2, node0, 1.0).unwrap();
            
            // Should now have cycle
            assert!(!graph.is_directed_acyclic_graph(py).unwrap());
        });
    }

    /// Test graph capacity management
    #[test]
    fn test_graph_capacity() {
        Python::with_gil(|py| {
            // Test graph with pre-allocated capacity
            let graph = PyRustworkxGraph::with_capacity(10, 15);
            assert_eq!(graph.node_count(py).unwrap(), 0);
            assert_eq!(graph.edge_count(py).unwrap(), 0);
            
            // Should be able to add nodes up to capacity without reallocation
            for i in 0..10 {
                let node = graph.add_node(py, i.to_string()).unwrap();
                assert_eq!(node, i);
            }
            
            // Clear graph
            graph.clear(py).unwrap();
            assert_eq!(graph.node_count(py).unwrap(), 0);
            assert_eq!(graph.edge_count(py).unwrap(), 0);
            assert!(graph.is_empty(py).unwrap());
        });
    }

    /// Test error handling for invalid operations
    #[test]
    fn test_error_handling() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Test operations on non-existent nodes
            let result = graph.add_edge(py, 999, 1000, 1.0);
            assert!(result.is_err());
            
            let result = graph.remove_node(py, 999);
            assert!(result.is_err());
            
            let result = graph.remove_edge(py, 999, 1000);
            assert!(result.is_err());
            
            // Test algorithms on empty graph
            let result = graph.dijkstra_shortest_paths(py, 0);
            assert!(result.is_err());
        });
    }

    /// Test graph properties and metadata
    #[test]
    fn test_graph_properties() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::new();
            
            // Test initial properties
            assert!(graph.is_empty(py).unwrap());
            assert_eq!(graph.node_count(py).unwrap(), 0);
            assert_eq!(graph.edge_count(py).unwrap(), 0);
            
            // Add nodes and edges
            let node1 = graph.add_node(py, "node1".to_string()).unwrap();
            let node2 = graph.add_node(py, "node2".to_string()).unwrap();
            graph.add_edge(py, node1, node2, 1.0).unwrap();
            
            // Check updated properties
            assert!(!graph.is_empty(py).unwrap());
            assert_eq!(graph.node_count(py).unwrap(), 2);
            assert_eq!(graph.edge_count(py).unwrap(), 1);
        });
    }

    /// Test large graph performance (basic smoke test)
    #[test]
    fn test_large_graph_basic() {
        Python::with_gil(|py| {
            let graph = PyRustworkxGraph::with_capacity(100, 150);
            
            // Create a simple linear graph
            let mut prev_node = None;
            for i in 0..100 {
                let node = graph.add_node(py, i.to_string()).unwrap();
                if let Some(prev) = prev_node {
                    graph.add_edge(py, prev, node, 1.0).unwrap();
                }
                prev_node = Some(node);
            }
            
            assert_eq!(graph.node_count(py).unwrap(), 100);
            assert_eq!(graph.edge_count(py).unwrap(), 99);
            
            // Test basic algorithm on large graph
            let start_node = 0;
            let distances = graph.dijkstra_shortest_paths(py, start_node).unwrap();
            let dist_dict: &PyDict = distances.extract(py).unwrap();
            
            assert_eq!(dist_dict.len(), 100);
        });
    }
}

/// Integration tests for graph operations
#[cfg(test)]
#[cfg(feature = "python")]
mod integration_tests {
    use super::*;
    use pyo3::prelude::*;

    /// Test complete workflow: create graph, add nodes/edges, run algorithms
    #[test]
    fn test_complete_workflow() {
        Python::with_gil(|py| {
            // Create graph representing a small network
            let graph = PyRustworkxGraph::new();
            
            // Add nodes (representing network devices)
            let router = graph.add_node(py, "router".to_string()).unwrap();
            let switch1 = graph.add_node(py, "switch1".to_string()).unwrap();
            let switch2 = graph.add_node(py, "switch2".to_string()).unwrap();
            let pc1 = graph.add_node(py, "pc1".to_string()).unwrap();
            let pc2 = graph.add_node(py, "pc2".to_string()).unwrap();
            let pc3 = graph.add_node(py, "pc3".to_string()).unwrap();
            
            // Add edges (network connections)
            graph.add_edge(py, router, switch1, 1.0).unwrap();
            graph.add_edge(py, router, switch2, 1.0).unwrap();
            graph.add_edge(py, switch1, pc1, 1.0).unwrap();
            graph.add_edge(py, switch1, pc2, 1.0).unwrap();
            graph.add_edge(py, switch2, pc3, 1.0).unwrap();
            
            // Run analysis algorithms
            let components = graph.connected_components(py).unwrap();
            let comp_list: Vec<Vec<usize>> = components.extract(py).unwrap();
            assert_eq!(comp_list.len(), 1); // All connected
            
            let centrality = graph.betweenness_centrality(py, true).unwrap();
            let cent_dict: &PyDict = centrality.extract(py).unwrap();
            
            // Router should have highest centrality
            let router_cent = cent_dict.get_item("router").unwrap().extract::<f64>().unwrap();
            assert!(router_cent > 0.0);
            
            // Test shortest path from router to all PCs
            let distances = graph.dijkstra_shortest_paths(py, router).unwrap();
            let dist_dict: &PyDict = distances.extract(py).unwrap();
            
            assert_eq!(dist_dict.len(), 6);
            assert_eq!(dist_dict.get_item("router").unwrap().extract::<f64>().unwrap(), 0.0);
            assert_eq!(dist_dict.get_item("pc1").unwrap().extract::<f64>().unwrap(), 2.0);
            assert_eq!(dist_dict.get_item("pc2").unwrap().extract::<f64>().unwrap(), 2.0);
            assert_eq!(dist_dict.get_item("pc3").unwrap().extract::<f64>().unwrap(), 2.0);
        });
    }

    /// Test graph algorithms robustness with various graph types
    #[test]
    fn test_algorithm_robustness() {
        Python::with_gil(|py| {
            // Test empty graph
            let empty_graph = PyRustworkxGraph::new();
            assert!(empty_graph.is_empty(py).unwrap());
            
            // Test single node graph
            let single_node = PyRustworkxGraph::new();
            single_node.add_node(py, "single".to_string()).unwrap();
            assert_eq!(single_node.node_count(py).unwrap(), 1);
            
            // Test complete graph (every node connected to every other)
            let complete = PyRustworkxGraph::new();
            let nodes: Vec<usize> = (0..5).map(|i| {
                complete.add_node(py, i.to_string()).unwrap()
            }).collect();
            
            for i in 0..nodes.len() {
                for j in i+1..nodes.len() {
                    complete.add_edge(py, nodes[i], nodes[j], 1.0).unwrap();
                }
            }
            
            assert_eq!(complete.node_count(py).unwrap(), 5);
            assert_eq!(complete.edge_count(py).unwrap(), 10); // C(5,2) = 10
            
            // Test tree structure
            let tree = PyRustworkxGraph::new();
            let root = tree.add_node(py, "root".to_string()).unwrap();
            let children: Vec<usize> = (0..3).map(|i| {
                tree.add_node(py, format!("child{}", i)).unwrap()
            }).collect();
            
            for &child in &children {
                tree.add_edge(py, root, child, 1.0).unwrap();
            }
            
            assert_eq!(tree.node_count(py).unwrap(), 4);
            assert_eq!(tree.edge_count(py).unwrap(), 3);
            
            // Tree should have no cycles
            let is_dag = tree.is_directed_acyclic_graph(py).unwrap();
            assert!(is_dag);
        });
    }
}