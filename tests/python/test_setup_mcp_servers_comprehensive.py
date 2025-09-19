"""
Comprehensive tests for fast_context setup_mcp_servers module to increase coverage.
"""

import pytest
import tempfile
import json
import os
from pathlib import Path
from unittest.mock import patch, mock_open, MagicMock

def test_setup_mcp_servers_check_dependencies_partial():
    """Test check_dependencies with some packages present."""
    from fast_context.setup_mcp_servers import check_dependencies
    
    # Test actual environment - should work without crashing
    result = check_dependencies()
    assert isinstance(result, bool)

def test_setup_mcp_servers_install_dependencies_basic():
    """Test install_dependencies basic functionality."""
    from fast_context.setup_mcp_servers import install_dependencies
    
    # Test that function exists and can be called
    assert callable(install_dependencies)

def test_setup_mcp_servers_find_claude_desktop_config_mock():
    """Test find_claude_desktop_config with mocked paths."""
    from fast_context.setup_mcp_servers import find_claude_desktop_config
    
    # Mock platform and paths
    with patch('fast_context.setup_mcp_servers.sys.platform', 'darwin'):
        with patch('pathlib.Path.exists', return_value=True):
            result = find_claude_desktop_config()
            assert result is not None

def test_setup_mcp_servers_find_claude_desktop_config_not_found():
    """Test find_claude_desktop_config when no config exists."""
    from fast_context.setup_mcp_servers import find_claude_desktop_config
    
    # Mock platform and paths (no existing config)
    with patch('fast_context.setup_mcp_servers.sys.platform', 'linux'):
        with patch('pathlib.Path.exists', return_value=False):
            result = find_claude_desktop_config()
            assert result is None

def test_setup_mcp_servers_create_claude_desktop_config_new_file():
    """Test create_claude_desktop_config creating new file."""
    from fast_context.setup_mcp_servers import create_claude_desktop_config
    
    with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
        temp_path = f.name
    
    try:
        # Remove the file so we can test creation
        os.unlink(temp_path)
        
        result = create_claude_desktop_config(temp_path)
        assert result is True
        assert os.path.exists(temp_path)
        
        # Check content
        with open(temp_path, 'r') as f:
            config = json.load(f)
            assert 'mcpServers' in config
            assert 'fast-context' in config['mcpServers']
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def test_setup_mcp_servers_create_claude_desktop_config_existing_file():
    """Test create_claude_desktop_config with existing file."""
    from fast_context.setup_mcp_servers import create_claude_desktop_config
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump({"existing": "config"}, f)
        temp_path = f.name
    
    try:
        result = create_claude_desktop_config(temp_path)
        assert result is True
        
        # Check that existing config is preserved and fast-context is added
        with open(temp_path, 'r') as f:
            config = json.load(f)
            assert 'existing' in config
            assert 'mcpServers' in config
            assert 'fast-context' in config['mcpServers']
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def test_setup_mcp_servers_create_claude_desktop_config_invalid_json():
    """Test create_claude_desktop_config with invalid existing JSON."""
    from fast_context.setup_mcp_servers import create_claude_desktop_config
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write('invalid json content')
        temp_path = f.name
    
    try:
        # Should handle invalid JSON gracefully
        result = create_claude_desktop_config(temp_path)
        assert result is True  # Should still succeed
        
        # Should create valid config
        with open(temp_path, 'r') as f:
            config = json.load(f)
            assert 'mcpServers' in config
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def test_setup_mcp_servers_test_server_import_failure():
    """Test test_server with import failure."""
    from fast_context.setup_mcp_servers import test_server
    
    # Test with non-existent server
    result = test_server("nonexistent.server")
    assert result is False

def test_setup_mcp_servers_test_server_help_failure():
    """Test test_server with help command failure."""
    from fast_context.setup_mcp_servers import test_server
    
    # Mock import to succeed but help to fail
    with patch('builtins.__import__', return_value=MagicMock()):
        with patch('fast_context.setup_mcp_servers.subprocess.run') as mock_run:
            mock_result = MagicMock()
            mock_result.returncode = 1
            mock_result.stderr = "Help command failed"
            mock_run.return_value = mock_result
            
            result = test_server("test.server")
            assert result is False

def test_setup_mcp_servers_test_server_timeout():
    """Test test_server with timeout."""
    from fast_context.setup_mcp_servers import test_server
    
    # Mock import to succeed but subprocess to timeout
    with patch('builtins.__import__', return_value=MagicMock()):
        with patch('fast_context.setup_mcp_servers.subprocess.run') as mock_run:
            import subprocess
            mock_run.side_effect = subprocess.TimeoutExpired(cmd=["test"], timeout=10)
            
            result = test_server("test.server")
            assert result is False

def test_setup_mcp_servers_print_usage_examples():
    """Test print_usage_examples function."""
    from fast_context.setup_mcp_servers import print_usage_examples
    
    # Should not crash
    print_usage_examples()  # This should just print and return

def test_setup_mcp_servers_main_test_only():
    """Test main function with --test-only flag."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                mock_check.return_value = True
                mock_test.return_value = True
                
                # Mock sys.argv
                with patch('sys.argv', ['setup_mcp_servers.py', '--test-only']):
                    result = main()
                    assert result == 0
                    mock_exit.assert_not_called()

def test_setup_mcp_servers_main_test_only_failure():
    """Test main function with --test-only flag and failing tests."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                mock_check.return_value = True
                mock_test.return_value = False
                
                # Mock sys.argv
                with patch('sys.argv', ['setup_mcp_servers.py', '--test-only']):
                    result = main()
                    assert result == 1
                    mock_exit.assert_not_called()

def test_setup_mcp_servers_main_skip_deps_check():
    """Test main function with --skip-deps-check flag."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                with patch('fast_context.setup_mcp_servers.create_claude_desktop_config') as mock_create:
                    mock_test.return_value = True
                    mock_create.return_value = True
                    
                    # Mock sys.argv
                    with patch('sys.argv', ['setup_mcp_servers.py', '--skip-deps-check']):
                        result = main()
                        assert result == 0
                        mock_check.assert_not_called()
                        mock_exit.assert_not_called()

def test_setup_mcp_servers_main_install_deps():
    """Test main function with --install-deps flag."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.install_dependencies') as mock_install:
            with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
                with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                    with patch('fast_context.setup_mcp_servers.create_claude_desktop_config') as mock_create:
                        mock_install.return_value = True
                        mock_check.return_value = True
                        mock_test.return_value = True
                        mock_create.return_value = True
                        
                        # Mock sys.argv
                        with patch('sys.argv', ['setup_mcp_servers.py', '--install-deps']):
                            result = main()
                            assert result == 0
                            mock_install.assert_called_once()
                            mock_exit.assert_not_called()

def test_setup_mcp_servers_main_config_path():
    """Test main function with --config-path flag."""
    from fast_context.setup_mcp_servers import main
    
    with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
        temp_path = f.name
    
    try:
        # Mock sys.exit to prevent actual exit
        with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
            with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
                with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                    with patch('fast_context.setup_mcp_servers.create_claude_desktop_config') as mock_create:
                        mock_check.return_value = True
                        mock_test.return_value = True
                        mock_create.return_value = True
                        
                        # Mock sys.argv
                        with patch('sys.argv', ['setup_mcp_servers.py', '--config-path', temp_path]):
                            result = main()
                            assert result == 0
                            mock_create.assert_called_once()
                            mock_exit.assert_not_called()
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def test_setup_mcp_servers_main_deps_check_failure():
    """Test main function when dependency check fails."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            mock_check.return_value = False
            
            # Mock sys.argv
            with patch('sys.argv', ['setup_mcp_servers.py']):
                result = main()
                assert result == 1
                mock_exit.assert_not_called()

def test_setup_mcp_servers_main_install_deps_failure():
    """Test main function when install dependencies fails."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.install_dependencies') as mock_install:
            mock_install.return_value = False
            
            # Mock sys.argv
            with patch('sys.argv', ['setup_mcp_servers.py', '--install-deps']):
                result = main()
                assert result == 1
                mock_exit.assert_not_called()

def test_setup_mcp_servers_main_server_tests_failure():
    """Test main function when server tests fail."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                mock_check.return_value = True
                mock_test.return_value = False
                
                # Mock sys.argv
                with patch('sys.argv', ['setup_mcp_servers.py']):
                    result = main()
                    assert result == 1
                    mock_exit.assert_not_called()

def test_setup_mcp_servers_main_config_creation_failure():
    """Test main function when config creation fails."""
    from fast_context.setup_mcp_servers import main
    
    # Mock sys.exit to prevent actual exit
    with patch('fast_context.setup_mcp_servers.sys.exit') as mock_exit:
        with patch('fast_context.setup_mcp_servers.check_dependencies') as mock_check:
            with patch('fast_context.setup_mcp_servers.test_server') as mock_test:
                with patch('fast_context.setup_mcp_servers.create_claude_desktop_config') as mock_create:
                    mock_check.return_value = True
                    mock_test.return_value = True
                    mock_create.return_value = False
                    
                    # Mock sys.argv
                    with patch('sys.argv', ['setup_mcp_servers.py']):
                        result = main()
                        assert result == 1
                        mock_exit.assert_not_called()

def test_setup_mcp_servers_platform_paths():
    """Test that platform-specific paths are handled correctly."""
    from fast_context.setup_mcp_servers import find_claude_desktop_config
    
    # Test different platforms
    platforms = ['darwin', 'win32', 'linux']
    
    for platform in platforms:
        with patch('fast_context.setup_mcp_servers.sys.platform', platform):
            with patch('pathlib.Path.exists', return_value=False):
                result = find_claude_desktop_config()
                assert result is None  # Should return None when no config exists

def test_setup_mcp_servers_all_imports():
    """Test that all functions from setup_mcp_servers can be imported."""
    try:
        from fast_context.setup_mcp_servers import (
            check_dependencies,
            install_dependencies,
            find_claude_desktop_config,
            create_claude_desktop_config,
            test_server,
            print_usage_examples,
            main
        )
        # All imports successful
        assert True
    except ImportError as e:
        pytest.fail(f"setup_mcp_servers import failed: {e}")

def test_setup_mcp_servers_functions_callable():
    """Test that all setup_mcp_servers functions are callable."""
    from fast_context.setup_mcp_servers import (
        check_dependencies,
        install_dependencies,
        find_claude_desktop_config,
        create_claude_desktop_config,
        test_server,
        print_usage_examples,
        main
    )
    
    functions = [
        check_dependencies,
        install_dependencies,
        find_claude_desktop_config,
        create_claude_desktop_config,
        test_server,
        print_usage_examples,
        main
    ]
    
    for func in functions:
        assert callable(func), f"Function {func.__name__} is not callable"