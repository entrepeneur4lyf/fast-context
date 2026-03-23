# Fast-Context API Reference

## Overview

Fast-Context provides a comprehensive codebase analysis engine with graph-powered code comprehension. This document covers all available APIs, their parameters, return values, and usage examples.

## Table of Contents

- [FastContextAnalyzer](#fastcontextanalyzer)
- [Configuration](#configuration)
- [Symbol Search APIs](#symbol-search-apis)
- [Dependency Analysis APIs](#dependency-analysis-apis)
- [File Analysis APIs](#file-analysis-apis)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)

## FastContextAnalyzer

The main class for codebase analysis operations.

### Constructor

```typescript
new FastContextAnalyzer(config: AnalyzerConfig): FastContextAnalyzer
```

Creates a new analyzer instance with the specified configuration.

**Parameters:**
- `config`: AnalyzerConfig - Configuration object (see [Configuration](#configuration))

**Returns:** FastContextAnalyzer instance

**Example:**
```typescript
const analyzer = new FastContextAnalyzer({
  projectRoot: "/path/to/project",
  languages: ["rust", "javascript", "typescript"],
  ignorePatterns: ["target/**", "node_modules/**"],
  enableCaching: true,
  enableWatching: false,
  maxFiles: 5000,
  parallelProcessing: true
});
```

## Configuration

### AnalyzerConfig

Configuration object for the analyzer.

```typescript
interface AnalyzerConfig {
  projectRoot: string
  languages?: string[]
  ignorePatterns?: string[]
  enableCaching?: boolean
  cachePolicy?: string
  enableWatching?: boolean
  maxFiles?: number
  parallelProcessing?: boolean
  enableExperimentalArchitecture?: boolean
}
```

**Default Values:**
- `languages`: all supported languages
- `ignorePatterns`: `node_modules/**`, `target/**`, `.git/**`
- `enableCaching`: optional
- `enableWatching`: optional
- `parallelProcessing`: `true`

## Symbol Search APIs

### findSymbolsByKind

Search for symbols by their kind/type.

```typescript
findSymbolsByKind(kind: string): string[]
```

**Parameters:**
- `kind`: string - Symbol kind to search for

**Supported Symbol Kinds:**
- `"function"` - Functions and methods
- `"class"` - Classes and types
- `"interface"` - Interfaces and traits
- `"module"` - Modules and namespaces
- `"variable"` - Variables and fields
- `"constant"` - Constants and static values
- `"enum"` - Enumerations
- `"struct"` - Structures and records
- `"trait"` - Traits and protocols
- `"type_alias"` - Type aliases and typedefs

**Returns:** Array of matching symbol names

**Example:**
```typescript
const functions = analyzer.findSymbolsByKind("function");
console.log(`Found ${functions.length} functions`);
```

**Input Validation:**
- Kind must be non-empty and <= 500 characters
- Invalid characters are sanitized
- SQL injection and XSS attempts are blocked

## File Analysis APIs

### findSymbolsInFile

Find all symbols in a specific file.

```typescript
findSymbolsInFile(filePath: string): string[]
```

**Parameters:**
- `filePath`: string - Path to the file to analyze, resolved relative to `projectRoot`

**Returns:** Array of `"<kind>: <name>"` entries

**Example:**
```typescript
const symbols = analyzer.findSymbolsInFile("src/main.rs");
console.log(`File contains ${symbols.length} exported symbol entries`);
```

**Security Features:**
- Path traversal protection (blocks `../`, `..\\`, etc.)
- Null byte injection protection
- Command injection protection
- File size limits enforced

**Caching:**
- Results are cached with LRU eviction
- Cache TTL: 5 minutes (configurable)
- Cache size: 100 entries (configurable)

## Dependency Analysis APIs

### findDependencies

Find symbols that the given symbol depends on (what this symbol uses).

```typescript
findDependencies(symbolName: string): string[]
```

**Parameters:**
- `symbolName`: string - Name of the symbol to find dependencies for

**Returns:** Array of formatted dependency strings

**Example:**
```typescript
const dependencies = analyzer.findDependencies("UserService");
console.log(`UserService depends on ${dependencies.length} symbols`);
```

### findComplexSymbols

Find files or symbols above a complexity threshold.

```typescript
findComplexSymbols(complexityThreshold: number): string[]
```

**Parameters:**
- `complexityThreshold`: number - Minimum complexity level

**Returns:** Array of formatted strings describing complex files/symbols

## Return Types

### QueryResult

Main result type for all query operations.

```typescript
interface QueryResult {
  symbols: SymbolInfo[];           // Array of matching symbols
  relationships: Relationship[];    // Relationships between symbols
  context: ContextInfo;            // Analysis context and metadata
  suggestions: string[];           // Optimization and improvement suggestions
}
```

### SymbolInfo

Information about a single symbol.

```typescript
interface SymbolInfo {
  symbol: Symbol;                  // Core symbol information
  file_path: string;              // File containing the symbol
  complexity: number;             // Cyclomatic complexity score
  dependencies: string[];         // Direct dependencies
  dependents: string[];          // Direct dependents
  related_files: string[];       // Related files
}
```

### ContextInfo

Analysis context and metadata.

```typescript
interface ContextInfo {
  total_symbols: number;          // Total number of symbols found
  files_involved: number;         // Number of files involved
  complexity_score: number;       // Average complexity score
  architectural_patterns: string[]; // Detected architectural patterns
  potential_issues: string[];     // Potential code issues
}
```

## Error Handling

All APIs use Result-based error handling. Possible error types:

### Common Errors

- **`InvalidInput`**: Invalid or malicious input detected
- **`FileNotFound`**: Specified file does not exist or is not accessible
- **`PathTraversal`**: Attempted path traversal attack blocked
- **`FileTooLarge`**: File exceeds maximum size limit
- **`AnalysisTimeout`**: Analysis operation timed out
- **`NotInitialized`**: Analyzer not properly initialized

### Error Handling Example

```typescript
try {
  const result = analyzer.findSymbolsByKind("function");
  // Process result
} catch (error) {
  if (error.message.includes("InvalidInput")) {
    console.error("Invalid input provided:", error);
  } else if (error.message.includes("AnalysisTimeout")) {
    console.error("Analysis timed out, try with smaller scope");
  } else {
    console.error("Unexpected error:", error);
  }
}
```

## Performance Considerations

### Caching Strategy

- **File Query Cache**: LRU cache with 5-minute TTL
- **Symbol Index Cache**: Persistent cache for symbol lookups
- **Dependency Cache**: Cached transitive dependency analysis

### Optimization Tips

1. **Use Specific Patterns**: Narrow include/exclude patterns improve performance
2. **Enable Caching**: Always enable caching for production use
3. **Set Reasonable Timeouts**: Balance thoroughness with responsiveness
4. **Incremental Analysis**: Enable incremental mode for large codebases
5. **File Size Limits**: Set appropriate file size limits to avoid memory issues

### Performance Benchmarks

Typical performance on modern hardware:

- **Symbol Search**: 10-100ms for medium codebases (10K-100K LOC)
- **File Analysis**: 1-50ms per file depending on size and complexity
- **Dependency Analysis**: 50-500ms depending on transitive depth
- **Cache Hit**: <1ms for cached results

### Memory Usage

- **Base Memory**: ~50MB for analyzer initialization
- **Per File**: ~1-10KB per analyzed file
- **Cache Memory**: ~100KB per 1000 cached entries
- **Peak Usage**: Typically 2-5x base memory during analysis
