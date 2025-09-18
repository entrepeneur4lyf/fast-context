//! Performance benchmarks for graph algorithms and operations
//! 
//! Measures execution time and memory usage for various graph operations

#[cfg(test)]
#[cfg(feature = "python")]
mod benchmarks {
    use super::*;
    use pyo3::prelude::*;
    use std::time::Instant;

    /// Benchmark basic graph operations
    #[test]
    fn benchmark_basic_operations() {
        Python::with_gil(|py| {
            let mut results = HashMap::new();
            
            // Benchmark node addition
            let start = Instant::now();
            let graph = PyRustworkxGraph::with_capacity(1000, 0);
            for i in 0..1000 {
                graph.add_node(py, i.to_string()).unwrap();
            }
            let node_add_time = start.elapsed();
            results.insert("node_addition_1000".to_string(), node_add_time.as_millis());
            
            // Benchmark edge addition
            let start = Instant::now();
            for i in 0..999 {
                graph.add_edge(py, i, i+1, 1.0).unwrap();
            }
            let edge_add_time = start.elapsed();
            results.insert("edge_addition_999".to_string(), edge_add_time.as_millis());
            
            // Benchmark graph traversal (BFS)
            let start = Instant::now();
            let _bfs_tree = graph.bfs_tree(py, 0).unwrap();
            let bfs_time = start.elapsed();
            results.insert("bfs_traversal_1000_nodes".to_string(), bfs_time.as_millis());
            
            // Benchmark Dijkstra's algorithm
            let start = Instant::now();
            let _distances = graph.dijkstra_shortest_paths(py, 0).unwrap();
            let dijkstra_time = start.elapsed();
            results.insert("dijkstra_1000_nodes".to_string(), dijkstra_time.as_millis());
            
            // Print results
            println!("Basic Operations Benchmark Results:");
            for (operation, time) in results {
                println!("  {}: {}ms", operation, time);
            }
            
            // Assert reasonable performance (adjust thresholds as needed)
            assert!(node_add_time.as_millis() < 100, "Node addition too slow");
            assert!(edge_add_time.as_millis() < 100, "Edge addition too slow");
            assert!(bfs_time.as_millis() < 50, "BFS traversal too slow");
            assert!(dijkstra_time.as_millis() < 200, "Dijkstra's algorithm too slow");
        });
    }

    /// Benchmark centrality algorithms
    #[test]
    fn benchmark_centrality_algorithms() {
        Python::with_gil(|py| {
            // Create a test graph
            let graph = PyRustworkxGraph::with_capacity(100, 200);
            
            // Add nodes
            for i in 0..100 {
                graph.add_node(py, i.to_string()).unwrap();
            }
            
            // Create a semi-random graph
            for i in 0..100 {
                for j in (i+1)..100 {
                    if (i + j) % 7 == 0 { // Create some connections
                        graph.add_edge(py, i, j, 1.0).unwrap();
                    }
                }
            }
            
            let mut results = HashMap::new();
            
            // Benchmark betweenness centrality
            let start = Instant::now();
            let _betweenness = graph.betweenness_centrality(py, true).unwrap();
            let betweenness_time = start.elapsed();
            results.insert("betweenness_centrality_100_nodes".to_string(), betweenness_time.as_millis());
            
            // Benchmark closeness centrality
            let start = Instant::now();
            let _closeness = graph.closeness_centrality(py, true).unwrap();
            let closeness_time = start.elapsed();
            results.insert("closeness_centrality_100_nodes".to_string(), closeness_time.as_millis());
            
            // Benchmark eigenvector centrality
            let start = Instant::now();
            let _eigenvector = graph.eigenvector_centrality(py, 100, 1e-6).unwrap();
            let eigenvector_time = start.elapsed();
            results.insert("eigenvector_centrality_100_nodes".to_string(), eigenvector_time.as_millis());
            
            // Benchmark PageRank
            let start = Instant::now();
            let _pagerank = graph.pagerank(py, 0.85, 1e-6, 100).unwrap();
            let pagerank_time = start.elapsed();
            results.insert("pagerank_100_nodes".to_string(), pagerank_time.as_millis());
            
            println!("Centrality Algorithms Benchmark Results:");
            for (algorithm, time) in results {
                println!("  {}: {}ms", algorithm, time);
            }
            
            // Centrality algorithms should complete in reasonable time
            assert!(betweenness_time.as_millis() < 1000, "Betweenness centrality too slow");
            assert!(closeness_time.as_millis() < 500, "Closeness centrality too slow");
            assert!(eigenvector_time.as_millis() < 1000, "Eigenvector centrality too slow");
            assert!(pagerank_time.as_millis() < 1000, "PageRank too slow");
        });
    }

    /// Benchmark graph analysis algorithms
    #[test]
    fn benchmark_graph_analysis() {
        Python::with_gil(|py| {
            // Create test graphs of different sizes
            let sizes = vec![10, 50, 100, 200];
            let mut results = HashMap::new();
            
            for &size in &sizes {
                let graph = PyRustworkxGraph::with_capacity(size, size * 2);
                
                // Add nodes
                for i in 0..size {
                    graph.add_node(py, i.to_string()).unwrap();
                }
                
                // Create random connections
                for i in 0..size {
                    for j in (i+1)..size {
                        if (i * j) % 5 == 0 {
                            graph.add_edge(py, i, j, 1.0).unwrap();
                        }
                    }
                }
                
                // Benchmark connected components
                let start = Instant::now();
                let _components = graph.connected_components(py).unwrap();
                let components_time = start.elapsed();
                results.insert(format!("connected_components_{}", size), components_time.as_millis());
                
                // Benchmark cycle detection
                let start = Instant::now();
                let _is_dag = graph.is_directed_acyclic_graph(py).unwrap();
                let dag_time = start.elapsed();
                results.insert(format!("dag_detection_{}", size), dag_time.as_millis());
                
                // Benchmark topological sort (if DAG)
                if _is_dag {
                    let start = Instant::now();
                    let _topo = graph.topological_sort(py).unwrap();
                    let topo_time = start.elapsed();
                    results.insert(format!("topological_sort_{}", size), topo_time.as_millis());
                }
            }
            
            println!("Graph Analysis Benchmark Results:");
            for (operation, time) in results {
                println!("  {}: {}ms", operation, time);
            }
            
            // Performance should scale reasonably (not exponentially)
            let size_10_components = results.get("connected_components_10").unwrap();
            let size_200_components = results.get("connected_components_200").unwrap();
            
            // Should not be more than 100x slower for 20x size increase
            assert!(*size_200_components < *size_10_components * 100, 
                "Connected components scaling is poor");
        });
    }

    /// Benchmark memory usage with large graphs
    #[test]
    fn benchmark_memory_usage() {
        Python::with_gil(|py| {
            let sizes = vec![1000, 5000, 10000];
            let mut memory_results = HashMap::new();
            
            for &size in &sizes {
                let start = Instant::now();
                
                let graph = PyRustworkxGraph::with_capacity(size, size * 2);
                
                // Add nodes
                for i in 0..size {
                    graph.add_node(py, i.to_string()).unwrap();
                }
                
                // Add edges in a sparse pattern
                for i in 0..size {
                    for j in 1..=5.min(size - i) {
                        graph.add_edge(py, i, i + j, 1.0).unwrap();
                    }
                }
                
                let creation_time = start.elapsed();
                
                // Measure basic operation time as proxy for memory efficiency
                let op_start = Instant::now();
                let node_count = graph.node_count(py).unwrap();
                let edge_count = graph.edge_count(py).unwrap();
                let basic_op_time = op_start.elapsed();
                
                memory_results.insert(format!("creation_{}_nodes", size), creation_time.as_millis());
                memory_results.insert(format!("basic_ops_{}_nodes", size), basic_op_time.as_millis());
                
                assert_eq!(node_count, size);
                assert!(edge_count > 0);
                
                println!("Memory benchmark for {} nodes: creation={}ms, basic_ops={}ms", 
                    size, creation_time.as_millis(), basic_op_time.as_millis());
            }
            
            // Verify that operations remain efficient at scale
            let size_1000_time = memory_results.get("basic_ops_1000_nodes").unwrap();
            let size_10000_time = memory_results.get("basic_ops_10000_nodes").unwrap();
            
            // Basic operations should scale linearly or better
            assert!(*size_10000_time < *size_1000_time * 20, 
                "Basic operations don't scale well with graph size");
        });
    }

    /// Benchmark scalability tests
    #[test]
    fn benchmark_scalability() {
        Python::with_gil(|py| {
            let test_sizes = vec![100, 500, 1000, 2000];
            let mut scalability_results = HashMap::new();
            
            for &size in &test_sizes {
                println!("Running scalability test with {} nodes...", size);
                
                // Create test graph
                let graph = PyRustworkxGraph::with_capacity(size, size * 3);
                
                // Add nodes
                for i in 0..size {
                    graph.add_node(py, i.to_string()).unwrap();
                }
                
                // Create random sparse connections
                for i in 0..size {
                    for j in (i+1)..size {
                        if (i * j) % 10 == 0 { // ~10% connectivity
                            graph.add_edge(py, i, j, ((i + j) % 10 + 1) as f64).unwrap();
                        }
                    }
                }
                
                // Run multiple algorithms and measure total time
                let start = Instant::now();
                
                let _components = graph.connected_components(py).unwrap();
                let _centrality = graph.betweenness_centrality(py, false).unwrap(); // Normalized for speed
                let _distances = graph.dijkstra_shortest_paths(py, 0).unwrap();
                
                let total_time = start.elapsed();
                scalability_results.insert(format!("total_analysis_{}", size), total_time.as_millis());
                
                println!("  Total analysis time: {}ms", total_time.as_millis());
            }
            
            println!("Scalability Results:");
            for (size, time) in scalability_results {
                println!("  {}: {}ms", size, time);
            }
            
            // Check that performance is reasonable (not exponential)
            let size_100_time = scalability_results.get("total_analysis_100").unwrap();
            let size_2000_time = scalability_results.get("total_analysis_2000").unwrap();
            
            // Should be less than 1000x slower for 20x size increase
            assert!(*size_2000_time < *size_100_time * 1000, 
                "Performance scaling is poor - may indicate exponential complexity");
        });
    }

    /// Benchmark specific algorithm performance
    #[test]
    fn benchmark_algorithm_comparison() {
        Python::with_gil(|py| {
            // Create different types of graphs for algorithm testing
            let graphs = create_test_graphs(py);
            
            for (graph_type, graph) in graphs {
                println!("Benchmarking algorithms on {} graph...", graph_type);
                
                let mut results = HashMap::new();
                
                // BFS vs DFS performance
                let start = Instant::now();
                let _bfs = graph.bfs_tree(py, 0).unwrap();
                let bfs_time = start.elapsed();
                results.insert("BFS".to_string(), bfs_time.as_millis());
                
                let start = Instant::now();
                let _dfs = graph.dfs_tree(py, 0).unwrap();
                let dfs_time = start.elapsed();
                results.insert("DFS".to_string(), dfs_time.as_millis());
                
                // Shortest path algorithms
                let start = Instant::now();
                let _dijkstra = graph.dijkstra_shortest_paths(py, 0).unwrap();
                let dijkstra_time = start.elapsed();
                results.insert("Dijkstra".to_string(), dijkstra_time.as_millis());
                
                // Graph properties
                let start = Instant::now();
                let _components = graph.connected_components(py).unwrap();
                let components_time = start.elapsed();
                results.insert("Connected Components".to_string(), components_time.as_millis());
                
                println!("  {} Graph Results:", graph_type);
                for (algorithm, time) in results {
                    println!("    {}: {}ms", algorithm, time);
                }
            }
        });
    }

    /// Helper function to create different types of test graphs
    fn create_test_graphs(py: Python) -> Vec<(String, PyRustworkxGraph)> {
        let mut graphs = Vec::new();
        
        // Linear graph
        let linear = PyRustworkxGraph::with_capacity(100, 99);
        for i in 0..100 {
            linear.add_node(py, i.to_string()).unwrap();
            if i > 0 {
                linear.add_edge(py, i-1, i, 1.0).unwrap();
            }
        }
        graphs.push(("Linear".to_string(), linear));
        
        // Complete graph (smaller due to O(n²) edges)
        let complete = PyRustworkxGraph::with_capacity(20, 190); // C(20,2) = 190
        for i in 0..20 {
            complete.add_node(py, i.to_string()).unwrap();
            for j in (i+1)..20 {
                complete.add_edge(py, i, j, 1.0).unwrap();
            }
        }
        graphs.push(("Complete".to_string(), complete));
        
        // Random sparse graph
        let sparse = PyRustworkxGraph::with_capacity(100, 150);
        for i in 0..100 {
            sparse.add_node(py, i.to_string()).unwrap();
            for j in (i+1)..100 {
                if (i * j) % 13 == 0 { // Sparse connections
                    sparse.add_edge(py, i, j, 1.0).unwrap();
                }
            }
        }
        graphs.push(("Sparse".to_string(), sparse));
        
        graphs
    }

    /// Stress test for concurrent operations (if applicable)
    #[test]
    fn benchmark_concurrent_operations() {
        Python::with_gil(|py| {
            // Test concurrent read operations
            let graph = create_large_test_graph(py, 1000);
            
            let start = Instant::now();
            
            // Simulate concurrent access by running multiple operations
            for _ in 0..10 {
                let _node_count = graph.node_count(py).unwrap();
                let _edge_count = graph.edge_count(py).unwrap();
                let _is_empty = graph.is_empty(py).unwrap();
                let _components = graph.connected_components(py).unwrap();
            }
            
            let concurrent_time = start.elapsed();
            println!("Concurrent operations (10 iterations): {}ms", concurrent_time.as_millis());
            
            // Should complete quickly as these are read-only operations
            assert!(concurrent_time.as_millis() < 1000, "Concurrent operations too slow");
        });
    }

    /// Helper function to create a large test graph
    fn create_large_test_graph(py: Python, size: usize) -> PyRustworkxGraph {
        let graph = PyRustworkxGraph::with_capacity(size, size * 2);
        
        for i in 0..size {
            graph.add_node(py, i.to_string()).unwrap();
        }
        
        // Create a mix of local and long-range connections
        for i in 0..size {
            // Local connections
            if i > 0 {
                graph.add_edge(py, i-1, i, 1.0).unwrap();
            }
            // Some random long-range connections
            for j in (i+10)..size {
                if (i + j) % 20 == 0 {
                    graph.add_edge(py, i, j, 2.0).unwrap();
                }
            }
        }
        
        graph
    }
}