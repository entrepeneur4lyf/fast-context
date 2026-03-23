"""
REAL PERFORMANCE benchmarks for Fast-Context SDK.
These tests validate actual performance with real codebases and operations.
"""

import pytest
import tempfile
import time
import json
import statistics
import os
from pathlib import Path
import multiprocessing

def test_performance_small_project_analysis():
    """Test performance with small project (< 10 files)."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a small project
            project_dir = Path(temp_dir) / "small_project"
            project_dir.mkdir()
            
            # Create 8 Python files with realistic content
            files_content = {
                "main.py": """
from app import Application
from config import Config

def main():
    config = Config()
    app = Application(config)
    return app.run()

if __name__ == "__main__":
    main()
""",
                "app.py": """
from services import UserService, DataService
from models import User, Data

class Application:
    def __init__(self, config):
        self.config = config
        self.user_service = UserService()
        self.data_service = DataService()
    
    def run(self):
        return "Application running successfully"
    
    def get_status(self):
        return {"status": "running", "users": len(self.user_service.get_all_users())}
""",
                "config.py": """
import os
from pathlib import Path

class Config:
    def __init__(self):
        self.debug = os.getenv('DEBUG', 'False').lower() == 'true'
        self.host = os.getenv('HOST', 'localhost')
        self.port = int(os.getenv('PORT', '8000'))
        self.database_url = os.getenv('DATABASE_URL', 'sqlite:///app.db')
    
    def get_database_config(self):
        return {
            'url': self.database_url,
            'echo': self.debug
        }
""",
                "services.py": """
from models import User, Data
from typing import List, Optional
import uuid

class UserService:
    def __init__(self):
        self.users = {}
    
    def create_user(self, name: str, email: str) -> User:
        user_id = str(uuid.uuid4())
        user = User(user_id, name, email)
        self.users[user_id] = user
        return user
    
    def get_user(self, user_id: str) -> Optional[User]:
        return self.users.get(user_id)
    
    def get_all_users(self) -> List[User]:
        return list(self.users.values())
    
    def update_user(self, user_id: str, name: str = None, email: str = None) -> Optional[User]:
        user = self.users.get(user_id)
        if user:
            if name:
                user.name = name
            if email:
                user.email = email
        return user

class DataService:
    def __init__(self):
        self.data_items = {}
    
    def create_data(self, content: str, user_id: str) -> Data:
        data_id = str(uuid.uuid4())
        data = Data(data_id, content, user_id)
        self.data_items[data_id] = data
        return data
    
    def get_data(self, data_id: str) -> Optional[Data]:
        return self.data_items.get(data_id)
    
    def get_user_data(self, user_id: str) -> List[Data]:
        return [data for data in self.data_items.values() if data.user_id == user_id]
""",
                "models.py": """
from dataclasses import dataclass
from datetime import datetime

@dataclass
class User:
    id: str
    name: str
    email: str
    created_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()

@dataclass
class Data:
    id: str
    content: str
    user_id: str
    created_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()
""",
                "utils.py": """
import hashlib
import json
from typing import Any

def hash_string(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()

def serialize_to_json(data: Any) -> str:
    return json.dumps(data, default=str)

def validate_email(email: str) -> bool:
    return '@' in email and '.' in email.split('@')[-1]

class ValidationError(Exception):
    pass

def validate_user_data(name: str, email: str) -> None:
    if not name or len(name.strip()) == 0:
        raise ValidationError("Name cannot be empty")
    if not validate_email(email):
        raise ValidationError("Invalid email format")
""",
                "database.py": """
import sqlite3
from contextlib import contextmanager
from typing import List, Dict, Any

class DatabaseManager:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        with self.get_connection() as conn:
            conn.execute('''
                CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    email TEXT UNIQUE NOT NULL,
                    created_at TIMESTAMP
                )
            ''')
            conn.execute('''
                CREATE TABLE IF NOT EXISTS data_items (
                    id TEXT PRIMARY KEY,
                    content TEXT NOT NULL,
                    user_id TEXT,
                    created_at TIMESTAMP,
                    FOREIGN KEY (user_id) REFERENCES users (id)
                )
            ''')
            conn.commit()
    
    @contextmanager
    def get_connection(self):
        conn = sqlite3.connect(self.db_path)
        try:
            yield conn
        finally:
            conn.close()
    
    def execute_query(self, query: str, params: tuple = ()) -> List[Dict[str, Any]]:
        with self.get_connection() as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(query, params)
            return [dict(row) for row in cursor.fetchall()]
""",
                "api.py": """
from flask import Flask, request, jsonify
from services import UserService, DataService
from models import User, Data
from utils import validate_user_data, ValidationError

app = Flask(__name__)

user_service = UserService()
data_service = DataService()

@app.route('/users', methods=['POST'])
def create_user():
    try:
        data = request.get_json()
        validate_user_data(data['name'], data['email'])
        user = user_service.create_user(data['name'], data['email'])
        return jsonify({'id': user.id, 'name': user.name, 'email': user.email}), 201
    except ValidationError as e:
        return jsonify({'error': str(e)}), 400

@app.route('/users/<user_id>', methods=['GET'])
def get_user(user_id):
    user = user_service.get_user(user_id)
    if user:
        return jsonify({'id': user.id, 'name': user.name, 'email': user.email})
    return jsonify({'error': 'User not found'}), 404

@app.route('/data', methods=['POST'])
def create_data():
    data = request.get_json()
    data_item = data_service.create_data(data['content'], data['user_id'])
    return jsonify({'id': data_item.id, 'content': data_item.content}), 201

if __name__ == '__main__':
    app.run(debug=True)
"""
            }
            
            # Write all files
            for filename, content in files_content.items():
                (project_dir / filename).write_text(content)
            
            # Measure analysis performance
            start_time = time.time()
            analyzer = fast_context.FastContextAnalyzer()
            result = analyzer.analyze(str(project_dir))
            end_time = time.time()
            
            analysis_time = end_time - start_time
            
            # Validate performance and results
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] == 8, f"Expected 8 files, found {result['total_files']}"
            
            # Performance benchmark: small project should be fast
            assert analysis_time < 2.0, f"Small project analysis took {analysis_time:.3f}s, expected < 2s"
            
            # Validate analysis quality
            if "symbols" in result:
                symbols = result["symbols"]
                total_symbols = sum(len(symbols.get(lang, [])) for lang in symbols)
                assert total_symbols > 20, f"Expected > 20 symbols, found {total_symbols}"
            
            print(f"✅ Small project analysis: {result['total_files']} files in {analysis_time:.3f}s")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Small project performance test failed: {e}")

def test_performance_medium_project_analysis():
    """Test performance with medium project (50-100 files)."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a medium project
            project_dir = Path(temp_dir) / "medium_project"
            project_dir.mkdir()
            
            # Create project structure
            (project_dir / "src").mkdir()
            (project_dir / "tests").mkdir()
            (project_dir / "docs").mkdir()
            
            # Generate 60 Python files with realistic content
            modules = ['auth', 'database', 'api', 'models', 'services', 'utils', 'config', 'cli']
            file_count = 0
            
            for module in modules:
                module_dir = project_dir / "src" / module
                module_dir.mkdir()
                
                # Create __init__.py
                (module_dir / "__init__.py").write_text(f'"""{module} module"""')
                file_count += 1
                
                # Create main module file
                (module_dir / f"{module}.py").write_text(f'''
"""{module} implementation"""

import logging
from typing import List, Dict, Any, Optional
from dataclasses import dataclass

logger = logging.getLogger(__name__)

@dataclass
class {module.capitalize()}Config:
    enabled: bool = True
    debug: bool = False
    max_items: int = 1000

class {module.capitalize()}Manager:
    def __init__(self, config: {module.capitalize()}Config = None):
        self.config = config or {module.capitalize()}Config()
        self.items = []
        logger.info(f"Initialized {module} manager")
    
    def add_item(self, item: Any) -> bool:
        if len(self.items) < self.config.max_items:
            self.items.append(item)
            return True
        return False
    
    def get_items(self) -> List[Any]:
        return self.items.copy()
    
    def find_item(self, predicate) -> Optional[Any]:
        for item in self.items:
            if predicate(item):
                return item
        return None
    
    def clear_items(self) -> None:
        self.items.clear()
        logger.info(f"Cleared {module} items")

def create_{module}_item(name: str, value: Any) -> Dict[str, Any]:
    """Create a {module} item"""
    return {{
        "name": name,
        "value": value,
        "created_at": "2024-01-01T00:00:00Z",
        "module": "{module}"
    }}

def validate_{module}_item(item: Dict[str, Any]) -> bool:
    """Validate a {module} item"""
    required_fields = ["name", "value", "created_at"]
    return all(field in item for field in required_fields)

class {module.capitalize()}Error(Exception):
    """{module} specific error"""
    pass

def process_{module}_data(data: List[Any]) -> List[Dict[str, Any]]:
    """Process {module} data"""
    results = []
    for item in data:
        try:
            processed = create_{module}_item(str(item), item)
            if validate_{module}_item(processed):
                results.append(processed)
        except Exception as e:
            logger.error(f"Error processing {{item}} in {module}: {{e}}")
    return results
''')
                file_count += 1
                
                # Create additional files for complex modules
                if module in ['api', 'services', 'database']:
                    for i in range(3):
                        extra_file = module_dir / f"{module}_extra_{i}.py"
                        extra_file.write_text(f'''
"""{module} extra functionality {i}"""

from .{module} import {module.capitalize()}Manager, {module.capitalize()}Config

class {module.capitalize()}Extra{i}({module.capitalize()}Manager):
    def __init__(self, config: {module.capitalize()}Config = None):
        super().__init__(config)
        self.extra_data = {{}}
    
    def add_extra_data(self, key: str, value: Any) -> None:
        self.extra_data[key] = value
    
    def get_extra_data(self, key: str) -> Any:
        return self.extra_data.get(key)
    
    def process_with_extra(self, data: List[Any]) -> List[Dict[str, Any]]:
        """Process data with extra functionality"""
        from .{module} import process_{module}_data
        results = process_{module}_data(data)
        for result in results:
            result["extra_processed"] = True
            result["processor"] = "extra{i}"
        return results
''')
                        file_count += 1
            
            # Create some test files
            for i in range(10):
                test_file = project_dir / "tests" / f"test_{i}.py"
                test_file.write_text(f'''
"""Test file {i}"""

import pytest
from src.auth import AuthManager
from src.services import ServiceManager

def test_function_{i}():
    """Test function {i}"""
    auth = AuthManager()
    service = ServiceManager()
    assert auth is not None
    assert service is not None

class TestClass{i}:
    def test_method(self):
        assert True
''')
                file_count += 1
            
            # Create documentation files
            for i in range(5):
                doc_file = project_dir / "docs" / f"doc_{i}.md"
                doc_file.write_text(f'''
# Documentation {i}

This is documentation file {i}.

## Features

- Feature 1
- Feature 2
- Feature 3

## Usage

```python
from src.{modules[i % len(modules)]} import {modules[i % len(modules)].capitalize()}Manager

manager = {modules[i % len(modules)].capitalize()}Manager()
result = manager.process_data([])
```
''')
                file_count += 1
            
            # Measure analysis performance
            start_time = time.time()
            analyzer = fast_context.FastContextAnalyzer()
            result = analyzer.analyze(str(project_dir))
            end_time = time.time()
            
            analysis_time = end_time - start_time
            
            # Validate performance and results
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] >= file_count - 5, f"Expected at least {file_count - 5} files, found {result['total_files']}"
            
            # Performance benchmark: medium project should be reasonable
            assert analysis_time < 10.0, f"Medium project analysis took {analysis_time:.3f}s, expected < 10s"
            
            # Validate analysis quality
            if "symbols" in result:
                symbols = result["symbols"]
                total_symbols = sum(len(symbols.get(lang, [])) for lang in symbols)
                assert total_symbols > 100, f"Expected > 100 symbols, found {total_symbols}"
            
            print(f"✅ Medium project analysis: {result['total_files']} files in {analysis_time:.3f}s")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Medium project performance test failed: {e}")

def test_performance_memory_usage():
    """Test memory usage with large analysis."""
    try:
        import fast_context
        import psutil
        import os
        
        process = psutil.Process(os.getpid())
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a project with memory-intensive content
            project_dir = Path(temp_dir) / "memory_test"
            project_dir.mkdir()
            
            # Create files with large content
            for i in range(20):
                file_path = project_dir / f"large_file_{i}.py"
                large_content = f'''
"""Large file {i}"""

# Generate many functions and classes
def function_{i}_1():
    return "result"

def function_{i}_2():
    return "result"

def function_{i}_3():
    return "result"

# Generate large class
class LargeClass{i}:
    def __init__(self):
        self.data = [j for j in range(1000)]
    
    def method_1(self):
        return self.data
    
    def method_2(self):
        return len(self.data)
    
    def method_3(self):
        return sum(self.data)
    
    def process_data(self):
        return [x * 2 for x in self.data]

# Generate many similar functions
for idx in range(50):
    exec(f'''
def generated_function_{i}_{idx}():
    return f"generated result {{idx}}"
''')

class AnotherClass{i}:
    def __init__(self):
        self.items = []
        for j in range(500):
            self.items.append(f"item_{{j}}")
    
    def get_items(self):
        return self.items.copy()
    
    def find_item(self, pattern):
        return [item for item in self.items if pattern in item]
'''
                file_path.write_text(large_content)
            
            # Measure memory before analysis
            memory_before = process.memory_info().rss / 1024 / 1024  # MB
            
            # Perform analysis
            start_time = time.time()
            analyzer = fast_context.FastContextAnalyzer()
            result = analyzer.analyze(str(project_dir))
            end_time = time.time()
            
            # Measure memory after analysis
            memory_after = process.memory_info().rss / 1024 / 1024  # MB
            
            analysis_time = end_time - start_time
            memory_increase = memory_after - memory_before
            
            # Validate performance and memory usage
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] == 20, f"Expected 20 files, found {result['total_files']}"
            
            # Performance benchmarks
            assert analysis_time < 15.0, f"Memory test took {analysis_time:.3f}s, expected < 15s"
            assert memory_increase < 100, f"Memory increase {memory_increase:.1f}MB, expected < 100MB"
            
            print(f"✅ Memory usage test: {result['total_files']} files in {analysis_time:.3f}s, +{memory_increase:.1f}MB")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Memory usage test failed: {e}")

def test_performance_concurrent_analysis():
    """Test performance with concurrent analysis operations."""
    try:
        import fast_context
        import threading
        import queue
        from concurrent.futures import ThreadPoolExecutor
        
        def analyze_project(project_path, result_queue):
            """Analyze a project and put result in queue"""
            try:
                analyzer = fast_context.FastContextAnalyzer()
                result = analyzer.analyze(project_path)
                result_queue.put(("success", result))
            except Exception as e:
                result_queue.put(("error", str(e)))
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create multiple similar projects
            projects = []
            for project_id in range(3):
                project_dir = Path(temp_dir) / f"project_{project_id}"
                project_dir.mkdir()
                
                # Create similar content in each project
                for i in range(10):
                    file_path = project_dir / f"file_{i}.py"
                    file_path.write_text(f'''
"""Project {project_id} file {i}"""

def function_{project_id}_{i}():
    return f"Result from project {project_id}, file {i}"

class Class{project_id}_{i}:
    def __init__(self):
        self.project_id = {project_id}
        self.file_id = {i}
    
    def get_id(self):
        return f"{{self.project_id}}_{{self.file_id}}"
    
    def process(self, data):
        return [f"processed_{{item}}" for item in data]
''')
                
                projects.append(str(project_dir))
            
            # Run concurrent analysis
            start_time = time.time()
            result_queue = queue.Queue()
            
            with ThreadPoolExecutor(max_workers=3) as executor:
                futures = [
                    executor.submit(analyze_project, project_path, result_queue)
                    for project_path in projects
                ]
                
                # Wait for all results
                results = []
                for _ in range(len(projects)):
                    status, result = result_queue.get()
                    if status == "success":
                        results.append(result)
                    else:
                        print(f"Error in concurrent analysis: {result}")
                
                # Wait for futures to complete
                for future in futures:
                    future.result()
            
            end_time = time.time()
            total_time = end_time - start_time
            
            # Validate concurrent performance
            assert len(results) >= 2, f"Expected at least 2 successful analyses, got {len(results)}"
            
            # Each result should be valid
            for i, result in enumerate(results):
                assert isinstance(result, dict)
                assert "total_files" in result
                assert result["total_files"] == 10, f"Project {i} should have 10 files, found {result['total_files']}"
            
            # Concurrent should be faster than sequential
            assert total_time < 20.0, f"Concurrent analysis took {total_time:.3f}s, expected < 20s"
            
            print(f"✅ Concurrent analysis: {len(results)} projects in {total_time:.3f}s")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Concurrent analysis test failed: {e}")

def test_performance_large_file_handling():
    """Test performance with large individual files."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a single large Python file
            large_file = Path(temp_dir / "large_file.py")
            
            # Generate a large file with many functions and classes
            lines = []
            lines.append('"""Large Python file for performance testing"""')
            lines.append('import os')
            lines.append('import sys')
            lines.append('from typing import List, Dict, Any')
            lines.append('')
            
            # Generate many functions
            for i in range(200):
                lines.append(f'def function_{i:03d}(param1, param2=None):')
                lines.append(f'    """Function {i:03d}"""')
                lines.append(f'    if param2 is None:')
                lines.append(f'        param2 = "default"')
                lines.append(f'    return f"function_{i:03d}_{{param1}}_{{param2}}"')
                lines.append('')
            
            # Generate many classes
            for i in range(50):
                lines.append(f'class LargeClass{i:03d}:')
                lines.append(f'    """Large class {i:03d}"""')
                lines.append(f'    def __init__(self):')
                lines.append(f'        self.data = list(range(100))')
                lines.append(f'        self.name = f"class_{i:03d}"')
                lines.append(f'    ')
                lines.append(f'    def method_1(self):')
                lines.append(f'        return len(self.data)')
                lines.append(f'    ')
                lines.append(f'    def method_2(self, value):')
                lines.append(f'        return [x + value for x in self.data]')
                lines.append(f'    ')
                lines.append(f'    def method_3(self, predicate):')
                lines.append(f'        return [x for x in self.data if predicate(x)]')
                lines.append('')
            
            large_file.write_text('\n'.join(lines))
            
            # Measure analysis performance
            start_time = time.time()
            analyzer = fast_context.FastContextAnalyzer()
            result = analyzer.analyze(str(temp_dir))
            end_time = time.time()
            
            analysis_time = end_time - start_time
            
            # Validate performance and results
            assert isinstance(result, dict)
            assert "total_files" in result
            assert result["total_files"] == 1, f"Expected 1 file, found {result['total_files']}"
            
            # Validate symbol extraction
            if "symbols" in result:
                symbols = result["symbols"]
                python_symbols = symbols.get("python", [])
                
                # Should find many functions and classes
                functions = [s for s in python_symbols if s.get("type") == "function"]
                classes = [s for s in python_symbols if s.get("type") == "class"]
                
                assert len(functions) >= 200, f"Expected >= 200 functions, found {len(functions)}"
                assert len(classes) >= 50, f"Expected >= 50 classes, found {len(classes)}"
            
            # Performance benchmark: large file should be handled efficiently
            assert analysis_time < 5.0, f"Large file analysis took {analysis_time:.3f}s, expected < 5s"
            
            print(f"✅ Large file analysis: {result['total_files']} files in {analysis_time:.3f}s")
            print(f"   Found {len(functions)} functions and {len(classes)} classes")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Large file handling test failed: {e}")

def test_performance_cache_efficiency():
    """Test cache efficiency with repeated analysis."""
    try:
        import fast_context
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a project
            project_dir = Path(temp_dir) / "cache_test"
            project_dir.mkdir()
            
            # Create some Python files
            for i in range(5):
                file_path = project_dir / f"file_{i}.py"
                file_path.write_text(f'''
"""File {i} for cache testing"""

def function_{i}():
    return f"result from file {i}"

class Class{i}:
    def method(self):
        return function_{i}()
''')
            
            # First analysis (should be slower)
            start_time = time.time()
            analyzer = fast_context.FastContextAnalyzer()
            result1 = analyzer.analyze(str(project_dir))
            first_time = time.time() - start_time
            
            # Second analysis (should be faster due to caching)
            start_time = time.time()
            result2 = analyzer.analyze(str(project_dir))
            second_time = time.time() - start_time
            
            # Third analysis (should be even faster)
            start_time = time.time()
            result3 = analyzer.analyze(str(project_dir))
            third_time = time.time() - start_time
            
            # Validate results are consistent
            assert result1 == result2 == result3, "Analysis results should be consistent"
            
            # Validate cache efficiency
            assert second_time < first_time, f"Second analysis ({second_time:.3f}s) should be faster than first ({first_time:.3f}s)"
            assert third_time <= second_time, f"Third analysis ({third_time:.3f}s) should be as fast as second ({second_time:.3f}s)"
            
            # Cache should provide significant improvement
            cache_improvement = (first_time - third_time) / first_time * 100
            assert cache_improvement > 20, f"Cache should provide >20% improvement, got {cache_improvement:.1f}%"
            
            print(f"✅ Cache efficiency:")
            print(f"   First analysis: {first_time:.3f}s")
            print(f"   Second analysis: {second_time:.3f}s")
            print(f"   Third analysis: {third_time:.3f}s")
            print(f"   Cache improvement: {cache_improvement:.1f}%")
            
    except ImportError as e:
        pytest.skip(f"fast_context core not available: {e}")
    except Exception as e:
        pytest.fail(f"Cache efficiency test failed: {e}")
