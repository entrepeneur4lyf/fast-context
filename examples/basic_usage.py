#!/usr/bin/env python3
"""
Basic Fast-Context Usage Example

This example demonstrates the core functionality of Fast-Context including:
- Codebase analysis
- Symbol extraction
- Graph operations
- Language detection
"""

import sys
import os
from pathlib import Path

# Add the fast_context module to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import fast_context
    from fast_context import PyRustworkxGraph, FastContextAnalyzer, AnalyzerConfig
    print("✅ Fast-Context imported successfully")
except ImportError as e:
    print(f"❌ Failed to import Fast-Context: {e}")
    sys.exit(1)

def create_sample_project():
    """Create a sample project for analysis"""
    sample_dir = Path("/tmp/sample_project")
    sample_dir.mkdir(exist_ok=True)
    
    # Create sample Python files
    (sample_dir / "main.py").write_text("""
import utils
from models import User

def main():
    user = User("Alice", 25)
    greeting = utils.create_greeting(user)
    print(greeting)
    
if __name__ == "__main__":
    main()
""")
    
    (sample_dir / "utils.py").write_text("""
from models import User

def create_greeting(user: User) -> str:
    return f"Hello, {user.name}! You are {user.age} years old."

def calculate_birthday(user: User) -> str:
    return f"Happy birthday, {user.name}!"
""")
    
    (sample_dir / "models.py").write_text("""
from dataclasses import dataclass
from typing import List

@dataclass
class User:
    name: str
    age: int
    
    def is_adult(self) -> bool:
        return self.age >= 18
    
    def get_info(self) -> str:
        return f"{self.name} ({self.age} years old)"
""")
    
    return sample_dir

def demonstrate_basic_analysis():
    """Demonstrate basic codebase analysis"""
    print("\n🔍 Basic Codebase Analysis")
    print("-" * 40)
    
    # Create sample project
    sample_dir = create_sample_project()
    print(f"📁 Created sample project at: {sample_dir}")
    
    try:
        # Analyze the codebase
        config = AnalyzerConfig()
        analyzer = FastContextAnalyzer(str(sample_dir), config)
        
        # Get basic analysis
        analysis = analyzer.analyze()
        print(f"📊 Analysis completed:")
        print(f"   - Symbols found: {getattr(analysis, 'symbol_count', 'N/A')}")
        print(f"   - Files analyzed: {getattr(analysis, 'file_count', 'N/A')}")
        print(f"   - Memory usage: {getattr(analysis, 'memory_usage_mb', 'N/A')} MB")
        
        # Test language detection
        lang = fast_context.detect_language(str(sample_dir / "main.py"))
        print(f"🌐 Detected language: {lang}")
        
        # Get supported languages
        supported = fast_context.get_supported_languages()
        print(f"🌍 Supported languages: {len(supported)}")
        
    except Exception as e:
        print(f"❌ Analysis failed: {e}")
    
    finally:
        # Clean up
        import shutil
        shutil.rmtree(sample_dir, ignore_errors=True)

def demonstrate_graph_operations():
    """Demonstrate graph operations"""
    print("\n📊 Graph Operations")
    print("-" * 40)
    
    try:
        # Create a graph
        graph = PyRustworkxGraph()
        print(f"📈 Created graph with {graph.node_count} nodes")
        
        # Add nodes
        node_a = graph.add_node("Function A")
        node_b = graph.add_node("Function B")
        node_c = graph.add_node("Function C")
        print(f"📊 Added 3 nodes, total: {graph.node_count}")
        
        # Add edges
        edge_ab = graph.add_edge(node_a, node_b, 1.0)
        edge_bc = graph.add_edge(node_b, node_c, 2.0)
        print(f"🔗 Added {graph.edge_count} edges")
        
        # Test shortest path
        try:
            path_result = graph.dijkstra_shortest_path(node_a, node_c)
            print(f"🛣️  Shortest path from A to C: {path_result}")
        except Exception as e:
            print(f"⚠️  Shortest path calculation failed: {e}")
        
        # Test centrality
        try:
            centrality = graph.pagerank_centrality()
            print(f"📊 PageRank centrality: {len(centrality)} nodes")
        except Exception as e:
            print(f"⚠️  Centrality calculation failed: {e}")
        
        # Test connectivity
        try:
            components = graph.connected_components()
            print(f"🔗 Connected components: {len(components)}")
        except Exception as e:
            print(f"⚠️  Connectivity test failed: {e}")
            
    except Exception as e:
        print(f"❌ Graph operations failed: {e}")

def demonstrate_advanced_features():
    """Demonstrate advanced features"""
    print("\n🚀 Advanced Features")
    print("-" * 40)
    
    try:
        # Test graph algorithms
        graph = PyRustworkxGraph()
        
        # Create a more complex graph
        nodes = [graph.add_node(f"Node {i}") for i in range(5)]
        edges = [
            graph.add_edge(nodes[0], nodes[1], 1.0),
            graph.add_edge(nodes[0], nodes[2], 2.0),
            graph.add_edge(nodes[1], nodes[3], 3.0),
            graph.add_edge(nodes[2], nodes[3], 1.0),
            graph.add_edge(nodes[3], nodes[4], 2.0),
        ]
        
        print(f"📊 Created complex graph: {graph.node_count} nodes, {graph.edge_count} edges")
        
        # Test flow algorithms
        try:
            max_flow = graph.maximum_flow(nodes[0], nodes[4])
            print(f"💧 Maximum flow: {max_flow}")
        except Exception as e:
            print(f"⚠️  Flow algorithm failed: {e}")
        
        # Test minimum cut
        try:
            min_cut = graph.minimum_cut(nodes[0], nodes[4])
            print(f"✂️  Minimum cut: {min_cut}")
        except Exception as e:
            print(f"⚠️  Minimum cut failed: {e}")
        
        # Test topological sort
        try:
            topo_sort = graph.topological_sort()
            print(f"📋 Topological sort: {len(topo_sort)} nodes")
        except Exception as e:
            print(f"⚠️  Topological sort failed: {e}")
            
    except Exception as e:
        print(f"❌ Advanced features failed: {e}")

def main():
    """Main function"""
    print("🚀 Fast-Context Basic Usage Example")
    print("=" * 50)
    
    # Demonstrate basic functionality
    demonstrate_basic_analysis()
    demonstrate_graph_operations()
    demonstrate_advanced_features()
    
    print("\n🎉 Example completed successfully!")
    print("\n💡 Next steps:")
    print("   - Try analyzing your own codebase")
    print("   - Explore the MCP server integration")
    print("   - Check out the advanced examples")

if __name__ == "__main__":
    main()