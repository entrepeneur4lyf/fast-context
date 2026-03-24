# Fast-Context API Reference

## Overview

This document describes the current public Node.js API exposed by the `fast-context` package.

The source of truth for the shipped TypeScript surface is:

- [index.d.ts](../index.d.ts)

## Package Exports

The package currently exports:

- `FastContextAnalyzer`
- `RustworkxGraph`
- `RustworkxDiGraph`
- `getVersion()`
- `getSupportedLanguages()`
- `detectLanguage(filePath)`
- `checkConfiguration(config?)`
- `getSystemInfo()`

## FastContextAnalyzer

Main entry point for codebase analysis.

### Constructor

```ts
new FastContextAnalyzer(config: AnalyzerConfig)
```

### AnalyzerConfig

```ts
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

Notes:

- `projectRoot` is required.
- `languages` defaults to auto-detecting all supported languages.
- `ignorePatterns` are added to the built-in project filters.
- `maxFiles: 0` means no limit.

### Methods

#### `analyze(): AnalysisResultJs`

Analyze the project rooted at `projectRoot`.

```ts
interface AnalysisResultJs {
  fileCount: number
  symbolCount: number
  relationshipCount: number
  languages: string[]
  durationMs: number
  memoryUsageMb?: number
  skippedFileCount: number
  skippedFiles: SkippedFileInfoJs[]
}
```

```ts
interface SkippedFileInfoJs {
  filePath: string
  stage: string
  reason: string
}
```

`skippedFiles` reports supported files that were skipped during read or parse, instead of silently omitting them.

#### `getAnalysis(): AnalysisResultJs | null`

Returns the last completed analysis result, or `null` if `analyze()` has not run yet.

#### `startWatching(): void`

Starts the project watcher.

#### `stopWatching(): void`

Stops the project watcher.

#### `findSymbolsByKind(kind: string): string[]`

Find symbols by kind, such as:

- `function`
- `class`
- `interface`
- `module`
- `variable`
- `enum`
- `struct`
- `trait`

#### `findSymbolsInFile(filePath: string): string[]`

Find symbols in one file.

Notes:

- `filePath` is resolved relative to `projectRoot`
- path traversal and invalid path input are rejected

#### `findDependencies(symbolName: string): string[]`

Returns the symbols directly depended on by the named symbol.

#### `findComplexSymbols(complexityThreshold: number): string[]`

Returns files or symbols above the supplied complexity threshold.

## Utility Functions

### `getVersion(): string`

Returns the package version.

### `getSupportedLanguages(): string[]`

Returns the list of supported languages.

### `detectLanguage(filePath: string): string | null`

Detects language from a file path or extension.

### `checkConfiguration(config?: AnalyzerConfig | null): string`

Returns validation output for a candidate analyzer config.

### `getSystemInfo(): string`

Returns basic environment and runtime information.

## Graph APIs

Two graph implementations are exported:

- `RustworkxGraph` for undirected graphs
- `RustworkxDiGraph` for directed graphs

Shared capabilities include:

- `addNode`
- `addEdge`
- `removeNode`
- `removeEdge`
- `nodeCount`
- `edgeCount`
- `neighbors`
- shortest-path helpers
- traversal helpers

`RustworkxDiGraph` also includes:

- `predecessors`
- `successors`

See [index.d.ts](../index.d.ts) for the full method list and signatures.

## Usage Example

```ts
import { FastContextAnalyzer, getSupportedLanguages, getVersion } from 'fast-context'

const analyzer = new FastContextAnalyzer({
  projectRoot: process.cwd(),
  ignorePatterns: ['coverage/**'],
  maxFiles: 5000,
  parallelProcessing: true,
})

const result = analyzer.analyze()

console.log({
  version: getVersion(),
  supportedLanguages: getSupportedLanguages().length,
  files: result.fileCount,
  symbols: result.symbolCount,
  skipped: result.skippedFileCount,
})
```

## Notes

- The Node.js API returns simple arrays for symbol/dependency queries rather than a rich `QueryResult` object.
- If you update the Rust/N-API surface, regenerate typings before publishing so this document stays aligned with [index.d.ts](../index.d.ts).
