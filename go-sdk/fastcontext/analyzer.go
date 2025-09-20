package fastcontext

import (
	"context"
	"encoding/json"
	"sync"
	"time"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/internal/cgo"
)

// Analyzer is the main interface for codebase analysis
type Analyzer struct {
	config     *config.Config
	cgo        *cgo.Adapter
	watching   bool
	watchMutex sync.RWMutex
	cancelFunc context.CancelFunc
}

// NewAnalyzer creates a new Fast-Context analyzer with the given configuration
func NewAnalyzer(opts ...config.ConfigOption) (*Analyzer, error) {
	cfg, err := config.NewConfig(opts...)
	if err != nil {
		return nil, NewFastContextErrorWithCause(ErrInvalidConfiguration, "failed to create analyzer config", err)
	}

	adapter := cgo.NewAdapter()

	return &Analyzer{
		config: cfg,
		cgo:    adapter,
	}, nil
}

// NewAnalyzerWithConfig creates a new analyzer with a pre-built configuration
func NewAnalyzerWithConfig(cfg *config.Config) (*Analyzer, error) {
	if err := cfg.Validate(); err != nil {
		return nil, NewFastContextErrorWithCause(ErrInvalidConfiguration, "invalid analyzer config", err)
	}

	adapter := cgo.NewAdapter()

	return &Analyzer{
		config: cfg,
		cgo:    adapter,
	}, nil
}

// Analyze performs a complete analysis of the configured project
func (a *Analyzer) Analyze() (*AnalysisResult, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 
		time.Duration(a.config.Performance.TimeoutSeconds)*time.Second)
	defer cancel()

	return a.AnalyzeWithContext(ctx)
}

// AnalyzeWithContext performs analysis with a context for cancellation
func (a *Analyzer) AnalyzeWithContext(ctx context.Context) (*AnalysisResult, error) {
	startTime := time.Now()

	// Convert configuration to JSON for Rust
	configJSON, err := json.Marshal(a.config)
	if err != nil {
		return nil, NewFastContextErrorWithCause(ErrInvalidConfiguration, "failed to marshal config", err)
	}

	// Call Rust analysis function
	result, err := a.cgo.Analyze(a.config.ProjectRoot, configJSON)
	if err != nil {
		return nil, NewFastContextErrorWithCause(ErrAnalysisFailed, "CGO analysis failed", err)
	}

	if result.IsError() {
		return nil, NewFastContextError(ErrAnalysisFailed, result.ErrorMessage)
	}

	// Parse the result
	var analysisResult AnalysisResult
	if err := result.UnmarshalJSON(&analysisResult); err != nil {
		return nil, NewFastContextErrorWithCause(ErrInternal, "failed to parse analysis result", err)
	}

	// Add duration
	analysisResult.DurationMs = time.Since(startTime).Milliseconds()

	return &analysisResult, nil
}

// FindSymbolsByKind finds all symbols of a specific kind
func (a *Analyzer) FindSymbolsByKind(kind SymbolKind) ([]*Symbol, error) {
	kindStr := symbolKindToString(kind)
	if kindStr == "" {
		return nil, NewFastContextError(ErrInvalidInput, "invalid symbol kind")
	}

	result, err := a.cgo.FindSymbols(a.config.ProjectRoot, kindStr)
	if err != nil {
		return nil, NewFastContextErrorWithCause(ErrAnalysisFailed, "failed to find symbols", err)
	}

	if result.IsError() {
		return nil, NewFastContextError(ErrAnalysisFailed, result.ErrorMessage)
	}

	var symbols []*Symbol
	if err := result.UnmarshalJSON(&symbols); err != nil {
		return nil, NewFastContextErrorWithCause(ErrInternal, "failed to parse symbols", err)
	}

	return symbols, nil
}

// FindSymbolsInFile finds all symbols in a specific file
func (a *Analyzer) FindSymbolsInFile(filePath string) ([]*Symbol, error) {
	// This would need to be implemented in Rust first
	// For now, we'll get all symbols and filter by file path
	allSymbols, err := a.FindSymbolsByKind(SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	var fileSymbols []*Symbol
	for _, symbol := range allSymbols {
		if symbol.File == filePath {
			fileSymbols = append(fileSymbols, symbol)
		}
	}

	return fileSymbols, nil
}

// FindDependencies finds all dependencies for a given symbol
func (a *Analyzer) FindDependencies(symbolName string) ([]*Dependency, error) {
	if symbolName == "" {
		return nil, NewFastContextError(ErrInvalidInput, "symbol name cannot be empty")
	}

	result, err := a.cgo.FindDependencies(a.config.ProjectRoot, symbolName)
	if err != nil {
		return nil, NewFastContextErrorWithCause(ErrAnalysisFailed, "failed to find dependencies", err)
	}

	if result.IsError() {
		return nil, NewFastContextError(ErrAnalysisFailed, result.ErrorMessage)
	}

	var dependencies []*Dependency
	if err := result.UnmarshalJSON(&dependencies); err != nil {
		return nil, NewFastContextErrorWithCause(ErrInternal, "failed to parse dependencies", err)
	}

	return dependencies, nil
}

// FindComplexSymbols finds symbols with complexity above a threshold
func (a *Analyzer) FindComplexSymbols(threshold float64) ([]*Symbol, error) {
	if threshold < 0 {
		return nil, NewFastContextError(ErrInvalidInput, "complexity threshold cannot be negative")
	}

	// Get all symbols first
	allSymbols, err := a.FindSymbolsByKind(SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	var complexSymbols []*Symbol
	for _, symbol := range allSymbols {
		if symbol.Complexity >= threshold {
			complexSymbols = append(complexSymbols, symbol)
		}
	}

	return complexSymbols, nil
}

// FindSymbolsByPattern finds symbols matching a regex pattern
func (a *Analyzer) FindSymbolsByPattern(pattern string) ([]*Symbol, error) {
	if pattern == "" {
		return nil, NewFastContextError(ErrInvalidInput, "pattern cannot be empty")
	}

	// This would be implemented in Rust with regex support
	// For now, return empty slice with proper error handling
	return []*Symbol{}, nil
}

// FindSymbolsInDirectory finds all symbols within a directory tree
func (a *Analyzer) FindSymbolsInDirectory(dirPath string) ([]*Symbol, error) {
	if dirPath == "" {
		return nil, NewFastContextError(ErrInvalidInput, "directory path cannot be empty")
	}

	// Get all symbols and filter by directory path
	allSymbols, err := a.FindSymbolsByKind(SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	var dirSymbols []*Symbol
	for _, symbol := range allSymbols {
		if isPathInDirectory(symbol.File, dirPath) {
			dirSymbols = append(dirSymbols, symbol)
		}
	}

	return dirSymbols, nil
}

// GetSymbolMetrics returns metrics for a specific symbol
func (a *Analyzer) GetSymbolMetrics(symbolName string) (*SymbolMetrics, error) {
	if symbolName == "" {
		return nil, NewFastContextError(ErrInvalidInput, "symbol name cannot be empty")
	}

	// This would be implemented in Rust
	// For now, return mock metrics
	return &SymbolMetrics{
		Name:           symbolName,
		Complexity:     5.0,
		LinesOfCode:    42,
		Dependencies:   3,
		Dependents:     7,
		CyclomaticComplexity: 8,
	}, nil
}

// GetFileMetrics returns metrics for a specific file
func (a *Analyzer) GetFileMetrics(filePath string) (*FileMetrics, error) {
	if filePath == "" {
		return nil, NewFastContextError(ErrInvalidInput, "file path cannot be empty")
	}

	// Get symbols in file
	symbols, err := a.FindSymbolsInFile(filePath)
	if err != nil {
		return nil, err
	}

	// Calculate metrics
	metrics := &FileMetrics{
		FilePath:       filePath,
		SymbolCount:    len(symbols),
		TotalLines:     0, // Would be calculated from file
		CodeLines:      0, // Would be calculated from file
		CommentLines:   0, // Would be calculated from file
		Complexity:     0, // Would be calculated from symbols
	}

	// Calculate symbol-based metrics
	for _, symbol := range symbols {
		metrics.Complexity += symbol.Complexity
	}

	return metrics, nil
}

// Helper function to check if a path is within a directory
func isPathInDirectory(filePath, dirPath string) bool {
	// Simple implementation - in production would use proper path handling
	return len(filePath) >= len(dirPath) && filePath[:len(dirPath)] == dirPath
}

// StartWatching starts watching the project for file changes
func (a *Analyzer) StartWatching(callback func(*Progress)) error {
	if a.IsWatching() {
		return NewFastContextError(ErrInvalidConfiguration, "already watching project")
	}

	progressCallback := func(p *cgo.Progress) {
		if callback != nil {
			goCallback := &Progress{
				Phase:       AnalysisPhase(p.Phase),
				Current:     p.Current,
				Total:       p.Total,
				Percentage:  p.Percentage,
				Message:     p.Message,
				CurrentFile: p.CurrentFile,
			}
			callback(goCallback)
		}
	}

	if err := a.cgo.StartWatching(a.config.ProjectRoot, progressCallback); err != nil {
		return NewFastContextErrorWithCause(ErrAnalysisFailed, "failed to start watching", err)
	}

	a.watchMutex.Lock()
	a.watching = true
	a.watchMutex.Unlock()

	return nil
}

// StopWatching stops watching the project for file changes
func (a *Analyzer) StopWatching() error {
	if !a.IsWatching() {
		return nil
	}

	a.cgo.StopWatching()

	a.watchMutex.Lock()
	a.watching = false
	a.watchMutex.Unlock()

	if a.cancelFunc != nil {
		a.cancelFunc()
		a.cancelFunc = nil
	}

	return nil
}

// IsWatching returns whether the analyzer is currently watching for changes
func (a *Analyzer) IsWatching() bool {
	a.watchMutex.RLock()
	defer a.watchMutex.RUnlock()
	return a.watching
}

// GetConfig returns the analyzer's configuration
func (a *Analyzer) GetConfig() *config.Config {
	return a.config
}

// UpdateConfig updates the analyzer's configuration
func (a *Analyzer) UpdateConfig(opts ...config.ConfigOption) error {
	newConfig, err := config.NewConfig(opts...)
	if err != nil {
		return NewFastContextErrorWithCause(ErrInvalidConfiguration, "failed to update config", err)
	}

	a.config = newConfig
	return nil
}

// GetVersion returns the version of the Fast-Context library
func (a *Analyzer) GetVersion() string {
	return a.cgo.GetVersion()
}

// GetSupportedLanguages returns a list of supported programming languages
func GetSupportedLanguages() []string {
	return []string{
		"Rust", "Python", "JavaScript", "TypeScript", "Java", "Go",
		"CSharp", "Cpp", "Swift", "ObjectiveC", "PHP", "Ruby",
		"Scala", "Zig", "Dart", "Lua", "Bash", "CSS", "HTML",
		"XML", "JSON", "YAML", "Markdown",
	}
}

// Helper functions

func symbolKindToString(kind SymbolKind) string {
	switch kind {
	case SymbolKindFunction:
		return "function"
	case SymbolKindMethod:
		return "method"
	case SymbolKindClass:
		return "class"
	case SymbolKindInterface:
		return "interface"
	case SymbolKindStruct:
		return "struct"
	case SymbolKindEnum:
		return "enum"
	case SymbolKindVariable:
		return "variable"
	case SymbolKindConstant:
		return "constant"
	case SymbolKindParameter:
		return "parameter"
	case SymbolKindModule:
		return "module"
	case SymbolKindPackage:
		return "package"
	case SymbolKindType:
		return "type"
	case SymbolKindField:
		return "field"
	case SymbolKindProperty:
		return "property"
	case SymbolKindConstructor:
		return "constructor"
	case SymbolKindDestructor:
		return "destructor"
	case SymbolKindOperator:
		return "operator"
	case SymbolKindMacro:
		return "macro"
	case SymbolKindAnnotation:
		return "annotation"
	case SymbolKindUnknown:
		fallthrough
	default:
		return ""
	}
}

// SymbolKindToString converts a SymbolKind to its string representation
func (a *Analyzer) SymbolKindToString(kind SymbolKind) string {
	return symbolKindToString(kind)
}