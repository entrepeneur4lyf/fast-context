#!/usr/bin/env python3
"""
Advanced Graph Analysis Example

This example demonstrates advanced graph analysis capabilities including:
- Complex graph algorithms
- Network analysis
- Performance optimization
- Real-world use cases
"""

import sys
import time
from pathlib import Path
from typing import List, Dict, Any

# Add the fast_context module to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import fast_context
    from fast_context import PyRustworkxGraph
    print("✅ Fast-Context imported successfully")
except ImportError as e:
    print(f"❌ Failed to import Fast-Context: {e}")
    sys.exit(1)

def create_dependency_graph():
    """Create a complex dependency graph"""
    print("\n🏗️  Creating Dependency Graph")
    print("-" * 40)
    
    graph = PyRustworkxGraph()
    
    # Define modules and their dependencies
    modules = {
        'app': ['auth', 'database', 'api'],
        'auth': ['utils', 'database'],
        'database': ['utils'],
        'api': ['auth', 'utils', 'cache'],
        'cache': ['utils'],
        'utils': [],
        'tests': ['app', 'auth', 'database']
    }
    
    # Add nodes for modules
    node_ids = {}
    for module in modules.keys():
        node_id = graph.add_node(module)
        node_ids[module] = node_id
    
    # Add dependency edges
    for module, dependencies in modules.items():
        for dep in dependencies:
            if dep in node_ids:
                edge_id = graph.add_edge(node_ids[module], node_ids[dep], 1.0)
    
    print(f"📊 Created graph: {graph.node_count} nodes, {graph.edge_count} edges")
    return graph, node_ids

def analyze_centrality(graph: PyRustworkxGraph, node_ids: Dict[str, int]):
    """Analyze centrality measures"""
    print("\n📊 Centrality Analysis")
    print("-" * 40)
    
    try:
        # PageRank centrality
        pagerank = graph.pagerank_centrality()
        print("📈 PageRank Centrality:")
        for node_name, node_id in sorted(node_ids.items()):
            if node_id < len(pagerank):
                score = pagerank[node_id]
                print(f"   {node_name}: {score:.4f}")
        
        # Betweenness centrality (if available)
        try:
            betweenness = graph.betweenness_centrality()
            print("\n🔗 Betweenness Centrality:")
            for node_name, node_id in sorted(node_ids.items()):
                if node_id < len(betweenness):
                    score = betweenness[node_id]
                    print(f"   {node_name}: {score:.4f}")
        except Exception as e:
            print(f"⚠️  Betweenness centrality not available: {e}")
        
        # Closeness centrality (if available)
        try:
            closeness = graph.closeness_centrality()
            print("\n🎯 Closeness Centrality:")
            for node_name, node_id in sorted(node_ids.items()):
                if node_id < len(closeness):
                    score = closeness[node_id]
                    print(f"   {node_name}: {score:.4f}")
        except Exception as e:
            print(f"⚠️  Closeness centrality not available: {e}")
            
    except Exception as e:
        print(f"❌ Centrality analysis failed: {e}")

def analyze_connectivity(graph: PyRustworkxGraph, node_ids: Dict[str, int]):
    """Analyze graph connectivity"""
    print("\n🔗 Connectivity Analysis")
    print("-" * 40)
    
    try:
        # Connected components
        components = graph.connected_components()
        print(f"🔗 Connected components: {len(components)}")
        for i, component in enumerate(components):
            print(f"   Component {i+1}: {len(component)} nodes")
        
        # Test if graph is connected
        is_connected = len(components) == 1
        print(f"✅ Graph is connected: {is_connected}")
        
        # Find articulation points (if available)
        try:
            articulation_points = graph.articulation_points()
            print(f"🎯 Articulation points: {len(articulation_points)}")
            for point in articulation_points:
                print(f"   - {point}")
        except Exception as e:
            print(f"⚠️  Articulation points not available: {e}")
        
        # Find cycles (if available)
        try:
            cycles = graph.cycle_basis()
            print(f"🔄 Cycle basis: {len(cycles)} cycles")
        except Exception as e:
            print(f"⚠️  Cycle analysis not available: {e}")
            
    except Exception as e:
        print(f"❌ Connectivity analysis failed: {e}")

def analyze_paths(graph: PyRustworkxGraph, node_ids: Dict[str, int]):
    """Analyze paths and distances"""
    print("\n🛣️  Path Analysis")
    print("-" * 40)
    
    try:
        # Test shortest paths between key modules
        key_pairs = [
            ('app', 'utils'),
            ('tests', 'database'),
            ('api', 'database')
        ]
        
        for source, target in key_pairs:
            if source in node_ids and target in node_ids:
                try:
                    path = graph.dijkstra_shortest_path(node_ids[source], node_ids[target])
                    distance = graph.dijkstra_shortest_path_lengths(node_ids[source], node_ids[target])
                    print(f"🛣️  {source} → {target}:")
                    print(f"   Path: {path}")
                    print(f"   Distance: {distance}")
                except Exception as e:
                    print(f"⚠️  No path from {source} to {target}: {e}")
        
        # All-pairs shortest paths (if available)
        try:
            all_pairs = graph.floyd_warshall_all_pairs()
            print(f"🌐 All-pairs shortest paths computed")
            # Show a few key distances
            if 'app' in node_ids and 'tests' in node_ids:
                app_id, tests_id = node_ids['app'], node_ids['tests']
                if app_id < len(all_pairs) and tests_id < len(all_pairs[app_id]):
                    distance = all_pairs[app_id][tests_id]
                    print(f"   app → tests: {distance}")
        except Exception as e:
            print(f"⚠️  All-pairs shortest paths not available: {e}")
            
    except Exception as e:
        print(f"❌ Path analysis failed: {e}")

def analyze_flow(graph: PyRustworkxGraph, node_ids: Dict[str, int]):
    """Analyze flow and cuts"""
    print("\n💧 Flow Analysis")
    print("-" * 40)
    
    try:
        # Test maximum flow between entry and exit points
        entry_points = ['app', 'tests']
        exit_points = ['utils']
        
        for entry in entry_points:
            for exit in exit_points:
                if entry in node_ids and exit in node_ids:
                    try:
                        max_flow = graph.maximum_flow(node_ids[entry], node_ids[exit])
                        print(f"💧 Max flow {entry} → {exit}: {max_flow}")
                    except Exception as e:
                        print(f"⚠️  Flow calculation failed for {entry} → {exit}: {e}")
        
        # Test minimum cuts
        for entry in entry_points:
            for exit in exit_points:
                if entry in node_ids and exit in node_ids:
                    try:
                        min_cut = graph.minimum_cut(node_ids[entry], node_ids[exit])
                        print(f"✂️  Min cut {entry} → {exit}: {min_cut}")
                    except Exception as e:
                        print(f"⚠️  Minimum cut failed for {entry} → {exit}: {e}")
            
    except Exception as e:
        print(f"❌ Flow analysis failed: {e}")

def analyze_performance(graph: PyRustworkxGraph):
    """Analyze graph performance"""
    print("\n⚡ Performance Analysis")
    print("-" * 40)
    
    try:
        # Test algorithm performance
        algorithms = [
            ("Connected Components", lambda g: g.connected_components()),
            ("PageRank", lambda g: g.pagerank_centrality()),
            ("Topological Sort", lambda g: g.topological_sort()),
        ]
        
        for name, func in algorithms:
            try:
                start_time = time.time()
                result = func(graph)
                end_time = time.time()
                duration = end_time - start_time
                print(f"⚡ {name}: {duration:.4f}s")
                
                if hasattr(result, '__len__'):
                    print(f"   Result size: {len(result)}")
            except Exception as e:
                print(f"⚠️  {name} failed: {e}")
        
        # Memory usage estimation
        try:
            import sys
            graph_size = sys.getsizeof(graph)
            print(f"💾 Graph memory size: {graph_size} bytes")
        except Exception as e:
            print(f"⚠️  Memory estimation failed: {e}")
            
    except Exception as e:
        print(f"❌ Performance analysis failed: {e}")

def demonstrate_real_world_use_case():
    """Demonstrate a real-world use case"""
    print("\n🌍 Real-World Use Case: Dependency Risk Assessment")
    print("-" * 50)
    
    try:
        # Create a more realistic dependency graph
        graph = PyRustworkxGraph()
        
        # Software system modules
        modules = [
            'frontend', 'backend', 'database', 'auth', 'cache',
            'logging', 'config', 'utils', 'tests', 'monitoring'
        ]
        
        # Add modules as nodes
        node_ids = {module: graph.add_node(module) for module in modules}
        
        # Add dependencies with weights (importance)
        dependencies = [
            ('frontend', 'backend', 5),
            ('frontend', 'auth', 3),
            ('backend', 'database', 8),
            ('backend', 'auth', 4),
            ('backend', 'cache', 3),
            ('backend', 'logging', 2),
            ('auth', 'database', 6),
            ('auth', 'config', 2),
            ('cache', 'config', 1),
            ('logging', 'config', 1),
            ('monitoring', 'backend', 3),
            ('monitoring', 'database', 2),
            ('tests', 'frontend', 2),
            ('tests', 'backend', 4),
            ('tests', 'auth', 2),
        ]
        
        for source, target, weight in dependencies:
            graph.add_edge(node_ids[source], node_ids[target], weight)
        
        print(f"📊 Created system dependency graph: {graph.node_count} modules, {graph.edge_count} dependencies")
        
        # Analyze critical paths
        print("\n🎯 Critical Path Analysis:")
        
        # Find most central modules
        try:
            centrality = graph.pagerank_centrality()
            sorted_modules = sorted(
                [(module, centrality[node_id]) for module, node_id in node_ids.items() if node_id < len(centrality)],
                key=lambda x: x[1],
                reverse=True
            )
            print("   Most critical modules:")
            for module, score in sorted_modules[:3]:
                print(f"   - {module}: {score:.4f}")
        except Exception as e:
            print(f"⚠️  Criticality analysis failed: {e}")
        
        # Identify potential bottlenecks
        print("\n🍾 Bottleneck Analysis:")
        try:
            # Modules with high in-degree (many dependencies)
            in_degrees = {}
            for module, node_id in node_ids.items():
                in_degrees[module] = sum(1 for edge in graph.edges() if edge[1] == node_id)
            
            bottlenecks = sorted(in_degrees.items(), key=lambda x: x[1], reverse=True)
            print("   Potential bottlenecks:")
            for module, degree in bottlenecks[:3]:
                print(f"   - {module}: {degree} incoming dependencies")
        except Exception as e:
            print(f"⚠️  Bottleneck analysis failed: {e}")
        
        # Risk assessment
        print("\n⚠️  Risk Assessment:")
        
        # Check for single points of failure
        try:
            components = graph.connected_components()
            if len(components) == 1:
                print("   ✅ System is fully connected")
            else:
                print(f"   ⚠️  System has {len(components)} disconnected components")
        except Exception as e:
            print(f"⚠️  Connectivity check failed: {e}")
        
    except Exception as e:
        print(f"❌ Real-world use case failed: {e}")

def main():
    """Main function"""
    print("🚀 Fast-Context Advanced Graph Analysis Example")
    print("=" * 60)
    
    # Create dependency graph
    graph, node_ids = create_dependency_graph()
    
    # Perform various analyses
    analyze_centrality(graph, node_ids)
    analyze_connectivity(graph, node_ids)
    analyze_paths(graph, node_ids)
    analyze_flow(graph, node_ids)
    analyze_performance(graph)
    
    # Demonstrate real-world use case
    demonstrate_real_world_use_case()
    
    print("\n🎉 Advanced Graph Analysis Example completed successfully!")
    print("\n💡 Next steps:")
    print("   - Apply these techniques to your own codebase")
    print("   - Experiment with different graph algorithms")
    print("   - Combine multiple analysis methods")

if __name__ == "__main__":
    main()