"""
Advanced tests for fast_context configuration system to increase coverage.
"""

import pytest
import tempfile
import os
import json
from pathlib import Path
from unittest.mock import patch, mock_open

def test_config_manager_default_paths():
    """Test ConfigManager default configuration paths."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    paths = manager._get_default_config_paths()
    
    assert isinstance(paths, list)
    assert len(paths) > 0
    
    # Should include home directory paths
    home_paths = [p for p in paths if str(p).startswith(str(Path.home()))]
    assert len(home_paths) > 0
    
    # Should include current working directory paths
    cwd_paths = [p for p in paths if str(p).startswith(str(Path.cwd()))]
    assert len(cwd_paths) > 0

def test_config_manager_load_config_with_path():
    """Test ConfigManager load_config with specific path."""
    from fast_context.config import ConfigManager, FastContextConfig
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump({
            "analysis": {"max_files": 500},
            "graph": {"cache_size": 2000}
        }, f)
        temp_path = f.name
    
    try:
        manager = ConfigManager()
        config = manager.load_config(temp_path)
        
        assert isinstance(config, FastContextConfig)
        assert config.analysis.max_files == 500
        assert config.graph.cache_size == 2000
    finally:
        os.unlink(temp_path)

def test_config_manager_load_config_nonexistent_path():
    """Test ConfigManager load_config with nonexistent path."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    
    with pytest.raises(FileNotFoundError):
        manager.load_config("/nonexistent/path/config.json")

def test_config_manager_validate_config_valid():
    """Test ConfigManager validate_config with valid data."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    valid_data = {
        "analysis": {
            "max_files": 1000,
            "parallel_processing": True,
            "exclude_patterns": ["*.tmp"]
        },
        "graph": {
            "cache_size": 1000,
            "enable_advanced_algorithms": True
        },
        "mcp": {
            "transport": "stdio",
            "port": 8000
        },
        "logging": {
            "level": "INFO",
            "enable_file_logging": False
        }
    }
    
    # Should not raise exception
    manager._validate_config(valid_data)

def test_config_manager_validate_config_invalid():
    """Test ConfigManager validate_config with invalid data."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    invalid_data = {
        "analysis": {
            "max_files": "invalid",  # Should be integer
            "parallel_processing": True
        },
        "graph": {
            "cache_size": -100,  # Should be >= 0
            "enable_advanced_algorithms": True
        }
    }
    
    with pytest.raises(ValueError, match="Configuration validation error"):
        manager._validate_config(invalid_data)

def test_config_manager_dict_to_config():
    """Test ConfigManager _dict_to_config method."""
    from fast_context.config import ConfigManager, AnalysisConfig, GraphConfig, MCPConfig, LoggingConfig, FastContextConfig
    
    manager = ConfigManager()
    test_data = {
        "analysis": {
            "max_files": 500,
            "parallel_processing": False,
            "exclude_patterns": ["*.test"]
        },
        "graph": {
            "cache_size": 2000,
            "enable_advanced_algorithms": False
        },
        "mcp": {
            "transport": "sse",
            "port": 9000,
            "timeout_seconds": 60
        },
        "logging": {
            "level": "DEBUG",
            "enable_file_logging": True,
            "log_file_path": "/tmp/test.log"
        }
    }
    
    config = manager._dict_to_config(test_data)
    
    assert isinstance(config, FastContextConfig)
    assert config.analysis.max_files == 500
    assert config.analysis.parallel_processing is False
    assert config.analysis.exclude_patterns == ["*.test"]
    assert config.graph.cache_size == 2000
    assert config.graph.enable_advanced_algorithms is False
    assert config.mcp.transport == "sse"
    assert config.mcp.port == 9000
    assert config.mcp.timeout_seconds == 60
    assert config.logging.level == "DEBUG"
    assert config.logging.enable_file_logging is True
    assert config.logging.log_file_path == "/tmp/test.log"

def test_config_manager_deep_merge():
    """Test ConfigManager _deep_merge method."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    base = {
        "analysis": {
            "max_files": 1000,
            "parallel_processing": True,
            "nested": {"value": "base"}
        },
        "graph": {
            "cache_size": 1000
        }
    }
    
    override = {
        "analysis": {
            "max_files": 2000,
            "nested": {"new_value": "override"}
        },
        "new_section": {"new_key": "new_value"}
    }
    
    result = manager._deep_merge(base, override)
    
    assert result["analysis"]["max_files"] == 2000  # Overridden
    assert result["analysis"]["parallel_processing"] is True  # Preserved
    assert result["analysis"]["nested"]["value"] == "base"  # Preserved
    assert result["analysis"]["nested"]["new_value"] == "override"  # Added
    assert result["graph"]["cache_size"] == 1000  # Preserved
    assert result["new_section"]["new_key"] == "new_value"  # Added

def test_config_manager_merge_config():
    """Test ConfigManager merge_config method."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    override_config = {
        "analysis": {"max_files": 3000},
        "mcp": {"port": 7000}
    }
    
    merged_config = manager.merge_config(override_config)
    
    assert merged_config.analysis.max_files == 3000
    assert merged_config.mcp.port == 7000
    # Default values should be preserved
    assert merged_config.analysis.parallel_processing is True
    assert merged_config.mcp.transport == "stdio"

def test_config_manager_create_default_config():
    """Test ConfigManager create_default_config method."""
    from fast_context.config import ConfigManager
    
    with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
        temp_path = f.name
    
    try:
        manager = ConfigManager()
        manager.create_default_config(temp_path, 'json')
        
        # File should be created
        assert os.path.exists(temp_path)
        
        # Content should be valid JSON
        with open(temp_path, 'r') as f:
            content = f.read()
            assert 'analysis' in content
            assert 'max_files' in content
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def test_config_manager_validate_config_file_valid():
    """Test ConfigManager validate_config_file with valid file."""
    from fast_context.config import ConfigManager
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump({
            "analysis": {"max_files": 100},
            "graph": {"cache_size": 500}
        }, f)
        temp_path = f.name
    
    try:
        manager = ConfigManager()
        is_valid = manager.validate_config_file(temp_path)
        assert is_valid is True
    finally:
        os.unlink(temp_path)

def test_config_manager_validate_config_file_invalid():
    """Test ConfigManager validate_config_file with invalid file."""
    from fast_context.config import ConfigManager
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write('{"invalid": "json"}')
        temp_path = f.name
    
    try:
        manager = ConfigManager()
        is_valid = manager.validate_config_file(temp_path)
        assert is_valid is False
    finally:
        os.unlink(temp_path)

def test_config_manager_validate_config_file_nonexistent():
    """Test ConfigManager validate_config_file with nonexistent file."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    is_valid = manager.validate_config_file("/nonexistent/path.json")
    assert is_valid is False

def test_config_manager_get_env_overrides():
    """Test ConfigManager get_env_overrides method."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    
    # Set some environment variables
    with patch.dict(os.environ, {
        'FAST_CONTEXT_MAX_FILES': '2000',
        'FAST_CONTEXT_MAX_MEMORY_MB': '1024',
        'FAST_CONTEXT_MCP_PORT': '9000',
        'FAST_CONTEXT_LOG_LEVEL': 'DEBUG'
    }):
        overrides = manager.get_env_overrides()
        
        assert overrides['analysis']['max_files'] == 2000
        assert overrides['analysis']['max_memory_mb'] == 1024
        assert overrides['mcp']['port'] == 9000
        assert overrides['logging']['level'] == 'DEBUG'

def test_config_manager_get_env_overrides_empty():
    """Test ConfigManager get_env_overrides with no environment variables."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    
    # Clear relevant environment variables
    env_vars = [
        'FAST_CONTEXT_MAX_FILES', 'FAST_CONTEXT_MAX_MEMORY_MB', 'FAST_CONTEXT_WORKER_THREADS',
        'FAST_CONTEXT_MCP_PORT', 'FAST_CONTEXT_MCP_TRANSPORT', 'FAST_CONTEXT_LOG_LEVEL'
    ]
    
    with patch.dict(os.environ, {}, clear=False):
        for var in env_vars:
            if var in os.environ:
                del os.environ[var]
        
        overrides = manager.get_env_overrides()
        assert overrides == {}

def test_config_manager_update_config():
    """Test ConfigManager update_config method."""
    from fast_context.config import ConfigManager, FastContextConfig
    
    manager = ConfigManager()
    original_config = manager.get_config()
    
    new_config = FastContextConfig()
    new_config.analysis.max_files = 5000
    
    manager.update_config(new_config)
    
    updated_config = manager.get_config()
    assert updated_config.analysis.max_files == 5000

def test_config_manager_get_config():
    """Test ConfigManager get_config method."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    config = manager.get_config()
    
    assert config is not None
    assert hasattr(config, 'analysis')
    assert hasattr(config, 'graph')
    assert hasattr(config, 'mcp')
    assert hasattr(config, 'logging')

def test_config_dataclass_defaults():
    """Test that all configuration dataclasses have proper defaults."""
    from fast_context.config import AnalysisConfig, GraphConfig, MCPConfig, LoggingConfig, FastContextConfig
    
    # Test AnalysisConfig defaults
    analysis = AnalysisConfig()
    assert analysis.max_files == 1000
    assert analysis.max_memory_mb == 512
    assert analysis.parallel_processing is True
    assert analysis.worker_threads == 4
    assert isinstance(analysis.exclude_patterns, list)
    assert len(analysis.exclude_patterns) > 0
    
    # Test GraphConfig defaults
    graph = GraphConfig()
    assert graph.cache_size == 1000
    assert graph.enable_advanced_algorithms is True
    assert graph.max_graph_nodes == 10000
    assert graph.max_graph_edges == 50000
    
    # Test MCPConfig defaults
    mcp = MCPConfig()
    assert mcp.transport == "stdio"
    assert mcp.port == 8000
    assert mcp.enable_sse is True
    assert mcp.host == "localhost"
    assert mcp.timeout_seconds == 30
    
    # Test LoggingConfig defaults
    logging = LoggingConfig()
    assert logging.level == "INFO"
    assert "asctime" in logging.format
    assert logging.enable_file_logging is False
    assert logging.log_file_path is None
    
    # Test FastContextConfig defaults
    config = FastContextConfig()
    assert isinstance(config.analysis, AnalysisConfig)
    assert isinstance(config.graph, GraphConfig)
    assert isinstance(config.mcp, MCPConfig)
    assert isinstance(config.logging, LoggingConfig)