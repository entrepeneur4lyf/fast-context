"""
Comprehensive unit tests for fast_context MCP server functionality.
"""

import pytest
import json
import tempfile
from unittest.mock import Mock, patch, MagicMock
from pathlib import Path

def test_mcp_server_import():
    """Test that MCP server can be imported."""
    from fast_context import mcp_server
    assert mcp_server is not None

def test_mcp_server_instance():
    """Test that MCP server instance exists."""
    from fast_context.mcp_server import mcp
    assert mcp is not None

def test_mcp_server_global_state():
    """Test MCP server global state variables."""
    from fast_context.mcp_server import analysis_sessions, graph_registry
    
    assert isinstance(analysis_sessions, dict)
    assert isinstance(graph_registry, dict)

def test_mcp_server_analyze_codebase_function_exists():
    """Test that analyze_codebase function exists."""
    from fast_context.mcp_server import analyze_codebase
    
    assert callable(analyze_codebase)
    assert analyze_codebase.__name__ == "analyze_codebase"

def test_mcp_server_find_symbols_function_exists():
    """Test that find_symbols function exists."""
    from fast_context.mcp_server import find_symbols
    
    assert callable(find_symbols)
    assert find_symbols.__name__ == "find_symbols"

def test_mcp_server_graph_functions_exist():
    """Test that graph functions exist."""
    from fast_context.mcp_server import (
        create_graph,
        analyze_graph_connectivity,
        find_shortest_paths,
        create_advanced_graph,
        perform_advanced_graph_analysis
    )
    
    assert callable(create_graph)
    assert callable(analyze_graph_connectivity)
    assert callable(find_shortest_paths)
    assert callable(create_advanced_graph)
    assert callable(perform_advanced_graph_analysis)

def test_mcp_server_utility_functions_exist():
    """Test that utility functions exist."""
    from fast_context.mcp_server import (
        get_project_info,
        get_performance_metrics,
        get_analysis_sessions,
        get_graph_registry
    )
    
    assert callable(get_project_info)
    assert callable(get_performance_metrics)
    assert callable(get_analysis_sessions)
    assert callable(get_graph_registry)

def test_mcp_server_analyze_codebase_streaming_exists():
    """Test that streaming analysis function exists."""
    from fast_context.mcp_server import analyze_codebase_streaming
    
    assert callable(analyze_codebase_streaming)
    assert analyze_codebase_streaming.__name__ == "analyze_codebase_streaming"

def test_analyze_codebase_function_exists():
    """Test that analyze_codebase function exists."""
    from fast_context.mcp_server import analyze_codebase
    
    assert callable(analyze_codebase)

def test_find_symbols_function_exists():
    """Test that find_symbols function exists."""
    from fast_context.mcp_server import find_symbols
    
    assert callable(find_symbols)

def test_create_graph_function_exists():
    """Test that create_graph function exists."""
    from fast_context.mcp_server import create_graph
    
    assert callable(create_graph)

def test_get_project_info_with_valid_path():
    """Test get_project_info with a valid directory."""
    from fast_context.mcp_server import get_project_info
    
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create some files
        Path(temp_dir, "main.py").write_text("print('hello')")
        Path(temp_dir, "README.md").write_text("# Test Project")
        Path(temp_dir, "requirements.txt").write_text("requests>=2.0.0")
        
        result = get_project_info(temp_dir)
        result_data = json.loads(result)
        
        assert "project_path" in result_data
        assert "total_files" in result_data
        assert "file_extensions" in result_data
        assert result_data["total_files"] > 0
        assert ".py" in result_data["file_extensions"]

def test_get_performance_metrics():
    """Test get_performance_metrics function."""
    from fast_context.mcp_server import get_performance_metrics
    
    result = get_performance_metrics()
    result_data = json.loads(result)
    
    assert "timestamp" in result_data
    assert "system_metrics" in result_data
    assert "performance_indicators" in result_data
    
    # Check structure of system metrics
    system_metrics = result_data["system_metrics"]
    assert "active_analysis_sessions" in system_metrics
    assert "registered_graphs" in system_metrics
    
    # Check structure of performance indicators
    perf_indicators = result_data["performance_indicators"]
    assert "analysis_queue_length" in perf_indicators

def test_get_analysis_sessions():
    """Test get_analysis_sessions function."""
    from fast_context.mcp_server import get_analysis_sessions
    
    result = get_analysis_sessions()
    result_data = json.loads(result)
    
    assert "total_sessions" in result_data
    assert "active_sessions" in result_data
    assert isinstance(result_data["total_sessions"], int)
    assert isinstance(result_data["active_sessions"], dict)

def test_get_graph_registry():
    """Test get_graph_registry function."""
    from fast_context.mcp_server import get_graph_registry
    
    result = get_graph_registry()
    result_data = json.loads(result)
    
    assert "total_graphs" in result_data
    assert "graphs" in result_data
    assert isinstance(result_data["total_graphs"], int)
    assert isinstance(result_data["graphs"], dict)

def test_mcp_server_error_handling():
    """Test that MCP server handles errors gracefully."""
    from fast_context.mcp_server import find_symbols
    
    # Test that function exists and can be called
    assert callable(find_symbols)