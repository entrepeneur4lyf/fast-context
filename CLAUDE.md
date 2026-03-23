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

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **fast-context** (8405 symbols, 22504 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## When Debugging

1. `gitnexus_query({query: "<error or symptom>"})` — find execution flows related to the issue
2. `gitnexus_context({name: "<suspect function>"})` — see all callers, callees, and process participation
3. `READ gitnexus://repo/fast-context/process/{processName}` — trace the full execution flow step by step
4. For regressions: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})` — see what your branch changed

## When Refactoring

- **Renaming**: MUST use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` first. Review the preview — graph edits are safe, text_search edits need manual review. Then run with `dry_run: false`.
- **Extracting/Splitting**: MUST run `gitnexus_context({name: "target"})` to see all incoming/outgoing refs, then `gitnexus_impact({target: "target", direction: "upstream"})` to find all external callers before moving code.
- After any refactor: run `gitnexus_detect_changes({scope: "all"})` to verify only expected files changed.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Tools Quick Reference

| Tool | When to use | Command |
|------|-------------|---------|
| `query` | Find code by concept | `gitnexus_query({query: "auth validation"})` |
| `context` | 360-degree view of one symbol | `gitnexus_context({name: "validateUser"})` |
| `impact` | Blast radius before editing | `gitnexus_impact({target: "X", direction: "upstream"})` |
| `detect_changes` | Pre-commit scope check | `gitnexus_detect_changes({scope: "staged"})` |
| `rename` | Safe multi-file rename | `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` |
| `cypher` | Custom graph queries | `gitnexus_cypher({query: "MATCH ..."})` |

## Impact Risk Levels

| Depth | Meaning | Action |
|-------|---------|--------|
| d=1 | WILL BREAK — direct callers/importers | MUST update these |
| d=2 | LIKELY AFFECTED — indirect deps | Should test |
| d=3 | MAY NEED TESTING — transitive | Test if critical path |

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/fast-context/context` | Codebase overview, check index freshness |
| `gitnexus://repo/fast-context/clusters` | All functional areas |
| `gitnexus://repo/fast-context/processes` | All execution flows |
| `gitnexus://repo/fast-context/process/{name}` | Step-by-step execution trace |

## Self-Check Before Finishing

Before completing any code modification task, verify:
1. `gitnexus_impact` was run for all modified symbols
2. No HIGH/CRITICAL risk warnings were ignored
3. `gitnexus_detect_changes()` confirms changes match expected scope
4. All d=1 (WILL BREAK) dependents were updated

## Keeping the Index Fresh

After committing code changes, the GitNexus index becomes stale. Re-run analyze to update it:

```bash
npx gitnexus analyze
```

If the index previously included embeddings, preserve them by adding `--embeddings`:

```bash
npx gitnexus analyze --embeddings
```

To check whether embeddings exist, inspect `.gitnexus/meta.json` — the `stats.embeddings` field shows the count (0 means no embeddings). **Running analyze without `--embeddings` will delete any previously generated embeddings.**

> Claude Code users: A PostToolUse hook handles this automatically after `git commit` and `git merge`.

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
