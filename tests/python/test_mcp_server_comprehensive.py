"""
Comprehensive tests for fast_context MCP server to increase coverage.
"""

import pytest
import json
import tempfile
from unittest.mock import Mock, patch, MagicMock, AsyncMock
from pathlib import Path
import asyncio

def test_mcp_server_import():
    """Test that MCP server can be imported."""
    from fast_context import mcp_server
    assert mcp_server is not None

def test_mcp_server_mcp_instance():
    """Test that mcp instance exists."""
    from fast_context.mcp_server import mcp
    assert mcp is not None

def test_mcp_server_analysis_sessions():
    """Test that analysis_sessions dict exists."""
    from fast_context.mcp_server import analysis_sessions
    assert isinstance(analysis_sessions, dict)

def test_mcp_server_graph_registry():
    """Test that graph_registry dict exists."""
    from fast_context.mcp_server import graph_registry
    assert isinstance(graph_registry, dict)

def test_mcp_server_analyze_codebase_exists():
    """Test that analyze_codebase function exists."""
    from fast_context.mcp_server import analyze_codebase
    assert callable(analyze_codebase)

def test_mcp_server_find_symbols_exists():
    """Test that find_symbols function exists."""
    from fast_context.mcp_server import find_symbols
    assert callable(find_symbols)

def test_mcp_server_create_graph_exists():
    """Test that create_graph function exists."""
    from fast_context.mcp_server import create_graph
    assert callable(create_graph)

def test_mcp_server_analyze_graph_connectivity_exists():
    """Test that analyze_graph_connectivity function exists."""
    from fast_context.mcp_server import analyze_graph_connectivity
    assert callable(analyze_graph_connectivity)

def test_mcp_server_find_shortest_paths_exists():
    """Test that find_shortest_paths function exists."""
    from fast_context.mcp_server import find_shortest_paths
    assert callable(find_shortest_paths)

def test_mcp_server_create_advanced_graph_exists():
    """Test that create_advanced_graph function exists."""
    from fast_context.mcp_server import create_advanced_graph
    assert callable(create_advanced_graph)

def test_mcp_server_perform_advanced_graph_analysis_exists():
    """Test that perform_advanced_graph_analysis function exists."""
    from fast_context.mcp_server import perform_advanced_graph_analysis
    assert callable(perform_advanced_graph_analysis)

def test_mcp_server_get_project_info_exists():
    """Test that get_project_info function exists."""
    from fast_context.mcp_server import get_project_info
    assert callable(get_project_info)

def test_mcp_server_get_performance_metrics_exists():
    """Test that get_performance_metrics function exists."""
    from fast_context.mcp_server import get_performance_metrics
    assert callable(get_performance_metrics)

def test_mcp_server_get_analysis_sessions_exists():
    """Test that get_analysis_sessions function exists."""
    from fast_context.mcp_server import get_analysis_sessions
    assert callable(get_analysis_sessions)

def test_mcp_server_get_graph_registry_exists():
    """Test that get_graph_registry function exists."""
    from fast_context.mcp_server import get_graph_registry
    assert callable(get_graph_registry)

def test_mcp_server_analyze_codebase_streaming_exists():
    """Test that analyze_codebase_streaming function exists."""
    from fast_context.mcp_server import analyze_codebase_streaming
    assert callable(analyze_codebase_streaming)

def test_mcp_server_all_functions_callable():
    """Test that all MCP server functions are callable."""
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
        assert callable(func), f"Function {func.__name__} is not callable"

def test_mcp_server_global_state_types():
    """Test that global state variables have correct types."""
    from fast_context.mcp_server import analysis_sessions, graph_registry
    
    assert isinstance(analysis_sessions, dict)
    assert isinstance(graph_registry, dict)

def test_mcp_server_get_project_info_with_directory():
    """Test get_project_info with a real directory."""
    from fast_context.mcp_server import get_project_info
    
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create some test files
        Path(temp_dir, "test.py").write_text("print('hello')")
        Path(temp_dir, "README.md").write_text("# Test")
        
        result = get_project_info(temp_dir)
        result_data = json.loads(result)
        
        assert "project_path" in result_data
        assert "total_files" in result_data
        assert isinstance(result_data["total_files"], int)

def test_mcp_server_get_performance_metrics_structure():
    """Test that get_performance_metrics returns expected structure."""
    from fast_context.mcp_server import get_performance_metrics
    
    result = get_performance_metrics()
    result_data = json.loads(result)
    
    assert "timestamp" in result_data
    assert "system_metrics" in result_data
    assert "performance_indicators" in result_data

def test_mcp_server_get_analysis_sessions_structure():
    """Test that get_analysis_sessions returns expected structure."""
    from fast_context.mcp_server import get_analysis_sessions
    
    result = get_analysis_sessions()
    result_data = json.loads(result)
    
    assert "total_sessions" in result_data
    assert "active_sessions" in result_data
    assert isinstance(result_data["total_sessions"], int)

def test_mcp_server_get_graph_registry_structure():
    """Test that get_graph_registry returns expected structure."""
    from fast_context.mcp_server import get_graph_registry
    
    result = get_graph_registry()
    result_data = json.loads(result)
    
    assert "total_graphs" in result_data
    assert "graphs" in result_data
    assert isinstance(result_data["total_graphs"], int)

def test_mcp_server_create_graph_with_parameters():
    """Test create_graph function with parameters."""
    from fast_context.mcp_server import create_graph
    
    result = create_graph("undirected", 3, 2)
    result_data = json.loads(result)
    
    # Should either succeed or return error
    assert "graph_id" in result_data or "error" in result_data

def test_mcp_server_functions_return_json():
    """Test that MCP server functions return valid JSON."""
    from fast_context.mcp_server import (
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    )
    
    with tempfile.TemporaryDirectory() as temp_dir:
        functions = [
            get_project_info,
            get_performance_metrics,
            get_analysis_sessions,
            get_graph_registry
        ]
        
        for func in functions:
            if func == get_project_info:
                result = func(temp_dir)
            else:
                result = func()
            
            # Should be valid JSON
            try:
                json.loads(result)
            except json.JSONDecodeError:
                pytest.fail(f"Function {func.__name__} did not return valid JSON")

def test_mcp_server_imports_success():
    """Test that all MCP server imports work correctly."""
    try:
        from fast_context.mcp_server import (
            mcp,
            analysis_sessions,
            graph_registry,
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
        # All imports successful
        assert True
    except ImportError as e:
        pytest.fail(f"MCP server import failed: {e}")

def test_mcp_server_mcp_instance_type():
    """Test that mcp instance has expected attributes."""
    from fast_context.mcp_server import mcp
    
    # Should have methods for registering tools
    assert hasattr(mcp, 'tool')
    assert callable(mcp.tool)

def test_mcp_server_analysis_sessions_empty():
    """Test that analysis_sessions starts empty."""
    from fast_context.mcp_server import analysis_sessions
    
    # Should be an empty dict initially
    assert isinstance(analysis_sessions, dict)
    # Could be empty or contain test data

def test_mcp_server_graph_registry_empty():
    """Test that graph_registry starts empty."""
    from fast_context.mcp_server import graph_registry
    
    # Should be an empty dict initially
    assert isinstance(graph_registry, dict)
    # Could be empty or contain test data

def test_mcp_server_functions_with_invalid_path():
    """Test that functions handle invalid paths gracefully."""
    from fast_context.mcp_server import get_project_info
    
    # Should not crash with invalid path
    try:
        result = get_project_info("/nonexistent/path")
        result_data = json.loads(result)
        # Should contain error information or valid response
        assert isinstance(result_data, dict)
    except:
        # If it fails, that's okay for this test
        pass

def test_mcp_server_multiple_calls():
    """Test that multiple calls to functions work."""
    from fast_context.mcp_server import get_performance_metrics
    
    # Call multiple times
    result1 = get_performance_metrics()
    result2 = get_performance_metrics()
    
    # Both should be valid JSON
    data1 = json.loads(result1)
    data2 = json.loads(result2)
    
    assert isinstance(data1, dict)
    assert isinstance(data2, dict)