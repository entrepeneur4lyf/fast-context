# Fast-Context CLI

Command-line interface for Fast-Context codebase analysis engine. Provides powerful tools for analyzing, searching, and understanding codebases with intelligent pattern detection and dependency analysis.

## Installation

```bash
# Install globally
npm install -g @fast-context/cli

# Or use npx
npx @fast-context/cli --help
```

## Quick Start

```bash
# Analyze a project
fast-context analyze ./my-project

# Search for symbols
fast-context search "UserService"

# Start interactive REPL
fast-context repl

# Show dependencies
fast-context deps UserService
```

## Commands

### `analyze` - Comprehensive Codebase Analysis

Perform deep analysis of your codebase including symbol extraction, dependency mapping, and architectural insights.

```bash
fast-context analyze <path> [options]

# Examples
fast-context analyze ./src --languages typescript javascript
fast-context analyze . --format json --output analysis.json
fast-context analyze ./app --include-tests --metrics-only
```

**Options:**
- `--languages <langs...>` - Specific languages to analyze
- `--ignore <patterns...>` - Patterns to ignore (e.g., node_modules, dist)
- `--depth <number>` - Maximum analysis depth (default: 10)
- `--max-files <number>` - Maximum files to analyze (default: 10000)
- `--format <format>` - Output format: table, json, yaml, markdown
- `--output <file>` - Save results to file
- `--include-tests` - Include test files
- `--include-docs` - Include documentation files
- `--metrics-only` - Show only metrics
- `--symbols-only` - Extract symbols only

### `search` - Symbol Search

Search for symbols across your codebase with advanced filtering and pattern matching.

```bash
fast-context search <query> [path] [options]

# Examples
fast-context search "UserService"
fast-context search "auth" --kind function class
fast-context search "user.*Service" --regex
```

**Options:**
- `--kind <types...>` - Symbol types (function, class, variable, etc.)
- `--files <patterns...>` - File patterns to search
- `--exclude <patterns...>` - File patterns to exclude
- `--limit <number>` - Maximum results (default: 50)
- `--case-sensitive` - Case-sensitive search
- `--regex` - Treat query as regular expression
- `--exact` - Exact match only

### `repl` - Interactive REPL

Start an interactive session for real-time codebase exploration and analysis.

```bash
fast-context repl [path] [options]

# REPL Commands:
# analyze [path]     - Analyze codebase
# search <query>     - Search symbols
# deps <symbol>      - Show dependencies
# patterns           - Detect patterns
# metrics [file]     - Show metrics
# export <format>    - Export results
# help               - Show help
# exit               - Exit REPL
```

**Options:**
- `--no-banner` - Disable welcome banner
- `--no-auto-analyze` - Disable automatic analysis
- `--history-file <path>` - Custom history file

### `deps` - Dependency Analysis

Analyze symbol dependencies and relationships with configurable depth and scope.

```bash
fast-context deps <symbol> [path] [options]

# Examples
fast-context deps UserService
fast-context deps UserService --depth 3 --include-external
```

**Options:**
- `--depth <number>` - Analysis depth (default: 5)
- `--include-external` - Include external dependencies
- `--include-reverse` - Include reverse dependencies

### `patterns` - Pattern Detection

Detect architectural and design patterns in your codebase.

```bash
fast-context patterns [path] [options]

# Examples
fast-context patterns ./src
fast-context patterns . --types singleton repository
```

### `metrics` - Code Metrics

Analyze code complexity, maintainability, and quality metrics.

```bash
fast-context metrics [path] [options]

# Examples
fast-context metrics ./src/UserService.ts
fast-context metrics . --format json
```

## Configuration

Fast-Context CLI supports multiple configuration sources with the following precedence:

1. Command-line options (highest priority)
2. Specified config file (`--config`)
3. Project config file (`.fast-context.json`, `.fast-context.yaml`)
4. Global config file (`~/.fast-context.json`)
5. Default configuration (lowest priority)

### Configuration File Example

```json
{
  "languages": ["typescript", "javascript"],
  "ignorePatterns": [
    "node_modules",
    "dist",
    "build",
    ".git",
    "coverage"
  ],
  "maxDepth": 10,
  "maxFiles": 10000,
  "enableCaching": true,
  "parallelProcessing": true,
  "includeTests": false,
  "includeDocs": false,
  "outputFormat": "table"
}
```

### Configuration Presets

```bash
# Use predefined presets
fast-context analyze . --preset typescript
fast-context analyze . --preset react
fast-context analyze . --preset minimal
```

Available presets:
- `typescript` - TypeScript/JavaScript projects
- `react` - React applications
- `node` - Node.js projects
- `python` - Python projects
- `rust` - Rust projects
- `minimal` - Minimal analysis
- `comprehensive` - Full analysis

## Output Formats

### Table Format (Default)
Human-readable tables with colored output for terminal display.

### JSON Format
Structured data perfect for programmatic processing and integration.

### YAML Format
Human-readable structured format ideal for configuration and documentation.

### Markdown Format
Documentation-friendly format for reports and README files.

## Global Options

- `-c, --config <path>` - Configuration file path
- `-v, --verbose` - Enable verbose output
- `-q, --quiet` - Suppress non-error output
- `--no-color` - Disable colored output
- `--json` - Force JSON output format
- `--debug` - Enable debug mode

## Examples

### Basic Analysis
```bash
# Analyze current directory
fast-context analyze .

# Analyze with specific languages
fast-context analyze ./src --languages typescript javascript

# Get JSON output
fast-context analyze . --format json --output analysis.json
```

### Advanced Search
```bash
# Search for authentication-related symbols
fast-context search "auth" --kind function class

# Regex search for service classes
fast-context search ".*Service$" --regex --kind class

# Search in specific files
fast-context search "user" --files "src/**/*.ts"
```

### Interactive Exploration
```bash
# Start REPL for current project
fast-context repl

# Start REPL with specific configuration
fast-context repl ./my-project --languages typescript
```

### Dependency Analysis
```bash
# Show UserService dependencies
fast-context deps UserService

# Deep dependency analysis
fast-context deps UserService --depth 5 --include-external
```

## Integration

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Analyze codebase
  run: |
    npx @fast-context/cli analyze . --format json --output analysis.json
    npx @fast-context/cli metrics . --format json --output metrics.json
```

### VS Code Integration
Use the CLI from VS Code terminal or integrate with tasks.json for automated analysis.

### Git Hooks
```bash
# Pre-commit hook
#!/bin/sh
fast-context analyze . --metrics-only --quiet || exit 1
```

## Performance

- **Parallel Processing**: Utilizes multiple CPU cores for faster analysis
- **Intelligent Caching**: Caches parsed results to speed up subsequent runs
- **Memory Efficient**: Streaming analysis for large codebases
- **Configurable Limits**: Control memory usage with file and depth limits

## Troubleshooting

### Common Issues

**Analysis fails with memory error:**
```bash
fast-context analyze . --max-files 5000 --depth 5
```

**Slow analysis:**
```bash
fast-context analyze . --no-parallel false --ignore "node_modules" "dist"
```

**Configuration not found:**
```bash
fast-context analyze . --config ./custom-config.json
```

### Debug Mode
```bash
fast-context analyze . --debug
```

## License

Apache-2.0 - See LICENSE file for details.

## Contributing

See the main repository for contribution guidelines and development setup.
