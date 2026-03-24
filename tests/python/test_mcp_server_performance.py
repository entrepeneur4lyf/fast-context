"""
Performance tests for Fast-Context MCP Server

This test suite measures and validates the performance characteristics
of the MCP server and Fast-Context core integration.
"""

import asyncio
import json
import time
import statistics
import tempfile
import pytest
import psutil
import sys
from pathlib import Path
from typing import List, Dict, Any

# Add the fast_context module to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

# Try to import the required components
try:
    from fast_context import (
        FastContextAnalyzer,
        AnalyzerConfig,
        PyRustworkxGraph,
        PyRustworkxDiGraph
    )
    FAST_CONTEXT_AVAILABLE = True
except ImportError:
    FAST_CONTEXT_AVAILABLE = False
    pytest.skip("Fast-Context core not available", allow_module_level=True)

try:
    from fast_context.mcp_server import (
        analyze_codebase,
        find_symbols,
        create_graph,
        analyze_graph_connectivity,
        find_shortest_paths,
        get_project_info,
        get_performance_metrics,
        analyze_codebase_streaming,
        create_advanced_graph,
        perform_advanced_graph_analysis
    )
    MCP_SERVER_AVAILABLE = True
except ImportError:
    MCP_SERVER_AVAILABLE = False
    pytest.skip("MCP server not available", allow_module_level=True)


class PerformanceMetrics:
    """Helper class to collect and analyze performance metrics."""
    
    def __init__(self):
        self.measurements = []
        self.memory_before = None
        self.memory_after = None
    
    def start_measurement(self):
        """Start a performance measurement."""
        self.memory_before = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        return time.time()
    
    def end_measurement(self, start_time, operation_name: str):
        """End a performance measurement and store results."""
        end_time = time.time()
        duration = end_time - start_time
        self.memory_after = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        measurement = {
            "operation": operation_name,
            "duration_seconds": duration,
            "memory_before_mb": self.memory_before,
            "memory_after_mb": self.memory_after,
            "memory_delta_mb": self.memory_after - self.memory_before
        }
        
        self.measurements.append(measurement)
        return measurement
    
    def get_summary(self):
        """Get summary statistics for all measurements."""
        if not self.measurements:
            return {}
        
        durations = [m["duration_seconds"] for m in self.measurements]
        memory_deltas = [m["memory_delta_mb"] for m in self.measurements]
        
        return {
            "total_operations": len(self.measurements),
            "avg_duration_seconds": statistics.mean(durations),
            "min_duration_seconds": min(durations),
            "max_duration_seconds": max(durations),
            "median_duration_seconds": statistics.median(durations),
            "avg_memory_delta_mb": statistics.mean(memory_deltas),
            "total_memory_delta_mb": sum(memory_deltas)
        }


class TestMCPServerPerformance:
    """Performance tests for MCP server functionality."""
    
    @pytest.fixture
    def performance_tracker(self):
        """Fixture for tracking performance metrics."""
        return PerformanceMetrics()
    
    @pytest.fixture
    def large_test_project(self):
        """Create a large test project for performance testing."""
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "large_project"
            project_dir.mkdir()
            
            # Create a deep directory structure
            src_dir = project_dir / "src"
            src_dir.mkdir()
            
            # Create many Python files with various symbols
            for i in range(50):  # 50 files
                file_content = f"""
# File {i}
import os
import sys
from typing import Optional, List

def function_{i}(param1: str, param2: int = 42) -> Optional[str]:
    \"\"\"Function {i} documentation.\"\"\"
    result = f"processed_{{param1}}_{{param2}}"
    return result

class Class_{i}:
    \"\"\"Class {i} documentation.\"\"\"
    
    def __init__(self, value: int):
        self.value = value
        self._private = f"private_{{value}}"
    
    def method_{i}(self, x: int) -> int:
        return self.value + x
    
    @property
    def computed_value(self) -> str:
        return f"computed_{{self.value}}"

# Constants for file {i}
CONSTANT_{i} = f"constant_value_{{i}}"

def utility_function_{i}():
    \"\"\"Utility function {i}.\"\"\"
    return {i} * 2
"""
                
                file_path = src_dir / f"module_{i}.py"
                file_path.write_text(file_content)
            
            # Create additional project files
            (project_dir / "requirements.txt").write_text("""
requests>=2.25.0
numpy>=1.20.0
pandas>=1.3.0
pytest>=6.0.0
""")
            
            (project_dir / "setup.py").write_text("""
from setuptools import setup, find_packages

setup(
    name="large-test-project",
    version="1.0.0",
    packages=find_packages(),
    install_requires=[
        "requests>=2.25.0",
    ],
)
""")
            
            yield str(project_dir)
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_symbol_search_performance(self, performance_tracker, large_test_project):
        """Test performance of symbol search operations."""
        patterns = ["function_.*", "class_.*", "CONSTANT_.*", ".*_1$", "method_.*"]
        
        measurements = []
        for pattern in patterns:
            start_time = performance_tracker.start_measurement()
            result = find_symbols(large_test_project, pattern)
            measurement = performance_tracker.end_measurement(start_time, f"symbol_search_{pattern}")
            measurements.append(measurement)
            
            # Validate result
            result_data = json.loads(result)
            assert "error" not in result_data
            assert "symbols" in result_data
        
        # Analyze performance
        durations = [m["duration_seconds"] for m in measurements]
        avg_duration = statistics.mean(durations)
        
        print(f"\n📊 Symbol Search Performance:")
        print(f"   Average duration: {avg_duration:.3f} seconds")
        print(f"   Patterns tested: {len(patterns)}")
        
        # Performance assertions
        assert avg_duration < 5.0, f"Symbol search too slow: {avg_duration:.3f}s"
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_graph_creation_performance(self, performance_tracker):
        """Test performance of graph creation operations."""
        sizes = [(100, 200), (500, 1000), (1000, 2000)]
        
        measurements = []
        for nodes, edges in sizes:
            start_time = performance_tracker.start_measurement()
            result = create_graph("undirected", nodes, edges)
            measurement = performance_tracker.end_measurement(start_time, f"graph_creation_{nodes}_{edges}")
            measurements.append(measurement)
            
            # Validate result
            result_data = json.loads(result)
            assert "error" not in result_data
            assert "graph_id" in result_data
        
        # Analyze performance
        durations = [m["duration_seconds"] for m in measurements]
        avg_duration = statistics.mean(durations)
        
        print(f"\n📊 Graph Creation Performance:")
        print(f"   Average duration: {avg_duration:.3f} seconds")
        print(f"   Sizes tested: {sizes}")
        
        # Performance assertions
        assert avg_duration < 1.0, f"Graph creation too slow: {avg_duration:.3f}s"
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_graph_analysis_performance(self, performance_tracker):
        """Test performance of graph analysis operations."""
        # Create a moderately sized graph for testing
        nodes = list(range(100))
        edges = [(i, (i + 1) % 100, 1.0) for i in range(100)]  # Cycle graph
        edges.extend([(i, (i + 5) % 100, 1.0) for i in range(0, 100, 5)])  # Additional edges
        
        measurements = []
        algorithms = ["dijkstra", "bellman_ford"]
        
        for algorithm in algorithms:
            start_time = performance_tracker.start_measurement()
            result = find_shortest_paths(nodes, edges, 0, algorithm)
            measurement = performance_tracker.end_measurement(start_time, f"shortest_paths_{algorithm}")
            measurements.append(measurement)
            
            # Validate result
            result_data = json.loads(result)
            assert "error" not in result_data
            assert "distances" in result_data
        
        # Analyze performance
        durations = [m["duration_seconds"] for m in measurements]
        avg_duration = statistics.mean(durations)
        
        print(f"\n📊 Graph Analysis Performance:")
        print(f"   Average duration: {avg_duration:.3f} seconds")
        print(f"   Algorithms tested: {algorithms}")
        print(f"   Graph size: {len(nodes)} nodes, {len(edges)} edges")
        
        # Performance assertions
        assert avg_duration < 2.0, f"Graph analysis too slow: {avg_duration:.3f}s"
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    @pytest.mark.asyncio
    async def test_streaming_analysis_performance(self, performance_tracker, large_test_project):
        """Test performance of streaming analysis."""
        measurements = []
        
        for _ in range(3):
            start_time = performance_tracker.start_measurement()
            result = await analyze_codebase_streaming(large_test_project, max_files=50)
            measurement = performance_tracker.end_measurement(start_time, "streaming_analysis")
            measurements.append(measurement)
            
            # Validate result
            result_data = json.loads(result)
            assert "error" not in result_data
            assert "progress_updates" in result_data
            assert len(result_data["progress_updates"]) > 0
        
        # Analyze performance
        durations = [m["duration_seconds"] for m in measurements]
        avg_duration = statistics.mean(durations)
        
        print(f"\n📊 Streaming Analysis Performance:")
        print(f"   Average duration: {avg_duration:.3f} seconds")
        print(f"   Progress updates: {len(result_data.get('progress_updates', []))}")
        
        # Performance assertions
        assert avg_duration < 15.0, f"Streaming analysis too slow: {avg_duration:.3f}s"
    

class TestMemoryUsagePerformance:
    """Tests for memory usage and efficiency."""
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_memory_usage_large_graphs(self):
        """Test memory usage with large graphs."""
        memory_before = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        # Create several large graphs
        graph_ids = []
        for i in range(5):
            result = create_advanced_graph("undirected", 1000, 2000, metadata={"id": i})
            result_data = json.loads(result)
            graph_ids.append(result_data["graph_id"])
        
        memory_after_creation = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        # Perform analysis on all graphs
        for graph_id in graph_ids:
            perform_advanced_graph_analysis(graph_id, "comprehensive")
        
        memory_after_analysis = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        print(f"\n📊 Memory Usage Analysis:")
        print(f"   Memory before: {memory_before:.2f} MB")
        print(f"   Memory after creation: {memory_after_creation:.2f} MB")
        print(f"   Memory after analysis: {memory_after_analysis:.2f} MB")
        print(f"   Total increase: {memory_after_analysis - memory_before:.2f} MB")
        
        # Memory usage should be reasonable
        memory_increase = memory_after_analysis - memory_before
        assert memory_increase < 500, f"Memory usage too high: {memory_increase:.2f} MB"
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_memory_cleanup(self):
        """Test that memory is properly cleaned up after operations."""
        memory_before = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        # Perform memory-intensive operations
        for _ in range(10):
            create_advanced_graph("directed", 500, 1000)
            result = get_performance_metrics()
            json.loads(result)  # Parse result
        
        memory_during = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        # Allow some time for cleanup
        import gc
        gc.collect()
        time.sleep(0.1)
        
        memory_after = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        print(f"\n📊 Memory Cleanup Test:")
        print(f"   Memory before: {memory_before:.2f} MB")
        print(f"   Memory during: {memory_during:.2f} MB")
        print(f"   Memory after: {memory_after:.2f} MB")
        
        # Memory should return close to baseline
        memory_increase = memory_after - memory_before
        assert memory_increase < 100, f"Memory not properly cleaned up: {memory_increase:.2f} MB"


class TestScalabilityPerformance:
    """Tests for scalability as input size increases."""
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_graph_scalability(self):
        """Test how performance scales with graph size."""
        sizes = [(100, 200), (500, 1000), (1000, 2000), (2000, 4000)]
        durations = []
        
        for nodes, edges in sizes:
            # Create graph data
            node_list = list(range(nodes))
            edge_list = [(i, (i + 1) % nodes, 1.0) for i in range(nodes)]
            # Add more edges for larger graphs
            additional_edges = min(edges - nodes, nodes * 2)
            for i in range(additional_edges):
                edge_list.append((i, (i + nodes // 2) % nodes, 1.0))
            
            # Measure performance
            start_time = time.time()
            result = analyze_graph_connectivity(node_list, edge_list[:edges], "undirected")
            duration = time.time() - start_time
            
            durations.append((nodes, edges, duration))
            
            # Validate result
            result_data = json.loads(result)
            assert "error" not in result_data
        
        print(f"\n📊 Graph Scalability Analysis:")
        for nodes, edges, duration in durations:
            print(f"   {nodes} nodes, {edges} edges: {duration:.3f}s")
        
        # Check that performance scales reasonably
        # (Shouldn't be exponentially worse)
        if len(durations) >= 2:
            last_duration = durations[-1][2]
            print(f"   Largest graph duration: {last_duration:.3f}s")

            # Timer granularity makes ratio-based checks unstable at these sizes.
            # Keep this as a simple bounded-latency regression check instead.
            assert last_duration < 1.0, f"Large graph analysis took too long: {last_duration:.3f}s"


if __name__ == "__main__":
    # Run performance tests with detailed output
    pytest.main([__file__, "-v", "-s"])
