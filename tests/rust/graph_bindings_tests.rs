//! Integration tests for Python graph bindings.

#![cfg(feature = "python")]

use fast_context::python_bindings_graph::{PyRustworkxDiGraph, PyRustworkxGraph};

#[test]
fn test_undirected_graph_basics() {
    let mut graph = PyRustworkxGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    let a = graph.add_node(Some("a".to_string()));
    let b = graph.add_node(Some("b".to_string()));
    assert_eq!(graph.add_edge(a, b, Some(1.5)), Some(0));

    assert!(!graph.is_empty());
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.get_node_weight(a), Some("a".to_string()));
    assert_eq!(graph.get_edge_weight(a, b), Some(1.5));

    let components = graph.connected_components();
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].size, 2);

    let cloned = graph.clone_graph();
    assert_eq!(cloned.node_count(), 2);
    assert_eq!(cloned.edge_count(), 1);
}

#[test]
fn test_undirected_shortest_paths() {
    let mut graph = PyRustworkxGraph::new();
    let a = graph.add_node(Some("a".to_string()));
    let b = graph.add_node(Some("b".to_string()));
    let c = graph.add_node(Some("c".to_string()));

    graph.add_edge(a, b, Some(1.0));
    graph.add_edge(b, c, Some(2.0));

    let result = graph.dijkstra_shortest_path(a, c).unwrap();
    assert_eq!(result.path, vec![a, c]);
    assert_eq!(result.distance, 3.0);

    let distances = graph.floyd_warshall_all_pairs();
    assert_eq!(distances[a][a], Some(0.0));
    assert_eq!(distances[a][c], Some(3.0));
}

#[test]
fn test_directed_graph_algorithms() {
    let mut graph = PyRustworkxDiGraph::new();
    let a = graph.add_node(Some("a".to_string()));
    let b = graph.add_node(Some("b".to_string()));
    let c = graph.add_node(Some("c".to_string()));

    graph.add_edge(a, b, Some(1.0));
    graph.add_edge(b, c, Some(1.0));

    assert!(graph.is_directed_acyclic_graph());

    let topo = graph.topological_sort();
    assert_eq!(topo.len(), 3);
    assert_eq!(topo.first(), Some(&a));

    let sccs = graph.strongly_connected_components();
    assert_eq!(sccs.len(), 3);

    let clone = graph.clone_graph();
    assert_eq!(clone.node_count(), 3);
    assert_eq!(clone.edge_count(), 2);
}
