# TypeScript Integration

Fast-Context provides comprehensive TypeScript support with auto-generated type definitions from Rust structs using `ts-rs`.

## Overview

All JavaScript interface types are automatically generated from Rust structs, ensuring:
- **Type Safety**: Compile-time checking of all API usage
- **Intellisense**: Full IDE support with auto-completion
- **Documentation**: JSDoc comments from Rust code
- **Consistency**: Types are always in sync with implementation

## Generated Types

The following types are automatically generated:

### Core Types
- `FastContextAnalyzer` - Main analyzer class
- `AnalyzerConfig` - Configuration options
- `AnalysisResultJs` - Analysis results
- `SymbolInfoJs` - Symbol information
- `QueryResultJs` - Query results

### File Watching Types
- `FileChangeEventJs` - Individual file change
- `FileChangeBatchJs` - Batch of file changes

### Query Types
- `QueryChunkJs` - Streaming query chunk
- `StreamingOptionsJs` - Streaming configuration
- `FilterOptionsJs` - Query filters
- `PaginationOptionsJs` - Pagination settings

### Export Types
- `ExportOptionsJs` - Export configuration
- `ContextInfoJs` - Context information

## Usage Examples

### Basic Configuration

```typescript
import { FastContextAnalyzer, AnalyzerConfig } from 'fast-context';

const config: AnalyzerConfig = {
    projectRoot: process.cwd(),
    languages: ['javascript', 'typescript', 'rust'],
    ignorePatterns: ['node_modules/**', 'target/**'],
    enableCaching: true,
    cachePolicy: 'adaptive',
    enableWatching: true,
    maxFiles: 10000,
    parallelProcessing: true
};

const analyzer = new FastContextAnalyzer(config);
```

### Type-Safe Analysis

```typescript
import { AnalysisResultJs, SymbolInfoJs } from 'fast-context';

const result: AnalysisResultJs = analyzer.analyze();

console.log(`Analyzed ${result.fileCount} files`);
console.log(`Found ${result.symbolCount} symbols`);
console.log(`Duration: ${result.durationMs}ms`);

if (result.memoryUsageMb) {
    console.log(`Memory: ${result.memoryUsageMb}MB`);
}
```

### File Watching with Types

```typescript
import { FileChangeBatchJs } from 'fast-context';

analyzer.startWatching((changeBatch: FileChangeBatchJs) => {
    console.log(`${changeBatch.changeCount} files changed`);
    console.log(`Impact level: ${changeBatch.impactLevel}`);
    
    changeBatch.changes.forEach(change => {
        console.log(`${change.changeType}: ${change.filePath}`);
        
        if (change.language) {
            console.log(`  Language: ${change.language}`);
        }
    });
    
    if (changeBatch.requiresReanalysis) {
        const newResult = analyzer.analyze();
        console.log(`Reanalyzed: ${newResult.symbolCount} symbols`);
    }
});
```

### Symbol Processing

```typescript
import { SymbolInfoJs } from 'fast-context';

function analyzeSymbols(symbols: SymbolInfoJs[]): void {
    // Type-safe filtering
    const functions = symbols.filter(s => s.kind === 'function');
    const complexSymbols = symbols.filter(s => s.complexity > 10);
    
    // Find most complex symbol
    const mostComplex = symbols.reduce((max, current) => 
        current.complexity > max.complexity ? current : max
    );
    
    console.log(`Most complex: ${mostComplex.name}`);
    console.log(`  Complexity: ${mostComplex.complexity}`);
    console.log(`  Location: ${mostComplex.filePath}:${mostComplex.startLine}`);
    
    if (mostComplex.signature) {
        console.log(`  Signature: ${mostComplex.signature}`);
    }
    
    if (mostComplex.documentation) {
        console.log(`  Docs: ${mostComplex.documentation}`);
    }
}
```

## Type Generation Process

Types are generated automatically during the build process:

1. **Rust Structs**: All `#[napi(object)]` structs with `#[derive(TS)]` and `#[ts(export)]`
2. **Test Execution**: `cargo test generate_typescript_types` generates raw TypeScript
3. **Post-Processing**: `scripts/update-types.js` cleans and formats types
4. **Integration**: Types are merged into `index.d.ts` with proper camelCase conversion

### Build Commands

```bash
# Generate types only
npm run generate-types

# Full build with type generation
npm run build

# Debug build with types
npm run build:debug
```

## Field Name Conversion

Rust snake_case fields are automatically converted to JavaScript camelCase:

| Rust Field | TypeScript Field |
|------------|------------------|
| `project_root` | `projectRoot` |
| `file_count` | `fileCount` |
| `symbol_count` | `symbolCount` |
| `start_line` | `startLine` |
| `change_type` | `changeType` |
| `memory_usage_mb` | `memoryUsageMb` |

## Type Guards

For runtime type safety, use type guards:

```typescript
function isValidAnalysisResult(result: any): result is AnalysisResultJs {
    return (
        typeof result === 'object' &&
        typeof result.fileCount === 'number' &&
        typeof result.symbolCount === 'number' &&
        typeof result.relationshipCount === 'number' &&
        Array.isArray(result.languages) &&
        typeof result.durationMs === 'number'
    );
}

function safeAnalyze(analyzer: FastContextAnalyzer): AnalysisResultJs | null {
    try {
        const result = analyzer.analyze();
        
        if (isValidAnalysisResult(result)) {
            return result;
        }
        
        console.error('Invalid analysis result structure');
        return null;
    } catch (error) {
        console.error('Analysis failed:', error);
        return null;
    }
}
```

## Advanced Usage

### Generic Type Helpers

```typescript
type AnalysisFields = keyof AnalysisResultJs;
type SymbolFields = keyof SymbolInfoJs;

// Extract specific fields
type BasicAnalysis = Pick<AnalysisResultJs, 'fileCount' | 'symbolCount' | 'durationMs'>;

// Optional configuration
type MinimalConfig = Partial<AnalyzerConfig> & Pick<AnalyzerConfig, 'projectRoot'>;

const minimalConfig: MinimalConfig = {
    projectRoot: '/path/to/project'
    // All other fields are optional
};
```

### Union Types for Enums

```typescript
// Symbol kinds (extend as needed)
type SymbolKind = 'function' | 'class' | 'variable' | 'constant' | 'method' | 'property';

// Change types for file watching
type ChangeType = 'created' | 'modified' | 'deleted' | 'renamed';

// Impact levels
type ImpactLevel = 'low' | 'medium' | 'high';

function handleChange(change: FileChangeEventJs): void {
    const changeType: ChangeType = change.changeType as ChangeType;
    
    switch (changeType) {
        case 'created':
            console.log(`New file: ${change.filePath}`);
            break;
        case 'modified':
            console.log(`Modified: ${change.filePath}`);
            break;
        case 'deleted':
            console.log(`Deleted: ${change.filePath}`);
            break;
        case 'renamed':
            console.log(`Renamed: ${change.oldPath} → ${change.filePath}`);
            break;
    }
}
```

## IDE Configuration

### VS Code

For optimal TypeScript support in VS Code:

```json
// .vscode/settings.json
{
    "typescript.preferences.includePackageJsonAutoImports": "on",
    "typescript.suggest.autoImports": true,
    "typescript.suggest.completeFunctionCalls": true,
    "typescript.inlayHints.parameterNames.enabled": "all",
    "typescript.inlayHints.variableTypes.enabled": true
}
```

### tsconfig.json

Recommended TypeScript configuration:

```json
{
    "compilerOptions": {
        "target": "ES2020",
        "module": "commonjs",
        "lib": ["ES2020"],
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true,
        "declaration": true,
        "declarationMap": true,
        "sourceMap": true
    },
    "include": [
        "src/**/*",
        "examples/**/*",
        "node_modules/fast-context/index.d.ts"
    ]
}
```

## Examples

See `examples/typescript-example.ts` for a comprehensive demonstration of all TypeScript features.

## Troubleshooting

### Types Not Found

```bash
# Regenerate types
npm run generate-types

# Check if types are in index.d.ts
grep -n "interface.*Js" index.d.ts
```

### Build Errors

```bash
# Clean and rebuild
rm -rf node_modules package-lock.json
npm install
npm run build
```

### IDE Not Recognizing Types

1. Restart TypeScript service in your IDE
2. Check `node_modules/fast-context/index.d.ts` exists
3. Verify tsconfig.json includes correct paths

## Contributing

When adding new Rust structs exposed to JavaScript:

1. Add `#[derive(TS)]` and `#[ts(export)]` to the struct
2. Add the struct to the test in `src/lib.rs`
3. Run `npm run generate-types` to update TypeScript definitions
4. Test the new types in TypeScript code

The type generation is fully automated - just follow the annotation pattern used by existing structs.