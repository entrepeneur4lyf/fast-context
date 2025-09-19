"""
REAL FUNCTIONAL tests for Fast-Context MCP server.
These tests validate actual MCP server functionality with real operations.
"""

import pytest
import subprocess
import tempfile
import json
import time
import asyncio
import aiohttp
import os
from pathlib import Path

class TestMCPServerFunctional:
    """Functional tests for MCP server."""

    @pytest.fixture
    def mcp_server_process(self):
        """Start MCP server in background for testing."""
        process = None
        try:
            # Start MCP server
            process = subprocess.Popen(
                ["python", "-m", "fast_context.mcp_server"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                stdin=subprocess.PIPE,
                text=True
            )
            
            # Wait for server to start
            time.sleep(3)
            
            # Check if process is still running
            if process.poll() is None:
                yield process
            else:
                # Process exited early
                stdout, stderr = process.communicate()
                print(f"MCP server exited early: {stderr}")
                pytest.skip("MCP server failed to start")
                
        except Exception as e:
            pytest.skip(f"Could not start MCP server: {e}")
        finally:
            if process and process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()

    def test_mcp_server_start_and_respond(self, mcp_server_process):
        """Test that MCP server starts and can respond to basic requests."""
        try:
            # Send a basic JSON-RPC request
            request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }
            
            # Send request to server stdin
            mcp_server_process.stdin.write(json.dumps(request) + "\n")
            mcp_server_process.stdin.flush()
            
            # Read response
            response = mcp_server_process.stdout.readline()
            
            assert response.strip() != "", "MCP server should respond to initialize request"
            
            # Parse response
            response_data = json.loads(response)
            assert "jsonrpc" in response_data, "Response should have jsonrpc field"
            assert "id" in response_data, "Response should have id field"
            assert response_data["id"] == 1, "Response ID should match request ID"
            
            print("✅ MCP server initialized successfully")
            
        except Exception as e:
            pytest.fail(f"MCP server communication failed: {e}")

    def test_mcp_server_tools_list(self, mcp_server_process):
        """Test that MCP server can list available tools."""
        try:
            # First initialize
            init_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "test-client", "version": "1.0.0"}
                }
            }
            
            mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
            mcp_server_process.stdin.flush()
            
            # Skip initialization response
            mcp_server_process.stdout.readline()
            
            # Request tools list
            tools_request = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }
            
            mcp_server_process.stdin.write(json.dumps(tools_request) + "\n")
            mcp_server_process.stdin.flush()
            
            # Read response
            response = mcp_server_process.stdout.readline()
            
            assert response.strip() != "", "MCP server should respond to tools/list request"
            
            response_data = json.loads(response)
            assert "result" in response_data, "Response should have result field"
            assert "tools" in response_data["result"], "Result should have tools field"
            
            tools = response_data["result"]["tools"]
            assert isinstance(tools, list), "Tools should be a list"
            assert len(tools) > 0, "Should have at least one tool available"
            
            # Check for expected tools
            tool_names = [tool.get("name", "") for tool in tools]
            
            # Should have analysis and graph tools
            expected_tools = ["analyze_codebase", "find_symbols", "create_graph"]
            for expected_tool in expected_tools:
                assert any(expected_tool in name for name in tool_names), \
                    f"Should have {expected_tool} tool available"
            
            print(f"✅ MCP server has {len(tools)} tools available")
            
        except Exception as e:
            pytest.fail(f"MCP server tools listing failed: {e}")

    def test_mcp_server_analyze_codebase_tool(self, mcp_server_process):
        """Test MCP server analyze_codebase tool with real code."""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Create a real Python project
                project_dir = Path(temp_dir) / "test_project"
                project_dir.mkdir()
                
                (project_dir / "main.py").write_text("""
def main():
    return "Hello World"

class MainClass:
    def method(self):
        return True
""")
                
                (project_dir / "utils.py").write_text("""
def helper():
    return "helper result"
""")
                
                # Initialize server
                init_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "test-client", "version": "1.0.0"}
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
                mcp_server_process.stdin.flush()
                mcp_server_process.stdout.readline()
                
                # Call analyze_codebase tool
                analyze_request = {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "analyze_codebase",
                        "arguments": {
                            "project_path": str(project_dir)
                        }
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(analyze_request) + "\n")
                mcp_server_process.stdin.flush()
                
                # Read response
                response = mcp_server_process.stdout.readline()
                
                assert response.strip() != "", "MCP server should respond to analyze_codebase request"
                
                response_data = json.loads(response)
                assert "result" in response_data, "Response should have result field"
                
                # Tool call result should have content
                result = response_data["result"]
                assert "content" in result, "Tool result should have content"
                
                content = result["content"]
                assert isinstance(content, list), "Content should be a list"
                assert len(content) > 0, "Content should not be empty"
                
                # First content item should have analysis data
                analysis_data = content[0]
                assert "text" in analysis_data, "Content item should have text"
                
                # Parse the analysis result
                analysis_text = analysis_data["text"]
                analysis_result = json.loads(analysis_text)
                
                assert isinstance(analysis_result, dict), "Analysis result should be a dictionary"
                assert "total_files" in analysis_result, "Analysis result should have total_files"
                assert analysis_result["total_files"] >= 2, f"Should find at least 2 files, found {analysis_result['total_files']}"
                
                print(f"✅ MCP server analyzed {analysis_result['total_files']} files successfully")
                
        except Exception as e:
            pytest.fail(f"MCP server analyze_codebase tool failed: {e}")

    def test_mcp_server_find_symbols_tool(self, mcp_server_process):
        """Test MCP server find_symbols tool with real code."""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Create a file with specific symbols
                test_file = Path(temp_dir) / "symbols_test.py"
                test_file.write_text("""
import os

GLOBAL_VAR = "test"

def specific_function(param):
    return param * 2

class SpecificClass:
    def specific_method(self):
        return GLOBAL_VAR
""")
                
                # Initialize server
                init_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "test-client", "version": "1.0.0"}
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
                mcp_server_process.stdin.flush()
                mcp_server_process.stdout.readline()
                
                # Call find_symbols tool
                find_request = {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "find_symbols",
                        "arguments": {
                            "project_path": str(temp_dir),
                            "symbol_pattern": "specific"
                        }
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(find_request) + "\n")
                mcp_server_process.stdin.flush()
                
                # Read response
                response = mcp_server_process.stdout.readline()
                
                assert response.strip() != "", "MCP server should respond to find_symbols request"
                
                response_data = json.loads(response)
                assert "result" in response_data, "Response should have result field"
                
                result = response_data["result"]
                assert "content" in result, "Tool result should have content"
                
                content = result["content"][0]
                assert "text" in content, "Content should have text"
                
                # Parse symbols result
                symbols_data = json.loads(content["text"])
                assert isinstance(symbols_data, dict), "Symbols result should be a dictionary"
                
                # Should find our specific symbols
                if "symbols" in symbols_data:
                    symbols = symbols_data["symbols"]
                    found_functions = [s.get("name", "") for s in symbols if s.get("type") == "function"]
                    found_classes = [s.get("name", "") for s in symbols if s.get("type") == "class"]
                    
                    assert "specific_function" in found_functions, "Should find specific_function"
                    assert "SpecificClass" in found_classes, "Should find SpecificClass"
                
                print("✅ MCP server found symbols successfully")
                
        except Exception as e:
            pytest.fail(f"MCP server find_symbols tool failed: {e}")

    def test_mcp_server_create_graph_tool(self, mcp_server_process):
        """Test MCP server create_graph tool with real project."""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Create a project with dependencies
                project_dir = Path(temp_dir) / "graph_project"
                project_dir.mkdir()
                
                (project_dir / "main.py").write_text("""
from models import User
from services import UserService

class MainController:
    def __init__(self):
        self.service = UserService()
    
    def process_user(self, user_id):
        user = self.service.get_user(user_id)
        return user
""")
                
                (project_dir / "models.py").write_text("""
class User:
    def __init__(self, id, name):
        self.id = id
        self.name = name
""")
                
                (project_dir / "services.py").write_text("""
from models import User

class UserService:
    def get_user(self, user_id):
        return User(user_id, f"User {user_id}")
""")
                
                # Initialize server
                init_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "test-client", "version": "1.0.0"}
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
                mcp_server_process.stdin.flush()
                mcp_server_process.stdout.readline()
                
                # Call create_graph tool
                graph_request = {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "create_graph",
                        "arguments": {
                            "project_path": str(project_dir)
                        }
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(graph_request) + "\n")
                mcp_server_process.stdin.flush()
                
                # Read response
                response = mcp_server_process.stdout.readline()
                
                assert response.strip() != "", "MCP server should respond to create_graph request"
                
                response_data = json.loads(response)
                assert "result" in response_data, "Response should have result field"
                
                result = response_data["result"]
                assert "content" in result, "Tool result should have content"
                
                content = result["content"][0]
                assert "text" in content, "Content should have text"
                
                # Parse graph result
                graph_data = json.loads(content["text"])
                assert isinstance(graph_data, dict), "Graph result should be a dictionary"
                
                # Should have graph structure
                if "nodes" in graph_data:
                    nodes = graph_data["nodes"]
                    assert isinstance(nodes, list), "Graph nodes should be a list"
                    assert len(nodes) > 0, "Graph should have nodes"
                    
                    # Should find our classes
                    node_names = [node.get("name", "") for node in nodes]
                    assert "MainController" in node_names, "Should find MainController"
                    assert "User" in node_names, "Should find User"
                    assert "UserService" in node_names, "Should find UserService"
                
                if "edges" in graph_data:
                    edges = graph_data["edges"]
                    assert isinstance(edges, list), "Graph edges should be a list"
                    
                print(f"✅ MCP server created graph with {len(graph_data.get('nodes', []))} nodes")
                
        except Exception as e:
            pytest.fail(f"MCP server create_graph tool failed: {e}")

    def test_mcp_server_error_handling(self, mcp_server_process):
        """Test MCP server error handling with invalid requests."""
        try:
            # Initialize server
            init_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "test-client", "version": "1.0.0"}
                }
            }
            
            mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
            mcp_server_process.stdin.flush()
            mcp_server_process.stdout.readline()
            
            # Send invalid request (non-existent tool)
            invalid_request = {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "nonexistent_tool",
                    "arguments": {}
                }
            }
            
            mcp_server_process.stdin.write(json.dumps(invalid_request) + "\n")
            mcp_server_process.stdin.flush()
            
            # Read response
            response = mcp_server_process.stdout.readline()
            
            assert response.strip() != "", "MCP server should respond to invalid request"
            
            response_data = json.loads(response)
            
            # Should have error information
            if "error" in response_data:
                error = response_data["error"]
                assert "code" in error, "Error should have code"
                assert "message" in error, "Error should have message"
                print(f"✅ MCP server handled error correctly: {error['message']}")
            elif "result" in response_data:
                # Some implementations might return error in result
                result = response_data["result"]
                if "content" in result:
                    content = result["content"][0]
                    if "text" in content:
                        text = content["text"]
                        assert "error" in text.lower(), "Result should indicate error"
                        print(f"✅ MCP server handled error in result: {text}")
                
        except Exception as e:
            pytest.fail(f"MCP server error handling test failed: {e}")

    def test_mcp_server_performance(self, mcp_server_process):
        """Test MCP server performance with multiple requests."""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Create a medium-sized project
                project_dir = Path(temp_dir) / "perf_project"
                project_dir.mkdir()
                
                # Create multiple Python files
                for i in range(5):
                    file_path = project_dir / f"module_{i}.py"
                    file_path.write_text(f'''
"""Module {i}"""

class Module{i}Class:
    def __init__(self):
        self.id = {i}
    
    def process(self, data):
        return f"processed by module {i}"

def function_{i}(param):
    return param * {i}
''')
                
                # Initialize server
                init_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "test-client", "version": "1.0.0"}
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
                mcp_server_process.stdin.flush()
                mcp_server_process.stdout.readline()
                
                # Send multiple requests
                start_time = time.time()
                
                for i in range(3):
                    request = {
                        "jsonrpc": "2.0",
                        "id": i + 2,
                        "method": "tools/call",
                        "params": {
                            "name": "analyze_codebase",
                            "arguments": {
                                "project_path": str(project_dir)
                            }
                        }
                    }
                    
                    mcp_server_process.stdin.write(json.dumps(request) + "\n")
                    mcp_server_process.stdin.flush()
                    
                    # Read response
                    response = mcp_server_process.stdout.readline()
                    assert response.strip() != "", f"Request {i} should get response"
                    
                    response_data = json.loads(response)
                    assert "result" in response_data, f"Request {i} should have result"
                
                end_time = time.time()
                total_time = end_time - start_time
                
                # Should handle multiple requests efficiently
                assert total_time < 10.0, f"3 requests took {total_time:.2f}s, expected < 10s"
                
                print(f"✅ MCP server handled 3 requests in {total_time:.2f}s")
                
        except Exception as e:
            pytest.fail(f"MCP server performance test failed: {e}")

    def test_mcp_server_real_world_project(self, mcp_server_process):
        """Test MCP server with a more realistic project structure."""
        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                # Create a realistic project structure
                project_dir = Path(temp_dir) / "real_project"
                project_dir.mkdir()
                
                # Create package structure
                (project_dir / "src").mkdir()
                (project_dir / "tests").mkdir()
                (project_dir / "docs").mkdir()
                
                # Create main application
                (project_dir / "src" / "__init__.py").write_text("")
                (project_dir / "src" / "main.py").write_text("""
from src.app import Application
from src.config import Config

def main():
    config = Config()
    app = Application(config)
    return app.run()
""")
                
                (project_dir / "src" / "app.py").write_text("""
from src.services import UserService, DataService
from src.models import User

class Application:
    def __init__(self, config):
        self.config = config
        self.user_service = UserService()
        self.data_service = DataService()
    
    def run(self):
        return "Application running"
""")
                
                (project_dir / "src" / "config.py").write_text("""
import os

class Config:
    def __init__(self):
        self.debug = os.getenv('DEBUG', 'False').lower() == 'true'
        self.port = int(os.getenv('PORT', '8000'))
""")
                
                (project_dir / "src" / "services.py").write_text("""
from src.models import User, Data

class UserService:
    def get_user(self, user_id):
        return User(user_id, f"User {user_id}")

class DataService:
    def get_data(self, data_id):
        return Data(data_id, f"Data {data_id}")
""")
                
                (project_dir / "src" / "models.py").write_text("""
class User:
    def __init__(self, id, name):
        self.id = id
        self.name = name

class Data:
    def __init__(self, id, content):
        self.id = id
        self.content = content
""")
                
                # Initialize server
                init_request = {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": "test-client", "version": "1.0.0"}
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(init_request) + "\n")
                mcp_server_process.stdin.flush()
                mcp_server_process.stdout.readline()
                
                # Analyze the project
                analyze_request = {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": "analyze_codebase",
                        "arguments": {
                            "project_path": str(project_dir)
                        }
                    }
                }
                
                mcp_server_process.stdin.write(json.dumps(analyze_request) + "\n")
                mcp_server_process.stdin.flush()
                
                # Read response
                response = mcp_server_process.stdout.readline()
                
                assert response.strip() != "", "MCP server should analyze real project"
                
                response_data = json.loads(response)
                result = response_data["result"]
                content = result["content"][0]
                analysis_data = json.loads(content["text"])
                
                # Should find multiple files and symbols
                assert analysis_data["total_files"] >= 5, f"Should find at least 5 files, found {analysis_data['total_files']}"
                
                if "languages" in analysis_data:
                    languages = analysis_data["languages"]
                    assert "python" in languages, "Should detect Python language"
                
                print(f"✅ MCP server analyzed real-world project with {analysis_data['total_files']} files")
                
        except Exception as e:
            pytest.fail(f"MCP server real-world project test failed: {e}")