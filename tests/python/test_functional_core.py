"""
REAL FUNCTIONAL tests for Fast-Context SDK core functionality.
These tests validate actual Rust-Python integration and real functionality.
"""

import pytest
import tempfile
import json
import subprocess
import time
import os
from pathlib import Path
import asyncio

def test_fast_context_core_import_and_version():
    """Test that we can import the real fast_context core and get version."""
    try:
        import fast_context
        assert hasattr(fast_context, 'get_version')
        
        version = fast_context.get_version()
        assert isinstance(version, str)
        assert len(version) > 0
        print(f"✅ Fast-Context version: {version}")
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Failed to get version: {e}")

def test_fast_context_analyzer_creation():
    """Test creating a real FastContextAnalyzer instance."""
    try:
        import fast_context
        
        # Test analyzer creation with proper config
        with tempfile.TemporaryDirectory() as temp_dir:
            config = fast_context.AnalyzerConfig(temp_dir)
            analyzer = fast_context.FastContextAnalyzer(config)
        assert analyzer is not None
        print("✅ FastContextAnalyzer created successfully")
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Failed to create analyzer: {e}")

def test_real_codebase_analysis():
    """Test analyzing a real Python codebase."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a real Python project
            project_dir = Path(temp_dir) / "test_project"
            project_dir.mkdir()
            
            # Create real Python files with actual code
            main_content = '''def main():
    print("Hello World")
    
if __name__ == "__main__":
    main()
'''
            
            utils_content = '''def helper_function():
    return "helper result"

class UtilityClass:
    def method(self):
        return True
'''
            
            config_content = '''CONFIG_VALUE = "test_value"

class Config:
    def __init__(self):
        self.setting = CONFIG_VALUE
'''
            
            (project_dir / "main.py").write_text(main_content)
            (project_dir / "utils.py").write_text(utils_content)
            (project_dir / "config.py").write_text(config_content)
            
            # Create analyzer and analyze
            config = fast_context.AnalyzerConfig(str(project_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            result = analyzer.analyze(str(project_dir))
            
            # Validate real analysis results
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] >= 3  # Should find at least our 3 files
            
            if "symbols" in result:
                symbols = result["symbols"]
                assert isinstance(symbols, dict)
                # Should find some real functions and classes
                total_symbols = sum(len(symbols.get(lang, [])) for lang in symbols)
                assert total_symbols > 0
                
            print(f"✅ Analyzed {result['total_files']} files successfully")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real codebase analysis failed: {e}")

def test_real_symbol_extraction():
    """Test extracting symbols from real code."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a test file with various symbols
            test_file = Path(temp_dir) / "test_symbols.py"
            test_content = '''# This is a real Python file with multiple symbol types
import os
import sys

GLOBAL_VAR = "test"

def function_one(param1, param2):
    return param1 + param2

def function_two():
    return GLOBAL_VAR

class TestClass:
    def __init__(self, value):
        self.value = value
    
    def method_one(self):
        return self.value

class AnotherClass:
    def another_method(self):
        pass
'''
            test_file.write_text(test_content)
            
            config = fast_context.AnalyzerConfig(str(temp_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            symbols = analyzer.extract_symbols(str(test_file))
            
            # Validate we found real symbols
            assert isinstance(symbols, dict)
            assert "python" in symbols
            
            python_symbols = symbols["python"]
            assert len(python_symbols) > 0
            
            # Check for specific symbol types
            function_names = [s.get('name', '') for s in python_symbols if s.get('type') == 'function']
            class_names = [s.get('name', '') for s in python_symbols if s.get('type') == 'class']
            
            # Should find our real functions
            assert "function_one" in function_names
            assert "function_two" in function_names
            
            # Should find our real classes
            assert "TestClass" in class_names
            assert "AnotherClass" in class_names
            
            print(f"✅ Extracted {len(python_symbols)} real symbols successfully")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real symbol extraction failed: {e}")

def test_real_dependency_analysis():
    """Test analyzing real dependencies between code elements."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create files with real dependencies
            main_content = '''from utils import helper_function, UtilityClass
from config import Config

def main():
    config = Config()
    util = UtilityClass()
    result = helper_function()
    return config.setting + result
'''
            
            utils_content = '''def helper_function():
    return "helper"

class UtilityClass:
    def method(self):
        return True
'''
            
            config_content = '''class Config:
    def __init__(self):
        self.setting = "config_value"
'''
            
            (Path(temp_dir) / "main.py").write_text(main_content)
            (Path(temp_dir) / "utils.py").write_text(utils_content)
            (Path(temp_dir) / "config.py").write_text(config_content)
            
            config = fast_context.AnalyzerConfig(str(temp_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            dependencies = analyzer.analyze_dependencies(str(temp_dir))
            
            # Validate real dependency analysis
            assert isinstance(dependencies, dict)
            
            if "imports" in dependencies:
                imports = dependencies["imports"]
                assert isinstance(imports, dict)
                
                # Should find real import relationships
                main_imports = imports.get("main.py", [])
                assert any("utils" in imp for imp in main_imports)
                assert any("config" in imp for imp in main_imports)
                
            print(f"✅ Analyzed real dependencies successfully")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real dependency analysis failed: {e}")

def test_real_graph_operations():
    """Test real graph operations on code structure."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a project with complex relationships
            main_content = '''from models import User, Post
from services import UserService, PostService

class ApplicationController:
    def __init__(self):
        self.user_service = UserService()
        self.post_service = PostService()
    
    def handle_request(self, user_id, post_id):
        user = self.user_service.get_user(user_id)
        post = self.post_service.get_post(post_id)
        return {"user": user, "post": post}
'''
            
            models_content = '''class User:
    def __init__(self, id, name):
        self.id = id
        self.name = name
        self.posts = []

class Post:
    def __init__(self, id, title, user_id):
        self.id = id
        self.title = title
        self.user_id = user_id
'''
            
            services_content = '''from models import User, Post

class UserService:
    def get_user(self, user_id):
        return User(user_id, f"User {user_id}")

class PostService:
    def get_post(self, post_id):
        return Post(post_id, f"Post {post_id}", 1)
'''
            
            (Path(temp_dir) / "app.py").write_text(main_content)
            (Path(temp_dir) / "models.py").write_text(models_content)
            (Path(temp_dir) / "services.py").write_text(services_content)
            
            config = fast_context.AnalyzerConfig(str(temp_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            graph = analyzer.create_dependency_graph(str(temp_dir))
            
            # Validate real graph creation
            assert isinstance(graph, dict)
            
            if "nodes" in graph:
                nodes = graph["nodes"]
                assert isinstance(nodes, list)
                assert len(nodes) > 0
                
                # Should find our real classes
                node_names = [node.get('name', '') for node in nodes]
                assert "ApplicationController" in node_names
                assert "User" in node_names
                assert "Post" in node_names
                
            print(f"✅ Created real dependency graph with {len(graph.get('nodes', []))} nodes")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real graph operations failed: {e}")

def test_real_multi_language_analysis():
    """Test analyzing a project with multiple programming languages."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a multi-language project
            project_dir = Path(temp_dir) / "multi_lang_project"
            project_dir.mkdir()
            
            # Python file
            python_content = '''import subprocess
import json

def run_command():
    result = subprocess.run(['node', 'script.js'], capture_output=True)
    return json.loads(result.stdout)
'''
            
            # JavaScript file
            js_content = '''const fs = require('fs');

function processData() {
    const data = fs.readFileSync('data.json', 'utf8');
    return JSON.parse(data);
}

module.exports = { processData };
'''
            
            # JSON file
            json_content = '''{
    "name": "Multi-language Project",
    "version": "1.0.0"
}'''
            
            (project_dir / "main.py").write_text(python_content)
            (project_dir / "script.js").write_text(js_content)
            (project_dir / "data.json").write_text(json_content)
            
            config = fast_context.AnalyzerConfig(str(temp_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            result = analyzer.analyze(str(project_dir))
            
            # Validate multi-language analysis
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] >= 3
            
            if "languages" in result:
                languages = result["languages"]
                assert isinstance(languages, dict)
                # Should detect multiple languages
                assert "python" in languages or "javascript" in languages
                
            print(f"✅ Multi-language analysis successful")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real multi-language analysis failed: {e}")

def test_real_configuration_integration():
    """Test real configuration integration."""
    try:
        import fast_context
        from fast_context.config import ConfigManager, FastContextConfig
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a real project
            project_dir = Path(temp_dir) / "config_test_project"
            project_dir.mkdir()
            
            # Create configuration file
            config_file = project_dir / "fast_context.toml"
            config_content = '''[analysis]
max_files = 1000
timeout_seconds = 30
enable_caching = true

[graph]
enabled = true
algorithm = "dependency"
max_depth = 10

[logging]
level = "INFO"
format = "structured"
'''
            config_file.write_text(config_content)
            
            # Create some Python files
            (project_dir / "main.py").write_text('''def main():
    return "Hello World"''')
            
            # Test configuration loading and usage
            config_manager = ConfigManager()
            config = config_manager.load_config(str(config_file))
            
            assert isinstance(config, FastContextConfig)
            assert config.analysis.max_files == 1000
            assert config.graph.enabled == True
            
            # Test analyzer with configuration
            analyzer_config = fast_context.AnalyzerConfig(str(project_dir))
            analyzer = fast_context.FastContextAnalyzer(analyzer_config)
            result = analyzer.analyze(str(project_dir))
            
            assert isinstance(result, dict)
            assert "total_files" in result
            
            print(f"✅ Configuration integration successful")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real configuration integration failed: {e}")

def test_cli_version_functional():
    """Test CLI version command actually works."""
    try:
        result = subprocess.run(
            ["python", "-m", "fast_context.cli", "version"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Command should succeed
        assert result.returncode == 0, f"CLI version command failed: {result.stderr}"
        
        # Should output version information
        output = result.stdout.strip()
        assert len(output) > 0, "CLI version command produced no output"
        assert "Fast-Context" in output, f"Expected 'Fast-Context' in output: {output}"
        
        print(f"✅ CLI version command: {output}")
        
    except subprocess.TimeoutExpired:
        pytest.fail("CLI version command timed out")
    except FileNotFoundError:
        pytest.skip("CLI module not found")
    except Exception as e:
        pytest.fail(f"CLI version test failed: {e}")

def test_cli_help_functional():
    """Test CLI help command works."""
    try:
        result = subprocess.run(
            ["python", "-m", "fast_context.cli", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode == 0, f"CLI help command failed: {result.stderr}"
        
        # Should show help information
        output = result.stdout.strip()
        assert len(output) > 0, "CLI help command produced no output"
        assert "Usage" in output, f"Expected 'Usage' in help output: {output}"
        
        print("✅ CLI help command works")
        
    except subprocess.TimeoutExpired:
        pytest.fail("CLI help command timed out")
    except FileNotFoundError:
        pytest.skip("CLI module not found")
    except Exception as e:
        pytest.fail(f"CLI help test failed: {e}")

def test_cli_config_init_functional():
    """Test CLI config init creates real configuration."""
    try:
        with tempfile.TemporaryDirectory() as temp_dir:
            # Change to temp directory
            original_cwd = os.getcwd()
            os.chdir(temp_dir)
            
            try:
                result = subprocess.run(
                    ["python", "-m", "fast_context.cli", "config", "init"],
                    capture_output=True,
                    text=True,
                    timeout=30
                )
                
                assert result.returncode == 0, f"CLI config init failed: {result.stderr}"
                
                # Check if config file was created
                config_files = ["fast_context.toml", "fast_context.yaml", "fast_context.json"]
                created_config = None
                
                for config_file in config_files:
                    if Path(config_file).exists():
                        created_config = config_file
                        break
                
                assert created_config is not None, "No configuration file was created"
                
                print(f"✅ CLI config init created {created_config}")
                
            finally:
                os.chdir(original_cwd)
                
    except subprocess.TimeoutExpired:
        pytest.fail("CLI config init timed out")
    except FileNotFoundError:
        pytest.skip("CLI module not found")
    except Exception as e:
        pytest.fail(f"CLI config init test failed: {e}")

def test_real_performance_with_project():
    """Test performance with a real project."""
    try:
        import fast_context
        import time
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a medium-sized project structure
            project_dir = Path(temp_dir) / "medium_project"
            project_dir.mkdir()
            
            # Create multiple Python modules
            modules = ['auth', 'database', 'api', 'models', 'services', 'utils']
            
            for module in modules:
                module_file = project_dir / f"{module}.py"
                module_content = f'''"""{module} module"""

class {module.capitalize()}Manager:
    def __init__(self):
        self.name = "{module}"
    
    def process(self, data):
        return f"processed by {{self.name}}"

def get_{module}_config():
    return {{"enabled": True, "version": "1.0.0"}}
'''
                module_file.write_text(module_content)
            
            # Create a main application file
            main_content = '''"""Main application"""

from auth import AuthManager
from database import DatabaseManager  
from api import APIManager
from models import ModelsManager
from services import ServicesManager
from utils import UtilsManager

class Application:
    def __init__(self):
        self.auth = AuthManager()
        self.db = DatabaseManager()
        self.api = APIManager()
        self.models = ModelsManager()
        self.services = ServicesManager()
        self.utils = UtilsManager()
    
    def run(self):
        return "Application running"

if __name__ == "__main__":
    app = Application()
    app.run()
'''
            (project_dir / "main.py").write_text(main_content)
            
            # Measure analysis time
            start_time = time.time()
            config = fast_context.AnalyzerConfig(str(temp_dir))
            analyzer = fast_context.FastContextAnalyzer(config)
            result = analyzer.analyze(str(project_dir))
            end_time = time.time()
            
            analysis_time = end_time - start_time
            
            # Validate results and performance
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] >= len(modules) + 1  # All modules + main.py
            
            # Performance should be reasonable (less than 5 seconds for this size)
            assert analysis_time < 5.0, f"Analysis took {analysis_time:.2f}s, expected < 5s"
            
            print(f"✅ Analyzed {result['total_files']} files in {analysis_time:.2f}s")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Real performance test failed: {e}")