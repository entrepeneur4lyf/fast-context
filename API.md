# Fast-Context API Reference

Complete API documentation for Fast-Context codebase analysis engine.

## Table of Contents

- [FastContextAnalyzer Class](#fastcontextanalyzer-class)
- [Configuration Objects](#configuration-objects)
- [Result Objects](#result-objects)
- [Error Handling](#error-handling)
- [Advanced APIs](#advanced-apis)
- [Streaming APIs](#streaming-apis)

## FastContextAnalyzer Class

The main class for performing codebase analysis and file monitoring.

### Constructor

```javascript
new FastContextAnalyzer(config)
```

**Parameters:**
- `config` (Object): Configuration options
  - `projectRoot` (string): Absolute path to project root directory
  - `ignorePatterns` (string[], optional): Glob patterns to ignore
  - `languageFilters` (string[], optional): Specific languages to analyze
  - `maxFileSize` (number, optional): Maximum file size in bytes (default: 1MB)
  - `parseTimeout` (number, optional): Parser timeout in milliseconds (default: 5000)
  - `enableCaching` (boolean, optional): Enable intelligent caching (default: true)

**Example:**
```javascript
const analyzer = new FastContextAnalyzer({
    projectRoot: '/path/to/project',
    ignorePatterns: ['node_modules/**', 'target/**', '.git/**'],
    languageFilters: ['javascript', 'typescript', 'rust'],
    maxFileSize: 2 * 1024 * 1024, // 2MB
    parseTimeout: 10000, // 10 seconds
    enableCaching: true
});
```

### Core Analysis Methods

#### `analyze(options?): AnalysisResult`

Performs comprehensive codebase analysis.

**Parameters:**
- `options` (Object, optional): Analysis options
  - `forceRefresh` (boolean): Bypass cache and force full analysis
  - `includeTests` (boolean): Include test files in analysis
  - `maxSymbols` (number): Maximum symbols to analyze (for large codebases)

**Returns:** [AnalysisResult](#analysisresult)

**Example:**
```javascript
const result = analyzer.analyze({
    forceRefresh: true,
    includeTests: false,
    maxSymbols: 10000
});

console.log(`Analyzed ${result.fileCount} files`);
console.log(`Found ${result.symbolCount} symbols`);
console.log(`Analysis completed in ${result.durationMs}ms`);
```

### File Watching Methods

#### `startWatching(callback, options?): void`

Starts real-time file monitoring with callback notifications.

**Parameters:**
- `callback` (function): Called when file changes are detected
  - `changeBatch` ([FileChangeBatch](#filechangebatch)): Batch of file changes
- `options` (Object, optional): Watching options
  - `debounceMs` (number): Debounce delay in milliseconds (default: 500)
  - `batchSize` (number): Maximum changes per batch (default: 100)
  - `watchTests` (boolean): Watch test files (default: false)

**Example:**
```javascript
analyzer.startWatching((changeBatch) => {
    console.log(`${changeBatch.changeCount} files changed`);
    console.log(`Impact level: ${changeBatch.impactLevel}`);
    
    if (changeBatch.requiresReanalysis) {
        const newResult = analyzer.analyze();
        console.log(`Reanalysis complete: ${newResult.symbolCount} symbols`);
    }
}, {
    debounceMs: 1000,
    batchSize: 50,
    watchTests: true
});
```

#### `stopWatching(): void`

Stops file monitoring and cleanup resources.

**Example:**
```javascript
analyzer.stopWatching();
```

### Symbol Query Methods

#### `findSymbolsByKind(kind, options?): Promise<Symbol[]>`

Find all symbols of a specific kind.

**Parameters:**
- `kind` (string): Symbol kind ('function', 'class', 'variable', 'interface', etc.)
- `options` (Object, optional): Query options
  - `limit` (number): Maximum results to return
  - `includePrivate` (boolean): Include private symbols
  - `sortBy` ('name' | 'complexity' | 'location'): Sort order

**Returns:** Promise<[Symbol[]](#symbol)>

**Example:**
```javascript
const functions = await analyzer.findSymbolsByKind('function', {
    limit: 100,
    includePrivate: false,
    sortBy: 'complexity'
});

functions.forEach(fn => {
    console.log(`${fn.name}: complexity ${fn.complexity}`);
});
```

#### `findSymbolsInFile(filePath, options?): Promise<Symbol[]>`

Find all symbols in a specific file.

**Parameters:**
- `filePath` (string): Path to file (relative to project root)
- `options` (Object, optional): Query options
  - `includeImports` (boolean): Include import symbols
  - `minComplexity` (number): Minimum complexity threshold

**Returns:** Promise<[Symbol[]](#symbol)>

**Example:**
```javascript
const symbols = await analyzer.findSymbolsInFile('src/main.js', {
    includeImports: true,
    minComplexity: 5
});
```

#### `findDependencies(symbolName, options?): Promise<Dependency[]>`

Find dependencies of a specific symbol.

**Parameters:**
- `symbolName` (string): Name or qualified name of symbol
- `options` (Object, optional): Query options
  - `depth` (number): Maximum dependency depth (default: 3)
  - `includeExternal` (boolean): Include external dependencies

**Returns:** Promise<[Dependency[]](#dependency)>

**Example:**
```javascript
const deps = await analyzer.findDependencies('MyClass', {
    depth: 2,
    includeExternal: true
});
```

#### `findComplexSymbols(threshold, options?): Promise<Symbol[]>`

Find symbols above a complexity threshold.

**Parameters:**
- `threshold` (number): Minimum complexity score
- `options` (Object, optional): Query options
  - `limit` (number): Maximum results
  - `sortBy` ('complexity' | 'name'): Sort order

**Returns:** Promise<[Symbol[]](#symbol)>

**Example:**
```javascript
const complexSymbols = await analyzer.findComplexSymbols(15, {
    limit: 20,
    sortBy: 'complexity'
});
```

### Streaming APIs

#### `findSymbolsStreaming(pattern, options, callback): void`

Stream symbol search results in chunks for large codebases.

**Parameters:**
- `pattern` (string): Search pattern (regex supported)
- `options` (Object): Streaming options
  - `chunkSize` (number): Symbols per chunk (default: 1000)
  - `includeProgress` (boolean): Include progress information
  - `filter` (Object): Additional filtering options
- `callback` (function): Called for each chunk
  - `chunk` ([SymbolChunk](#symbolchunk)): Chunk of symbols with metadata

**Example:**
```javascript
analyzer.findSymbolsStreaming(
    'test.*',
    {
        chunkSize: 500,
        includeProgress: true,
        filter: { kind: 'function', minComplexity: 5 }
    },
    (chunk) => {
        console.log(`Chunk ${chunk.chunkIndex}: ${chunk.symbols.length} symbols`);
        console.log(`Progress: ${chunk.progress}%`);
        
        chunk.symbols.forEach(symbol => {
            console.log(`  ${symbol.name} (${symbol.kind})`);
        });
    }
);
```

## Configuration Objects

### AnalysisOptions

```typescript
interface AnalysisOptions {
    forceRefresh?: boolean;     // Bypass cache
    includeTests?: boolean;     // Include test files
    maxSymbols?: number;        // Limit for large codebases
    timeout?: number;           // Analysis timeout (ms)
}
```

### WatchingOptions

```typescript
interface WatchingOptions {
    debounceMs?: number;        // Debounce delay
    batchSize?: number;         // Max changes per batch
    watchTests?: boolean;       // Monitor test files
    watchConfig?: boolean;      // Monitor config files
}
```

## Result Objects

### AnalysisResult

Main result object returned by analysis operations.

```typescript
interface AnalysisResult {
    fileCount: number;              // Total files analyzed
    symbolCount: number;            // Total symbols found
    relationshipCount: number;      // Total relationships
    languages: string[];            // Programming languages detected
    durationMs: number;             // Analysis duration
    memoryUsageMb: number;         // Memory usage during analysis
    cacheHitRate?: number;         // Cache effectiveness (0-1)
    errors?: AnalysisError[];      // Non-fatal errors encountered
}
```

### FileChangeBatch

Batch of file changes from file watching.

```typescript
interface FileChangeBatch {
    changes: FileChange[];          // Individual file changes
    changeCount: number;            // Number of changes
    impactLevel: 'low' | 'medium' | 'high';  // Estimated impact
    requiresReanalysis: boolean;    // Whether full reanalysis needed
    batchTimestamp: number;         // Unix timestamp
    affectedLanguages: string[];    // Languages affected by changes
}
```

### FileChange

Individual file change event.

```typescript
interface FileChange {
    changeType: 'created' | 'modified' | 'deleted' | 'renamed';
    filePath: string;               // Path to changed file
    language?: string;              // Detected programming language
    affectsAnalysis: boolean;       // Whether change affects analysis
    timestamp: number;              // Unix timestamp
    previousPath?: string;          // For renamed files
}
```

### Symbol

Represents a code symbol (function, class, variable, etc.).

```typescript
interface Symbol {
    id: string;                     // Unique identifier
    name: string;                   // Symbol name
    qualifiedName: string;          // Fully qualified name
    kind: SymbolKind;              // Type of symbol
    filePath: string;              // Source file path
    language: string;              // Programming language
    location: Location;            // Position in file
    complexity: number;            // Cyclomatic complexity
    signature?: string;            // Function/method signature
    documentation?: string;        // Associated documentation
    scopeChain: string[];          // Scope hierarchy
    modifiers: string[];           // Access modifiers, keywords
    dependencies: string[];        // Direct dependencies
    dependents: string[];          // Symbols that depend on this
}
```

### Location

Source code location information.

```typescript
interface Location {
    startLine: number;             // Starting line number (1-based)
    startColumn: number;           // Starting column (0-based)
    endLine: number;               // Ending line number
    endColumn: number;             // Ending column
}
```

### Dependency

Represents a dependency relationship between symbols.

```typescript
interface Dependency {
    fromSymbol: string;            // Source symbol ID
    toSymbol: string;              // Target symbol ID
    dependencyType: DependencyType; // Type of dependency
    filePath: string;              // File where dependency occurs
    location: Location;            // Location of dependency
    confidence: number;            // Confidence score (0-1)
}
```

### SymbolChunk

Chunk of symbols for streaming operations.

```typescript
interface SymbolChunk {
    symbols: Symbol[];             // Symbols in this chunk
    chunkIndex: number;            // Zero-based chunk index
    totalChunks: number;           // Total number of chunks
    progress: number;              // Progress percentage (0-100)
    isLast: boolean;               // Whether this is the final chunk
}
```

## Enums

### SymbolKind

```typescript
enum SymbolKind {
    Function = 'function',
    Class = 'class',
    Interface = 'interface',
    Variable = 'variable',
    Constant = 'constant',
    Method = 'method',
    Property = 'property',
    Field = 'field',
    Parameter = 'parameter',
    Enum = 'enum',
    EnumMember = 'enum_member',
    Struct = 'struct',
    Union = 'union',
    Typedef = 'typedef',
    Macro = 'macro',
    Module = 'module',
    Namespace = 'namespace',
    Import = 'import',
    Export = 'export'
}
```

### DependencyType

```typescript
enum DependencyType {
    Calls = 'calls',
    Imports = 'imports',
    Inherits = 'inherits',
    Implements = 'implements',
    Uses = 'uses',
    Contains = 'contains',
    References = 'references',
    Returns = 'returns',
    Throws = 'throws'
}
```

## Error Handling

### AnalysisError

```typescript
interface AnalysisError {
    type: 'parse_error' | 'file_error' | 'timeout' | 'memory_limit';
    message: string;
    filePath?: string;
    lineNumber?: number;
    severity: 'warning' | 'error';
}
```

### Common Error Scenarios

```javascript
try {
    const result = analyzer.analyze();
    
    // Check for errors
    if (result.errors && result.errors.length > 0) {
        result.errors.forEach(error => {
            if (error.severity === 'error') {
                console.error(`Error in ${error.filePath}: ${error.message}`);
            } else {
                console.warn(`Warning in ${error.filePath}: ${error.message}`);
            }
        });
    }
} catch (error) {
    // Handle fatal errors
    console.error('Analysis failed:', error.message);
}
```

## Performance Considerations

### Memory Management

For large codebases, use these strategies:

```javascript
// Use streaming for large symbol queries
analyzer.findSymbolsStreaming('.*', {
    chunkSize: 1000,
    includeProgress: true
}, (chunk) => {
    // Process chunk immediately
    processSymbols(chunk.symbols);
    
    // Optional: Clear processed data
    chunk.symbols = null;
});

// Limit analysis scope
const result = analyzer.analyze({
    maxSymbols: 50000,
    includeTests: false
});
```

### Caching

Enable intelligent caching for better performance:

```javascript
const analyzer = new FastContextAnalyzer({
    projectRoot: '/path/to/project',
    enableCaching: true
});

// First analysis builds cache
const result1 = analyzer.analyze();

// Subsequent analyses use cache
const result2 = analyzer.analyze(); // Much faster
```

### File Watching Optimization

Configure file watching for optimal performance:

```javascript
analyzer.startWatching(callback, {
    debounceMs: 1000,      // Batch rapid changes
    batchSize: 50,         // Reasonable batch size
    watchTests: false      // Exclude test files
});
```

## Language Support

Fast-Context supports 20+ programming languages with varying levels of analysis depth:

### Fully Supported Languages
- **JavaScript/TypeScript**: Complete symbol extraction, dependency analysis
- **Rust**: Full support including traits, impl blocks, macros  
- **Python**: Classes, functions, imports, decorators
- **Java**: Classes, methods, interfaces, annotations
- **Go**: Functions, structs, interfaces, packages

### Partially Supported Languages  
- **C/C++**: Functions, classes, headers
- **C#**: Classes, methods, properties
- **Swift**: Classes, functions, protocols
- **PHP**: Classes, functions, namespaces
- **Ruby**: Classes, methods, modules

### Basic Support
- **HTML/CSS**: Structure analysis
- **JSON/YAML**: Configuration analysis  
- **Markdown**: Documentation structure
- **Shell Scripts**: Function detection

## Migration Guide

### From rustworkx v0.15.x to v0.16.x

Breaking changes:
- `analyze()` now returns `AnalysisResult` instead of raw data
- File watching callback signature changed to include `FileChangeBatch`
- Constructor requires object parameter instead of positional arguments

Migration example:
```javascript
// Old (v0.15.x)
const analyzer = new FastContextAnalyzer('/path/to/project', ['node_modules/**']);
const result = analyzer.analyze();
console.log(result.symbols.length);

// New (v0.16.x)
const analyzer = new FastContextAnalyzer({
    projectRoot: '/path/to/project',
    ignorePatterns: ['node_modules/**']
});
const result = analyzer.analyze();
console.log(result.symbolCount);
```