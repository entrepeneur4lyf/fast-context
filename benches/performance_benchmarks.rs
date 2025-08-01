//! Performance benchmarks for fast-context
//! Measures core algorithm performance and graph operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

// Since this is a NAPI crate, we'll benchmark the core graph algorithms
// that are available through the rustworkx-core dependency
use petgraph::{Directed, Graph};

type TestGraph = Graph<i32, f64, Directed>;

fn create_test_graph(nodes: usize, edges: usize) -> TestGraph {
    let mut graph = Graph::new();

    // Add nodes
    let node_indices: Vec<_> = (0..nodes).map(|i| graph.add_node(i as i32)).collect();

    // Add edges with random weights
    for i in 0..edges {
        let from = node_indices[i % nodes];
        let to = node_indices[(i + 1) % nodes];
        graph.add_edge(from, to, (i as f64) * 0.1 + 1.0);
    }

    graph
}

fn benchmark_graph_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_creation");
    group.measurement_time(Duration::from_secs(5));

    let sizes = vec![
        (10, 20),     // Small graph
        (100, 200),   // Medium graph
        (1000, 2000), // Large graph
    ];

    for (nodes, edges) in sizes {
        group.bench_with_input(
            BenchmarkId::new("create_graph", format!("{nodes}n_{edges}e")),
            &(nodes, edges),
            |b, &(nodes, edges)| {
                b.iter(|| {
                    let graph = create_test_graph(black_box(nodes), black_box(edges));
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_traversal");
    group.measurement_time(Duration::from_secs(5));

    let graph = create_test_graph(1000, 2000);

    group.bench_function("node_count", |b| {
        b.iter(|| {
            let count = graph.node_count();
            black_box(count)
        });
    });

    group.bench_function("edge_count", |b| {
        b.iter(|| {
            let count = graph.edge_count();
            black_box(count)
        });
    });

    group.bench_function("node_iteration", |b| {
        b.iter(|| {
            let nodes: Vec<_> = graph.node_indices().collect();
            black_box(nodes)
        });
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(5));

    let workload_sizes = vec![10, 50, 100, 500];

    for size in workload_sizes {
        group.bench_with_input(
            BenchmarkId::new("repeated_graph_creation", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let graph =
                            create_test_graph(black_box(10 + i % 10), black_box(20 + i % 20));
                        black_box(graph);
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    group.measurement_time(Duration::from_secs(5));

    let medium_string = "fn test() { println!(\"hello\"); }".repeat(100);
    let large_string = "class MyClass { constructor() {} }".repeat(1000);

    let test_strings = vec![
        ("small", "function main() { return 42; }"),
        ("medium", medium_string.as_str()),
        ("large", large_string.as_str()),
    ];

    for (size_name, test_string) in test_strings {
        group.bench_with_input(
            BenchmarkId::new("string_processing", size_name),
            test_string,
            |b, s| {
                b.iter(|| {
                    let processed = s.chars().filter(|c| c.is_alphanumeric()).count();
                    black_box(processed)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_graph_creation,
    benchmark_graph_traversal,
    benchmark_memory_usage,
    benchmark_string_operations
);

criterion_main!(benches);
