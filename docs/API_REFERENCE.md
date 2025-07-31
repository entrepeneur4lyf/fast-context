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
  project_root: "/path/to/project",
  include_patterns: ["**/*.rs", "**/*.js", "**/*.ts"],
  exclude_patterns: ["target/**", "node_modules/**"],
  max_file_size: 1024 * 1024, // 1MB
  enable_caching: true,
  cache_ttl_seconds: 300,
  analysis_timeout_seconds: 30
});
```

## Configuration

### AnalyzerConfig

Configuration object for the analyzer.

```typescript
interface AnalyzerConfig {
  project_root: string;                    // Root directory of the project
  include_patterns: string[];              // Glob patterns for files to include
  exclude_patterns: string[];              // Glob patterns for files to exclude
  max_file_size: number;                   // Maximum file size in bytes
  follow_symlinks: boolean;                // Whether to follow symbolic links
  respect_gitignore: boolean;              // Whether to respect .gitignore files
  analysis_timeout_seconds: number;        // Timeout for analysis operations
  enable_caching: boolean;                 // Enable result caching
  cache_ttl_seconds: number;               // Cache time-to-live in seconds
  max_cache_size: number;                  // Maximum number of cached entries
  enable_incremental: boolean;             // Enable incremental analysis
  language_config: Record<string, any>;    // Language-specific configuration
}
```

**Default Values:**
- `max_file_size`: 5MB
- `follow_symlinks`: false
- `respect_gitignore`: true
- `analysis_timeout_seconds`: 60
- `enable_caching`: true
- `cache_ttl_seconds`: 300 (5 minutes)
- `max_cache_size`: 1000
- `enable_incremental`: true

## Symbol Search APIs

### find_symbols_by_kind

Search for symbols by their kind/type.

```typescript
find_symbols_by_kind(kind: string): QueryResult
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

**Returns:** QueryResult with matching symbols

**Example:**
```typescript
const functions = analyzer.find_symbols_by_kind("function");
console.log(`Found ${functions.symbols.length} functions`);

functions.symbols.forEach(symbol => {
  console.log(`${symbol.symbol.name} at ${symbol.file_path}:${symbol.symbol.location.start_line}`);
});
```

**Input Validation:**
- Kind must be non-empty and <= 500 characters
- Invalid characters are sanitized
- SQL injection and XSS attempts are blocked

## File Analysis APIs

### find_symbols_in_file

Find all symbols in a specific file.

```typescript
find_symbols_in_file(file_path: string): QueryResult
```

**Parameters:**
- `file_path`: string - Path to the file to analyze

**Returns:** QueryResult with symbols found in the file

**Example:**
```typescript
const symbols = analyzer.find_symbols_in_file("src/main.rs");
console.log(`File contains ${symbols.symbols.length} symbols`);

// Group symbols by kind
const byKind = symbols.symbols.reduce((acc, symbol) => {
  const kind = symbol.symbol.kind;
  acc[kind] = (acc[kind] || 0) + 1;
  return acc;
}, {});

console.log("Symbol distribution:", byKind);
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

### find_dependents

Find symbols that depend on the given symbol (who uses this symbol).

```typescript
find_dependents(symbol_name: string): QueryResult
```

**Parameters:**
- `symbol_name`: string - Name of the symbol to find dependents for

**Returns:** QueryResult with symbols that depend on the target symbol

**Features:**
- **Transitive Analysis**: Finds dependencies up to 5 levels deep
- **Cycle Detection**: Prevents infinite loops in dependency chains
- **Impact Analysis**: Includes complexity scoring and architectural insights

**Example:**
```typescript
const dependents = analyzer.find_dependents("DatabaseConnection");
console.log(`${dependents.symbols.length} symbols depend on DatabaseConnection`);

// Analyze impact
console.log(`Total complexity impact: ${dependents.context.complexity_score}`);
console.log(`Files affected: ${dependents.context.files_involved}`);

// Review suggestions
dependents.suggestions.forEach(suggestion => {
  console.log(`Suggestion: ${suggestion}`);
});
```

### find_dependencies

Find symbols that the given symbol depends on (what this symbol uses).

```typescript
find_dependencies(symbol_name: string): QueryResult
```

**Parameters:**
- `symbol_name`: string - Name of the symbol to find dependencies for

**Returns:** QueryResult with symbols that the target symbol depends on

**Features:**
- **Transitive Analysis**: Finds dependencies up to 5 levels deep
- **Dependency Chain Analysis**: Tracks complete dependency paths
- **Circular Dependency Detection**: Identifies potential circular references

**Example:**
```typescript
const dependencies = analyzer.find_dependencies("UserService");
console.log(`UserService depends on ${dependencies.symbols.length} symbols`);

// Check for potential issues
if (dependencies.context.potential_issues.length > 0) {
  console.log("Potential issues found:");
  dependencies.context.potential_issues.forEach(issue => {
    console.log(`- ${issue}`);
  });
}
```

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
  const result = analyzer.find_symbols_by_kind("function");
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
