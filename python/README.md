# Fast-Context Python SDK

[![PyPI version](https://badge.fury.io/py/fast-context.svg)](https://badge.fury.io/py/fast-context)
[![Python versions](https://img.shields.io/pypi/pyversions/fast-context.svg)](https://pypi.org/project/fast-context/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

High-performance codebase analysis engine with graph-powered code comprehension, built in Rust for maximum performance.

## 🚀 Features

- **Multi-Language Support**: 20+ programming languages via Tree-sitter parsers
- **Graph-Powered Analysis**: Advanced dependency and relationship analysis using rustworkx
- **High Performance**: Rust-based core with Python bindings for optimal speed
- **AI Assistant Integration**: Built-in MCP (Model Context Protocol) server for AI tools
- **Streaming Analysis**: Real-time progressive analysis with cancellation support
- **Rich CLI Tools**: Comprehensive command-line interface with interactive features
- **Type Safety**: Full type annotations and runtime validation

## 📦 Installation

```bash
pip install fast-context
```

### Optional Dependencies

```bash
# For CLI tools
pip install fast-context[cli]

# For configuration management
pip install fast-context[config]

# For development
pip install fast-context[dev]

# Install everything
pip install fast-context[all]
```

## 🔧 Quick Start

### Basic Analysis

```python
import fast_context

# Initialize analyzer
analyzer = fast_context.FastContextAnalyzer("/path/to/your/project")

# Analyze codebase
result = analyzer.analyze()

print(f"Found {len(result.symbols)} symbols")
print(f"Analysis took {result.performance_metrics.total_time_ms}ms")
```

### Advanced Usage

```python
import fast_context

# Configure analyzer
config = fast_context.AnalyzerConfig(
    languages=["python", "javascript", "rust"],
    include_patterns=["src/**/*.py", "lib/**/*.js"],
    exclude_patterns=["**/test_*.py", "**/node_modules/**"],
    max_file_size_mb=10
)

analyzer = fast_context.FastContextAnalyzer("/path/to/project", config)

# Get symbols by type
functions = analyzer.find_symbols_by_kind("function")
classes = analyzer.find_symbols_by_kind("class")

# Analyze dependencies
deps = analyzer.analyze_dependencies()
print(f"Found {len(deps.internal)} internal dependencies")
```

### Graph Analysis

```python
import fast_context

analyzer = fast_context.FastContextAnalyzer("/path/to/project")

# Get dependency graph
graph = analyzer.get_dependency_graph()

# Analyze graph properties
print(f"Nodes: {graph.node_count()}")
print(f"Edges: {graph.edge_count()}")

# Find strongly connected components
components = graph.strongly_connected_components()
print(f"Found {len(components)} components")
```

## 🖥️ Command Line Interface

Fast-Context includes a powerful CLI:

```bash
# Analyze current directory
fast-context analyze

# Analyze specific directory
fast-context analyze /path/to/project

# Search for symbols
fast-context search "function_name"

# Get dependency information
fast-context deps --format json

# Interactive REPL mode
fast-context repl
```

## 🤖 AI Assistant Integration

Fast-Context includes an MCP server for AI assistant integration:

```bash
# Start MCP server
fast-context-mcp

# Or run as module
python -m fast_context.mcp_server
```

### Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "fast-context": {
      "command": "fast-context-mcp",
      "args": ["--project-root", "/path/to/your/project"]
    }
  }
}
```

## 📚 Supported Languages

Fast-Context supports 20+ programming languages:

**Systems**: Rust, C, C++, Go, Zig
**Web**: JavaScript, TypeScript, HTML, CSS
**Enterprise**: Java, C#, Scala, Swift, Objective-C
**Scripting**: Python, Ruby, PHP, Lua, Bash
**Data**: JSON, YAML, XML, Markdown

## 🔧 Configuration

### Configuration File

Create a `.fast-context.toml` file in your project root:

```toml
[analysis]
languages = ["python", "javascript"]
include_patterns = ["src/**/*", "lib/**/*"]
exclude_patterns = ["**/test_*", "**/node_modules/**"]
max_file_size_mb = 10

[graph]
enable_dependency_analysis = true
max_depth = 5

[mcp]
enable_server = true
port = 8080
```

### Environment Variables

```bash
export FAST_CONTEXT_LOG_LEVEL=info
export FAST_CONTEXT_CACHE_DIR=/tmp/fast-context
export FAST_CONTEXT_MAX_WORKERS=8
```

## 🚀 Performance

Fast-Context is built for performance:

- **Rust Core**: Maximum performance with memory safety
- **Parallel Processing**: Multi-threaded analysis using Rayon
- **Efficient Parsing**: Tree-sitter parsers with caching
- **Memory Optimization**: Streaming analysis for large codebases

### Benchmarks

| Project Size | Analysis Time | Memory Usage |
|-------------|---------------|--------------|
| Small (1K files) | ~100ms | ~50MB |
| Medium (10K files) | ~1s | ~200MB |
| Large (100K files) | ~10s | ~1GB |

## 🛠️ Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/entrepeneur4lyf/rustworkx-nodejs.git
cd rustworkx-nodejs

# Install Python dependencies
pip install -e ".[dev]"

# Build Rust extension
maturin develop --features python

# Run tests
pytest tests/
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/entrepeneur4lyf/rustworkx-nodejs/blob/main/LICENSE) for details.

## 🔗 Links

- **Repository**: https://github.com/entrepeneur4lyf/rustworkx-nodejs
- **Documentation**: https://github.com/entrepeneur4lyf/rustworkx-nodejs#readme
- **Issues**: https://github.com/entrepeneur4lyf/rustworkx-nodejs/issues
- **PyPI**: https://pypi.org/project/fast-context/

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [PyO3](https://pyo3.rs/) - Rust-Python bindings
- [Tree-sitter](https://tree-sitter.github.io/) - Incremental parsing
- [rustworkx](https://github.com/Qiskit/rustworkx) - Graph algorithms
- [Maturin](https://github.com/PyO3/maturin) - Build tool for Rust-Python extensions