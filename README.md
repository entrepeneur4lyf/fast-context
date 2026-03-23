# Fast-Context

[![Repository](https://img.shields.io/badge/GitHub-fast--context-black.svg)](https://github.com/entrepeneur4lyf/fast-context)
[![npm](https://img.shields.io/npm/v/fast-context.svg)](https://www.npmjs.com/package/fast-context)
[![PyPI](https://img.shields.io/pypi/v/fast-context.svg)](https://pypi.org/project/fast-context/)

A Rust codebase analysis engine with Node.js and Python bindings. Fast-Context provides symbol extraction, dependency analysis, graph operations, and file watching for coding assistants and developer tooling.

## Current Status

Fast-Context is no longer in the "hand-wavy production ready" state the older docs claimed, but it is materially healthier now.

Validated recently:
- Rust core: `cargo check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Node package: `npm run build`, `npm test`, `npm pack`, and clean tarball install smoke test
- Python bindings: `cargo check --features python` and the full `pytest tests/python` suite on Python 3.11
- Security audits: `npm audit --audit-level moderate` and `cargo audit`

Current CI/runtime notes:
- GitHub Actions currently tests Node.js 18
- Local source builds for Python are validated on Python 3.11
- Wheel builds target CPython 3.8 through 3.12
- Native publish artifacts are driven by the GitHub release workflows; treat CI as the source of truth for cross-platform binary status

## Features

- 20+ language grammars via Tree-sitter, including Rust, JavaScript, TypeScript, Python, Java, Go, C++, C#, Swift, Objective-C, PHP, Ruby, Lua, Bash, Zig, CSS, HTML, XML, JSON, YAML, Markdown, JSDoc, and Regex
- Context-aware symbol extraction with scope information
- Dependency and relationship extraction across supported languages
- Graph-powered analysis primitives and export tooling
- Parallel analysis, caching, and large-file streaming
- Node.js and Python bindings over the same Rust core

## Installation

### Node.js

```bash
npm install fast-context
```

### Python

```bash
pip install fast-context
```

## Quick Start

### Node.js

```javascript
const { FastContextAnalyzer } = require('fast-context');

const analyzer = new FastContextAnalyzer({
  projectRoot: process.cwd(),
  ignorePatterns: ['node_modules/**', '.git/**', 'target/**']
});

const result = await analyzer.analyze();
console.log(`Found ${result.symbolCount} symbols in ${result.fileCount} files`);
console.log(`Languages: ${result.languages.join(', ')}`);
console.log(`Analysis completed in ${result.durationMs}ms`);
console.log(`Skipped files: ${result.skippedFileCount}`);
```

### Python

```python
import fast_context

config = fast_context.AnalyzerConfig(
    project_root=".",
    languages=["python", "javascript", "rust"],
)
analyzer = fast_context.FastContextAnalyzer(config)
result = analyzer.analyze()

print(f"Files analyzed: {result.file_count}")
print(f"Symbols found: {result.symbol_count}")
print(f"Languages: {result.languages}")
print(f"Skipped files: {len(result.skipped_files)}")
```

## Node.js API Notes

### FastContextAnalyzer

The main class for codebase analysis.

#### Constructor

```javascript
new FastContextAnalyzer(config)
```

**Parameters:**
- `config` (Object): Configuration options
  - `projectRoot` (string): Path to project root directory
  - `ignorePatterns` (string[]): Glob patterns to ignore (optional)
  - `languages` (string[]): Specific languages to analyze (optional)
  - `enableCaching` (boolean): Enable caching (optional)
  - `cachePolicy` (string): Cache policy (optional)
  - `enableWatching` (boolean): Enable file watching (optional)
  - `maxFiles` (number): Maximum files to analyze (optional)
  - `parallelProcessing` (boolean): Enable parallel processing (optional)
  - `enableExperimentalArchitecture` (boolean): Enable experimental features (optional, default: false)

#### Methods

##### `analyze(): AnalysisResult`

Performs comprehensive codebase analysis.

**Returns:** `AnalysisResult`
- `fileCount` (number): Total files analyzed
- `symbolCount` (number): Total symbols found
- `relationshipCount` (number): Total relationships discovered
- `languages` (string[]): Programming languages detected
- `durationMs` (number): Analysis duration in milliseconds
- `memoryUsageMb` (number): Memory usage during analysis, when available
- `skippedFileCount` (number): Count of supported files skipped during analysis
- `skippedFiles` (array): Structured skipped-file diagnostics with `filePath`, `stage`, and `reason`

##### `start_watching(): void`

Starts real-time file monitoring.

##### `stop_watching(): void`

Stops file monitoring.

##### `getAnalysis(): AnalysisResult`

Gets the current analysis results without re-analyzing.

**Returns:** Same as `analyze()` method.

##### Symbol Query Methods

###### `findSymbolsByKind(kind: string): string[]`

Find symbols by type (function, class, etc.).

**Parameters:**
- `kind` (string): Symbol type to search for

**Returns:** Array of symbol names.

###### `findSymbolsInFile(filePath: string): string[]`

Find symbols in a specific file.

**Parameters:**
- `filePath` (string): Path to file

**Returns:** Array of symbol names.

###### `findDependencies(symbolName: string): string[]`

Find dependencies for a symbol.

**Parameters:**
- `symbolName` (string): Name of symbol

**Returns:** Array of dependency names.

###### `findComplexSymbols(complexityThreshold: number): string[]`

Find symbols above complexity threshold.

**Parameters:**
- `complexityThreshold` (number): Minimum complexity level

**Returns:** Array of symbol names.

## Supported Languages

Fast-Context supports 20+ programming languages with Tree-sitter precision:

- **Web Technologies**: JavaScript, TypeScript, HTML, CSS, JSDoc
- **Systems Programming**: Rust, C, C++, Go, Zig
- **Enterprise**: Java, C#, Scala, Swift, Objective-C
- **Scripting Languages**: Python, Ruby, PHP, Lua, Bash
- **Data Formats**: JSON, YAML, XML, Markdown, Regex
- **Comprehensive Coverage**: Full symbol extraction, dependency analysis, and cross-language relationships

## Configuration

### Ignore Patterns

Common ignore patterns for different project types:

```javascript
// Node.js project
ignorePatterns: [
    'node_modules/**',
    'dist/**',
    'build/**',
    '.git/**',
    '**/*.min.js'
]

// Rust project
ignorePatterns: [
    'target/**',
    'Cargo.lock',
    '.git/**'
]

// Multi-language project
ignorePatterns: [
    'node_modules/**',
    'target/**',
    '.git/**',
    'vendor/**',
    'build/**',
    'dist/**'
]
```

### Language Filtering

Analyze only specific languages:

```javascript
const analyzer = new FastContextAnalyzer({
    projectRoot: './src',
    languages: ['javascript', 'typescript', 'rust']
});
```

## 🔧 Advanced Usage

### Symbol Querying

Find specific symbols and relationships:

```javascript
// Find all functions
const functions = analyzer.findSymbolsByKind('function');

// Find symbols in specific file
const fileSymbols = analyzer.findSymbolsInFile('src/main.js');

// Find dependencies
const deps = analyzer.findDependencies('MyClass');

// Find complex code
const complex = analyzer.findComplexSymbols(10); // complexity > 10
```

### Utility Functions

```javascript
// Get supported languages
const languages = getSupportedLanguages();

// Detect language from file
const language = detectLanguage('src/main.rs');

// Get version
const version = getVersion();

// Check configuration
const isValid = checkConfiguration(config);
```

## Architecture

Fast-Context uses a sophisticated multi-layer architecture:

1. **Tree-sitter Parsing**: Language-agnostic syntax tree generation with 20+ language support
2. **Symbol Extraction**: Intelligent symbol identification with full scope tracking and cross-file references
3. **Graph Building**: Advanced dependency mapping using 80+ graph algorithms with rustworkx-core
4. **Streaming Processing**: Memory-efficient file processing with automatic chunking and size-based optimization
5. **Multi-Level Caching**: Intelligent caching with adaptive eviction strategies and background compaction
6. **Security Layer**: Comprehensive input validation and path protection throughout the pipeline
7. **Real-time Monitoring**: File system event processing with debouncing and incremental analysis

## Testing

Representative validation commands:

```bash
# Rust core tests
cargo test

# Node.js integration tests
npm test

# Integration tests
npm run test:integration

# Performance benchmarks
cargo bench
```

## Troubleshooting

### Common Issues

**Performance Issues:**
- Ensure ignore patterns exclude large directories (`node_modules`, `target`)
- Use language filters to focus on relevant code
- Enable caching for repeated analyses

**File Watching Not Working:**
- Check file permissions in project directory
- Verify ignore patterns aren't excluding relevant files
- Ensure callback function is properly defined

**Memory Usage:**
- Use streaming analysis for very large codebases
- Adjust chunk sizes based on available memory
- Monitor memory usage with the built-in metrics

## Performance Tips

1. **Use Ignore Patterns**: Exclude irrelevant directories (`node_modules`, `target`, `.git`)
2. **Language Filtering**: Focus on specific languages when possible
3. **Streaming Processing**: Automatic for large files (>1MB) with configurable chunk sizes
4. **Multi-Level Caching**: Enable intelligent caching for repeated analyses
5. **Incremental Updates**: Use file watching instead of full re-analysis
6. **Memory Management**: Configure appropriate limits for your system resources
7. **Parallel Processing**: Leverage multi-core capabilities for large codebases

## Contributing

Core implementation details:

- **Rust** for memory-safe core analysis engine
- **NAPI-RS** for high-performance Node.js bindings  
- **Tree-sitter** for language-agnostic parsing
- **Tokio** for async runtime and streaming
- **Rayon** for parallel data processing
- **rustworkx-core** for advanced graph algorithms
- **PyO3** for Python bindings (optional)

## Advanced Features

### AI Assistant Integration
- **MCP Protocol**: Native Model Context Protocol support
- **LLM Optimized**: Designed for large language model interactions
- **Knowledge Graph**: Comprehensive code understanding for AI assistance

### Extensibility
- **Custom Languages**: Easy addition of new programming languages
- **Plugin Architecture**: Modular design for custom analysis rules
- **Release Artifacts**: Native packages are built through the GitHub release workflows

## License

Apache-2.0

## Support

For issues and questions:
- GitHub Issues: [entrepeneur4lyf/fast-context](https://github.com/entrepeneur4lyf/fast-context/issues)
- Documentation: [Repository README](https://github.com/entrepeneur4lyf/fast-context#readme) and [docs/README.md](https://github.com/entrepeneur4lyf/fast-context/blob/main/docs/README.md)

---

**Fast-Context** - Powering the next generation of code analysis tools.
