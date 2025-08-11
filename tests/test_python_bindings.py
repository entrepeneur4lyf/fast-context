"""
Tests for fast-context Python bindings.
"""

import pytest
import tempfile
import os
from pathlib import Path

try:
    from fast_context import (
        FastContextAnalyzer,
        AnalyzerConfig,
        get_supported_languages,
        detect_language,
        get_version,
    )
except ImportError:
    pytest.skip("fast_context not available", allow_module_level=True)


class TestUtilityFunctions:
    """Test utility functions."""
    
    def test_get_version(self):
        """Test version retrieval."""
        version = get_version()
        assert isinstance(version, str)
        assert len(version) > 0
    
    def test_get_supported_languages(self):
        """Test supported languages retrieval."""
        languages = get_supported_languages()
        assert isinstance(languages, list)
        assert len(languages) > 0
        assert "rust" in [lang.lower() for lang in languages]
    
    def test_detect_language(self):
        """Test language detection."""
        test_cases = [
            ("main.rs", "Rust"),
            ("app.py", "Python"),
            ("script.js", "JavaScript"),
            ("style.css", "CSS"),
            ("unknown.xyz", None),
        ]
        
        for filename, expected in test_cases:
            result = detect_language(filename)
            if expected is None:
                assert result is None
            else:
                assert result is not None
                assert expected.lower() in result.lower()


class TestAnalyzerConfig:
    """Test AnalyzerConfig class."""
    
    def test_config_creation(self):
        """Test config creation with defaults."""
        config = AnalyzerConfig("/tmp/test")
        assert config.project_root == "/tmp/test"
        assert isinstance(config.languages, list)
        assert len(config.languages) > 0
        assert config.enable_caching is True
        assert config.enable_watching is False
    
    def test_config_with_custom_values(self):
        """Test config creation with custom values."""
        config = AnalyzerConfig(
            project_root="/custom/path",
            languages=["rust", "python"],
            ignore_patterns=["*.tmp"],
            enable_caching=False,
            enable_watching=True,
            max_files=500,
            parallel_processing=False,
        )
        
        assert config.project_root == "/custom/path"
        assert config.languages == ["rust", "python"]
        assert config.ignore_patterns == ["*.tmp"]
        assert config.enable_caching is False
        assert config.enable_watching is True
        assert config.max_files == 500
        assert config.parallel_processing is False


class TestFastContextAnalyzer:
    """Test FastContextAnalyzer class."""
    
    @pytest.fixture
    def temp_project(self):
        """Create a temporary project for testing."""
        temp_dir = tempfile.mkdtemp(prefix="fast_context_test_")
        
        # Create a simple Rust file
        rust_file = Path(temp_dir) / "main.rs"
        rust_file.write_text("""
fn main() {
    println!("Hello, world!");
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Point {
    x: f64,
    y: f64,
}
""")
        
        # Create a simple Python file
        python_file = Path(temp_dir) / "test.py"
        python_file.write_text("""
def greet(name):
    return f"Hello, {name}!"

class Calculator:
    def __init__(self):
        self.value = 0
    
    def add(self, n):
        self.value += n
        return self
""")
        
        yield temp_dir
        
        # Cleanup
        import shutil
        shutil.rmtree(temp_dir, ignore_errors=True)
    
    def test_analyzer_creation(self, temp_project):
        """Test analyzer creation."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        assert analyzer is not None
    
    def test_analyzer_creation_invalid_path(self):
        """Test analyzer creation with invalid path."""
        config = AnalyzerConfig("/nonexistent/path")
        # Should still create analyzer, but analysis might fail
        analyzer = FastContextAnalyzer(config)
        assert analyzer is not None
    
    def test_analysis(self, temp_project):
        """Test basic analysis functionality."""
        config = AnalyzerConfig(
            project_root=temp_project,
            languages=["rust", "python"],
            enable_caching=False,
        )
        analyzer = FastContextAnalyzer(config)
        
        result = analyzer.analyze()
        
        assert result.file_count >= 2  # At least our test files
        assert result.symbol_count > 0
        assert result.duration_ms >= 0
        assert isinstance(result.languages, list)
        assert len(result.languages) > 0
    
    def test_find_symbols_by_kind(self, temp_project):
        """Test finding symbols by kind."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Analyze first
        analyzer.analyze()
        
        # Find functions
        functions = analyzer.find_symbols_by_kind("function")
        assert isinstance(functions, list)
        # Should find at least 'main', 'add', 'greet'
        assert len(functions) >= 3
    
    def test_find_symbols_in_file(self, temp_project):
        """Test finding symbols in specific file."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Analyze first
        analyzer.analyze()
        
        # Find symbols in main.rs
        symbols = analyzer.find_symbols_in_file("main.rs")
        assert isinstance(symbols, list)
        assert len(symbols) > 0
    
    def test_find_dependencies(self, temp_project):
        """Test finding dependencies."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Analyze first
        analyzer.analyze()
        
        # Find dependencies (might be empty for simple test)
        deps = analyzer.find_dependencies("Point")
        assert isinstance(deps, list)
    
    def test_find_complex_symbols(self, temp_project):
        """Test finding complex symbols."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Analyze first
        analyzer.analyze()
        
        # Find complex symbols (threshold 10 should return few/none)
        complex_symbols = analyzer.find_complex_symbols(10)
        assert isinstance(complex_symbols, list)
    
    def test_file_watching(self, temp_project):
        """Test file watching functionality."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Test start/stop watching
        analyzer.start_watching()
        analyzer.stop_watching()
        # If no exception, test passes
    
    def test_get_analysis_before_analyze(self, temp_project):
        """Test getting analysis before running analyze."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        result = analyzer.get_analysis()
        assert result is None
    
    def test_get_analysis_after_analyze(self, temp_project):
        """Test getting analysis after running analyze."""
        config = AnalyzerConfig(temp_project)
        analyzer = FastContextAnalyzer(config)
        
        # Analyze first
        analyzer.analyze()
        
        result = analyzer.get_analysis()
        assert result is not None
        assert result.file_count > 0
