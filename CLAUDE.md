# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fast-Context is an intelligent codebase analysis engine built in Rust with Node.js and Python bindings. It provides graph-powered code comprehension through Tree-sitter parsers and sophisticated dependency analysis.

## Build System & Commands

### Rust Core Development
```bash
# Build the Rust library
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate TypeScript types
cargo test generate_typescript_types

# Build with release optimizations
cargo build --release
```

### Node.js Development
```bash
# Install dependencies
npm install

# Build native Node.js module
npm run build

# Build debug version
npm run build:debug

# Generate TypeScript definitions
npm run generate-types

# Run Node.js tests
npm test

# Run integration tests
npm run test:integration

# Run TypeScript example
npm run example
```

### Python Development
```bash
# Install Python development dependencies
pip install -e ".[dev]"

# Run Python tests
pytest tests/

# Run with specific Python version
python -m pytest tests/
```

## Architecture

### Core Domains (Domain-Driven Design)
- **Graph Domain** (`src/domains/graph.rs`): Pure graph algorithms and data structures using rustworkx-core
- **Analysis Domain** (`src/domains/analysis.rs`): Codebase analysis and intelligence features  
- **Core Domain** (`src/domains/core.rs`): Shared utilities and abstractions

### Multi-Language Support
Supports 20+ programming languages through Tree-sitter parsers:
- **Systems**: Rust, C, C++, Go, Zig
- **Web**: JavaScript, TypeScript, HTML, CSS
- **Enterprise**: Java, C#, Scala, Swift, Objective-C
- **Scripting**: Python, Ruby, PHP, Lua, Bash
- **Data**: JSON, YAML, XML, Markdown

### Key Components
- **CoreAnalyzer** (`src/core/mod.rs`): Shared Send+Sync analyzer used by both Node.js and Python bindings
- **ParserFactory** (`src/parsers/mod.rs`): Multi-language Tree-sitter parser factory with caching
- **SymbolExtractorFactory** (`src/symbols/mod.rs`): Language-specific symbol extraction
- **FastContextAnalyzer** (`src/analyzer/mod.rs`): Main Node.js API with NAPI bindings

### Build Features
- `nodejs`: Enables NAPI-RS bindings for Node.js
- `python`: Enables PyO3 bindings for Python
- Default: `nodejs`

## Development Workflow

### Adding New Language Support
1. Add Tree-sitter grammar to `Cargo.toml`
2. Implement language extractor in `src/symbols/extractors/`
3. Add language to `LanguageId` enum in `src/parsers/mod.rs`
4. Add dependency extractor in `src/symbols/dependency_extractor/`

### Testing Strategy
- **Unit tests**: `cargo test` for Rust core functionality
- **Integration tests**: `npm run test:integration` for end-to-end testing
- **Python tests**: `pytest tests/` for Python binding validation
- **Performance benchmarks**: `cargo bench` for performance optimization

### Code Organization
- Use domain-driven design with clear separation of concerns
- Prefer composition over inheritance
- Implement error handling with `thiserror`
- Use async/await with Tokio runtime
- Follow Rust idioms and conventions

## Important Files

### Configuration
- `Cargo.toml`: Rust project configuration and dependencies
- `package.json`: Node.js package configuration and scripts
- `pyproject.toml`: Python package configuration

### Core Implementation
- `src/lib.rs`: Main library entry point and module organization
- `src/core/mod.rs`: Cross-language CoreAnalyzer implementation
- `src/parsers/mod.rs`: Multi-language Tree-sitter integration
- `src/symbols/mod.rs`: Symbol extraction and management

### Bindings
- `src/analyzer/mod.rs`: Node.js NAPI bindings
- `src/python_bindings.rs`: Python PyO3 bindings

### Testing
- `tests/integration.test.mjs`: Node.js integration tests
- `tests/test_python_bindings.py`: Python binding tests

## Performance Considerations

- Uses LRU caching with adaptive strategies
- Parallel processing with Rayon for CPU-intensive tasks
- Streaming analysis for large codebases
- Memory-efficient tree-sitter parsing
- Configurable resource limits and ignore patterns

## Common Patterns

### Analyzer Usage
```rust
// Create analyzer for specific use case
let analyzer = CoreAnalyzer::new(project_root, languages, ignore_patterns);

// Analyze codebase
let result = analyzer.analyze()?;

// Query symbols
let functions = analyzer.find_symbols_by_kind("function".to_string())?;
```

### Language Detection
```rust
// Auto-detect language from file extension
let language = LanguageId::from_extension("rs")?; // Some(LanguageId::Rust)

// Parse file with auto-detection
let parse_result = parser_factory.parse_file(content, "main.rs")?;
```

### Error Handling
- Use `thiserror` for structured error types
- Return `Result<T, String>` for simple error cases
- Provide context in error messages
- Handle file system errors gracefully

## Build Targets

### Native Libraries
- Linux x64: `fast-context.linux-x64-gnu.node`
- macOS x64: `fast-context.darwin-x64.node`
- macOS ARM64: `fast-context.darwin-arm64.node`
- Windows x64: `fast-context.win32-x64-msvc.node`

### Cross-Platform Support
- Universal builds with `npm run universal`
- Platform-specific builds with `npm run build`
- Artifact management with `npm run artifacts`