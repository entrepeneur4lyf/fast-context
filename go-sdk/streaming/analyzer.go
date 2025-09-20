package streaming

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
)

// Analyzer provides streaming analysis capabilities
type Analyzer struct {
	config         *config.Config
	baseAnalyzer   *fastcontext.Analyzer
	progressChan   chan *Progress
	resultChan     chan *PartialResult
	errorChan      chan error
	cancelFunc     context.CancelFunc
	wg             sync.WaitGroup
	isStreaming    bool
	streamMutex    sync.RWMutex
	stats          *StreamStats
}

// StreamingOptions defines options for streaming analysis
type StreamingOptions struct {
	BufferSize        int           `json:"bufferSize"`
	BatchSize         int           `json:"batchSize"`
	FlushInterval     time.Duration `json:"flushInterval"`
	ProgressInterval  time.Duration `json:"progressInterval"`
	MaxConcurrent     int           `json:"maxConcurrent"`
	EnableMetrics     bool          `json:"enableMetrics"`
	RealTimeUpdates   bool          `json:"realTimeUpdates"`
	OnErrorStrategy   string        `json:"onErrorStrategy"` // "continue", "stop", "retry"
	MaxRetries        int           `json:"maxRetries"`
}

// Progress represents streaming analysis progress
type Progress struct {
	Phase           fastcontext.AnalysisPhase `json:"phase"`
	CurrentFile     string                   `json:"currentFile"`
	ProcessedFiles  int                      `json:"processedFiles"`
	TotalFiles      int                      `json:"totalFiles"`
	ProcessedBytes  int64                    `json:"processedBytes"`
	TotalBytes      int64                    `json:"totalBytes"`
	CurrentSymbols  int                      `json:"currentSymbols"`
	TotalSymbols    int                      `json:"totalSymbols"`
	Percentage      float64                  `json:"percentage"`
	StartTime       time.Time                `json:"startTime"`
	EstimatedTime   time.Duration            `json:"estimatedTime"`
	CurrentThroughput float64                `json:"currentThroughput"`
	AverageThroughput float64                `json:"averageThroughput"`
	Message         string                   `json:"message"`
	Warnings        []string                 `json:"warnings"`
}

// PartialResult represents a partial analysis result during streaming
type PartialResult struct {
	BatchID       int                     `json:"batchId"`
	Symbols       []*fastcontext.Symbol   `json:"symbols"`
	Dependencies  []*fastcontext.Dependency `json:"dependencies"`
	Files         []string                `json:"files"`
	Metrics       *StreamMetrics         `json:"metrics"`
	Timestamp     time.Time               `json:"timestamp"`
	IsComplete    bool                    `json:"isComplete"`
	HasError      bool                    `json:"hasError"`
	Error         error                   `json:"error,omitempty"`
}

// StreamStats contains statistics for the streaming session
type StreamStats struct {
	StartTime          time.Time     `json:"startTime"`
	EndTime            time.Time     `json:"endTime,omitempty"`
	TotalDuration      time.Duration `json:"totalDuration"`
	FilesProcessed     int           `json:"filesProcessed"`
	SymbolsFound       int           `json:"symbolsFound"`
	DependenciesFound  int           `json:"dependenciesFound"`
	BytesProcessed     int64         `json:"bytesProcessed"`
	ErrorCount         int           `json:"errorCount"`
	WarningCount       int           `json:"warningCount"`
	Throughput         float64       `json:"throughput"` // files per second
	PeakMemory         int64         `json:"peakMemory"`
	AverageThroughput  float64       `json:"averageThroughput"`
}

// StreamMetrics contains metrics for the current batch
type StreamMetrics struct {
	BatchNumber        int                      `json:"batchNumber"`
	FilesInBatch       int                      `json:"filesInBatch"`
	SymbolsInBatch     int                      `json:"symbolsInBatch"`
	DependenciesInBatch int                      `json:"dependenciesInBatch"`
	BatchDuration      time.Duration            `json:"batchDuration"`
	BatchThroughput    float64                  `json:"batchThroughput"`
	CPUUsage          float64                  `json:"cpuUsage"`
	MemoryUsage       int64                    `json:"memoryUsage"`
	DiskUsage         int64                    `json:"diskUsage"`
	LanguageStats     map[string]int           `json:"languageStats"`
	SymbolTypeStats   map[string]int           `json:"symbolTypeStats"`
	FileTypeStats     map[string]int           `json:"fileTypeStats"`
}

// NewAnalyzer creates a new streaming analyzer
func NewAnalyzer(cfg *config.Config, opts *StreamingOptions) (*Analyzer, error) {
	if cfg == nil {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "config cannot be nil")
	}

	baseAnalyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)
	if err != nil {
		return nil, err
	}

	if opts == nil {
		opts = &StreamingOptions{
			BufferSize:       1000,
			BatchSize:        50,
			FlushInterval:    5 * time.Second,
			ProgressInterval: 1 * time.Second,
			MaxConcurrent:    4,
			EnableMetrics:    true,
			RealTimeUpdates:  true,
			OnErrorStrategy:  "continue",
			MaxRetries:       3,
		}
	}

	return &Analyzer{
		config:        cfg,
		baseAnalyzer:  baseAnalyzer,
		progressChan:  make(chan *Progress, opts.BufferSize),
		resultChan:    make(chan *PartialResult, opts.BufferSize),
		errorChan:     make(chan error, 100),
		stats:         &StreamStats{StartTime: time.Now()},
	}, nil
}

// AnalyzeStream starts streaming analysis
func (a *Analyzer) AnalyzeStream(ctx context.Context) error {
	a.streamMutex.Lock()
	if a.isStreaming {
		a.streamMutex.Unlock()
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "analysis already in progress")
	}
	a.isStreaming = true
	a.streamMutex.Unlock()

	// Create context for cancellation
	analysisCtx, cancel := context.WithCancel(ctx)
	a.cancelFunc = cancel

	// Start the streaming analysis
	a.wg.Add(1)
	go a.runStreamingAnalysis(analysisCtx)

	return nil
}

// GetProgress returns the current progress channel
func (a *Analyzer) GetProgress() <-chan *Progress {
	return a.progressChan
}

// GetResults returns the result channel
func (a *Analyzer) GetResults() <-chan *PartialResult {
	return a.resultChan
}

// GetErrors returns the error channel
func (a *Analyzer) GetErrors() <-chan error {
	return a.errorChan
}

// Stop stops the streaming analysis
func (a *Analyzer) Stop() error {
	a.streamMutex.Lock()
	defer a.streamMutex.Unlock()

	if !a.isStreaming {
		return nil
	}

	if a.cancelFunc != nil {
		a.cancelFunc()
	}

	// Close channels
	close(a.progressChan)
	close(a.resultChan)
	close(a.errorChan)

	a.isStreaming = false
	a.stats.EndTime = time.Now()
	a.stats.TotalDuration = a.stats.EndTime.Sub(a.stats.StartTime)

	// Calculate final throughput
	if a.stats.TotalDuration > 0 {
		a.stats.Throughput = float64(a.stats.FilesProcessed) / a.stats.TotalDuration.Seconds()
	}

	a.wg.Wait()

	return nil
}

// IsStreaming returns whether the analyzer is currently streaming
func (a *Analyzer) IsStreaming() bool {
	a.streamMutex.RLock()
	defer a.streamMutex.RUnlock()
	return a.isStreaming
}

// GetStats returns the current streaming statistics
func (a *Analyzer) GetStats() *StreamStats {
	// Return a copy of the stats
	statsCopy := *a.stats
	return &statsCopy
}

// Pause pauses the streaming analysis
func (a *Analyzer) Pause() error {
	a.streamMutex.Lock()
	defer a.streamMutex.Unlock()

	if !a.isStreaming {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "no analysis in progress")
	}

	if a.cancelFunc != nil {
		a.cancelFunc()
	}

	return nil
}

// Resume resumes the streaming analysis
func (a *Analyzer) Resume(ctx context.Context) error {
	a.streamMutex.Lock()
	defer a.streamMutex.Unlock()

	if a.isStreaming {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "analysis already in progress")
	}

	analysisCtx, cancel := context.WithCancel(ctx)
	a.cancelFunc = cancel

	a.isStreaming = true

	a.wg.Add(1)
	go a.runStreamingAnalysis(analysisCtx)

	return nil
}

// runStreamingAnalysis is the main streaming analysis loop
func (a *Analyzer) runStreamingAnalysis(ctx context.Context) {
	defer a.wg.Done()

	// Send initial progress
	progress := &Progress{
		Phase:      fastcontext.PhaseDiscovery,
		StartTime:  time.Now(),
		Message:    "Starting analysis...",
	}
	a.progressChan <- progress

	// Simulate streaming analysis with batches
	batchID := 0
	totalFiles := 100 // Would be determined from actual file discovery
	processedFiles := 0

	for processedFiles < totalFiles {
		select {
		case <-ctx.Done():
			// Context was cancelled
			a.sendCompletionSignal(batchID, true, ctx.Err())
			return
		default:
			// Process next batch
			batchResult := a.processBatch(ctx, batchID, processedFiles, totalFiles)
			
			// Send result
			a.resultChan <- batchResult
			
			// Update stats
			a.stats.FilesProcessed += len(batchResult.Files)
			a.stats.SymbolsFound += len(batchResult.Symbols)
			a.stats.DependenciesFound += len(batchResult.Dependencies)
			
			// Send progress update
			processedFiles += len(batchResult.Files)
			progress = a.createProgress(progress, processedFiles, totalFiles, batchResult)
			a.progressChan <- progress
			
			batchID++
			
			// Small delay to simulate processing
			time.Sleep(100 * time.Millisecond)
		}
	}

	// Send completion signal
	a.sendCompletionSignal(batchID, false, nil)
}

// processBatch processes a batch of files
func (a *Analyzer) processBatch(ctx context.Context, batchID, processedFiles, totalFiles int) *PartialResult {
	startTime := time.Now()
	
	// Simulate batch processing
	batchSize := 10
	if processedFiles+batchSize > totalFiles {
		batchSize = totalFiles - processedFiles
	}

	// Mock data for demonstration
	symbols := []*fastcontext.Symbol{
		{
			ID:         fmt.Sprintf("symbol_%d_%d", batchID, 0),
			Name:       fmt.Sprintf("function_%d", batchID),
			Kind:       fastcontext.SymbolKindFunction,
			Language:   "Go",
			File:       fmt.Sprintf("file_%d.go", processedFiles),
			LineStart:  10,
			LineEnd:    25,
			Complexity: 3.5,
		},
	}

	dependencies := []*fastcontext.Dependency{
		{
			From:     fmt.Sprintf("symbol_%d_%d", batchID, 0),
			To:       "fmt",
			Type:     fastcontext.DepTypeImports,
			Strength: 1.0,
		},
	}

	files := []string{}
	for i := 0; i < batchSize; i++ {
		files = append(files, fmt.Sprintf("file_%d.go", processedFiles+i))
	}

	metrics := &StreamMetrics{
		BatchNumber:    batchID,
		FilesInBatch:   batchSize,
		SymbolsInBatch: len(symbols),
		DependenciesInBatch: len(dependencies),
		BatchDuration:  time.Since(startTime),
		BatchThroughput: float64(batchSize) / time.Since(startTime).Seconds(),
		LanguageStats: map[string]int{
			"Go": batchSize,
		},
		SymbolTypeStats: map[string]int{
			"function": len(symbols),
		},
		FileTypeStats: map[string]int{
			".go": batchSize,
		},
	}

	return &PartialResult{
		BatchID:      batchID,
		Symbols:      symbols,
		Dependencies: dependencies,
		Files:        files,
		Metrics:      metrics,
		Timestamp:    time.Now(),
		IsComplete:   false,
		HasError:     false,
	}
}

// createProgress creates a progress update
func (a *Analyzer) createProgress(prevProgress *Progress, processedFiles, totalFiles int, result *PartialResult) *Progress {
	now := time.Now()
	percentage := float64(processedFiles) / float64(totalFiles) * 100
	
	// Calculate estimated time remaining
	remainingFiles := totalFiles - processedFiles
	timePerFile := now.Sub(prevProgress.StartTime).Seconds() / float64(processedFiles)
	estimatedTimeRemaining := time.Duration(float64(remainingFiles) * timePerFile) * time.Second
	
	// Calculate throughput
	elapsed := now.Sub(prevProgress.StartTime)
	currentThroughput := float64(len(result.Files)) / result.Metrics.BatchDuration.Seconds()
	averageThroughput := float64(processedFiles) / elapsed.Seconds()

	return &Progress{
		Phase:            fastcontext.PhaseSymbolExtraction,
		CurrentFile:      result.Files[len(result.Files)-1],
		ProcessedFiles:   processedFiles,
		TotalFiles:       totalFiles,
		CurrentSymbols:   a.stats.SymbolsFound + len(result.Symbols),
		TotalSymbols:     totalFiles * 2, // Estimate
		Percentage:       percentage,
		StartTime:        prevProgress.StartTime,
		EstimatedTime:    estimatedTimeRemaining,
		CurrentThroughput: currentThroughput,
		AverageThroughput: averageThroughput,
		Message:          fmt.Sprintf("Processed batch %d", result.BatchID),
		Warnings:         []string{},
	}
}

// sendCompletionSignal sends the final result
func (a *Analyzer) sendCompletionSignal(batchID int, cancelled bool, err error) {
	result := &PartialResult{
		BatchID:    batchID,
		Timestamp:  time.Now(),
		IsComplete: true,
		HasError:   cancelled,
		Error:      err,
	}
	
	a.resultChan <- result
	
	// Send final progress
	finalProgress := &Progress{
		Phase:           fastcontext.PhaseComplete,
		ProcessedFiles:  a.stats.FilesProcessed,
		TotalFiles:      a.stats.FilesProcessed,
		Percentage:      100.0,
		StartTime:       a.stats.StartTime,
		EstimatedTime:   0,
		Message:         "Analysis complete",
	}
	
	if cancelled {
		finalProgress.Message = "Analysis cancelled"
		finalProgress.Warnings = append(finalProgress.Warnings, "Analysis was cancelled before completion")
	}
	
	a.progressChan <- finalProgress
}

// GetRealTimeMetrics returns real-time metrics if enabled
func (a *Analyzer) GetRealTimeMetrics() (*StreamMetrics, error) {
	if !a.IsStreaming() {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "no active streaming session")
	}

	// Mock real-time metrics
	return &StreamMetrics{
		CPUUsage:     45.5,
		MemoryUsage:  1024 * 1024 * 100, // 100MB
		DiskUsage:    1024 * 1024 * 50,  // 50MB
	}, nil
}

// GetBatchHistory returns the history of processed batches
func (a *Analyzer) GetBatchHistory() []*PartialResult {
	// Would maintain a history of batches
	return []*PartialResult{}
}

// ExportProgress exports the current progress in various formats
func (a *Analyzer) ExportProgress(format string) (string, error) {
	progress := &Progress{
		Phase:           fastcontext.PhaseSymbolExtraction,
		ProcessedFiles:  a.stats.FilesProcessed,
		TotalFiles:      a.stats.FilesProcessed + 50, // Estimate
		Percentage:      float64(a.stats.FilesProcessed) / float64(a.stats.FilesProcessed+50) * 100,
		StartTime:       a.stats.StartTime,
		Message:         "Analysis in progress",
	}

	switch format {
	case "json":
		data, err := json.MarshalIndent(progress, "", "  ")
		if err != nil {
			return "", err
		}
		return string(data), nil
	case "text":
		return fmt.Sprintf("Progress: %.1f%% (%d/%d files)", progress.Percentage, progress.ProcessedFiles, progress.TotalFiles), nil
	default:
		return "", fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "unsupported format")
	}
}

// ValidateStreamingOptions validates the streaming options
func ValidateStreamingOptions(opts *StreamingOptions) error {
	if opts.BufferSize <= 0 {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "buffer size must be positive")
	}
	if opts.BatchSize <= 0 {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "batch size must be positive")
	}
	if opts.MaxConcurrent <= 0 {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "max concurrent must be positive")
	}
	if opts.FlushInterval <= 0 {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "flush interval must be positive")
	}
	
	switch opts.OnErrorStrategy {
	case "continue", "stop", "retry":
		// Valid strategies
	default:
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "invalid error strategy")
	}
	
	return nil
}