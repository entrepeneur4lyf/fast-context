# Fast-Context

[![Repository](https://img.shields.io/badge/GitHub-fast--context-black.svg)](https://github.com/entrepeneur4lyf/fast-context)
[![npm](https://img.shields.io/npm/v/fast-context.svg)](https://www.npmjs.com/package/fast-context)
[![PyPI](https://img.shields.io/pypi/v/fast-context.svg)](https://pypi.org/project/fast-context/)

Fast-Context is a Rust codebase analysis library with Node.js and Python bindings.

It is built around Tree-sitter parsing, symbol extraction, dependency analysis, and graph operations. The project is meant for code search, indexing, editor tooling, and assistant-style codebase analysis.

## What It Does

- analyzes multi-language repositories
- extracts symbols and relationships
- finds symbols by kind or file
- resolves symbol dependencies
- exposes graph types for traversal and metrics
- supports file watching and incremental workflows

## Current Surface

The actively maintained surfaces are:

- Rust core library
- Node.js package
- Python bindings
- Codex skills
- Claude plugin skeleton

## Installation

### Node.js

```bash
npm install fast-context
```

### Python

```bash
pip install fast-context
```

### Hosts and MCP

For host-specific installation and setup, see [docs/HOST_SETUP.md](docs/HOST_SETUP.md).

That document covers:

- Codex skills, MCP setup, and optional hooks
- Claude plugin installation, bundled skills, plugin MCP, and optional hooks
- standalone MCP installation for non-Codex/non-Claude clients

## Quick Start

### Node.js

```js
const { FastContextAnalyzer } = require('fast-context')

const analyzer = new FastContextAnalyzer({
  projectRoot: process.cwd(),
  ignorePatterns: ['node_modules/**', '.git/**', 'target/**'],
})

const result = analyzer.analyze()

console.log({
  files: result.fileCount,
  symbols: result.symbolCount,
  relationships: result.relationshipCount,
  skipped: result.skippedFileCount,
})
```

### Python

```python
import fast_context

config = fast_context.AnalyzerConfig(
    project_root=".",
    ignore_patterns=["node_modules/**", ".git/**", "target/**"],
)

analyzer = fast_context.FastContextAnalyzer(config)
result = analyzer.analyze()

print({
    "files": result.file_count,
    "symbols": result.symbol_count,
    "relationships": len(result.relationships),
    "skipped": len(result.skipped_files),
})
```

## Node API Summary

`FastContextAnalyzer` currently exposes:

- `analyze()`
- `getAnalysis()`
- `startWatching()`
- `stopWatching()`
- `findSymbolsByKind(kind)`
- `findSymbolsInFile(filePath)`
- `findDependencies(symbolName)`
- `findComplexSymbols(complexityThreshold)`

Utility exports:

- `getVersion()`
- `getSupportedLanguages()`
- `detectLanguage(filePath)`
- `checkConfiguration(config?)`
- `getSystemInfo()`

For the current typed contract, see [index.d.ts](index.d.ts) and [docs/API_REFERENCE.md](docs/API_REFERENCE.md).

## Supported Languages

Fast-Context includes parsers for:

- Rust
- JavaScript
- TypeScript
- Python
- Java
- Go
- C and C++
- C#
- Swift
- Objective-C
- PHP
- Ruby
- Scala
- Zig
- Lua
- Bash
- CSS
- HTML
- XML
- JSON
- YAML
- Markdown

## Development

### Rust

```bash
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run --bin fast-context --features cli -- --help
```

### Node.js

```bash
npm install
npm run build:debug
npm test
```

### Python

```bash
pytest tests/python
```

If you are building native bindings locally, use the versions and host environments exercised by CI where possible.

## Documentation

- [API reference](docs/API_REFERENCE.md)
- [Deployment guide](docs/DEPLOYMENT_GUIDE.md)
- [Host setup](docs/HOST_SETUP.md)
- [Host integration spec](docs/HOST_INTEGRATION_SPEC.md)
- [Host integration plan](docs/HOST_INTEGRATION_PLAN.md)
- [Release architecture](docs/RELEASE_ARCHITECTURE.md)
- [Documentation index](docs/README.md)

## Repository Notes

- The generated Node typings live in [index.d.ts](index.d.ts).
- The Node package metadata is rewritten during build/publish steps, so local `package.json` churn after native builds is expected.
- Cross-platform native artifact truth should come from the GitHub Actions workflows, not a single local machine.

## License

Apache-2.0

## Support

- [GitHub repository](https://github.com/entrepeneur4lyf/fast-context)
- [Issue tracker](https://github.com/entrepeneur4lyf/fast-context/issues)
