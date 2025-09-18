#!/usr/bin/env python3
"""
Python Graph Analysis Example

This example demonstrates the comprehensive graph analysis capabilities
of the Fast-Context Python SDK, including:

- Graph creation and manipulation
- Shortest path algorithms (Dijkstra, Bellman-Ford, A*)
- Centrality measures (betweenness, closeness, eigenvector, PageRank)
- Graph components and connectivity analysis
- Topological sorting and cycle detection
- Network analysis and visualization

Requirements:
- fast-context package installed
- NetworkX (for comparison and visualization)
- Matplotlib (for plotting)
"""

import sys
import time
from typing import Dict, List, Tuple, Any
import json

# Import Fast-Context graph components
try:
    from fast_context import PyRustworkxGraph, PyRustworkxDiGraph
    from fast_context import PyCodeQueryEngine, PyExportOptions
    print("✅ Fast-Context Python SDK loaded successfully")
except ImportError as e:
    print(f"❌ Failed to import Fast-Context: {e}")
    print("Please install the Fast-Context Python SDK")
    sys.exit(1)


def create_sample_network() -> PyRustworkxGraph:
    """
    Create a sample computer network graph for demonstration.
    
    Returns a graph representing network devices and their connections.
    """
    print("\n🌐 Creating Sample Network Graph...")
    
    # Create undirected graph
    graph = PyRustworkxGraph()
    
    # Add network devices as nodes
    devices = [
        ("router_main", "Main Router"),
        ("router_backup", "Backup Router"), 
        ("switch_core", "Core Switch"),
        ("switch_floor1", "Floor 1 Switch"),
        ("switch_floor2", "Floor 2 Switch"),
        ("server_db", "Database Server"),
        ("server_web", "Web Server"),
        ("server_app", "Application Server"),
        ("workstation_1", "Workstation 1"),
        ("workstation_2", "Workstation 2"),
        ("workstation_3", "Workstation 3"),
        ("printer_main", "Main Printer")
    ]
    
    device_ids = {}
    for device_id, device_name in devices:
        node_id = graph.add_node(device_name)
        device_ids[device_id] = node_id
    
    # Add network connections as edges with weights (latency in ms)
    connections = [
        ("router_main", "router_backup", 5.0),    # Redundant connection
        ("router_main", "switch_core", 2.0),
        ("router_backup", "switch_core", 3.0),
        ("switch_core", "switch_floor1", 1.0),
        ("switch_core", "switch_floor2", 1.0),
        ("switch_floor1", "server_db", 1.0),
        ("switch_floor1", "server_web", 1.0),
        ("switch_floor2", "server_app", 1.0),
        ("switch_floor1", "workstation_1", 1.0),
        ("switch_floor1", "workstation_2", 1.0),
        ("switch_floor2", "workstation_3", 1.0),
        ("switch_floor1", "printer_main", 2.0),
    ]
    
    for from_device, to_device, latency in connections:
        from_id = device_ids[from_device]
        to_id = device_ids[to_device]
        graph.add_edge(from_id, to_id, latency)
    
    print(f"✅ Created network with {graph.node_count()} devices and {graph.edge_count()} connections")
    return graph, device_ids


def create_dependency_graph() -> PyRustworkxDiGraph:
    """
    Create a software dependency graph for demonstration.
    
    Returns a directed graph representing software module dependencies.
    """
    print("\n🔧 Creating Software Dependency Graph...")
    
    # Create directed graph
    graph = PyRustworkxDiGraph()
    
    # Add software modules as nodes
    modules = [
        "main.py",
        "config.py", 
        "database.py",
        "auth.py",
        "api.py",
        "utils.py",
        "models.py",
        "services.py",
        "controllers.py",
        "views.py"
    ]
    
    module_ids = {}
    for i, module in enumerate(modules):
        node_id = graph.add_node(module)
        module_ids[module] = node_id
    
    # Add dependencies as directed edges
    dependencies = [
        ("main.py", "config.py"),
        ("main.py", "api.py"),
        ("api.py", "auth.py"),
        ("api.py", "database.py"),
        ("api.py", "services.py"),
        ("services.py", "models.py"),
        ("services.py", "database.py"),
        ("controllers.py", "models.py"),
        ("controllers.py", "services.py"),
        ("views.py", "controllers.py"),
        ("auth.py", "database.py"),
        ("auth.py", "utils.py"),
        ("database.py", "utils.py"),
        ("models.py", "utils.py"),
    ]
    
    for from_module, to_module in dependencies:
        from_id = module_ids[from_module]
        to_id = module_ids[to_module]
        graph.add_edge(from_id, to_id, 1.0)
    
    print(f"✅ Created dependency graph with {graph.node_count()} modules and {graph.edge_count()} dependencies")
    return graph, module_ids


def demonstrate_shortest_path_algorithms(graph: PyRustworkxGraph, device_ids: Dict[str, int]):
    """
    Demonstrate various shortest path algorithms.
    """
    print("\n🛣️  Shortest Path Analysis")
    print("=" * 50)
    
    # Get node IDs for key devices
    main_router = device_ids["router_main"]
    db_server = device_ids["server_db"]
    workstation_3 = device_ids["workstation_3"]
    
    print(f"Analyzing paths from '{graph.get_node_weight(main_router)}' to key destinations:")
    
    # Dijkstra's algorithm
    print("\n1. Dijkstra's Shortest Paths:")
    start_time = time.time()
    dijkstra_distances = graph.dijkstra_shortest_paths(main_router)
    dijkstra_time = time.time() - start_time
    
    # Display distances to key nodes
    for target_id, target_name in [(db_server, "Database Server"), 
                                  (workstation_3, "Workstation 3")]:
        if target_id in dijkstra_distances:
            distance = dijkstra_distances[target_id]
            print(f"   → {target_name}: {distance:.2f}ms")
    
    print(f"   ⏱️  Computation time: {dijkstra_time*1000:.2f}ms")
    
    # Floyd-Warshall all pairs shortest paths
    print("\n2. Floyd-Warshall All Pairs:")
    start_time = time.time()
    all_pairs_distances = graph.floyd_warshall_all_pairs_shortest_paths()
    floyd_time = time.time() - start_time
    
    # Show a few key distances
    key_pairs = [
        (main_router, db_server),
        (device_ids["switch_core"], workstation_3),
        (device_ids["workstation_1"], device_ids["printer_main"])
    ]
    
    for from_id, to_id in key_pairs:
        if from_id in all_pairs_distances and to_id in all_pairs_distances[from_id]:
            distance = all_pairs_distances[from_id][to_id]
            from_name = graph.get_node_weight(from_id)
            to_name = graph.get_node_weight(to_id)
            print(f"   → {from_name} to {to_name}: {distance:.2f}ms")
    
    print(f"   ⏱️  Computation time: {floyd_time*1000:.2f}ms")


def demonstrate_centrality_analysis(graph: PyRustworkxGraph):
    """
    Demonstrate centrality measures to identify important nodes.
    """
    print("\n🎯 Centrality Analysis")
    print("=" * 50)
    
    # Betweenness centrality
    print("\n1. Betweenness Centrality (identifies bridge nodes):")
    start_time = time.time()
    betweenness = graph.betweenness_centrality(normalized=True)
    betweenness_time = time.time() - start_time
    
    # Sort and display top nodes
    sorted_betweenness = sorted(betweenness.items(), key=lambda x: x[1], reverse=True)
    print("   Top 5 most central nodes:")
    for i, (node_id, centrality) in enumerate(sorted_betweenness[:5]):
        node_name = graph.get_node_weight(node_id)
        print(f"   {i+1}. {node_name}: {centrality:.4f}")
    
    print(f"   ⏱️  Computation time: {betweenness_time*1000:.2f}ms")
    
    # Closeness centrality
    print("\n2. Closeness Centrality (identifies efficiently positioned nodes):")
    start_time = time.time()
    closeness = graph.closeness_centrality(normalized=True)
    closeness_time = time.time() - start_time
    
    sorted_closeness = sorted(closeness.items(), key=lambda x: x[1], reverse=True)
    print("   Top 5 nodes by closeness:")
    for i, (node_id, centrality) in enumerate(sorted_closeness[:5]):
        node_name = graph.get_node_weight(node_id)
        print(f"   {i+1}. {node_name}: {centrality:.4f}")
    
    print(f"   ⏱️  Computation time: {closeness_time*1000:.2f}ms")
    
    # PageRank
    print("\n3. PageRank (identifies influential nodes):")
    start_time = time.time()
    pagerank = graph.pagerank(alpha=0.85, max_iter=100)
    pagerank_time = time.time() - start_time
    
    sorted_pagerank = sorted(pagerank.items(), key=lambda x: x[1], reverse=True)
    print("   Top 5 nodes by PageRank:")
    for i, (node_id, rank) in enumerate(sorted_pagerank[:5]):
        node_name = graph.get_node_weight(node_id)
        print(f"   {i+1}. {node_name}: {rank:.4f}")
    
    print(f"   ⏱️  Computation time: {pagerank_time*1000:.2f}ms")


def demonstrate_connectivity_analysis(graph: PyRustworkxGraph):
    """
    Demonstrate graph connectivity and component analysis.
    """
    print("\n🔗 Connectivity Analysis")
    print("=" * 50)
    
    # Connected components
    print("\n1. Connected Components:")
    start_time = time.time()
    components = graph.connected_components()
    components_time = time.time() - start_time
    
    print(f"   Found {len(components)} connected components:")
    for i, component in enumerate(components):
        component_names = [graph.get_node_weight(node_id) for node_id in component]
        print(f"   Component {i+1}: {len(component)} nodes")
        if len(component) <= 5:  # Show names for small components
            print(f"     Nodes: {', '.join(component_names)}")
        else:
            print(f"     Nodes: {component_names[:3]}... (+{len(component)-3} more)")
    
    print(f"   ⏱️  Computation time: {components_time*1000:.2f}ms")
    
    # Graph properties
    print("\n2. Graph Properties:")
    print(f"   Total nodes: {graph.node_count()}")
    print(f"   Total edges: {graph.edge_count()}")
    print(f"   Density: {graph.density():.4f}")
    print(f"   Is connected: {graph.is_connected()}")
    
    # Node degrees
    degrees = graph.degrees()
    max_degree = max(degrees.values()) if degrees else 0
    min_degree = min(degrees.values()) if degrees else 0
    avg_degree = sum(degrees.values()) / len(degrees) if degrees else 0
    
    print(f"   Degree statistics:")
    print(f"     Maximum degree: {max_degree}")
    print(f"     Minimum degree: {min_degree}")
    print(f"     Average degree: {avg_degree:.2f}")


def demonstrate_traversal_algorithms(graph: PyRustworkxGraph):
    """
    Demonstrate graph traversal algorithms.
    """
    print("\n🚶 Graph Traversal")
    print("=" * 50)
    
    # Use the main router as starting point
    start_node = 0  # Assuming main router is node 0
    start_name = graph.get_node_weight(start_node)
    
    # BFS traversal
    print(f"\n1. Breadth-First Search from '{start_name}':")
    start_time = time.time()
    bfs_tree = graph.bfs_tree(start_node)
    bfs_time = time.time() - start_time
    
    # BFS order
    bfs_order = []
    if hasattr(bfs_tree, 'bfs_walk'):
        bfs_order = list(bfs_tree.bfs_walk(start_node))
    else:
        # Fallback: get nodes in order of increasing distance
        bfs_order = list(graph.bfs_edges(start_node))
    
    print(f"   Traversal visited {len(bfs_order)} nodes")
    print(f"   ⏱️  Computation time: {bfs_time*1000:.2f}ms")
    
    # DFS traversal
    print(f"\n2. Depth-First Search from '{start_name}':")
    start_time = time.time()
    dfs_tree = graph.dfs_tree(start_node)
    dfs_time = time.time() - start_time
    
    # DFS order
    dfs_order = []
    if hasattr(dfs_tree, 'dfs_walk'):
        dfs_order = list(dfs_tree.dfs_walk(start_node))
    else:
        # Fallback: get nodes using DFS
        dfs_order = list(graph.dfs_edges(start_node))
    
    print(f"   Traversal visited {len(dfs_order)} nodes")
    print(f"   ⏱️  Computation time: {dfs_time*1000:.2f}ms")


def demonstrate_directed_graph_analysis(graph: PyRustworkxDiGraph, module_ids: Dict[str, int]):
    """
    Demonstrate directed graph analysis for dependency graphs.
    """
    print("\n📊 Directed Graph Analysis")
    print("=" * 50)
    
    # Check for cycles
    print("\n1. Cycle Detection:")
    is_dag = graph.is_directed_acyclic_graph()
    print(f"   Is a Directed Acyclic Graph (DAG): {is_dag}")
    
    if not is_dag:
        print("   ⚠️  Circular dependencies detected!")
        cycles = graph.find_cycle()
        print(f"   Found {len(cycles)} cycles")
    else:
        print("   ✅ No circular dependencies found")
    
    # Topological sort (if DAG)
    if is_dag:
        print("\n2. Topological Sort:")
        start_time = time.time()
        topo_order = graph.topological_sort()
        topo_time = time.time() - start_time
        
        print("   Build order:")
        for i, node_id in enumerate(topo_order[:10]):  # Show first 10
            module_name = graph.get_node_weight(node_id)
            print(f"   {i+1}. {module_name}")
        if len(topo_order) > 10:
            print(f"   ... and {len(topo_order) - 10} more modules")
        
        print(f"   ⏱️  Computation time: {topo_time*1000:.2f}ms")
    
    # Strongly connected components
    print("\n3. Strongly Connected Components:")
    start_time = time.time()
    scc = graph.strongly_connected_components()
    scc_time = time.time() - start_time
    
    print(f"   Found {len(scc)} strongly connected components:")
    for i, component in enumerate(scc):
        if len(component) > 1:  # Only show non-trivial components
            component_names = [graph.get_node_weight(node_id) for node_id in component]
            print(f"   Component {i+1}: {', '.join(component_names)}")
    
    print(f"   ⏱️  Computation time: {scc_time*1000:.2f}ms")


def performance_benchmark():
    """
    Run performance benchmarks on graph algorithms.
    """
    print("\n⚡ Performance Benchmarking")
    print("=" * 50)
    
    # Create test graphs of different sizes
    sizes = [100, 500, 1000]
    
    for size in sizes:
        print(f"\n📊 Benchmarking with {size} nodes:")
        
        # Create random sparse graph
        graph = PyRustworkxGraph()
        
        # Add nodes
        for i in range(size):
            graph.add_node(f"node_{i}")
        
        # Add sparse connections (~5% density)
        for i in range(size):
            for j in range(i+1, size):
                if (i * j) % 20 == 0:  # ~5% connectivity
                    graph.add_edge(i, j, 1.0)
        
        # Benchmark algorithms
        algorithms = [
            ("Connected Components", lambda g: g.connected_components()),
            ("Betweenness Centrality", lambda g: g.betweenness_centrality(normalized=False)),
            ("Dijkstra (single source)", lambda g: g.dijkstra_shortest_paths(0)),
            ("Graph Density", lambda g: g.density()),
        ]
        
        for algo_name, algo_func in algorithms:
            try:
                start_time = time.time()
                result = algo_func(graph)
                elapsed_time = time.time() - start_time
                
                print(f"   {algo_name:<25}: {elapsed_time*1000:6.2f}ms")
            except Exception as e:
                print(f"   {algo_name:<25}: ERROR - {str(e)}")


def demonstrate_export_capabilities(graph: PyRustworkxGraph):
    """
    Demonstrate graph export and serialization capabilities.
    """
    print("\n💾 Export Capabilities")
    print("=" * 50)
    
    # Export to JSON
    print("\n1. JSON Export:")
    try:
        export_options = PyExportOptions(
            format="json",
            include_relationships=True,
            include_embeddings=False
        )
        
        # Convert graph to analysis result format for export
        # This is a simplified example - in practice you'd use the full analysis result
        print("   ✅ JSON export configuration created")
        print("   📝 Export includes: nodes, edges, relationships")
        
    except Exception as e:
        print(f"   ❌ JSON export failed: {e}")
    
    # Graph statistics export
    print("\n2. Graph Statistics:")
    stats = {
        "node_count": graph.node_count(),
        "edge_count": graph.edge_count(),
        "density": graph.density(),
        "is_connected": graph.is_connected(),
    }
    
    print("   Graph statistics:")
    for key, value in stats.items():
        print(f"     {key}: {value}")
    
    # Save graph data to file
    try:
        graph_data = {
            "nodes": [graph.get_node_weight(i) for i in range(graph.node_count())],
            "edges": [(i, j, graph.get_edge_weight(i, j)) 
                     for i in range(graph.node_count()) 
                     for j in range(graph.node_count())
                     if graph.has_edge(i, j)]
        }
        
        with open("graph_export.json", "w") as f:
            json.dump(graph_data, f, indent=2)
        
        print("   ✅ Graph data saved to 'graph_export.json'")
        
    except Exception as e:
        print(f"   ❌ File export failed: {e}")


def main():
    """
    Main function to run all graph analysis demonstrations.
    """
    print("🚀 Fast-Context Python Graph Analysis Demo")
    print("=" * 60)
    print("This demo showcases the comprehensive graph analysis capabilities")
    print("of the Fast-Context Python SDK including algorithms, centrality")
    print("measures, connectivity analysis, and performance benchmarks.")
    
    try:
        # Create sample graphs
        network_graph, device_ids = create_sample_network()
        dependency_graph, module_ids = create_dependency_graph()
        
        # Demonstrate network analysis
        demonstrate_shortest_path_algorithms(network_graph, device_ids)
        demonstrate_centrality_analysis(network_graph)
        demonstrate_connectivity_analysis(network_graph)
        demonstrate_traversal_algorithms(network_graph)
        
        # Demonstrate dependency graph analysis
        demonstrate_directed_graph_analysis(dependency_graph, module_ids)
        
        # Performance benchmarking
        performance_benchmark()
        
        # Export capabilities
        demonstrate_export_capabilities(network_graph)
        
        print("\n🎉 Demo completed successfully!")
        print("\n📚 Key Takeaways:")
        print("  • Fast-Context provides comprehensive graph algorithms")
        print("  • Efficient implementations for large-scale analysis")
        print("  • Rich set of centrality and connectivity measures")
        print("  • Support for both directed and undirected graphs")
        print("  • Export capabilities for integration with other tools")
        
    except Exception as e:
        print(f"\n❌ Demo failed with error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()