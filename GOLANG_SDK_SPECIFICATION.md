# Fast-Context Golang SDK Specification

## 🎯 Overview

This specification defines a feature-complete Golang SDK for Fast-Context that provides idiomatic Go APIs while maintaining full compatibility with the existing TypeScript and Python SDKs. The SDK will leverage Go's strengths including goroutines, channels, interfaces, and the context package.

## 📦 Package Structure

```
github.com/fast-context/go-sdk/
├── fastcontext/           # Main package with core analyzer
├── config/               # Configuration management
├── query/                # Advanced query engine
├── graph/                # Graph operations and algorithms
├── streaming/            # Streaming analysis with progress tracking
├── export/               # Export functionality (JSON, GraphML, etc.)
├── cli/                  # Command-line interface
├── examples/             # Usage examples and tutorials
├── internal/             # Internal utilities and CGO bindings
└── tools/                # Development and build tools
```

## 🏗️ Core Architecture

### Integration Strategy
- **CGO Bindings**: Direct integration with Rust core via C-compatible interface
- **Memory Management**: Safe memory handling between Go and Rust with proper cleanup
- **Thread Safety**: Goroutine-safe APIs with proper synchronization
- **Error Handling**: Idiomatic Go error handling with detailed error types

### Design Principles
1. **Idiomatic Go**: Follow Go conventions and best practices
2. **Interface-Driven**: Use interfaces for extensibility and testing
3. **Context-Aware**: Support context.Context for cancellation and timeouts
4. **Channel-Based**: Use channels for streaming and progress updates
5. **Functional Options**: Use functional options pattern for configuration
6. **Zero Dependencies**: Minimize external dependencies for core functionality

## 🔧 Core API Specification

### 1. Main Analyzer Package (`fastcontext`)

```go
package fastcontext

import (
    "context"
    "time"
)

// Analyzer provides high-performance codebase analysis
type Analyzer struct {
    config Config
    // internal fields
}

// NewAnalyzer creates a new analyzer with the given configuration
func NewAnalyzer(config Config) (*Analyzer, error)

// Analyze performs complete codebase analysis
func (a *Analyzer) Analyze(ctx context.Context) (*AnalysisResult, error)

// AnalyzeStream performs streaming analysis with progress updates
func (a *Analyzer) AnalyzeStream(ctx context.Context) (<-chan Progress, error)

// FindSymbolsByKind finds symbols by their type (function, class, etc.)
func (a *Analyzer) FindSymbolsByKind(kind SymbolKind) ([]Symbol, error)

// FindSymbolsInFile finds all symbols in a specific file
func (a *Analyzer) FindSymbolsInFile(filePath string) ([]Symbol, error)

// FindDependencies finds dependencies of a given symbol
func (a *Analyzer) FindDependencies(symbolName string, opts ...DependencyOption) ([]Dependency, error)

// FindComplexSymbols finds symbols with complexity above threshold
func (a *Analyzer) FindComplexSymbols(threshold int) ([]Symbol, error)

// GetQueryEngine returns the advanced query engine
func (a *Analyzer) GetQueryEngine() *query.Engine

// StartWatching starts file system watching for changes
func (a *Analyzer) StartWatching(ctx context.Context) (<-chan FileChangeEvent, error)

// GetSupportedLanguages returns list of supported programming languages
func GetSupportedLanguages() []string

// DetectLanguage detects the programming language of a file
func DetectLanguage(filePath string) (string, error)

// GetVersion returns the SDK version
func GetVersion() string
```

### 2. Configuration Package (`config`)

```go
package config

import (
    "time"
    "path/filepath"
)

// Config represents analyzer configuration
type Config struct {
    ProjectRoot         string
    Languages          []string
    IgnorePatterns     []string
    EnableCaching      bool
    CachePolicy        CachePolicy
    EnableWatching     bool
    MaxFiles           int
    ParallelProcessing bool
    Performance        PerformanceConfig
}

// PerformanceConfig contains performance-related settings
type PerformanceConfig struct {
    MaxMemoryMB     int
    TimeoutDuration time.Duration
    WorkerThreads   int
    ChunkSize       int
}

// CachePolicy defines caching behavior
type CachePolicy string

const (
    CachePolicyAuto       CachePolicy = "auto"
    CachePolicyMinimal    CachePolicy = "minimal"
    CachePolicyBalanced   CachePolicy = "balanced"
    CachePolicyAdaptive   CachePolicy = "adaptive"
    CachePolicyPersistent CachePolicy = "persistent"
)

// ConfigOption is a functional option for configuration
type ConfigOption func(*Config)

// NewConfig creates a new configuration with the given project root
func NewConfig(projectRoot string, opts ...ConfigOption) Config

// Configuration options
func WithLanguages(languages []string) ConfigOption
func WithIgnorePatterns(patterns []string) ConfigOption
func WithCaching(enabled bool) ConfigOption
func WithCachePolicy(policy CachePolicy) ConfigOption
func WithWatching(enabled bool) ConfigOption
func WithMaxFiles(maxFiles int) ConfigOption
func WithParallelProcessing(enabled bool) ConfigOption
func WithPerformance(perf PerformanceConfig) ConfigOption

// Preset configurations
func FastPreset(projectRoot string) Config
func BalancedPreset(projectRoot string) Config
func ThoroughPreset(projectRoot string) Config

// LoadFromFile loads configuration from file (JSON, YAML, TOML)
func LoadFromFile(filePath string) (Config, error)

// LoadFromEnv loads configuration from environment variables
func LoadFromEnv() Config

// Validate validates the configuration
func (c Config) Validate() error
```

### 3. Core Types

```go
// AnalysisResult contains the results of codebase analysis
type AnalysisResult struct {
    FileCount         int
    SymbolCount       int
    RelationshipCount int
    Languages         []string
    DurationMs        int64
    MemoryUsageMB     float64
    Performance       PerformanceMetrics
    Summary           string
    Insights          []string
    Recommendations   []string
}

// PerformanceMetrics contains performance information
type PerformanceMetrics struct {
    MemoryUsageMB            float64
    CPUUsagePercent          float64
    ElapsedMs                int64
    EstimatedRemainingMs     int64
    ThroughputFilesPerSecond float64
}

// Progress represents analysis progress
type Progress struct {
    Phase               AnalysisPhase
    FilesProcessed      int
    TotalFiles          int
    CurrentFile         string
    SymbolsFound        int
    RelationshipsFound  int
    Errors              []AnalysisError
    Performance         PerformanceMetrics
    Timestamp           time.Time
}

// AnalysisPhase represents the current phase of analysis
type AnalysisPhase string

const (
    PhaseInitializing AnalysisPhase = "initializing"
    PhaseParsing      AnalysisPhase = "parsing"
    PhaseExtracting   AnalysisPhase = "extracting"
    PhaseAnalyzing    AnalysisPhase = "analyzing"
    PhaseIndexing     AnalysisPhase = "indexing"
    PhaseComplete     AnalysisPhase = "complete"
    PhaseError        AnalysisPhase = "error"
)

// Symbol represents a code symbol
type Symbol struct {
    Name          string
    Kind          SymbolKind
    FilePath      string
    Line          int
    Column        int
    Scope         string
    Language      string
    Documentation string
    Signature     string
    Complexity    int
}

// SymbolKind represents the type of symbol
type SymbolKind string

const (
    SymbolKindFunction    SymbolKind = "function"
    SymbolKindClass       SymbolKind = "class"
    SymbolKindInterface   SymbolKind = "interface"
    SymbolKindType        SymbolKind = "type"
    SymbolKindVariable    SymbolKind = "variable"
    SymbolKindConstant    SymbolKind = "constant"
    SymbolKindEnum        SymbolKind = "enum"
    SymbolKindModule      SymbolKind = "module"
    SymbolKindNamespace   SymbolKind = "namespace"
    SymbolKindProperty    SymbolKind = "property"
    SymbolKindMethod      SymbolKind = "method"
    SymbolKindConstructor SymbolKind = "constructor"
    SymbolKindField       SymbolKind = "field"
    SymbolKindParameter   SymbolKind = "parameter"
    SymbolKindImport      SymbolKind = "import"
    SymbolKindExport      SymbolKind = "export"
)

// Dependency represents a code dependency relationship
type Dependency struct {
    From   string
    To     string
    Type   DependencyType
    Weight float64
}

// DependencyType represents the type of dependency
type DependencyType string

const (
    DependencyTypeCalls      DependencyType = "calls"
    DependencyTypeImports    DependencyType = "imports"
    DependencyTypeExtends    DependencyType = "extends"
    DependencyTypeImplements DependencyType = "implements"
    DependencyTypeUses       DependencyType = "uses"
)

// FileChangeEvent represents a file system change
type FileChangeEvent struct {
    Type     FileChangeType
    FilePath string
    Time     time.Time
}

// FileChangeType represents the type of file change
type FileChangeType string

const (
    FileChangeTypeCreated  FileChangeType = "created"
    FileChangeTypeModified FileChangeType = "modified"
    FileChangeTypeDeleted  FileChangeType = "deleted"
)

// AnalysisError represents an error during analysis
type AnalysisError struct {
    Message string
    Code    string
    Context map[string]interface{}
}

func (e AnalysisError) Error() string {
    return e.Message
}
```

### 4. Query Engine Package (`query`)

```go
package query

import (
    "context"
    "github.com/fast-context/go-sdk/fastcontext"
)

// Engine provides advanced querying capabilities
type Engine struct {
    analyzer *fastcontext.Analyzer
    // internal fields
}

// NewEngine creates a new query engine
func NewEngine(analyzer *fastcontext.Analyzer) *Engine

// SemanticQuery represents a semantic search query
type SemanticQuery struct {
    Text        string
    Kind        fastcontext.SymbolKind
    Language    string
    MaxResults  int
    Similarity  float64
}

// DependencyOptions configures dependency analysis
type DependencyOptions struct {
    Depth              int
    IncludeTransitive  bool
    Direction          DependencyDirection
}

// DependencyDirection specifies dependency search direction
type DependencyDirection string

const (
    DependencyDirectionIncoming DependencyDirection = "incoming"
    DependencyDirectionOutgoing DependencyDirection = "outgoing"
    DependencyDirectionBoth     DependencyDirection = "both"
)

// ComplexityOptions configures complexity analysis
type ComplexityOptions struct {
    Threshold      int
    IncludeMetrics bool
    SortBy         ComplexitySortBy
}

// ComplexitySortBy specifies how to sort complexity results
type ComplexitySortBy string

const (
    ComplexitySortByComplexity ComplexitySortBy = "complexity"
    ComplexitySortByName       ComplexitySortBy = "name"
    ComplexitySortByFile       ComplexitySortBy = "file"
)

// FindSymbols performs semantic symbol search
func (e *Engine) FindSymbols(ctx context.Context, query SemanticQuery) ([]fastcontext.Symbol, error)

// GetSymbolDependencies analyzes symbol dependencies
func (e *Engine) GetSymbolDependencies(ctx context.Context, symbol string, opts DependencyOptions) (*DependencyGraph, error)

// GetSymbolUsages finds all usages of a symbol
func (e *Engine) GetSymbolUsages(ctx context.Context, symbol string) ([]fastcontext.Symbol, error)

// DetectPatterns detects architectural patterns
func (e *Engine) DetectPatterns(ctx context.Context) ([]ArchitecturalPattern, error)

// AnalyzeComplexity analyzes code complexity
func (e *Engine) AnalyzeComplexity(ctx context.Context, opts ComplexityOptions) (*ComplexityReport, error)

// FindCodeSmells finds potential code issues
func (e *Engine) FindCodeSmells(ctx context.Context) ([]fastcontext.Symbol, error)

// FindSimilarCode finds similar code patterns
func (e *Engine) FindSimilarCode(ctx context.Context, codeSnippet string, similarity float64) ([]CodeMatch, error)

// ClearCache clears the query cache
func (e *Engine) ClearCache(ctx context.Context) error

// DependencyGraph represents a dependency graph
type DependencyGraph struct {
    Nodes []fastcontext.Symbol
    Edges []DependencyEdge
}

// DependencyEdge represents an edge in the dependency graph
type DependencyEdge struct {
    From   string
    To     string
    Type   fastcontext.DependencyType
    Weight float64
}

// ArchitecturalPattern represents a detected architectural pattern
type ArchitecturalPattern struct {
    Name        string
    Description string
    Confidence  float64
    Examples    []string
}

// ComplexityReport contains complexity analysis results
type ComplexityReport struct {
    AverageComplexity float64
    MaxComplexity     int
    ComplexSymbols    []ComplexSymbol
    Recommendations   []string
}

// ComplexSymbol represents a symbol with complexity information
type ComplexSymbol struct {
    Symbol     fastcontext.Symbol
    Complexity int
}

// CodeMatch represents a similar code match
type CodeMatch struct {
    Symbol     fastcontext.Symbol
    Similarity float64
    Snippet    string
}
```

### 5. Graph Package (`graph`)

```go
package graph

import (
    "context"
)

// Graph represents an undirected graph
type Graph interface {
    AddNode(weight float64) int
    RemoveNode(nodeIndex int) bool
    AddEdge(source, target int, weight float64) error
    RemoveEdge(source, target int) bool
    NodeCount() int
    EdgeCount() int
    IsEmpty() bool
    Clear()
    GetNodeWeight(nodeIndex int) (float64, bool)
    SetNodeWeight(nodeIndex int, weight float64) bool
    GetEdgeWeight(source, target int) (float64, bool)
    SetEdgeWeight(source, target int, weight float64) bool
    Neighbors(nodeIndex int) []int
    Edges(nodeIndex int) []Edge
    HasEdge(source, target int) bool

    // Graph algorithms
    DijkstraShortestPath(source, target int) (*PathResult, error)
    FloydWarshallAllPairs() ([][]float64, error)
    ConnectedComponents() ([]ConnectedComponent, error)
    BetweennessCentrality(normalized bool) ([]CentralityResult, error)
    ClosenessCentrality(wfImproved bool) ([]CentralityResult, error)
    PageRank(alpha float64, tolerance float64, maxIter int) ([]CentralityResult, error)
    MinimumSpanningTree() (*Graph, error)
    IsConnected() bool
}

// DiGraph represents a directed graph
type DiGraph interface {
    Graph // Embed Graph interface

    Successors(nodeIndex int) []int
    Predecessors(nodeIndex int) []int
    OutEdges(nodeIndex int) []Edge
    InEdges(nodeIndex int) []Edge

    // Directed graph specific algorithms
    StronglyConnectedComponents() ([]ConnectedComponent, error)
    WeaklyConnectedComponents() ([]ConnectedComponent, error)
    TopologicalSort() ([]int, error)
    IsDAG() bool
    TransitiveClosure() (*DiGraph, error)
}

// NewGraph creates a new undirected graph
func NewGraph() Graph

// NewGraphWithCapacity creates a new undirected graph with initial capacity
func NewGraphWithCapacity(nodes, edges int) Graph

// NewDiGraph creates a new directed graph
func NewDiGraph() DiGraph

// NewDiGraphWithCapacity creates a new directed graph with initial capacity
func NewDiGraphWithCapacity(nodes, edges int) DiGraph

// Edge represents a graph edge
type Edge struct {
    Source int
    Target int
    Weight float64
}

// PathResult represents a shortest path result
type PathResult struct {
    Path     []int
    Distance float64
    Found    bool
}

// CentralityResult represents centrality calculation result
type CentralityResult struct {
    NodeIndex   int
    Centrality  float64
}

// ConnectedComponent represents a connected component
type ConnectedComponent struct {
    Nodes []int
    Size  int
}
```

### 6. Streaming Package (`streaming`)

```go
package streaming

import (
    "context"
    "time"
    "github.com/fast-context/go-sdk/fastcontext"
)

// Analyzer provides streaming analysis capabilities
type Analyzer struct {
    analyzer *fastcontext.Analyzer
    // internal fields
}

// NewAnalyzer creates a new streaming analyzer
func NewAnalyzer(analyzer *fastcontext.Analyzer) *Analyzer

// StreamingOptions configures streaming analysis
type StreamingOptions struct {
    ProgressInterval      time.Duration
    EnableDetailedProgress bool
    BatchSize            int
}

// AnalyzeStream performs streaming analysis with progress updates
func (s *Analyzer) AnalyzeStream(ctx context.Context, opts StreamingOptions) (<-chan fastcontext.Progress, error)

// Cancel cancels the current streaming analysis
func (s *Analyzer) Cancel()

// IsAnalyzing returns true if analysis is currently running
func (s *Analyzer) IsAnalyzing() bool
```

### 7. Export Package (`export`)

```go
package export

import (
    "context"
    "io"
    "github.com/fast-context/go-sdk/fastcontext"
    "github.com/fast-context/go-sdk/graph"
)

// Format represents export format
type Format string

const (
    FormatJSON     Format = "json"
    FormatYAML     Format = "yaml"
    FormatXML      Format = "xml"
    FormatGraphML  Format = "graphml"
    FormatDOT      Format = "dot"
    FormatCSV      Format = "csv"
    FormatMarkdown Format = "markdown"
)

// Options configures export behavior
type Options struct {
    Format          Format
    IncludeMetadata bool
    Compress        bool
    PrettyPrint     bool
    FilterOptions   FilterOptions
}

// FilterOptions configures what to include in export
type FilterOptions struct {
    Languages    []string
    SymbolKinds  []fastcontext.SymbolKind
    MinComplexity int
    MaxComplexity int
}

// Exporter handles exporting analysis results
type Exporter struct {
    // internal fields
}

// NewExporter creates a new exporter
func NewExporter() *Exporter

// ExportAnalysis exports analysis results
func (e *Exporter) ExportAnalysis(ctx context.Context, result *fastcontext.AnalysisResult, writer io.Writer, opts Options) error

// ExportGraph exports a graph
func (e *Exporter) ExportGraph(ctx context.Context, g graph.Graph, writer io.Writer, opts Options) error

// ExportSymbols exports symbols
func (e *Exporter) ExportSymbols(ctx context.Context, symbols []fastcontext.Symbol, writer io.Writer, opts Options) error

// GetSupportedFormats returns list of supported export formats
func GetSupportedFormats() []Format

// ValidateOptions validates export options
func ValidateOptions(opts Options) error
```

### 8. CLI Package (`cli`)

```go
package cli

import (
    "context"
    "github.com/spf13/cobra"
    "github.com/fast-context/go-sdk/fastcontext"
    "github.com/fast-context/go-sdk/config"
    "github.com/fast-context/go-sdk/export"
)

// App represents the CLI application
type App struct {
    rootCmd *cobra.Command
    config  config.Config
}

// NewApp creates a new CLI application
func NewApp() *App

// Execute runs the CLI application
func (a *App) Execute() error

// Commands:
// - analyze: Analyze codebase
// - symbols: Find and list symbols
// - dependencies: Analyze dependencies
// - complexity: Analyze code complexity
// - patterns: Detect architectural patterns
// - export: Export analysis results
// - config: Manage configuration
// - watch: Watch for file changes
// - serve: Start analysis server
// - version: Show version information

// AnalyzeCommand handles the analyze command
type AnalyzeCommand struct {
    ProjectRoot    string
    Languages      []string
    IgnorePatterns []string
    OutputFormat   export.Format
    OutputFile     string
    Verbose        bool
    Stream         bool
}

// Execute runs the analyze command
func (c *AnalyzeCommand) Execute(ctx context.Context) error

// SymbolsCommand handles the symbols command
type SymbolsCommand struct {
    ProjectRoot string
    Kind        fastcontext.SymbolKind
    File        string
    Format      export.Format
    OutputFile  string
}

// Execute runs the symbols command
func (c *SymbolsCommand) Execute(ctx context.Context) error

// DependenciesCommand handles the dependencies command
type DependenciesCommand struct {
    ProjectRoot string
    Symbol      string
    Depth       int
    Direction   string
    Format      export.Format
    OutputFile  string
}

// Execute runs the dependencies command
func (c *DependenciesCommand) Execute(ctx context.Context) error

// ComplexityCommand handles the complexity command
type ComplexityCommand struct {
    ProjectRoot string
    Threshold   int
    SortBy      string
    Format      export.Format
    OutputFile  string
}

// Execute runs the complexity command
func (c *ComplexityCommand) Execute(ctx context.Context) error

// WatchCommand handles the watch command
type WatchCommand struct {
    ProjectRoot string
    Verbose     bool
}

// Execute runs the watch command
func (c *WatchCommand) Execute(ctx context.Context) error

// ServeCommand handles the serve command
type ServeCommand struct {
    Port        int
    Host        string
    ProjectRoot string
    ConfigFile  string
}

// Execute runs the serve command
func (c *ServeCommand) Execute(ctx context.Context) error
```

## 🔧 Implementation Details

### CGO Integration

The SDK will integrate with the Rust core through CGO bindings:

```go
package internal

/*
#cgo LDFLAGS: -L. -lfastcontext
#include "fastcontext.h"
*/
import "C"
import (
    "unsafe"
    "runtime"
)

// RustAnalyzer wraps the Rust analyzer
type RustAnalyzer struct {
    ptr C.analyzer_t
}

// NewRustAnalyzer creates a new Rust analyzer
func NewRustAnalyzer(config *Config) (*RustAnalyzer, error) {
    configStr := C.CString(config.ToJSON())
    defer C.free(unsafe.Pointer(configStr))

    ptr := C.analyzer_new(configStr)
    if ptr == nil {
        return nil, errors.New("failed to create analyzer")
    }

    analyzer := &RustAnalyzer{ptr: ptr}
    runtime.SetFinalizer(analyzer, (*RustAnalyzer).finalize)
    return analyzer, nil
}

// Analyze performs analysis
func (a *RustAnalyzer) Analyze() (*AnalysisResult, error) {
    resultPtr := C.analyzer_analyze(a.ptr)
    if resultPtr == nil {
        return nil, errors.New("analysis failed")
    }
    defer C.analysis_result_free(resultPtr)

    return parseAnalysisResult(resultPtr), nil
}

// finalize cleans up the Rust analyzer
func (a *RustAnalyzer) finalize() {
    if a.ptr != nil {
        C.analyzer_free(a.ptr)
        a.ptr = nil
    }
}
```

### Error Handling

```go
// Error types for different categories of errors
type ErrorCode string

const (
    ErrorCodeConfiguration ErrorCode = "CONFIGURATION_ERROR"
    ErrorCodeAnalysis      ErrorCode = "ANALYSIS_ERROR"
    ErrorCodeIO            ErrorCode = "IO_ERROR"
    ErrorCodeCancelled     ErrorCode = "CANCELLED_ERROR"
    ErrorCodeTimeout       ErrorCode = "TIMEOUT_ERROR"
    ErrorCodeMemory        ErrorCode = "MEMORY_ERROR"
)

// FastContextError provides detailed error information
type FastContextError struct {
    Code    ErrorCode
    Message string
    Context map[string]interface{}
    Cause   error
}

func (e *FastContextError) Error() string {
    if e.Cause != nil {
        return fmt.Sprintf("%s: %s (caused by: %v)", e.Code, e.Message, e.Cause)
    }
    return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

func (e *FastContextError) Unwrap() error {
    return e.Cause
}

// Error constructors
func NewConfigurationError(message string, cause error) *FastContextError {
    return &FastContextError{
        Code:    ErrorCodeConfiguration,
        Message: message,
        Cause:   cause,
    }
}

func NewAnalysisError(message string, context map[string]interface{}) *FastContextError {
    return &FastContextError{
        Code:    ErrorCodeAnalysis,
        Message: message,
        Context: context,
    }
}
```

## 📚 Usage Examples

### Basic Usage

```go
package main

import (
    "context"
    "fmt"
    "log"

    "github.com/fast-context/go-sdk/fastcontext"
    "github.com/fast-context/go-sdk/config"
)

func main() {
    // Create configuration
    cfg := config.NewConfig("./my-project",
        config.WithLanguages([]string{"go", "javascript", "python"}),
        config.WithCaching(true),
        config.WithParallelProcessing(true),
    )

    // Create analyzer
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        log.Fatal(err)
    }

    // Perform analysis
    ctx := context.Background()
    result, err := analyzer.Analyze(ctx)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Analyzed %d files, found %d symbols\n",
        result.FileCount, result.SymbolCount)
}
```

### Streaming Analysis

```go
func streamingAnalysis() {
    cfg := config.BalancedPreset("./my-project")
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        log.Fatal(err)
    }

    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
    defer cancel()

    progressCh, err := analyzer.AnalyzeStream(ctx)
    if err != nil {
        log.Fatal(err)
    }

    for progress := range progressCh {
        fmt.Printf("Phase: %s, Progress: %d/%d files\n",
            progress.Phase, progress.FilesProcessed, progress.TotalFiles)

        if progress.Phase == fastcontext.PhaseComplete {
            fmt.Printf("Analysis complete! Found %d symbols\n", progress.SymbolsFound)
            break
        }
    }
}
```

### Advanced Querying

```go
func advancedQuerying() {
    cfg := config.ThoroughPreset("./my-project")
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        log.Fatal(err)
    }

    // Get query engine
    queryEngine := analyzer.GetQueryEngine()

    ctx := context.Background()

    // Semantic search
    symbols, err := queryEngine.FindSymbols(ctx, query.SemanticQuery{
        Text:       "user authentication",
        Kind:       fastcontext.SymbolKindFunction,
        MaxResults: 10,
        Similarity: 0.7,
    })
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Found %d authentication-related functions\n", len(symbols))

    // Dependency analysis
    deps, err := queryEngine.GetSymbolDependencies(ctx, "UserService", query.DependencyOptions{
        Depth:             3,
        IncludeTransitive: true,
        Direction:         query.DependencyDirectionBoth,
    })
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("UserService has %d dependencies\n", len(deps.Edges))

    // Complexity analysis
    complexity, err := queryEngine.AnalyzeComplexity(ctx, query.ComplexityOptions{
        Threshold:      10,
        IncludeMetrics: true,
        SortBy:         query.ComplexitySortByComplexity,
    })
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Average complexity: %.2f, %d complex symbols found\n",
        complexity.AverageComplexity, len(complexity.ComplexSymbols))
}
```

### Graph Operations

```go
func graphOperations() {
    // Create a directed graph
    g := graph.NewDiGraph()

    // Add nodes
    node1 := g.AddNode(1.0)
    node2 := g.AddNode(2.0)
    node3 := g.AddNode(3.0)

    // Add edges
    g.AddEdge(node1, node2, 5.0)
    g.AddEdge(node2, node3, 3.0)
    g.AddEdge(node1, node3, 10.0)

    // Find shortest path
    path, err := g.DijkstraShortestPath(node1, node3)
    if err != nil {
        log.Fatal(err)
    }

    if path.Found {
        fmt.Printf("Shortest path from %d to %d: %v (distance: %.2f)\n",
            node1, node3, path.Path, path.Distance)
    }

    // Calculate centrality
    centrality, err := g.BetweennessCentrality(true)
    if err != nil {
        log.Fatal(err)
    }

    for _, result := range centrality {
        fmt.Printf("Node %d centrality: %.4f\n", result.NodeIndex, result.Centrality)
    }

    // Find strongly connected components
    components, err := g.StronglyConnectedComponents()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Found %d strongly connected components\n", len(components))
}
```

### File Watching

```go
func fileWatching() {
    cfg := config.NewConfig("./my-project", config.WithWatching(true))
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        log.Fatal(err)
    }

    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    // Start watching for file changes
    changesCh, err := analyzer.StartWatching(ctx)
    if err != nil {
        log.Fatal(err)
    }

    // Handle file changes
    go func() {
        for change := range changesCh {
            fmt.Printf("File %s was %s at %s\n",
                change.FilePath, change.Type, change.Time.Format(time.RFC3339))

            // Trigger incremental analysis if needed
            if change.Type == fastcontext.FileChangeTypeModified {
                // Perform incremental analysis
                symbols, err := analyzer.FindSymbolsInFile(change.FilePath)
                if err == nil {
                    fmt.Printf("Re-analyzed %s, found %d symbols\n",
                        change.FilePath, len(symbols))
                }
            }
        }
    }()

    // Keep watching for 1 minute
    time.Sleep(1 * time.Minute)
}
```

### Export Functionality

```go
func exportResults() {
    cfg := config.BalancedPreset("./my-project")
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        log.Fatal(err)
    }

    ctx := context.Background()
    result, err := analyzer.Analyze(ctx)
    if err != nil {
        log.Fatal(err)
    }

    // Export to JSON
    exporter := export.NewExporter()

    file, err := os.Create("analysis_result.json")
    if err != nil {
        log.Fatal(err)
    }
    defer file.Close()

    err = exporter.ExportAnalysis(ctx, result, file, export.Options{
        Format:          export.FormatJSON,
        IncludeMetadata: true,
        PrettyPrint:     true,
        FilterOptions: export.FilterOptions{
            Languages:   []string{"go", "javascript"},
            SymbolKinds: []fastcontext.SymbolKind{
                fastcontext.SymbolKindFunction,
                fastcontext.SymbolKindClass,
            },
        },
    })
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Analysis results exported to analysis_result.json")
}
```

## 🚀 Development Plan

### Phase 1: Core Foundation (Weeks 1-3)
**Objective**: Establish basic functionality and CGO integration

**Deliverables**:
- [ ] CGO bindings to Rust core
- [ ] Basic analyzer implementation
- [ ] Configuration management
- [ ] Core types and interfaces
- [ ] Error handling system
- [ ] Basic unit tests

**Key Tasks**:
1. Set up Go module structure
2. Create C-compatible wrapper in Rust
3. Implement CGO bindings
4. Design and implement core types
5. Create configuration system with validation
6. Implement basic analyzer functionality
7. Set up testing framework

### Phase 2: Advanced Features (Weeks 4-6)
**Objective**: Implement advanced analysis and query capabilities

**Deliverables**:
- [ ] Query engine implementation
- [ ] Graph operations package
- [ ] Streaming analysis with progress tracking
- [ ] Symbol and dependency analysis
- [ ] File watching capabilities
- [ ] Integration tests

**Key Tasks**:
1. Implement query engine with semantic search
2. Create graph package with algorithms
3. Add streaming analysis with channels
4. Implement dependency analysis
5. Add file system watching
6. Create comprehensive integration tests
7. Performance optimization

### Phase 3: Ecosystem Integration (Weeks 7-8)
**Objective**: Complete ecosystem features and tooling

**Deliverables**:
- [ ] CLI tool with cobra framework
- [ ] Export functionality (multiple formats)
- [ ] Configuration file support
- [ ] Logging and metrics integration
- [ ] Documentation and examples
- [ ] Performance benchmarks

**Key Tasks**:
1. Build CLI tool with all commands
2. Implement export functionality
3. Add configuration file support (YAML, JSON, TOML)
4. Integrate logging and metrics
5. Create comprehensive documentation
6. Write usage examples and tutorials
7. Set up performance benchmarks

### Phase 4: Production Readiness (Weeks 9-10)
**Objective**: Ensure production quality and release preparation

**Deliverables**:
- [ ] Performance optimization
- [ ] Memory leak detection and fixes
- [ ] Comprehensive test suite (>95% coverage)
- [ ] CI/CD pipeline
- [ ] Release automation
- [ ] Security audit

**Key Tasks**:
1. Profile and optimize performance
2. Fix memory leaks and race conditions
3. Achieve comprehensive test coverage
4. Set up CI/CD with GitHub Actions
5. Create release automation
6. Conduct security audit
7. Prepare release documentation

## 🧪 Testing Strategy

### Unit Testing
```go
func TestAnalyzer_Analyze(t *testing.T) {
    tests := []struct {
        name        string
        config      config.Config
        expectError bool
        expectFiles int
    }{
        {
            name: "successful analysis",
            config: config.NewConfig("./testdata/simple-project",
                config.WithLanguages([]string{"go"}),
            ),
            expectError: false,
            expectFiles: 5,
        },
        {
            name: "invalid project root",
            config: config.NewConfig("./nonexistent",
                config.WithLanguages([]string{"go"}),
            ),
            expectError: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            analyzer, err := fastcontext.NewAnalyzer(tt.config)
            if err != nil {
                if !tt.expectError {
                    t.Fatalf("unexpected error: %v", err)
                }
                return
            }

            ctx := context.Background()
            result, err := analyzer.Analyze(ctx)

            if tt.expectError {
                assert.Error(t, err)
                return
            }

            assert.NoError(t, err)
            assert.Equal(t, tt.expectFiles, result.FileCount)
        })
    }
}
```

### Integration Testing
```go
func TestIntegration_FullAnalysis(t *testing.T) {
    if testing.Short() {
        t.Skip("skipping integration test in short mode")
    }

    // Test with real project
    cfg := config.BalancedPreset("./testdata/real-project")
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    require.NoError(t, err)

    ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
    defer cancel()

    // Test streaming analysis
    progressCh, err := analyzer.AnalyzeStream(ctx)
    require.NoError(t, err)

    var finalProgress fastcontext.Progress
    for progress := range progressCh {
        finalProgress = progress
        if progress.Phase == fastcontext.PhaseComplete {
            break
        }
    }

    assert.Equal(t, fastcontext.PhaseComplete, finalProgress.Phase)
    assert.Greater(t, finalProgress.SymbolsFound, 0)

    // Test query engine
    queryEngine := analyzer.GetQueryEngine()
    symbols, err := queryEngine.FindSymbols(ctx, query.SemanticQuery{
        Text:       "main function",
        Kind:       fastcontext.SymbolKindFunction,
        MaxResults: 10,
    })
    require.NoError(t, err)
    assert.Greater(t, len(symbols), 0)
}
```

### Benchmark Testing
```go
func BenchmarkAnalyzer_Analyze(b *testing.B) {
    cfg := config.FastPreset("./testdata/benchmark-project")
    analyzer, err := fastcontext.NewAnalyzer(cfg)
    if err != nil {
        b.Fatal(err)
    }

    ctx := context.Background()

    b.ResetTimer()
    for i := 0; i < b.N; i++ {
        _, err := analyzer.Analyze(ctx)
        if err != nil {
            b.Fatal(err)
        }
    }
}

func BenchmarkGraph_DijkstraShortestPath(b *testing.B) {
    g := graph.NewDiGraphWithCapacity(1000, 5000)

    // Build test graph
    for i := 0; i < 1000; i++ {
        g.AddNode(float64(i))
    }
    for i := 0; i < 5000; i++ {
        source := rand.Intn(1000)
        target := rand.Intn(1000)
        if source != target {
            g.AddEdge(source, target, rand.Float64()*10)
        }
    }

    b.ResetTimer()
    for i := 0; i < b.N; i++ {
        _, err := g.DijkstraShortestPath(0, 999)
        if err != nil {
            b.Fatal(err)
        }
    }
}
```

### Memory and Race Testing
```bash
# Memory leak detection
go test -race -memprofile=mem.prof ./...

# Race condition detection
go test -race ./...

# CPU profiling
go test -cpuprofile=cpu.prof -bench=. ./...
```

## 📊 Quality Standards

### Code Coverage
- **Minimum**: 90% test coverage for all public APIs
- **Target**: 95% test coverage including edge cases
- **Critical paths**: 100% coverage for core analysis functions

### Performance Benchmarks
- **Analysis Speed**: Process 1000 Go files in <30 seconds
- **Memory Usage**: Stay under 512MB for typical projects
- **Streaming Latency**: Progress updates every 100ms
- **Query Response**: Symbol queries complete in <1 second

### Error Handling
- All public functions return proper error types
- Context cancellation respected throughout
- Resource cleanup guaranteed via finalizers
- Detailed error messages with context

### Documentation
- Godoc comments for all public APIs
- Usage examples for all major features
- Architecture documentation
- Performance tuning guide

## 🔧 Build and Release

### Build Configuration
```makefile
# Makefile
.PHONY: build test bench clean install

# Build the SDK
build:
	go build -v ./...

# Run tests
test:
	go test -v -race ./...

# Run benchmarks
bench:
	go test -bench=. -benchmem ./...

# Build with CGO
build-cgo:
	CGO_ENABLED=1 go build -v ./...

# Install dependencies
deps:
	go mod download
	go mod verify

# Clean build artifacts
clean:
	go clean -cache -testcache -modcache

# Install CLI tool
install:
	go install ./cmd/fast-context-cli
```

### Release Process
1. **Version Tagging**: Semantic versioning (v1.0.0, v1.1.0, etc.)
2. **Automated Testing**: Full test suite on multiple Go versions
3. **Cross-Platform Builds**: Linux, macOS, Windows support
4. **Documentation Updates**: Changelog and API documentation
5. **Binary Releases**: Pre-built binaries for major platforms

## 🎯 Success Criteria

### Functional Requirements
- ✅ Feature parity with TypeScript and Python SDKs
- ✅ Support for 20+ programming languages
- ✅ Streaming analysis with progress tracking
- ✅ Advanced query engine with semantic search
- ✅ Graph algorithms and analysis
- ✅ Export functionality (multiple formats)
- ✅ CLI tool with comprehensive commands
- ✅ File watching and incremental updates

### Non-Functional Requirements
- ✅ Performance: 2x faster than Python SDK
- ✅ Memory efficiency: 50% less memory usage than TypeScript SDK
- ✅ Reliability: 99.9% uptime for long-running analysis
- ✅ Usability: Intuitive Go-idiomatic API
- ✅ Maintainability: Comprehensive test coverage and documentation

## 🚀 Conclusion

This specification defines a comprehensive, feature-complete Golang SDK for Fast-Context that:

1. **Maintains Feature Parity**: Provides all capabilities available in TypeScript and Python SDKs
2. **Follows Go Idioms**: Uses channels, contexts, interfaces, and error handling patterns
3. **Optimizes Performance**: Leverages Go's concurrency and CGO integration with Rust
4. **Ensures Quality**: Comprehensive testing, documentation, and error handling
5. **Enables Ecosystem**: CLI tools, export formats, and integration examples

The SDK will serve as a production-ready solution for Go developers who need intelligent codebase analysis, making Fast-Context accessible to the entire Go ecosystem while maintaining the high performance and accuracy of the Rust core engine.

**Next Steps**: Begin Phase 1 implementation with CGO bindings and core analyzer functionality.
```
```
```
```
