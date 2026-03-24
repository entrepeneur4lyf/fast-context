"""
Tests for fast_context CLI functionality.
"""

import re
from unittest.mock import patch, MagicMock

import pytest
from click.testing import CliRunner

def test_cli_import():
    """Test that CLI can be imported."""
    from fast_context.cli import app
    assert app is not None
    assert app.info.name == "fast-context"

def test_cli_version_command():
    """Test CLI version command."""
    from fast_context.cli import version
    
    runner = CliRunner()
    result = runner.invoke(version)
    
    assert result.exit_code == 0
    assert "Fast-Context" in result.output

def test_cli_analyze_command_exists():
    """Test that analyze commands exist."""
    from fast_context.cli import analyze_project, find_symbols, analyze_dependencies
    
    assert analyze_project is not None
    assert find_symbols is not None  
    assert analyze_dependencies is not None

def test_cli_config_commands_exist():
    """Test that config commands exist."""
    from fast_context.cli import config_init, config_validate, config_show, config_env
    
    assert config_init is not None
    assert config_validate is not None
    assert config_show is not None
    assert config_env is not None

def test_cli_graph_commands_exist():
    """Test that graph commands exist."""
    from fast_context.cli import graph_create, graph_analyze, graph_visualize
    
    assert graph_create is not None
    assert graph_analyze is not None
    assert graph_visualize is not None

@patch('fast_context.cli.get_version')
def test_cli_version_mocked(mock_get_version):
    """Test CLI version with mocked version."""
    from fast_context.cli import version
    
    mock_get_version.return_value = "1.2.3"
    
    runner = CliRunner()
    result = runner.invoke(version)
    
    assert result.exit_code == 0
    assert "1.2.3" in result.output

def test_cli_help():
    """Test CLI help functionality."""
    from fast_context.cli import app
    
    runner = CliRunner()
    result = runner.invoke(app, ['--help'])
    clean_output = re.sub(r"\x1b\[[0-9;]*m", "", result.output)
    
    assert result.exit_code == 0
    assert "fast-context" in clean_output
    assert "--help" in clean_output
