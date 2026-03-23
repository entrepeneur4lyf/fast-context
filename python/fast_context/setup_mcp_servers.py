#!/usr/bin/env python3
"""
Fast-Context MCP Server Setup Script

This script helps set up and configure the Fast-Context MCP servers
for use with Claude Desktop and other MCP clients.

Usage:
    python setup_mcp_servers.py [--install-deps] [--config-path PATH]
"""

import json
import os
import sys
import shutil
from pathlib import Path
import subprocess
import argparse


SERVER_CONFIG = {
    "version": "1.0",
    "servers": {
        "fast-context": {
            "name": "fast-context",
            "command": "python",
            "args": ["-m", "fast_context.mcp_server"],
            "env": {
                "PYTHONUNBUFFERED": "1",
                "PYTHONPATH": str(Path(__file__).parent.parent),
            },
            "description": "Fast-Context MCP server",
        }
    },
}


def create_server_config(command: str, args=None, env=None):
    """Create a normalized MCP server config dictionary."""
    return {
        "command": command,
        "args": args or [],
        "env": env or {},
    }


def validate_server_config(config):
    """Validate a basic MCP server config shape."""
    if not isinstance(config, dict):
        return False
    if not isinstance(config.get("command"), str) or not config["command"]:
        return False
    if not isinstance(config.get("args", []), list):
        return False
    if not isinstance(config.get("env", {}), dict):
        return False
    return True


def list_available_servers():
    """List configured server names."""
    return list(SERVER_CONFIG["servers"].keys())


def get_server_info(server_name):
    """Get metadata for a configured server."""
    server = SERVER_CONFIG["servers"].get(server_name)
    if server is None:
        return {}
    return {
        "name": server_name,
        "command": server["command"],
        "args": server["args"],
        "env": server["env"],
        "description": server.get("description", ""),
    }


def setup_fast_context_server():
    """Run a basic setup check for the Fast-Context MCP server."""
    try:
        result = subprocess.run(
            [sys.executable, "-m", "fast_context.mcp_server", "--help"],
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            return {
                "success": True,
                "message": "Fast-Context MCP server setup completed successfully",
                "details": {
                    "returncode": result.returncode,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                },
            }
        return {
            "success": False,
            "message": "Failed to setup Fast-Context MCP server",
            "details": {
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
            },
        }
    except Exception as e:
        return {
            "success": False,
            "message": "Error setting up Fast-Context MCP server",
            "details": {"error": str(e)},
        }


def check_dependencies():
    """Check if required dependencies are installed."""
    required_packages = ["mcp", "fastmcp", "click", "uvicorn", "starlette"]
    missing_packages = []
    
    for package in required_packages:
        try:
            __import__(package)
        except ImportError:
            missing_packages.append(package)
    
    if missing_packages:
        print("❌ Missing required packages:")
        for package in missing_packages:
            print(f"   - {package}")
        print("\nInstall with:")
        print("   pip install mcp fastmcp click uvicorn starlette")
        return False
    
    print("✅ All required dependencies are installed")
    return True


def install_dependencies():
    """Install required dependencies using pip."""
    print("📦 Installing MCP server dependencies...")
    
    packages = [
        "mcp>=1.0.0",
        "fastmcp>=0.1.0", 
        "click>=8.0.0",
        "uvicorn>=0.20.0",
        "starlette>=0.25.0",
        "anyio>=3.7.0"
    ]
    
    try:
        for package in packages:
            print(f"   Installing {package}...")
            subprocess.run([sys.executable, "-m", "pip", "install", package], 
                         check=True, capture_output=True)
        
        print("✅ Dependencies installed successfully")
        return True
    
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False


def find_claude_desktop_config():
    """Find Claude Desktop configuration file path."""
    possible_paths = []
    
    # macOS
    if sys.platform == "darwin":
        possible_paths.extend([
            Path.home() / "Library" / "Application Support" / "Claude" / "claude_desktop_config.json",
            Path.home() / "Library" / "Application Support" / "Claude Desktop" / "claude_desktop_config.json"
        ])
    
    # Windows
    elif sys.platform == "win32":
        possible_paths.extend([
            Path.home() / "AppData" / "Roaming" / "Claude" / "claude_desktop_config.json",
            Path.home() / "AppData" / "Roaming" / "Claude Desktop" / "claude_desktop_config.json"
        ])
    
    # Linux
    else:
        possible_paths.extend([
            Path.home() / ".config" / "Claude" / "claude_desktop_config.json",
            Path.home() / ".config" / "Claude Desktop" / "claude_desktop_config.json"
        ])
    
    for path in possible_paths:
        if path.exists():
            return str(path)
    
    return None


def create_claude_desktop_config(config_path=None):
    """Create Claude Desktop configuration file."""
    if config_path is None:
        config_path = find_claude_desktop_config()
        
        if config_path is None:
            # Use default path based on platform
            if sys.platform == "darwin":
                config_path = Path.home() / "Library" / "Application Support" / "Claude" / "claude_desktop_config.json"
            elif sys.platform == "win32":
                config_path = Path.home() / "AppData" / "Roaming" / "Claude" / "claude_desktop_config.json"
            else:
                config_path = Path.home() / ".config" / "Claude" / "claude_desktop_config.json"
    
    config_path = Path(config_path)
    
    # Create directory if it doesn't exist
    config_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Load existing config if it exists
    existing_config = {}
    if config_path.exists():
        try:
            with open(config_path, 'r') as f:
                existing_config = json.load(f)
        except (json.JSONDecodeError, IOError):
            print(f"⚠️  Could not read existing config at {config_path}")
            existing_config = {}
    
    # Ensure mcpServers section exists
    if "mcpServers" not in existing_config:
        existing_config["mcpServers"] = {}
    
    # Add Fast-Context server
    existing_config["mcpServers"]["fast-context"] = {
        "command": "python",
        "args": ["-m", "fast_context.mcp_server"],
        "env": {}
    }
    
    # Write config file
    try:
        with open(config_path, 'w') as f:
            json.dump(existing_config, f, indent=2)
        
        print(f"✅ Claude Desktop configuration created at: {config_path}")
        print("   Please restart Claude Desktop for changes to take effect")
        return True
    
    except IOError as e:
        print(f"❌ Failed to write config file: {e}")
        return False


def test_server(server_name="fast_context.mcp_server"):
    """Test if MCP server can be imported and run basic checks."""
    try:
        print(f"🧪 Testing {server_name}...")
        
        # Test import
        __import__(server_name)
        print("   ✅ Server imports successfully")
        
        # Test basic functionality by running help
        result = subprocess.run([
            sys.executable, "-m", server_name, "--help"
        ], capture_output=True, text=True, timeout=10)
        
        if result.returncode == 0:
            print("   ✅ Server responds to help command")
            return True
        else:
            print(f"   ❌ Server help command failed: {result.stderr}")
            return False
    
    except ImportError as e:
        print(f"   ❌ Failed to import server: {e}")
        return False
    except subprocess.TimeoutExpired:
        print("   ❌ Server help command timed out")
        return False
    except Exception as e:
        print(f"   ❌ Unexpected error: {e}")
        return False


def print_usage_examples():
    """Print usage examples for the MCP servers."""
    print("\n📖 Usage Examples")
    print("=" * 50)
    
    print("\n1. Start FastMCP server with stdio:")
    print("   python -m fast_context.mcp_server")
    print("   python -m fast_context.mcp_server --transport stdio")
    
    print("\n2. Start FastMCP server with SSE:")
    print("   python -m fast_context.mcp_server --transport sse --port 8000")
    
    print("\n3. Test server connectivity:")
    print("   python -m fast_context.mcp_server --help")
    
    print("\n5. Example usage with Claude Desktop:")
    print("   Once configured, you can use tools like:")
    print("   - Analyze codebase: 'Analyze the Python project in ./my_project'")
    print("   - Find symbols: 'Find all functions named process_* in ./src'")
    print("   - Graph analysis: 'Create a dependency graph of this codebase'")


def main():
    """Main setup function."""
    parser = argparse.ArgumentParser(description="Setup Fast-Context MCP Servers")
    parser.add_argument("--install-deps", action="store_true", help="Install required dependencies")
    parser.add_argument("--config-path", help="Path to Claude Desktop config file")
    parser.add_argument("--skip-deps-check", action="store_true", help="Skip dependency checking")
    parser.add_argument("--test-only", action="store_true", help="Only test existing setup")
    
    args = parser.parse_args()
    
    print("🚀 Fast-Context MCP Server Setup")
    print("=" * 50)
    
    if args.test_only:
        print("🧪 Testing existing setup...")
        
        if not args.skip_deps_check:
            if not check_dependencies():
                return 1
        
        success = True
        success &= test_server("fast_context.mcp_server")
        
        if success:
            print("\n✅ All tests passed! MCP servers are ready to use.")
        else:
            print("\n❌ Some tests failed. Check the error messages above.")
            return 1
        
        return 0
    
    # Install dependencies if requested
    if args.install_deps:
        if not install_dependencies():
            return 1
    
    # Check dependencies
    if not args.skip_deps_check:
        if not check_dependencies():
            if not args.install_deps:
                print("\nRun with --install-deps to install required packages")
                return 1
            return 1
    
    # Test servers
    print("\n🧪 Testing MCP server...")
    success = True
    success &= test_server("fast_context.mcp_server")
    
    if not success:
        print("\n❌ Server tests failed. Please check your installation.")
        return 1
    
    # Create Claude Desktop configuration
    print("\n⚙️  Configuring Claude Desktop...")
    if create_claude_desktop_config(args.config_path):
        print("✅ Configuration created successfully")
    else:
        print("❌ Failed to create configuration")
        return 1
    
    # Print success message and usage examples
    print("\n🎉 Setup completed successfully!")
    print_usage_examples()
    
    print("\n📝 Next Steps:")
    print("1. Restart Claude Desktop to load the new configuration")
    print("2. You can now use Fast-Context tools in your conversations")
    print("3. The consolidated server includes all features in one package")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
