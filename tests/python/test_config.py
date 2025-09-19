"""
Basic unit tests for fast_context configuration system.
"""

import pytest
import tempfile
import os
from pathlib import Path

def test_config_import():
    """Test that configuration modules can be imported."""
    from fast_context.config import (
        FastContextConfig,
        AnalysisConfig,
        GraphConfig,
        MCPConfig,
        LoggingConfig,
        ConfigManager,
        load_config,
        save_config,
        create_default_config
    )
    # Test that all classes can be instantiated
    assert AnalysisConfig()
    assert GraphConfig()
    assert MCPConfig()
    assert LoggingConfig()
    assert FastContextConfig()
    assert ConfigManager()

def test_config_default_values():
    """Test configuration default values."""
    from fast_context.config import AnalysisConfig, GraphConfig, MCPConfig, LoggingConfig
    
    analysis = AnalysisConfig()
    assert analysis.max_files == 1000
    assert analysis.max_memory_mb == 512
    assert analysis.parallel_processing is True
    assert analysis.worker_threads == 4
    assert len(analysis.exclude_patterns) > 0
    
    graph = GraphConfig()
    assert graph.cache_size == 1000
    assert graph.enable_advanced_algorithms is True
    assert graph.max_graph_nodes == 10000
    assert graph.max_graph_edges == 50000
    
    mcp = MCPConfig()
    assert mcp.transport == "stdio"
    assert mcp.port == 8000
    assert mcp.enable_sse is True
    assert mcp.host == "localhost"
    assert mcp.timeout_seconds == 30
    
    logging = LoggingConfig()
    assert logging.level == "INFO"
    assert "asctime" in logging.format
    assert logging.enable_file_logging is False
    assert logging.log_file_path is None

def test_config_manager_basic():
    """Test basic ConfigManager functionality."""
    from fast_context.config import ConfigManager
    
    manager = ConfigManager()
    config = manager.get_config()
    
    assert config is not None
    assert config.analysis.max_files == 1000
    assert config.mcp.transport == "stdio"

def test_config_serialization():
    """Test configuration serialization."""
    from fast_context.config import FastContextConfig, asdict
    
    config = FastContextConfig()
    config_dict = asdict(config)
    
    assert "analysis" in config_dict
    assert "graph" in config_dict
    assert "mcp" in config_dict
    assert "logging" in config_dict
    
    # Check nested structure
    assert "max_files" in config_dict["analysis"]
    assert "cache_size" in config_dict["graph"]
    assert "transport" in config_dict["mcp"]
    assert "level" in config_dict["logging"]

def test_config_validation_schema():
    """Test configuration validation schema."""
    from fast_context.config import CONFIG_SCHEMA
    
    assert CONFIG_SCHEMA is not None
    assert "type" in CONFIG_SCHEMA
    assert CONFIG_SCHEMA["type"] == "object"
    assert "properties" in CONFIG_SCHEMA
    assert "analysis" in CONFIG_SCHEMA["properties"]
    assert "graph" in CONFIG_SCHEMA["properties"]
    assert "mcp" in CONFIG_SCHEMA["properties"]
    assert "logging" in CONFIG_SCHEMA["properties"]