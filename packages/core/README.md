# @fast-context/core

Enhanced TypeScript SDK for Fast-Context with streaming analysis, advanced querying, and comprehensive type safety.

## 🚀 Features

- **Streaming Analysis**: Progressive analysis with real-time progress tracking and cancellation support
- **Advanced Query Engine**: Semantic search, relationship analysis, and architectural pattern detection
- **Type Safety**: Strict TypeScript with runtime validation using Zod schemas
- **Configuration Management**: Environment-based configuration with validation and presets
- **Performance Monitoring**: Real-time memory and throughput tracking
- **Error Handling**: Comprehensive error types with Result<T, E> pattern

## 📦 Installation

```bash
npm install @fast-context/core
```

**Prerequisites:**
- Node.js 18+ 
- TypeScript 5.0+
- The main `fast-context` package (peer dependency)

## 🏁 Quick Start

### Basic Usage

```typescript
import { EnhancedFastContextAnalyzer, createAnalyzerFromPreset } from '@fast-context/core';

// Create analyzer with preset configuration
const analyzer = createAnalyzerFromPreset('balanced', './my-project');

// Stream analysis with progress tracking
for await (const progress of analyzer.analyzeStream()) {
  console.log(`Progress: ${progress.filesProcessed}/${progress.totalFiles} files`);
  console.log(`Phase: ${progress.phase}, Symbols: ${progress.symbolsFound}`);
}
```

### Advanced Query Engine

```typescript
// Get the query engine
const queryEngine = analyzer.getQueryEngine();

// Semantic symbol search
const symbolsResult = await queryEngine.findSymbols({
  text: 'user authentication',
  kind: 'function',
  maxResults: 10
});

if (symbolsResult.success) {
  console.log('Found symbols:', symbolsResult.data);
}

// Dependency analysis
const depsResult = await queryEngine.getSymbolDependencies('UserService', {
  depth: 3,
  includeTransitive: true
});

// Complexity analysis
const complexityResult = await queryEngine.analyzeComplexity({
  threshold: 10,
  includeMetrics: true
});

// Architectural pattern detection
const patternsResult = await queryEngine.detectPatterns();
```

### Configuration Management

```typescript
import { ConfigurationManager, createSmartConfig } from '@fast-context/core';

// Use preset configurations
const fastConfig = ConfigurationManager.getPreset('fast');
const balancedConfig = ConfigurationManager.getPreset('balanced');
const thoroughConfig = ConfigurationManager.getPreset('thorough');

// Load from environment variables
const envConfig = ConfigurationManager.loadFromEnvironment();

// Smart configuration based on project detection
const smartConfig = await createSmartConfig('./my-project');

// Validate configuration
const validation = ConfigurationManager.validate(myConfig);
if (!validation.success) {
  console.error('Invalid config:', validation.error.message);
}
```

### Error Handling with Result Types

```typescript
import { Result, Ok, Err, AnalysisError } from '@fast-context/core';

// All major operations return Result<T, E> for type-safe error handling
const analyzerResult = EnhancedFastContextAnalyzer.create(config);

if (analyzerResult.success) {
  const analyzer = analyzerResult.data;
  // Use analyzer safely
} else {
  const error = analyzerResult.error;
  console.error(`Error ${error.code}: ${error.message}`);
}
```

## 📋 Configuration Options

### AnalysisConfig

```typescript
interface AnalysisConfig {
  projectRoot: string;                    // Project directory path
  languages?: string[];                   // Languages to analyze (auto-detect if not specified)
  ignorePatterns?: string[];             // File patterns to ignore
  enableCaching?: boolean;               // Enable intelligent caching (default: true)
  cachePolicy?: 'auto' | 'minimal' | 'balanced' | 'adaptive' | 'persistent';
  enableWatching?: boolean;              // Enable file watching (default: false)
  maxFiles?: number;                     // Maximum files to analyze
  parallelProcessing?: boolean;          // Enable parallel processing (default: true)
  performance?: {
    maxMemoryMb?: number;                // Memory limit in MB (default: 1024)
    timeoutMs?: number;                  // Analysis timeout in ms (default: 30000)
    workerThreads?: number;              // Number of worker threads (default: 4)
    chunkSize?: number;                  // Processing chunk size (default: 100)
  };
}
```

### Performance Presets

| Preset | Use Case | Memory | Timeout | Cache Policy |
|--------|----------|--------|---------|--------------|
| `fast` | CI/CD, Quick Analysis | 256MB | 10s | minimal |
| `balanced` | Development, General Use | 512MB | 30s | adaptive |
| `thorough` | Production, Deep Analysis | 2GB | 2m | persistent |

## 🔍 Query Engine API

### Symbol Search

```typescript
// Semantic search
const symbols = await queryEngine.findSymbols({
  text: 'authentication logic',
  kind: 'function',
  language: 'typescript',
  maxResults: 20,
  similarity: 0.8
});

// Find similar code
const similar = await queryEngine.findSimilarCode(codeSnippet, 0.7);
```

### Relationship Analysis

```typescript
// Get symbol dependencies
const deps = await queryEngine.getSymbolDependencies('MyClass', {
  depth: 3,
  includeTransitive: true,
  direction: 'both'
});

// Find symbol usages
const usages = await queryEngine.getSymbolUsages('MyFunction');
```

### Code Quality Analysis

```typescript
// Complexity analysis
const complexity = await queryEngine.analyzeComplexity({
  threshold: 10,
  includeMetrics: true,
  sortBy: 'complexity'
});

// Find code smells
const smells = await queryEngine.findCodeSmells();

// Detect architectural patterns
const patterns = await queryEngine.detectPatterns();
```

## 📊 Streaming Analysis

### Progress Tracking

```typescript
for await (const progress of analyzer.analyzeStream()) {
  console.log(`Phase: ${progress.phase}`);
  console.log(`Files: ${progress.filesProcessed}/${progress.totalFiles}`);
  console.log(`Symbols: ${progress.symbolsFound}`);
  console.log(`Memory: ${progress.performance.memoryUsageMb}MB`);
  console.log(`Throughput: ${progress.performance.throughputFilesPerSecond} files/sec`);
  
  if (progress.performance.estimatedRemainingMs) {
    console.log(`ETA: ${progress.performance.estimatedRemainingMs}ms`);
  }
}
```

### Cancellation Support

```typescript
// Start analysis
const analysisPromise = analyzer.analyzeStream();

// Cancel after 5 seconds
setTimeout(() => {
  analyzer.cancel();
}, 5000);

try {
  for await (const progress of analysisPromise) {
    // Handle progress
  }
} catch (error) {
  if (error instanceof AnalysisCancelledError) {
    console.log('Analysis was cancelled');
  }
}
```

## 🛡️ Error Handling

### Error Types

- `AnalysisError`: Base error class for all analysis-related errors
- `ValidationError`: Configuration validation errors
- `AnalysisCancelledError`: Analysis cancellation errors
- `ConfigurationError`: Configuration loading/parsing errors

### Result Pattern

```typescript
// All major operations return Result<T, E>
const result = await queryEngine.findSymbols(query);

if (result.success) {
  // Type-safe access to data
  const symbols = result.data;
} else {
  // Type-safe error handling
  const error = result.error;
  console.error(`${error.code}: ${error.message}`);
}
```

## 🧪 Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Run with coverage
npm run test:coverage
```

## 📚 Examples

See the [examples](./examples/) directory for comprehensive usage examples:

- [Basic Usage](./examples/basic-usage.ts) - Core functionality demonstration
- [Streaming Analysis](./examples/streaming-analysis.ts) - Advanced streaming features
- [Query Engine](./examples/query-engine.ts) - Advanced querying capabilities
- [Configuration](./examples/configuration.ts) - Configuration management

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

Apache-2.0 - see [LICENSE](../../LICENSE) for details.

## 🔗 Related Components

- Root `fast-context` crate - Rust library, Node bindings, CLI, and MCP server
