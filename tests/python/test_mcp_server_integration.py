"""
Integration tests for Fast-Context MCP Server

This test suite verifies that the MCP server works correctly with the actual
Fast-Context core implementation, testing all major functionality.
"""

import asyncio
import json
import pytest
import tempfile
import sys
from pathlib import Path
from unittest.mock import Mock, patch

# Add the fast_context module to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

# Try to import the actual Fast-Context components
try:
    from fast_context import (
        FastContextAnalyzer,
        AnalyzerConfig,
        get_supported_languages,
        detect_language,
        get_version,
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


class TestFastContextCoreIntegration:
    """Test integration with Fast-Context core functionality."""
    
    @pytest.mark.skipif(not FAST_CONTEXT_AVAILABLE, reason="Fast-Context core not available")
    def test_fast_context_core_basics(self):
        """Test basic Fast-Context core functionality."""
        # Test version
        version = get_version()
        assert isinstance(version, str)
        assert len(version) > 0
        
        # Test supported languages
        languages = get_supported_languages()
        assert isinstance(languages, list)
        assert len(languages) > 0
        assert "python" in [lang.lower() for lang in languages]
        
        # Test language detection
        detected = detect_language("test.py")
        assert detected == "python"
        
        detected = detect_language("test.rs")
        assert detected == "rust"
        
        detected = detect_language("unknown.xyz")
        assert detected is None
    
    @pytest.mark.skipif(not FAST_CONTEXT_AVAILABLE, reason="Fast-Context core not available")
    def test_analyzer_config_creation(self):
        """Test AnalyzerConfig creation and configuration."""
        config = AnalyzerConfig(
            project_root="/tmp",
            languages=["python", "rust"],
            max_files=1000,
            enable_caching=True,
            enable_watching=False
        )
        
        assert config.project_root == "/tmp"
        assert "python" in config.languages
        assert "rust" in config.languages
        assert config.max_files == 1000
        assert config.enable_caching is True
        assert config.enable_watching is False
    
    @pytest.mark.skipif(not FAST_CONTEXT_AVAILABLE, reason="Fast-Context core not available")
    def test_fast_context_analyzer_creation(self):
        """Test FastContextAnalyzer creation."""
        config = AnalyzerConfig(project_root="/tmp", max_files=100)
        analyzer = FastContextAnalyzer("/tmp")
        
        assert analyzer is not None
        # Test that we can get the configuration back
        # This will depend on the actual implementation
        
    @pytest.mark.skipif(not FAST_CONTEXT_AVAILABLE, reason="Fast-Context core not available")
    def test_graph_creation(self):
        """Test basic graph functionality."""
        # Test undirected graph
        graph = PyRustworkxGraph()
        assert graph.node_count == 0
        assert graph.edge_count == 0
        
        # Test directed graph
        digraph = PyRustworkxDiGraph()
        assert digraph.node_count == 0
        assert digraph.edge_count == 0
        
        # Test graph with capacity
        graph_with_capacity = PyRustworkxGraph.with_capacity(10, 20)
        assert graph_with_capacity.node_count == 0
        assert graph_with_capacity.edge_count == 0


class TestMCPServerIntegration:
    """Test MCP server integration with Fast-Context core."""
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    @pytest.mark.asyncio
    async def test_find_symbols_integration(self):
        """Test symbol finding with real Fast-Context core."""
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "test_project"
            project_dir.mkdir()
            
            # Create a test file with symbols
            test_file = project_dir / "test.py"
            test_file.write_text("""
def hello_world():
    return "Hello"

class TestClass:
    def test_method(self):
        pass

def utility_function():
    pass
""")
            
            # Test finding symbols by pattern
            result = await find_symbols(str(project_dir), ".*")
            result_data = json.loads(result)
            
            assert "symbols" in result_data
            assert "total_matches" in result_data
            
            # Should find some symbols
            symbols = result_data["symbols"]
            assert len(symbols) > 0
            
            # Test filtering by symbol type
            result = find_symbols(str(project_dir), ".*", symbol_type="function")
            result_data = json.loads(result)
            
            function_symbols = [s for s in result_data["symbols"] if s["kind"] == "function"]
            assert len(function_symbols) > 0
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_graph_operations_integration(self):
        """Test graph operations with real Fast-Context core."""
        # Test basic graph creation
        result = create_graph("undirected", 5, 10)
        result_data = json.loads(result)
        
        assert "graph_id" in result_data
        assert "graph_type" in result_data
        assert result_data["graph_type"] == "undirected"
        assert "node_count" in result_data
        assert "edge_count" in result_data
        
        # Test graph connectivity analysis
        nodes = ["A", "B", "C", "D"]
        edges = [("A", "B", 1.0), ("B", "C", 1.0), ("C", "D", 1.0)]
        
        result = analyze_graph_connectivity(nodes, edges, "undirected")
        result_data = json.loads(result)
        
        assert "num_components" in result_data
        assert "is_connected" in result_data
        assert "density" in result_data
        
        # Test shortest paths
        result = find_shortest_paths(nodes, edges, "A", "dijkstra")
        result_data = json.loads(result)
        
        assert "distances" in result_data
        assert "algorithm" in result_data
        assert result_data["algorithm"] == "dijkstra"
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_advanced_graph_operations_integration(self):
        """Test advanced graph operations."""
        # Test advanced graph creation
        result = create_advanced_graph(
            "directed", 
            10, 
            20,
            metadata={"test": "integration"}
        )
        result_data = json.loads(result)
        
        assert "graph_id" in result_data
        assert "metadata" in result_data
        assert result_data["metadata"]["test"] == "integration"
        
        # Test advanced graph analysis
        graph_id = result_data["graph_id"]
        result = perform_advanced_graph_analysis(graph_id, "comprehensive")
        result_data = json.loads(result)
        
        assert "results" in result_data
        assert "analysis_type" in result_data
        
        # Should contain centrality, connectivity, and metrics
        results = result_data["results"]
        assert "centrality" in results
        assert "connectivity" in results
        assert "metrics" in results
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    @pytest.mark.asyncio
    async def test_streaming_analysis_integration(self):
        """Test streaming analysis functionality."""
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "test_project"
            project_dir.mkdir()
            
            # Create test files
            for i in range(5):
                (project_dir / f"file_{i}.py").write_text(f"# Test file {i}\ndef func_{i}():\n    pass\n")
            
            # Test streaming analysis
            result = await analyze_codebase_streaming(str(project_dir))
            result_data = json.loads(result)
            
            assert "session_id" in result_data
            assert "status" in result_data
            assert result_data["status"] == "completed"
            
            # Should have progress updates
            assert "progress_updates" in result_data
            progress_updates = result_data["progress_updates"]
            assert len(progress_updates) > 0
            
            # Check that progress increases
            progress_values = [update["progress"] for update in progress_updates]
            assert progress_values[-1] >= progress_values[0]  # Progress should increase or stay same
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_project_info_integration(self):
        """Test project information gathering."""
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "test_project"
            project_dir.mkdir()
            
            # Create a mixed project
            (project_dir / "package.json").write_text('{"name": "test"}')
            (project_dir / "main.py").write_text("print('hello')")
            (project_dir / "README.md").write_text("# Test Project")
            
            result = get_project_info(str(project_dir))
            result_data = json.loads(result)
            
            assert "project_path" in result_data
            assert "total_files" in result_data
            assert "file_extensions" in result_data
            assert "detected_types" in result_data
            
            # Should detect project types
            detected_types = result_data["detected_types"]
            assert len(detected_types) > 0
            
            # Should count file extensions
            file_extensions = result_data["file_extensions"]
            assert ".py" in file_extensions
            assert ".json" in file_extensions
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_performance_metrics_integration(self):
        """Test performance metrics functionality."""
        result = get_performance_metrics()
        result_data = json.loads(result)
        
        assert "timestamp" in result_data
        assert "system_metrics" in result_data
        assert "performance_indicators" in result_data
        
        # Check system metrics
        system_metrics = result_data["system_metrics"]
        assert "active_analysis_sessions" in system_metrics
        assert "registered_graphs" in system_metrics
        
        # Check performance indicators
        perf_indicators = result_data["performance_indicators"]
        assert "analysis_queue_length" in perf_indicators
        assert "memory_efficiency" in perf_indicators


class TestErrorHandlingIntegration:
    """Test error handling in integration scenarios."""
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_invalid_graph_operations(self):
        """Test handling of invalid graph operations."""
        # Test with invalid graph type
        result = create_graph("invalid_type")
        result_data = json.loads(result)
        
        assert "error" in result_data
        assert "Unsupported graph type" in result_data["error"]
        
        # Test analysis with non-existent graph
        result = perform_advanced_graph_analysis("non_existent_graph")
        result_data = json.loads(result)
        
        assert "error" in result_data
        assert "Invalid or missing graph_id" in result_data["error"]
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_malformed_input_handling(self):
        """Test handling of malformed input data."""
        # Test with invalid JSON in graph analysis
        try:
            # This should handle malformed input gracefully
            result = find_shortest_paths([], [], "invalid_node")
            result_data = json.loads(result)
            
            # Should either return an error or handle gracefully
            assert "error" in result_data or "distances" in result_data
        except Exception as e:
            # If it raises an exception, that's also acceptable error handling
            assert isinstance(e, (ValueError, KeyError, IndexError))


class TestResourceIntegration:
    """Test MCP resource functionality."""
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_analysis_sessions_resource(self):
        """Test analysis sessions resource."""
        from fast_context.mcp_server import get_analysis_sessions
        
        result = get_analysis_sessions()
        result_data = json.loads(result)
        
        assert "total_sessions" in result_data
        assert "active_sessions" in result_data
        
        # Should be JSON serializable
        assert isinstance(result_data["total_sessions"], int)
        assert isinstance(result_data["active_sessions"], dict)
    
    @pytest.mark.skipif(not MCP_SERVER_AVAILABLE, reason="MCP server not available")
    def test_graph_registry_resource(self):
        """Test graph registry resource."""
        from fast_context.mcp_server import get_graph_registry
        
        result = get_graph_registry()
        result_data = json.loads(result)
        
        assert "total_graphs" in result_data
        assert "graphs" in result_data
        
        # Should be JSON serializable
        assert isinstance(result_data["total_graphs"], int)
        assert isinstance(result_data["graphs"], dict)


@pytest.mark.skipif(not (FAST_CONTEXT_AVAILABLE and MCP_SERVER_AVAILABLE), 
                    reason="Both Fast-Context core and MCP server required")
class TestEndToEndIntegration:
    """End-to-end integration tests."""


if __name__ == "__main__":
    # Run tests if executed directly
    pytest.main([__file__, "-v"])
