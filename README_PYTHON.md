# 🐍 Fast-Context Python Bindings

High-performance codebase analysis for Python, powered by Rust.

## 🚀 Installation

### From PyPI (when published)
```bash
pip install fast-context
```

### Development Installation
```bash
# Install maturin for building Rust extensions
pip install maturin

# Clone the repository
git clone https://github.com/entrepeneur4lyf/fast-context.git
cd fast-context

# Build and install in development mode
maturin develop --features python
```

## 📖 Quick Start

```python
from fast_context import FastContextAnalyzer, AnalyzerConfig

# Configure the analyzer
config = AnalyzerConfig(
    project_root="./my-project",
    languages=["python", "rust", "javascript"],
    ignore_patterns=["node_modules/**", "target/**", ".git/**"],
    enable_caching=True,
    parallel_processing=True,
)

# Create analyzer
analyzer = FastContextAnalyzer(config)

# Analyze the codebase
result = analyzer.analyze()
print(f"Analyzed {result.file_count} files")
print(f"Found {result.symbol_count} symbols")
print(f"Detected languages: {', '.join(result.languages)}")
```

## 🔍 Advanced Usage

### Symbol Analysis

```python
# Find all functions
functions = analyzer.find_symbols_by_kind("function")
print(f"Functions: {functions}")

# Find all classes
classes = analyzer.find_symbols_by_kind("class")
print(f"Classes: {classes}")

# Find symbols in specific file
file_symbols = analyzer.find_symbols_in_file("src/main.py")
print(f"Symbols in main.py: {file_symbols}")
```

### Dependency Analysis

```python
# Find dependencies of a symbol
deps = analyzer.find_dependencies("MyClass")
print(f"Dependencies of MyClass: {deps}")

# Find complex code (high complexity)
complex_code = analyzer.find_complex_symbols(threshold=10)
print(f"Complex symbols: {complex_code}")
```

### File Watching

```python
# Start watching for file changes
analyzer.start_watching()

# Your application logic here...

# Stop watching
analyzer.stop_watching()
```

### Utility Functions

```python
from fast_context import get_supported_languages, detect_language, get_version

# Check supported languages
languages = get_supported_languages()
print(f"Supported: {languages}")

# Detect file language
lang = detect_language("script.py")
print(f"Language: {lang}")  # Output: Python

# Get version
version = get_version()
print(f"Version: {version}")
```

## 🏗️ API Reference

### AnalyzerConfig

Configuration class for the analyzer.

```python
config = AnalyzerConfig(
    project_root: str,                    # Required: path to project
    languages: List[str] = None,          # Languages to analyze
    ignore_patterns: List[str] = None,    # Patterns to ignore
    enable_caching: bool = True,          # Enable caching
    enable_watching: bool = False,        # Enable file watching
    max_files: int = 10000,              # Maximum files to process
    parallel_processing: bool = True,     # Enable parallel processing
)
```

### FastContextAnalyzer

Main analyzer class.

#### Methods

- `analyze() -> AnalysisResult`: Analyze the codebase
- `find_symbols_by_kind(kind: str) -> List[str]`: Find symbols by type
- `find_symbols_in_file(file_path: str) -> List[str]`: Find symbols in file
- `find_dependencies(symbol_name: str) -> List[str]`: Find symbol dependencies
- `find_complex_symbols(threshold: int) -> List[str]`: Find complex symbols
- `start_watching() -> None`: Start file watching
- `stop_watching() -> None`: Stop file watching
- `get_analysis() -> Optional[AnalysisResult]`: Get current analysis

### AnalysisResult

Results from codebase analysis.

#### Properties

- `file_count: int`: Number of files analyzed
- `symbol_count: int`: Number of symbols found
- `relationship_count: int`: Number of relationships found
- `languages: List[str]`: Detected languages
- `duration_ms: int`: Analysis duration in milliseconds
- `memory_usage_mb: Optional[float]`: Memory usage in MB

## 🧪 Testing

```bash
# Install test dependencies
pip install pytest pytest-asyncio

# Run tests
pytest tests/
```

## 🔧 Development

### Building from Source

```bash
# Install Rust and Python dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pip install maturin

# Build the extension
maturin build --features python

# Or build and install in development mode
maturin develop --features python
```

### Type Checking

```bash
# Install mypy
pip install mypy

# Run type checking
mypy python/fast_context/
```

### Code Formatting

```bash
# Install black and ruff
pip install black ruff

# Format code
black python/
ruff check python/
```

## 🚀 Performance

The Python bindings provide the same high performance as the Rust core:

- **Fast Analysis**: Analyze large codebases in seconds
- **Low Memory**: Efficient memory usage with intelligent caching
- **Parallel Processing**: Multi-threaded analysis for maximum speed
- **Real-time Updates**: File watching with minimal overhead

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## 📄 License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## 🔗 Links

- [GitHub Repository](https://github.com/entrepeneur4lyf/fast-context)
- [Documentation](https://github.com/entrepeneur4lyf/fast-context#readme)
- [Issues](https://github.com/entrepeneur4lyf/fast-context/issues)
