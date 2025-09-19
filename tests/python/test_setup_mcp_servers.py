"""
Tests for fast_context setup_mcp_servers module.
"""

import pytest
import tempfile
import os
import json
from pathlib import Path
from unittest.mock import patch, mock_open, MagicMock

def test_setup_mcp_servers_import():
    """Test that setup_mcp_servers can be imported."""
    import fast_context.setup_mcp_servers
    assert fast_context.setup_mcp_servers is not None

def test_setup_mcp_servers_has_main_functions():
    """Test that main functions exist in setup_mcp_servers."""
    from fast_context.setup_mcp_servers import (
        setup_fast_context_server,
        create_server_config,
        validate_server_config,
        list_available_servers,
        get_server_info,
        SERVER_CONFIG
    )
    
    assert callable(setup_fast_context_server)
    assert callable(create_server_config)
    assert callable(validate_server_config)
    assert callable(list_available_servers)
    assert callable(get_server_info)
    assert isinstance(SERVER_CONFIG, dict)

def test_server_config_structure():
    """Test that SERVER_CONFIG has the expected structure."""
    from fast_context.setup_mcp_servers import SERVER_CONFIG
    
    assert isinstance(SERVER_CONFIG, dict)
    assert "servers" in SERVER_CONFIG
    assert isinstance(SERVER_CONFIG["servers"], dict)
    
    # Should contain fast-context server
    assert "fast-context" in SERVER_CONFIG["servers"]
    fast_context_config = SERVER_CONFIG["servers"]["fast-context"]
    
    assert "command" in fast_context_config
    assert "args" in fast_context_config
    assert "env" in fast_context_config

def test_list_available_servers():
    """Test list_available_servers function."""
    from fast_context.setup_mcp_servers import list_available_servers
    
    servers = list_available_servers()
    
    assert isinstance(servers, list)
    assert len(servers) > 0
    assert "fast-context" in servers

def test_get_server_info():
    """Test get_server_info function."""
    from fast_context.setup_mcp_servers import get_server_info
    
    info = get_server_info("fast-context")
    
    assert isinstance(info, dict)
    assert "name" in info
    assert "command" in info
    assert "args" in info
    assert "description" in info
    
    # Test with non-existent server
    info = get_server_info("non-existent")
    assert info == {}

def test_validate_server_config_valid():
    """Test validate_server_config with valid config."""
    from fast_context.setup_mcp_servers import validate_server_config
    
    valid_config = {
        "command": "python",
        "args": ["-m", "fast_context.mcp_server"],
        "env": {"DEBUG": "true"}
    }
    
    is_valid = validate_server_config(valid_config)
    assert is_valid is True

def test_validate_server_config_invalid():
    """Test validate_server_config with invalid config."""
    from fast_context.setup_mcp_servers import validate_server_config
    
    # Missing required command
    invalid_config = {
        "args": ["-m", "fast_context.mcp_server"],
        "env": {"DEBUG": "true"}
    }
    
    is_valid = validate_server_config(invalid_config)
    assert is_valid is False

def test_validate_server_config_empty():
    """Test validate_server_config with empty config."""
    from fast_context.setup_mcp_servers import validate_server_config
    
    is_valid = validate_server_config({})
    assert is_valid is False

def test_create_server_config():
    """Test create_server_config function."""
    from fast_context.setup_mcp_servers import create_server_config
    
    config = create_server_config(
        command="python",
        args=["-m", "test.server"],
        env={"TEST": "true"}
    )
    
    assert isinstance(config, dict)
    assert config["command"] == "python"
    assert config["args"] == ["-m", "test.server"]
    assert config["env"]["TEST"] == "true"

def test_create_server_config_defaults():
    """Test create_server_config with default values."""
    from fast_context.setup_mcp_servers import create_server_config
    
    config = create_server_config(command="test-command")
    
    assert config["command"] == "test-command"
    assert config["args"] == []
    assert config["env"] == {}

@patch('fast_context.setup_mcp_servers.subprocess.run')
def test_setup_fast_context_server_success(mock_run):
    """Test setup_fast_context_server with successful execution."""
    from fast_context.setup_mcp_servers import setup_fast_context_server
    
    # Mock successful subprocess execution
    mock_result = MagicMock()
    mock_result.returncode = 0
    mock_result.stdout = "Server setup successful"
    mock_result.stderr = ""
    mock_run.return_value = mock_result
    
    result = setup_fast_context_server()
    
    assert result["success"] is True
    assert result["message"] == "Fast-Context MCP server setup completed successfully"
    assert result["details"]["returncode"] == 0

@patch('fast_context.setup_mcp_servers.subprocess.run')
def test_setup_fast_context_server_failure(mock_run):
    """Test setup_fast_context_server with failed execution."""
    from fast_context.setup_mcp_servers import setup_fast_context_server
    
    # Mock failed subprocess execution
    mock_result = MagicMock()
    mock_result.returncode = 1
    mock_result.stdout = "Setup failed"
    mock_result.stderr = "Error: something went wrong"
    mock_run.return_value = mock_result
    
    result = setup_fast_context_server()
    
    assert result["success"] is False
    assert "Failed to setup Fast-Context MCP server" in result["message"]
    assert result["details"]["returncode"] == 1

@patch('fast_context.setup_mcp_servers.subprocess.run')
def test_setup_fast_context_server_exception(mock_run):
    """Test setup_fast_context_server with exception."""
    from fast_context.setup_mcp_servers import setup_fast_context_server
    
    # Mock subprocess exception
    mock_run.side_effect = FileNotFoundError("python not found")
    
    result = setup_fast_context_server()
    
    assert result["success"] is False
    assert "Error setting up Fast-Context MCP server" in result["message"]
    assert "python not found" in result["details"]["error"]

def test_setup_mcp_servers_config_schema():
    """Test that configuration schema exists and is valid."""
    from fast_context.setup_mcp_servers import SERVER_CONFIG
    
    # Check that the schema is reasonable
    assert "version" in SERVER_CONFIG
    assert isinstance(SERVER_CONFIG["version"], str)
    
    servers = SERVER_CONFIG["servers"]
    for server_name, server_config in servers.items():
        assert isinstance(server_name, str)
        assert isinstance(server_config, dict)
        assert "command" in server_config
        assert isinstance(server_config["command"], str)
        assert "args" in server_config
        assert isinstance(server_config["args"], list)

def test_setup_mcp_servers_env_parsing():
    """Test that environment variables are properly handled."""
    from fast_context.setup_mcp_servers import SERVER_CONFIG
    
    fast_context_config = SERVER_CONFIG["servers"]["fast-context"]
    
    # Check that env section exists
    assert "env" in fast_context_config
    env = fast_context_config["env"]
    
    # Should be a dictionary
    assert isinstance(env, dict)
    
    # Common environment variables should be present
    if "PYTHONPATH" in env:
        assert isinstance(env["PYTHONPATH"], str)
    
    if "PYTHONUNBUFFERED" in env:
        assert isinstance(env["PYTHONUNBUFFERED"], str)