# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **RustWorkX**, a high-performance graph library for Python implemented in Rust. Despite the directory name containing "nodejs", this is a Python library that uses PyO3 bindings to expose Rust graph algorithms to Python.

- **Language**: Rust with Python bindings via PyO3
- **Package Name**: `rustworkx` (formerly `retworkx`)  
- **Version**: 0.16.0
- **Build System**: setuptools-rust + Cargo
- **Test Framework**: nox (preferred) or tox (deprecated)

## Development Commands

### Building and Installation
```bash
# Install in development mode (debug build)
python setup.py develop

# Install release build
pip install .

# Install with optional dependencies
pip install '.[all]'  # matplotlib + graphviz support
pip install '.[mpl]'  # matplotlib only
pip install '.[graphviz]'  # graphviz only
```

### Testing
```bash
# Run tests with nox (preferred)
nox -e test

# Run tests for specific Python versions
nox -e test_with_version

# Run tests with tox (deprecated but still works)
tox -e py39
```

### Linting and Formatting
```bash
# Run all linting
nox -e lint

# Format Python code
nox -e black

# Check Rust formatting
cargo fmt --all -- --check

# Format Rust code
cargo fmt --all

# Check Python with ruff
ruff check rustworkx retworkx setup.py

# Check for typos
nox -e typos
```

### Documentation
```bash
# Build documentation
nox -e docs

# Clean documentation build
nox -e docs_clean
```

### Type Checking
```bash
# Check stub files
nox -e stubs
```

## Architecture

### Core Components

1. **rustworkx-core/**: Pure Rust graph algorithms library
   - Independent of Python bindings
   - Contains core graph data structures and algorithms
   - Used by both the main library and potentially other projects

2. **src/**: Python binding implementation
   - PyO3-based bindings that expose Rust functionality to Python
   - Organized by functionality (centrality, generators, shortest_path, etc.)
   - Entry point: `src/lib.rs`

3. **rustworkx/**: Python package structure
   - Python shim and type stubs
   - Visualization modules for matplotlib and graphviz

### Key Rust Modules

- **Graph Types**: `src/digraph.rs`, `src/graph.rs` - Core graph data structures
- **Algorithms**: Organized in subdirectories like `src/shortest_path/`, `src/centrality/`
- **Generators**: `src/generators.rs` - Graph generation functions
- **I/O**: `src/graphml.rs`, `src/json/` - Import/export functionality
- **Visualization**: `src/layout/` - Graph layout algorithms

### Python Integration

- Uses PyO3 with `abi3-py39` for stable ABI (Python 3.9+)
- Numpy integration for matrix operations
- Custom return types and iterators for Python compatibility

## Environment Variables

- `RUSTWORKX_DEBUG=1`: Force debug build
- `RUSTWORKX_PKG_NAME`: Override package name (used for retworkx compatibility)
- `RUSTWORKX_TEST_PRESERVE_IMAGES`: Preserve test images during visualization tests

## Testing Strategy

- **Python tests**: Located in `tests/` directory, organized by graph type (digraph/, graph/)
- **Rust tests**: Unit tests within Rust modules and integration tests in `rustworkx-core/tests/`
- **Test runner**: stestr for parallel test execution
- **Test dependencies**: networkx for comparison/validation

## Release Process

- Uses semantic versioning
- Release notes in `releasenotes/notes/` using reno
- Automated builds via GitHub Actions for multiple platforms
- PyPI deployment with precompiled wheels

## Dependencies

### Rust Dependencies
- **petgraph**: Core graph data structure
- **ndarray**: Numerical arrays with numpy integration  
- **rayon**: Data parallelism
- **pyo3**: Python bindings
- **indexmap**: Ordered hash maps
- **hashbrown**: High-performance hash maps

### Python Dependencies
- **numpy**: Required for array operations
- **matplotlib**: Optional, for visualization
- **pillow**: Optional, for graphviz support

## Common Pitfalls

1. **Development Installation**: Use `python setup.py develop` for debug builds, not `pip install -e`
2. **Name Conflicts**: Don't run Python from repo root due to package shim conflicts
3. **Rust Changes**: Recompile with `pip install .` after Rust code changes
4. **Test Location**: Tests must be run from `tests/` directory, not repo root