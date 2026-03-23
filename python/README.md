# Fast-Context Python SDK

[![PyPI version](https://badge.fury.io/py/fast-context.svg)](https://badge.fury.io/py/fast-context)
[![Python versions](https://img.shields.io/pypi/pyversions/fast-context.svg)](https://pypi.org/project/fast-context/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Python bindings for the Fast-Context Rust analysis engine.

## Status

Validated recently:
- `cargo check --features python`
- `pytest tests/python` on Python 3.11
- wheel builds in CI for CPython 3.8 through 3.12

For local source builds, Python 3.11 is the safest supported path today.

## Features

- 20+ language grammars via Tree-sitter
- Rust-backed analysis with Python bindings
- Compatibility wrapper for the existing Python API surface
- MCP and CLI helpers for local tooling integrations
- Graph bindings and dependency analysis helpers

## Installation

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

## Quick Start

### Basic Analysis

```python
import fast_context

config = fast_context.AnalyzerConfig(
    project_root="/path/to/your/project",
    languages=["python", "javascript", "rust"],
)
analyzer = fast_context.FastContextAnalyzer(config)

result = analyzer.analyze()

print(f"Files analyzed: {result.file_count}")
print(f"Symbols found: {result.symbol_count}")
print(f"Languages: {result.languages}")
print(f"Skipped files: {len(result.skipped_files)}")
```

### Compatibility Helpers

```python
import fast_context

analyzer = fast_context.FastContextAnalyzer("/path/to/project")

functions = analyzer.find_symbols_by_kind("function")
symbols = analyzer.extract_symbols("/path/to/project/app.py")
deps = analyzer.analyze_dependencies("/path/to/project")
graph = analyzer.create_dependency_graph("/path/to/project")

print(functions[:5])
print(symbols)
print(graph["edges"][:5])
```

### Asynchronous Native API

```python
import asyncio
import fast_context

async def main():
    analyzer = fast_context.FastContextAnalyzer(
        fast_context.AnalyzerConfig(project_root=".")
    )
    result = await analyzer.analyze_async()
    print(result.file_count, result.symbol_count)

asyncio.run(main())
```

## Command Line Interface

Fast-Context includes a CLI:

```bash
# Analyze current directory
fast-context analyze

# Analyze specific directory
fast-context analyze /path/to/project

# Show current configuration
fast-context config show
```

## AI Assistant Integration

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

## Supported Languages

Fast-Context supports 20+ programming languages:

**Systems**: Rust, C, C++, Go, Zig
**Web**: JavaScript, TypeScript, HTML, CSS
**Enterprise**: Java, C#, Scala, Swift, Objective-C
**Scripting**: Python, Ruby, PHP, Lua, Bash
**Data**: JSON, YAML, XML, Markdown

## Configuration

The most reliable configuration path today is constructing `AnalyzerConfig` directly in Python.

```python
config = fast_context.AnalyzerConfig(
    project_root=".",
    languages=["python", "javascript"],
    ignore_patterns=["node_modules/**", ".git/**", "target/**"],
    max_files=5000,
    parallel_processing=True,
)
```

## Performance

Fast-Context is built for performance:

- **Rust Core**: Maximum performance with memory safety
- **Parallel Processing**: Multi-threaded analysis using Rayon
- **Efficient Parsing**: Tree-sitter parsers with caching
- **Memory Optimization**: Streaming analysis for large codebases

Benchmarks in the older docs were aspirational. Treat the automated test and release workflows as the current source of truth for validated performance and platform support.

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/entrepeneur4lyf/fast-context.git
cd fast-context

# Install Python dependencies
pip install -e ".[dev]"

# Build Rust extension
maturin develop --features python

# Run tests
python -m pytest tests/python
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/entrepeneur4lyf/fast-context/blob/main/LICENSE) for details.

## Links

- **Repository**: https://github.com/entrepeneur4lyf/fast-context
- **Documentation**: https://github.com/entrepeneur4lyf/fast-context#readme
- **Issues**: https://github.com/entrepeneur4lyf/fast-context/issues
- **PyPI**: https://pypi.org/project/fast-context/

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [PyO3](https://pyo3.rs/) - Rust-Python bindings
- [Tree-sitter](https://tree-sitter.github.io/) - Incremental parsing
- [rustworkx](https://github.com/Qiskit/rustworkx) - Graph algorithms
- [Maturin](https://github.com/PyO3/maturin) - Build tool for Rust-Python extensions
