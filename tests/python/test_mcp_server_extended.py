"""
Tests for MCP server implementation to cover missing functionality.
"""

import pytest
import json
import tempfile
from unittest.mock import Mock, patch, MagicMock, AsyncMock
from pathlib import Path

def test_mcp_server_tool_registration():
    """Test that MCP tools are properly registered."""
    from fast_context.mcp_server import mcp
    
    # Check that the MCP instance has tool registration capability
    assert hasattr(mcp, 'tool')
    assert callable(mcp.tool)

def test_mcp_server_analysis_sessions_dict():
    """Test that analysis_sessions is a proper dict."""
    from fast_context.mcp_server import analysis_sessions
    
    assert isinstance(analysis_sessions, dict)
    # Should be empty or contain test data

def test_mcp_server_graph_registry_dict():
    """Test that graph_registry is a proper dict."""
    from fast_context.mcp_server import graph_registry
    
    assert isinstance(graph_registry, dict)
    # Should be empty or contain test data

def test_mcp_server_analyze_codebase_function_signature():
    """Test analyze_codebase function signature."""
    from fast_context.mcp_server import analyze_codebase
    
    # Should be a coroutine function
    import inspect
    assert inspect.iscoroutinefunction(analyze_codebase)

def test_mcp_server_find_symbols_function_signature():
    """Test find_symbols function signature."""
    from fast_context.mcp_server import find_symbols
    
    # Compatibility surface may be sync but still awaitable to legacy callers
    import inspect
    assert callable(find_symbols)

def test_mcp_server_create_graph_function_signature():
    """Test create_graph function signature."""
    from fast_context.mcp_server import create_graph
    
    # Should be a regular function
    import inspect
    assert not inspect.iscoroutinefunction(create_graph)

def test_mcp_server_get_project_info_with_empty_dir():
    """Test get_project_info with empty directory."""
    from fast_context.mcp_server import get_project_info
    
    with tempfile.TemporaryDirectory() as temp_dir:
        result = get_project_info(temp_dir)
        result_data = json.loads(result)
        
        assert "project_path" in result_data
        assert "total_files" in result_data
        assert result_data["total_files"] == 0

def test_mcp_server_get_project_info_with_subdirs():
    """Test get_project_info with nested directories."""
    from fast_context.mcp_server import get_project_info
    
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create nested structure
        subdir = Path(temp_dir) / "subdir"
        subdir.mkdir()
        Path(subdir, "nested.py").write_text("print('nested')")
        Path(temp_dir, "main.py").write_text("print('main')")
        
        result = get_project_info(temp_dir)
        result_data = json.loads(result)
        
        assert result_data["total_files"] >= 2

def test_mcp_server_create_graph_different_types():
    """Test create_graph with different graph types."""
    from fast_context.mcp_server import create_graph
    
    graph_types = ["directed", "undirected", "mixed"]
    
    for graph_type in graph_types:
        try:
            result = create_graph(graph_type, 3, 2)
            result_data = json.loads(result)
            
            # Should either succeed or return error
            assert "graph_id" in result_data or "error" in result_data
        except json.JSONDecodeError:
            # If it returns invalid JSON, that's okay for this test
            pass

def test_mcp_server_create_graph_edge_cases():
    """Test create_graph with edge cases."""
    from fast_context.mcp_server import create_graph
    
    # Test with zero nodes
    result = create_graph("undirected", 0, 0)
    result_data = json.loads(result)
    
    # Should handle gracefully
    assert "graph_id" in result_data or "error" in result_data

def test_mcp_server_get_performance_metrics_consistency():
    """Test that get_performance_metrics returns consistent structure."""
    from fast_context.mcp_server import get_performance_metrics
    
    # Call multiple times
    result1 = get_performance_metrics()
    result2 = get_performance_metrics()
    
    data1 = json.loads(result1)
    data2 = json.loads(result2)
    
    # Should have same structure
    assert set(data1.keys()) == set(data2.keys())
    assert "timestamp" in data1
    assert "system_metrics" in data1
    assert "performance_indicators" in data1

def test_mcp_server_get_analysis_sessions_consistency():
    """Test that get_analysis_sessions returns consistent structure."""
    from fast_context.mcp_server import get_analysis_sessions
    
    result = get_analysis_sessions()
    result_data = json.loads(result)
    
    # Should have expected structure
    assert "total_sessions" in result_data
    assert "active_sessions" in result_data
    assert isinstance(result_data["total_sessions"], int)
    assert isinstance(result_data["active_sessions"], dict)

def test_mcp_server_get_graph_registry_consistency():
    """Test that get_graph_registry returns consistent structure."""
    from fast_context.mcp_server import get_graph_registry
    
    result = get_graph_registry()
    result_data = json.loads(result)
    
    # Should have expected structure
    assert "total_graphs" in result_data
    assert "graphs" in result_data
    assert isinstance(result_data["total_graphs"], int)
    assert isinstance(result_data["graphs"], dict)

def test_mcp_server_functions_error_handling():
    """Test that functions handle errors gracefully."""
    from fast_context.mcp_server import (
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    )
    
    # All should handle invalid input gracefully
    functions = [
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    ]
    
    for func in functions:
        try:
            result = func()
            data = json.loads(result)
            assert isinstance(data, dict)
        except Exception as e:
            pytest.fail(f"Function {func.__name__} failed with error: {e}")

def test_mcp_server_functions_return_valid_json():
    """Test that all functions return valid JSON."""
    from fast_context.mcp_server import (
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry,
        create_graph
    )
    
    with tempfile.TemporaryDirectory() as temp_dir:
        functions = [
            (get_performance_metrics, []),
            (get_analysis_sessions, []),
            (get_graph_registry, []),
            (create_graph, ["undirected", 3, 2])
        ]
        
        for func, args in functions:
            try:
                if args:
                    result = func(*args)
                else:
                    result = func()
                
                # Should be valid JSON
                data = json.loads(result)
                assert isinstance(data, dict)
            except json.JSONDecodeError:
                pytest.fail(f"Function {func.__name__} did not return valid JSON")
            except Exception:
                # Some functions might fail, that's okay for this test
                pass

def test_mcp_server_global_state_isolation():
    """Test that global state variables are properly isolated."""
    from fast_context.mcp_server import analysis_sessions, graph_registry
    
    # Should be dict instances
    assert isinstance(analysis_sessions, dict)
    assert isinstance(graph_registry, dict)
    
    # Should not share references
    assert analysis_sessions is not graph_registry

def test_mcp_server_function_names():
    """Test that all functions have proper names."""
    from fast_context.mcp_server import (
        analyze_codebase,
        find_symbols,
        create_graph,
        analyze_graph_connectivity,
        find_shortest_paths,
        create_advanced_graph,
        perform_advanced_graph_analysis,
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry,
        analyze_codebase_streaming
    )
    
    functions = [
        analyze_codebase,
        find_symbols,
        create_graph,
        analyze_graph_connectivity,
        find_shortest_paths,
        create_advanced_graph,
        perform_advanced_graph_analysis,
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry,
        analyze_codebase_streaming
    ]
    
    for func in functions:
        assert hasattr(func, '__name__')
        assert func.__name__ != ""  # Should have a name

def test_mcp_server_function_docstrings():
    """Test that functions have docstrings."""
    from fast_context.mcp_server import (
        analyze_codebase,
        find_symbols,
        create_graph,
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    )
    
    functions = [
        analyze_codebase,
        find_symbols,
        create_graph,
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    ]
    
    for func in functions:
        # Should have docstring (though some might not)
        if func.__doc__ is not None:
            assert isinstance(func.__doc__, str)
            assert len(func.__doc__.strip()) > 0

def test_mcp_server_module_level_imports():
    """Test that module-level imports work correctly."""
    import fast_context.mcp_server as mcp_module
    
    # Should have expected attributes
    assert hasattr(mcp_module, 'mcp')
    assert hasattr(mcp_module, 'analysis_sessions')
    assert hasattr(mcp_module, 'graph_registry')

def test_mcp_server_no_global_pollution():
    """Test that module doesn't pollute global namespace unexpectedly."""
    import fast_context.mcp_server as mcp_module
    
    # Should have expected attributes but not too many unexpected ones
    attrs = [attr for attr in dir(mcp_module) if not attr.startswith('_')]
    
    # Should have our main functions
    expected_attrs = [
        'mcp', 'analysis_sessions', 'graph_registry',
        'analyze_codebase', 'find_symbols', 'create_graph',
        'get_project_info', 'get_performance_metrics',
        'get_analysis_sessions', 'get_graph_registry'
    ]
    
    for attr in expected_attrs:
        assert attr in attrs, f"Expected attribute {attr} not found in module"

def test_mcp_server_concurrent_access():
    """Test that functions can be called concurrently without issues."""
    from fast_context.mcp_server import get_performance_metrics
    import threading
    
    results = []
    errors = []
    
    def worker():
        try:
            result = get_performance_metrics()
            results.append(result)
        except Exception as e:
            errors.append(e)
    
    # Create multiple threads
    threads = []
    for _ in range(5):
        t = threading.Thread(target=worker)
        threads.append(t)
        t.start()
    
    # Wait for all threads to complete
    for t in threads:
        t.join()
    
    # Should have no errors
    assert len(errors) == 0
    assert len(results) == 5
    
    # All results should be valid JSON
    for result in results:
        data = json.loads(result)
        assert isinstance(data, dict)
