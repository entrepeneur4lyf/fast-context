"""
Basic unit tests for fast_context CLI functionality.
"""

import pytest
import subprocess
import tempfile
import os
from pathlib import Path

def test_cli_import():
    """Test that CLI modules can be imported."""
    from fast_context.cli import app
    
    assert app is not None
    assert app.info.name == "fast-context"

def test_cli_version_command():
    """Test CLI version command."""
    from fast_context.cli import version
    from click.testing import CliRunner
    
    runner = CliRunner()
    result = runner.invoke(version)
    
    assert result.exit_code == 0
    assert "Fast-Context" in result.output

def test_cli_config_commands_exist():
    """Test that CLI config commands exist."""
    from fast_context.cli import (
        config_init,
        config_validate,
        config_show,
        config_env
    )
    
    assert config_init is not None
    assert config_validate is not None
    assert config_show is not None
    assert config_env is not None

def test_cli_analyze_commands_exist():
    """Test that CLI analyze commands exist."""
    from fast_context.cli import (
        analyze_project,
        find_symbols,
        analyze_dependencies
    )
    
    assert analyze_project is not None
    assert find_symbols is not None
    assert analyze_dependencies is not None

def test_cli_graph_commands_exist():
    """Test that CLI graph commands exist."""
    from fast_context.cli import (
        graph_create,
        graph_analyze,
        graph_visualize
    )
    
    assert graph_create is not None
    assert graph_analyze is not None
    assert graph_visualize is not None

def test_cli_mcp_commands_exist():
    """Test that CLI MCP commands exist."""
    from fast_context.cli import (
        mcp_serve,
        mcp_info
    )
    
    assert mcp_serve is not None
    assert mcp_info is not None