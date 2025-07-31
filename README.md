# Fast-Context: Intelligent Codebase Analysis Engine

A high-performance, graph-powered codebase analysis engine built in Rust with Node.js bindings. Fast-Context provides deep code understanding through symbol extraction, dependency analysis, and real-time file monitoring.

## 🚀 Features

- **Lightning Fast Analysis**: Process large codebases in milliseconds using Rust's performance
- **Multi-Language Support**: 20+ programming languages via Tree-sitter parsers
- **Real-time File Watching**: Live updates with intelligent change detection
- **Graph-Powered Insights**: Advanced dependency and relationship analysis
- **Streaming API**: Handle massive codebases with memory-efficient streaming
- **Smart Caching**: Multi-level caching for instant subsequent analyses

## 📊 Performance

Real-world performance on a typical project:
- **86 files, 26,529 symbols, 6,026 relationships** analyzed in **667ms**
- **15MB memory usage** during analysis
- **Sub-second** incremental updates with file watching

## 🛠 Installation

```bash
npm install fast-context
```

## 🏁 Quick Start

### Basic Codebase Analysis

```javascript
const { FastContextAnalyzer } = require('fast-context');

// Initialize analyzer
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: ['node_modules/**', '.git/**', 'target/**']
});

// Analyze codebase
const result = analyzer.analyze();
console.log(`Found ${result.symbolCount} symbols in ${result.fileCount} files`);
console.log(`Languages: ${result.languages.join(', ')}`);
console.log(`Analysis completed in ${result.durationMs}ms`);
```

### Real-time File Watching

```javascript
// Set up file change monitoring
analyzer.startWatching((changeBatch) => {
    console.log(`File changes detected: ${changeBatch.changeCount}`);
    console.log(`Impact level: ${changeBatch.impactLevel}`);
    console.log(`Requires reanalysis: ${changeBatch.requiresReanalysis}`);
    
    changeBatch.changes.forEach(change => {
        console.log(`${change.changeType.toUpperCase()}: ${change.filePath}`);
        if (change.language) {
            console.log(`  Language: ${change.language}`);
        }
    });
});

// Stop watching when done
// analyzer.stopWatching();
```

## 📖 API Reference

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
  - `languageFilters` (string[]): Specific languages to analyze (optional)

#### Methods

##### `analyze(config?): AnalysisResult`

Performs comprehensive codebase analysis.

**Returns:** `AnalysisResult`
- `fileCount` (number): Total files analyzed
- `symbolCount` (number): Total symbols found
- `relationshipCount` (number): Total relationships discovered
- `languages` (string[]): Programming languages detected
- `durationMs` (number): Analysis duration in milliseconds
- `memoryUsageMb` (number): Memory usage during analysis

##### `startWatching(callback): void`

Starts real-time file monitoring.

**Parameters:**
- `callback` (function): Called when file changes are detected
  - `changeBatch` (Object): Batch of file changes
    - `changes` (FileChange[]): Array of individual changes
    - `changeCount` (number): Number of changes in batch
    - `impactLevel` ('low'|'medium'|'high'): Estimated impact
    - `requiresReanalysis` (boolean): Whether full reanalysis is needed
    - `batchTimestamp` (number): Unix timestamp of batch

**FileChange Object:**
- `changeType` ('created'|'modified'|'deleted'|'renamed'): Type of change
- `filePath` (string): Path to changed file
- `language` (string): Detected programming language
- `affectsAnalysis` (boolean): Whether change affects code analysis
- `timestamp` (number): Unix timestamp of change

##### `stopWatching(): void`

Stops file monitoring.

## 🌐 Supported Languages

Fast-Context supports 20+ programming languages:

- **Web**: JavaScript, TypeScript, HTML, CSS
- **Systems**: Rust, C, C++, Go, Zig
- **Enterprise**: Java, C#, Scala, Swift, Objective-C
- **Scripting**: Python, Ruby, PHP, Lua, Bash
- **Data**: JSON, YAML, XML, Markdown
- **Documentation**: JSDoc, Markdown

## ⚙️ Configuration

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
    languageFilters: ['javascript', 'typescript', 'rust']
});
```

## 🔧 Advanced Usage

### Streaming Analysis

For very large codebases, use streaming analysis:

```javascript
analyzer.findSymbolsStreaming(
    'function',  // Search pattern
    {
        chunkSize: 1000,
        includeProgress: true
    },
    (chunk) => {
        console.log(`Processing chunk: ${chunk.symbols.length} symbols`);
        console.log(`Progress: ${chunk.progress}%`);
    }
);
```

### Symbol Querying

Find specific symbols and relationships:

```javascript
// Find all functions
const functions = await analyzer.findSymbolsByKind('function');

// Find symbols in specific file
const fileSymbols = await analyzer.findSymbolsInFile('src/main.js');

// Find dependencies
const deps = await analyzer.findDependencies('MyClass');

// Find complex code
const complex = await analyzer.findComplexSymbols(10); // complexity > 10
```

## 🏗 Architecture

Fast-Context uses a sophisticated multi-layer architecture:

1. **Tree-sitter Parsing**: Language-agnostic syntax tree generation
2. **Symbol Extraction**: Intelligent symbol identification and classification
3. **Graph Building**: Dependency and relationship mapping
4. **Caching System**: Multi-level (L1/L2/L3) intelligent caching
5. **Real-time Monitoring**: File system event processing with debouncing

## 🧪 Testing

Run the test suite:

```bash
# Basic functionality test
node test_basic.js

# File watching test
node test_file_watcher.js
```

## 🔍 Troubleshooting

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

## 📈 Performance Tips

1. **Use Ignore Patterns**: Exclude irrelevant directories
2. **Language Filtering**: Focus on specific languages when possible
3. **Streaming**: Use streaming for codebases with >50k symbols
4. **Caching**: Enable caching for repeated analyses
5. **Incremental Updates**: Use file watching instead of full re-analysis

## 🤝 Contributing

Contributions welcome! This project uses:
- **Rust** for core analysis engine
- **NAPI-RS** for Node.js bindings
- **Tree-sitter** for parsing
- **Tokio** for async runtime

## 📄 License

Apache-2.0

## 📞 Support

For issues and questions:
- GitHub Issues: [fast-context/fast-context](https://github.com/fast-context/fast-context)
- Documentation: [https://docs.fast-context.dev](https://docs.fast-context.dev)

---

**Fast-Context** - Powering the next generation of code analysis tools.