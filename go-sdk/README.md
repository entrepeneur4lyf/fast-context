# Fast-Context Go SDK

[![Go Reference](https://pkg.go.dev/badge/github.com/fast-context/go-sdk.svg)](https://pkg.go.dev/github.com/fast-context/go-sdk)
[![Build Status](https://github.com/fast-context/go-sdk/workflows/CI/badge.svg)](https://github.com/fast-context/go-sdk/actions)
[![Coverage Status](https://coveralls.io/repos/github/fast-context/go-sdk/badge.svg)](https://coveralls.io/github/fast-context/go-sdk)
[![Go Report Card](https://goreportcard.com/badge/github.com/fast-context/go-sdk)](https://goreportcard.com/report/github.com/fast-context/go-sdk)

Fast-Context Go SDK is an intelligent codebase analysis engine that provides comprehensive code comprehension through graph-powered dependency analysis and multi-language symbol extraction.

## Features

- **Multi-Language Support**: Analyze code written in 20+ programming languages
- **Graph-Powered Analysis**: Advanced dependency graph algorithms and visualization
- **Symbol Extraction**: Detailed symbol information with complexity metrics
- **Streaming Analysis**: Real-time analysis with progress tracking for large codebases
- **File Watching**: Automatic re-analysis on file changes
- **Export Capabilities**: Multiple export formats (JSON, YAML, XML, GraphML, DOT, CSV, Markdown)
- **Comprehensive CLI**: Full-featured command-line interface
- **Structured Logging**: Advanced logging with metrics collection
- **Configuration Management**: Flexible configuration with YAML/JSON/TOML support

## Installation

### Go Module

```bash
go get github.com/fast-context/go-sdk
```

### CLI Installation

```bash
# Install CLI tool
go install github.com/fast-context/go-sdk/cmd/fast-context@latest

# Or build from source
git clone https://github.com/fast-context/go-sdk.git
cd go-sdk
make build
```

## Quick Start

### Basic Usage

```go
package main

import (
	"fmt"
	"log"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
)

func main() {
	// Create analyzer with default configuration
	analyzer, err := fastcontext.NewAnalyzer()
	if err != nil {
		log.Fatal(err)
	}

	// Analyze current directory
	result, err := analyzer.Analyze()
	if err != nil {
		log.Fatal(err)
	}

	// Print summary
	fmt.Printf("Analyzed %d files, found %d symbols\n", 
		result.FileCount, result.SymbolCount)
	fmt.Printf("Languages: %v\n", result.Languages)
}
```

### With Custom Configuration

```go
// Create custom configuration
cfg, err := config.NewConfig(
	config.WithProjectRoot("/path/to/project"),
	config.WithLanguages([]string{"Go", "Rust", "Python"}),
	config.WithMaxMemory(2048),
	config.WithTimeout(600),
	config.WithParallelProcessing(true),
)
if err != nil {
	log.Fatal(err)
}

// Create analyzer with custom config
analyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)
if err != nil {
	log.Fatal(err)
}

result, err := analyzer.Analyze()
```

### Preset Configurations

```go
// Fast analysis (speed over depth)
fastCfg, err := config.FastConfig("/path/to/project")

// Balanced analysis (recommended)
balancedCfg, err := config.BalancedConfig("/path/to/project")

// Thorough analysis (depth over speed)
thoroughCfg, err := config.ThoroughConfig("/path/to/project")
```

## CLI Usage

### Basic Commands

```bash
# Analyze current directory
fast-context analyze

# Analyze specific project
fast-context analyze /path/to/project

# Export results to JSON
fast-context analyze -o results.json

# Use specific preset
fast-context analyze --preset fast

# Analyze with custom memory limit
fast-context analyze --memory 2048 --timeout 600
```

### Finding Symbols

```bash
# Find all functions
fast-context symbols --kind function

# Find symbols in specific file
fast-context symbols --file src/main.go

# Find complex symbols (complexity > 10)
fast-context symbols --complexity 10

# Find symbols matching pattern
fast-context symbols --pattern ".*Service.*"
```

### Dependency Analysis

```bash
# Find dependencies for a symbol
fast-context dependencies "MyFunction"

# Show analysis complexity
fast-context complexity

# Find code patterns
fast-context patterns ".*Error.*"
```

### Export and Configuration

```bash
# Export to different formats
fast-context export json -o analysis.json
fast-context export yaml -o analysis.yaml
fast-context export graphml -o dependencies.graphml
fast-context export dot -o graph.dot
fast-context export markdown -o report.md

# Configuration management
fast-context config --show           # Show current config
fast-context config --validate       # Validate config
fast-context config --init           # Initialize config file
fast-context config --preset fast   # Apply fast preset
```

### File Watching

```bash
# Watch project for changes
fast-context watch

# Watch for 5 minutes
fast-context watch 300

# Watch specific directory
fast-context watch --project /path/to/project
```

## Advanced Features

### Streaming Analysis

```go
// Create streaming analyzer for large projects
streamingAnalyzer := streaming.NewAnalyzer(cfg)

// Set up progress callback
progressCallback := func(p *fastcontext.Progress) {
	fmt.Printf("Progress: %.1f%% - %s\n", p.Percentage, p.Message)
}

// Analyze with streaming
resultChan := streamingAnalyzer.AnalyzeStream(progressCallback)

// Process results as they come
for partialResult := range resultChan {
	fmt.Printf("Partial result: %d symbols processed\n", 
		partialResult.SymbolCount)
}
```

### File Watching

```go
// Start watching for file changes
err := analyzer.StartWatching(func(p *fastcontext.Progress) {
	fmt.Printf("Analysis progress: %.1f%%\n", p.Percentage)
})
if err != nil {
	log.Fatal(err)
}

// Stop watching when done
defer analyzer.StopWatching()

// File changes will trigger automatic re-analysis
```

### Export Functionality

```go
import "github.com/fast-context/go-sdk/export"

// Create exporter with custom options
exporter := export.NewExporter(analyzer,
	export.WithFormat(export.FormatGraphML),
	export.WithOutputFile("dependencies.graphml"),
	export.WithIndent(true),
	export.WithIncludeMetrics(true),
)

// Export analysis results
err := exporter.ExportToFile(result)
if err != nil {
	log.Fatal(err)
}

// Or get export data directly
data, err := exporter.Export(result)
```

### Query Engine

```go
import "github.com/fast-context/go-sdk/query"

// Create query engine
engine := query.NewEngine(analyzer)

// Semantic search
query := &query.SemanticQuery{
	Query:       "user authentication",
	Languages:   []string{"Go"},
	SymbolKinds:  []query.SymbolKindFunction,
	MaxResults:  10,
}

results, err := engine.FindSymbols(context.Background(), query)
if err != nil {
	log.Fatal(err)
}

for _, result := range results.Symbols {
	fmt.Printf("Found: %s (%s:%d)\n", result.Name, result.File, result.LineStart)
}
```

### Graph Algorithms

```go
import "github.com/fast-context/go-sdk/graph"

// Create graph from analysis results
g := graph.NewGraph()

// Add nodes and edges from analysis results
for _, symbol := range result.Symbols {
	g.AddNode(symbol.ID, symbol.Name)
}

for _, dep := range result.Dependencies {
	g.AddEdge(dep.From, dep.To, dep.Strength)
}

// Find shortest path
path, err := graph.DijkstraShortestPath(g, "node1", "node2")
if err != nil {
	log.Fatal(err)
}
fmt.Printf("Shortest path: %v\n", path.Path)

// Calculate centrality measures
centrality := graph.BetweennessCentrality(g)
for node, score := range centrality {
	fmt.Printf("Node %s centrality: %.3f\n", node, score)
}
```

## Configuration

### Configuration Files

Fast-Context supports configuration files in YAML, JSON, and TOML formats:

#### YAML Configuration
```yaml
# .fast-context.yaml
projectRoot: "."
languages:
  - "Go"
  - "Rust"
  - "Python"
  - "JavaScript"

ignorePatterns:
  - "node_modules/**"
  - "target/**"
  - "build/**"
  - "dist/**"
  - "*.min.js"
  - "__pycache__/**"
  - ".git/**"

performance:
  maxMemoryMB: 1024
  maxConcurrentFiles: 50
  timeoutSeconds: 300
  cachePolicy: "balanced"
  enableParallel: true
  enableStreaming: true
  enableWatching: false
  analysisDepth: 3

enableProgress: true
enableMetrics: true
logLevel: "info"
maxFileSizeKB: 1024
maxFiles: 10000
```

#### JSON Configuration
```json
{
  "projectRoot": ".",
  "languages": ["Go", "Rust", "Python"],
  "ignorePatterns": [
    "node_modules/**",
    "target/**",
    "build/**"
  ],
  "performance": {
    "maxMemoryMB": 1024,
    "timeoutSeconds": 300,
    "cachePolicy": "balanced"
  },
  "enableProgress": true,
  "logLevel": "info"
}
```

### Environment Variables

```bash
export FAST_CONTEXT_PROJECT_ROOT="/path/to/project"
export FAST_CONTEXT_LOG_LEVEL="debug"
export FAST_CONTEXT_MAX_MEMORY_MB="2048"
export FAST_CONTEXT_TIMEOUT_SECONDS="600"
export FAST_CONTEXT_CACHE_POLICY="aggressive"
export FAST_CONTEXT_ENABLE_PARALLEL="true"
export FAST_CONTEXT_ENABLE_PROGRESS="false"
```

## Supported Languages

Fast-Context supports analysis of code written in the following languages:

- **Systems Languages**: Rust, C, C++, Go, Zig
- **Web Development**: JavaScript, TypeScript, HTML, CSS
- **Enterprise**: Java, C#, Scala, Swift, Objective-C
- **Scripting**: Python, Ruby, PHP, Lua, Bash
- **Data Formats**: JSON, YAML, XML, Markdown

## API Reference

### Core Types

#### `fastcontext.Analyzer`
Main interface for codebase analysis.

```go
type Analyzer struct {
    // contains filtered or unexported fields
}

func NewAnalyzer(opts ...config.ConfigOption) (*Analyzer, error)
func NewAnalyzerWithConfig(cfg *config.Config) (*Analyzer, error)
func (a *Analyzer) Analyze() (*AnalysisResult, error)
func (a *Analyzer) FindSymbolsByKind(kind SymbolKind) ([]*Symbol, error)
func (a *Analyzer) FindDependencies(symbolName string) ([]*Dependency, error)
func (a *Analyzer) StartWatching(callback func(*Progress)) error
func (a *Analyzer) StopWatching() error
```

#### `fastcontext.AnalysisResult`
Contains the results of codebase analysis.

```go
type AnalysisResult struct {
    FileCount         int                `json:"fileCount"`
    SymbolCount       int                `json:"symbolCount"`
    RelationshipCount int                `json:"relationshipCount"`
    Symbols           []*Symbol          `json:"symbols"`
    Dependencies      []*Dependency      `json:"dependencies"`
    Languages         []string           `json:"languages"`
    DurationMs        int64              `json:"durationMs"`
    MemoryUsed        int64              `json:"memoryUsed"`
    Progress          []Progress         `json:"progress,omitempty"`
    Metadata          map[string]interface{} `json:"metadata,omitempty"`
}
```

#### `fastcontext.Symbol`
Represents a code symbol with metadata.

```go
type Symbol struct {
    ID            string      `json:"id"`
    Name          string      `json:"name"`
    Kind          SymbolKind  `json:"kind"`
    Language      string      `json:"language"`
    File          string      `json:"file"`
    LineStart     int         `json:"lineStart"`
    LineEnd       int         `json:"lineEnd"`
    Complexity    float64     `json:"complexity"`
    IsPublic      bool        `json:"isPublic"`
    IsExported    bool        `json:"isExported"`
    IsTest        bool        `json:"isTest"`
    IsDeprecated  bool        `json:"isDeprecated"`
    Documentation string      `json:"documentation,omitempty"`
    Children      []*Symbol   `json:"children,omitempty"`
    Properties    map[string]interface{} `json:"properties,omitempty"`
    Tags          []string    `json:"tags,omitempty"`
}
```

### Configuration Types

#### `config.Config`
Configuration for the Fast-Context analyzer.

```go
type Config struct {
    ProjectRoot       string            `json:"projectRoot"`
    Languages         []string          `json:"languages"`
    IgnorePatterns    []string          `json:"ignorePatterns"`
    IncludePatterns   []string          `json:"includePatterns"`
    Performance       PerformanceConfig `json:"performance"`
    EnableProgress    bool              `json:"enableProgress"`
    EnableMetrics     bool              `json:"enableMetrics"`
    LogLevel          string            `json:"logLevel"`
    MaxFileSizeKB     int               `json:"maxFileSizeKB"`
    MaxFiles          int               `json:"maxFiles"`
}

// Configuration options
func WithProjectRoot(root string) ConfigOption
func WithLanguages(languages []string) ConfigOption
func WithMaxMemory(mb int) ConfigOption
func WithTimeout(seconds int) ConfigOption
func WithParallelProcessing(enabled bool) ConfigOption
func WithLogLevel(level string) ConfigOption
```

### Export Types

#### `export.Exporter`
Handles exporting analysis results to various formats.

```go
type Exporter struct {
    // contains filtered or unexported fields
}

func NewExporter(analyzer *fastcontext.Analyzer, opts ...func(*Options)) *Exporter
func (e *Exporter) Export(result *fastcontext.AnalysisResult) ([]byte, error)
func (e *Exporter) ExportToFile(result *fastcontext.AnalysisResult) error

// Export formats
type Format int
const (
    FormatJSON Format = iota
    FormatYAML
    FormatXML
    FormatGraphML
    FormatDOT
    FormatCSV
    FormatMarkdown
)
```

## Performance Considerations

### Memory Usage

Fast-Context is designed to be memory-efficient, but large codebases may require tuning:

```go
// For large codebases (>100K files)
cfg, err := config.NewConfig(
    config.WithMaxMemory(4096),        // 4GB memory limit
    config.WithMaxConcurrentFiles(25),  // Reduce concurrency
    config.WithCachePolicy(config.CachePolicyPersistent),
)
```

### Parallel Processing

Control parallel processing based on your system capabilities:

```go
// High-performance server (many cores)
cfg, err := config.NewConfig(
    config.WithMaxConcurrentFiles(100),
    config.WithParallelProcessing(true),
)

// Limited resource environment
cfg, err := config.NewConfig(
    config.WithMaxConcurrentFiles(10),
    config.WithParallelProcessing(false),
)
```

### Caching Strategies

Choose appropriate caching strategy for your use case:

```go
// Development: frequent changes, minimal caching
cfg, err := config.NewConfig(
    config.WithCachePolicy(config.CachePolicyMinimal),
)

// CI/CD: one-time analysis, aggressive caching
cfg, err := config.NewConfig(
    config.WithCachePolicy(config.CachePolicyAggressive),
)
```

## Error Handling

Fast-Context provides comprehensive error handling:

```go
result, err := analyzer.Analyze()
if err != nil {
    // Check specific error types
    var fastErr *fastcontext.FastContextError
    if errors.As(err, &fastErr) {
        switch fastErr.Code {
        case fastcontext.ErrInvalidProjectRoot:
            log.Fatal("Invalid project root:", fastErr.Message)
        case fastcontext.ErrAnalysisTimeout:
            log.Fatal("Analysis timeout:", fastErr.Message)
        case fastcontext.ErrMemoryLimitExceeded:
            log.Fatal("Memory limit exceeded:", fastErr.Message)
        default:
            log.Fatal("Analysis error:", fastErr.Message)
        }
    } else {
        log.Fatal("Unexpected error:", err)
    }
}
```

## Logging and Metrics

### Structured Logging

```go
import "github.com/fast-context/go-sdk/logging"

// Initialize logger with custom level
logger := logging.NewStructuredLogger(logging.LevelDebug,
    logging.WithConsole(true),
    logging.WithJSON(false),
    logging.WithColor(true),
)

// Log with structured fields
logger.Info("Analysis started", 
    "project", "/path/to/project",
    "files_count", 150,
)

// Context-aware logging
contextLogger := logging.NewContextLogger(logger, map[string]interface{}{
    "request_id": "123",
    "user": "developer",
})

contextLogger.Info("Processing file", "file", "main.go")
```

### Metrics Collection

```go
import "github.com/fast-context/go-sdk/logging"

// Access metrics collector
metrics := logging.GetMetricsCollector()

// Record custom metrics
metrics.IncrementCounter("analysis_started", "language", "Go")
metrics.SetGauge("memory_usage", 1024.0)
metrics.ObserveHistogram("analysis_duration", 5.2)

// Time operations
duration := logging.TimeFunction("file_analysis", func() {
    // Your analysis code here
}, "file_type", "go")
```

## Examples

### Basic Project Analysis

```go
package main

import (
	"fmt"
	"log"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/fast-context/go-sdk/export"
)

func main() {
	// Create configuration
	cfg, err := config.BalancedConfig(".")
	if err != nil {
		log.Fatal(err)
	}

	// Create analyzer
	analyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)
	if err != nil {
		log.Fatal(err)
	}

	// Analyze project
	fmt.Println("Analyzing project...")
	result, err := analyzer.Analyze()
	if err != nil {
		log.Fatal(err)
	}

	// Print summary
	fmt.Printf("\nAnalysis Results:\n")
	fmt.Printf("Files: %d\n", result.FileCount)
	fmt.Printf("Symbols: %d\n", result.SymbolCount)
	fmt.Printf("Dependencies: %d\n", result.RelationshipCount)
	fmt.Printf("Languages: %v\n", result.Languages)
	fmt.Printf("Duration: %dms\n", result.DurationMs)

	// Export to Markdown report
	exporter := export.NewExporter(analyzer,
		export.WithFormat(export.FormatMarkdown),
		export.WithOutputFile("analysis_report.md"),
	)

	if err := exporter.ExportToFile(result); err != nil {
		log.Printf("Warning: Failed to export report: %v", err)
	} else {
		fmt.Println("Report saved to: analysis_report.md")
	}
}
```

### Advanced Query Example

```go
package main

import (
	"context"
	"fmt"
	"log"
	"sort"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/fast-context/go-sdk/query"
)

func main() {
	analyzer, err := fastcontext.NewAnalyzer()
	if err != nil {
		log.Fatal(err)
	}

	engine := query.NewEngine(analyzer)

	// Find complex functions
	query := &query.SemanticQuery{
		Query:      "complex business logic",
		SymbolKinds: []query.SymbolKindFunction,
		MinComplexity: 10.0,
		MaxResults:  20,
	}

	results, err := engine.FindSymbols(context.Background(), query)
	if err != nil {
		log.Fatal(err)
	}

	// Sort by complexity
	sort.Slice(results.Symbols, func(i, j int) bool {
		return results.Symbols[i].Complexity > results.Symbols[j].Complexity
	})

	fmt.Printf("Found %d complex functions:\n", len(results.Symbols))
	for i, symbol := range results.Symbols {
		fmt.Printf("%d. %s (%s:%d) - complexity: %.1f\n",
			i+1, symbol.Name, symbol.File, symbol.LineStart, symbol.Complexity)
	}
}
```

### Dependency Visualization

```go
package main

import (
	"fmt"
	"log"
	"os"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/export"
	"github.com/fast-context/go-sdk/fastcontext"
)

func main() {
	analyzer, err := fastcontext.NewAnalyzer()
	if err != nil {
		log.Fatal(err)
	}

	result, err := analyzer.Analyze()
	if err != nil {
		log.Fatal(err)
	}

	// Export as DOT graph for Graphviz
	exporter := export.NewExporter(analyzer,
		export.WithFormat(export.FormatDOT),
		export.WithOutputFile("dependencies.dot"),
	)

	if err := exporter.ExportToFile(result); err != nil {
		log.Fatal(err)
	}

	fmt.Println("Dependency graph saved to: dependencies.dot")
	fmt.Println("Generate visualization with:")
	fmt.Println("  dot -Tpng dependencies.dot -o dependencies.png")
}
```

## Best Practices

### 1. Configuration Management

- Use preset configurations for common scenarios
- Store configuration in version control for reproducible builds
- Use environment variables for deployment-specific settings
- Validate configuration before starting analysis

```go
// Good: Use presets
cfg, err := config.BalancedConfig(projectPath)

// Better: Customize presets
cfg, err := config.BalancedConfig(projectPath)
if err == nil {
    // Customize for specific needs
    cfg.Performance.MaxMemoryMB = 2048
    cfg.IgnorePatterns = append(cfg.IgnorePatterns, "**/generated/**")
}

// Best: Validate configuration
if err := cfg.Validate(); err != nil {
    log.Fatal("Invalid configuration:", err)
}
```

### 2. Error Handling

- Always check for errors and handle them appropriately
- Use specific error types for different failure scenarios
- Provide context in error messages for debugging
- Implement retry logic for transient failures

```go
result, err := analyzer.Analyze()
if err != nil {
    if errors.Is(err, fastcontext.ErrAnalysisTimeout) {
        // Retry with longer timeout
        cfg.Performance.TimeoutSeconds *= 2
        result, err = analyzer.Analyze()
    }
    if err != nil {
        log.Fatal("Analysis failed:", err)
    }
}
```

### 3. Performance Optimization

- Choose appropriate caching strategy based on use case
- Monitor memory usage and adjust limits accordingly
- Use streaming analysis for large codebases
- Implement progress reporting for long-running operations

```go
// For large codebases
if fileCount > 10000 {
    cfg.Performance.EnableStreaming = true
    cfg.Performance.CachePolicy = config.CachePolicyPersistent
    cfg.Performance.MaxConcurrentFiles = 25 // Reduce memory pressure
}
```

### 4. Resource Management

- Always stop file watchers when no longer needed
- Close file handles and clean up resources
- Use context cancellation for long-running operations
- Monitor memory usage and implement limits

```go
// Proper resource cleanup
analyzer, err := fastcontext.NewAnalyzer()
if err != nil {
    log.Fatal(err)
}

if err := analyzer.StartWatching(progressCallback); err != nil {
    log.Fatal(err)
}
defer analyzer.StopWatching() // Ensure cleanup
```

## Troubleshooting

### Common Issues

#### Analysis Timeout

```bash
# Increase timeout
fast-context analyze --timeout 600

# Or in configuration
performance:
  timeoutSeconds: 600
```

#### Memory Limit Exceeded

```bash
# Increase memory limit
fast-context analyze --memory 2048

# Or use streaming for large projects
fast-context analyze --preset thorough --enable-streaming
```

#### File Not Found Errors

```bash
# Check project path
fast-context config --show
fast-context analyze --project /correct/path

# Verify file permissions
ls -la /path/to/project
```

#### Parse Errors

```bash
# Enable debug logging for detailed error information
FAST_CONTEXT_LOG_LEVEL=debug fast-context analyze

# Check for unsupported file types in ignore patterns
ignorePatterns:
  - "**/*.binary"
  - "**/*.min.js"
```

### Performance Debugging

```go
// Enable metrics collection
cfg, err := config.NewConfig(
    config.WithEnableMetrics(true),
    config.WithLogLevel("debug"),
)

analyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)

// Check metrics after analysis
metrics := logging.GetMetricsCollector()
allMetrics := metrics.GetAllMetrics()
fmt.Printf("Analysis metrics: %+v\n", allMetrics)
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/fast-context/go-sdk.git
cd go-sdk

# Install dependencies
go mod download
go mod tidy

# Run tests
make test

# Run linting
make lint

# Build project
make build
```

### Running Tests

```bash
# Run all tests
go test ./...

# Run with coverage
go test -cover ./...

# Run specific package tests
go test ./fastcontext/...
go test ./config/...
go test ./export/...
```

## Requirements

- Go 1.19 or higher
- Rust toolchain (for building the native library)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: [https://docs.fast-context.dev](https://docs.fast-context.dev)
- **Issues**: [GitHub Issues](https://github.com/fast-context/go-sdk/issues)
- **Discussions**: [GitHub Discussions](https://github.com/fast-context/go-sdk/discussions)
- **Email**: [support@fast-context.dev](mailto:support@fast-context.dev)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes and version history.