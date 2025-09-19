# Fast-Context Go SDK

A high-performance Go SDK for intelligent codebase analysis powered by Rust core. Provides graph-powered code comprehension through Tree-sitter parsers and sophisticated dependency analysis.

## Features

- **Multi-Language Support**: 20+ programming languages (Rust, Python, JavaScript, TypeScript, Java, Go, C#, C++, Swift, Objective-C, PHP, Ruby, Scala, Zig, Dart, Lua, Bash, CSS, HTML, XML, JSON, YAML, Markdown)
- **Graph Algorithms**: Built-in graph operations and analysis algorithms
- **Streaming Analysis**: Real-time codebase analysis with progress tracking
- **Symbol Extraction**: Advanced symbol extraction and relationship mapping
- **Dependency Analysis**: Comprehensive dependency graph construction
- **Performance Optimized**: CGO-based Rust integration for maximum performance
- **Cross-Platform**: Linux, macOS, Windows support

## Installation

```bash
go get github.com/fast-context/go-sdk
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
    
    "github.com/fast-context/go-sdk/fastcontext"
)

func main() {
    // Create analyzer with default configuration
    analyzer, err := fastcontext.NewAnalyzer(
        fastcontext.WithProjectRoot("./my-project"),
    )
    if err != nil {
        log.Fatal(err)
    }
    
    // Analyze codebase
    result, err := analyzer.Analyze()
    if err != nil {
        log.Fatal(err)
    }
    
    fmt.Printf("Found %d files with %d symbols\n", 
        result.FileCount, result.SymbolCount)
}
```

## Documentation

- [API Reference](docs/api.md)
- [Examples](examples/)
- [Configuration Guide](docs/configuration.md)
- [Performance Guide](docs/performance.md)

## Requirements

- Go 1.19 or higher
- Rust toolchain (for building the native library)

## License

MIT License - see LICENSE file for details.