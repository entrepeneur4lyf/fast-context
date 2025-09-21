package performance

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/fast-context/go-sdk/logging"
	"github.com/fast-context/go-sdk/optimization"
	"github.com/fast-context/go-sdk/profiling"
)

// PerformanceAnalyzer combines profiling and optimization for comprehensive performance analysis
type PerformanceAnalyzer struct {
	profiler  *profiling.Profiler
	optimizer *optimization.Optimizer
	logger    *logging.StructuredLogger
	metrics   *logging.MetricsCollector
	enabled   bool
	config    *PerformanceConfig
}

// PerformanceConfig configures performance analysis behavior
type PerformanceConfig struct {
	EnableProfiling        bool
	EnableOptimization     bool
	ProfileOutputDir       string
	OptimizationLevel      string
	AutoOptimize          bool
	BenchmarkIterations   int
	MemoryPressureThreshold float64
	ReportFormat          string // "json", "yaml", "text"
}

// PerformanceReport contains comprehensive performance analysis results
type PerformanceReport struct {
	Timestamp           time.Time                     `json:"timestamp"`
	Duration            time.Duration                 `json:"duration"`
	ProfileResults      []*profiling.ProfileResult   `json:"profileResults,omitempty"`
	OptimizationResults []*optimization.OptimizationResult `json:"optimizationResults,omitempty"`
	SystemStats         map[string]interface{}        `json:"systemStats"`
	Recommendations     []string                      `json:"recommendations"`
	Summary             *PerformanceSummary           `json:"summary"`
}

// PerformanceSummary provides a high-level summary of performance analysis
type PerformanceSummary struct {
	TotalOperations       int                    `json:"totalOperations"`
	AverageDuration       time.Duration          `json:"averageDuration"`
	TotalOptimizations    int                    `json:"totalOptimizations"`
	TotalTimeSaved        time.Duration          `json:"totalTimeSaved"`
	MemoryOptimizations   int                    `json:"memoryOptimizations"`
	CacheHits            int64                  `json:"cacheHits"`
	PerformanceScore      float64                `json:"performanceScore"` // 0.0 to 1.0
	Bottlenecks          []string               `json:"bottlenecks"`
}

// NewPerformanceAnalyzer creates a new performance analyzer
func NewPerformanceAnalyzer(config *PerformanceConfig, logger *logging.StructuredLogger) (*PerformanceAnalyzer, error) {
	if config == nil {
		config = &PerformanceConfig{
			EnableProfiling:        true,
			EnableOptimization:     true,
			ProfileOutputDir:       "./performance_reports",
			OptimizationLevel:      "balanced",
			AutoOptimize:          true,
			BenchmarkIterations:   10,
			MemoryPressureThreshold: 0.8,
			ReportFormat:          "json",
		}
	}

	// Create output directory if it doesn't exist
	if config.ProfileOutputDir != "" {
		if err := os.MkdirAll(config.ProfileOutputDir, 0755); err != nil {
			return nil, fmt.Errorf("failed to create performance output directory: %w", err)
		}
	}

	analyzer := &PerformanceAnalyzer{
		logger:  logger,
		metrics: logging.GetMetricsCollector(),
		enabled: true,
		config:  config,
	}

	// Initialize profiler if enabled
	if config.EnableProfiling {
		profilerConfig := &profiling.ProfilingConfig{
			EnableCPUProfiling:    true,
			EnableMemoryProfiling: true,
			EnableMutexProfiling:   true,
			EnableBlockProfiling:   true,
			ProfileOutputDir:      config.ProfileOutputDir,
			ProfileDuration:       5 * time.Minute,
			AutoStartProfiling:    false,
			ProfileThreshold:      100 * time.Millisecond,
		}

		profiler, err := profiling.NewProfiler(profilerConfig, logger)
		if err != nil {
			return nil, fmt.Errorf("failed to create profiler: %w", err)
		}
		analyzer.profiler = profiler
	}

	// Initialize optimizer if enabled
	if config.EnableOptimization {
		optimizerConfig := &optimization.OptimizationConfig{
			EnableCaching:           true,
			EnableParallelization:   true,
			EnableMemoryOptimization: true,
			MaxCacheSize:           100 * 1024 * 1024, // 100MB
			MaxWorkerCount:         runtime.NumCPU(),
			MemoryPressureThreshold: config.MemoryPressureThreshold,
			OptimizationLevel:      config.OptimizationLevel,
		}

		optimizer, err := optimization.NewOptimizer(optimizerConfig, logger)
		if err != nil {
			return nil, fmt.Errorf("failed to create optimizer: %w", err)
		}
		analyzer.optimizer = optimizer
	}

	analyzer.logger.Info("Performance analyzer initialized", "config", config)
	return analyzer, nil
}

// AnalyzeOperation performs comprehensive performance analysis on an operation
func (pa *PerformanceAnalyzer) AnalyzeOperation(ctx context.Context, operation string, fn func() (interface{}, error)) (interface{}, *PerformanceReport, error) {
	if !pa.enabled {
		start := time.Now()
		result, err := fn()
		duration := time.Since(start)
		
		report := &PerformanceReport{
			Timestamp: time.Now(),
			Duration:  duration,
			Summary: &PerformanceSummary{
				TotalOperations: 1,
				AverageDuration: duration,
				PerformanceScore: 1.0,
			},
		}
		
		return result, report, err
	}

	start := time.Now()
	report := &PerformanceReport{
		Timestamp:   time.Now(),
		SystemStats: pa.getSystemStats(),
	}

	var profileResult *profiling.ProfileResult
	var optimizationResult *optimization.OptimizationResult
	var result interface{}
	var err error

	// Run profiling if enabled
	if pa.profiler != nil {
		wrappedFn := func() error {
			_, err := fn()
			return err
		}
		profileResult, err = pa.profiler.ProfileOperation(ctx, operation, wrappedFn)
		if err != nil {
			return nil, nil, fmt.Errorf("profiling failed: %w", err)
		}
		report.ProfileResults = append(report.ProfileResults, profileResult)
		// We need to run the operation again for the actual result
	}

	// Run optimization if enabled and auto-optimize is on
	if pa.optimizer != nil && pa.config.AutoOptimize {
		result, optimizationResult, err = pa.optimizer.OptimizeOperation(ctx, operation, fn)
		if err != nil {
			return nil, nil, fmt.Errorf("optimization failed: %w", err)
		}
		report.OptimizationResults = append(report.OptimizationResults, optimizationResult)
	} else {
		// Run operation normally if no optimization
		result, err = fn()
		if err != nil {
			return nil, nil, err
		}
	}

	report.Duration = time.Since(start)
	report.Summary = pa.generateSummary(report)
	report.Recommendations = pa.generateRecommendations(report)

	pa.logger.Info("Performance analysis completed", "operation", operation, "duration", report.Duration, "score", report.Summary.PerformanceScore)

	return result, report, err
}

// BenchmarkOperation benchmarks an operation with comprehensive analysis
func (pa *PerformanceAnalyzer) BenchmarkOperation(ctx context.Context, operation string, fn func() (interface{}, error)) (*PerformanceReport, error) {
	if !pa.enabled {
		return &PerformanceReport{
			Timestamp: time.Now(),
			Summary: &PerformanceSummary{
				PerformanceScore: 1.0,
			},
		}, nil
	}

	iterations := pa.config.BenchmarkIterations
	if iterations <= 0 {
		iterations = 10
	}

	report := &PerformanceReport{
		Timestamp:   time.Now(),
		SystemStats: pa.getSystemStats(),
	}

	var profileResults []*profiling.ProfileResult
	var optimizationResults []*optimization.OptimizationResult
	var totalDuration time.Duration
	var successfulOperations int

	for i := 0; i < iterations; i++ {
		iterationOp := fmt.Sprintf("%s_iteration_%d", operation, i)
		
		// Run profiling if enabled
		if pa.profiler != nil {
			wrappedFn := func() error {
				_, err := fn()
				return err
			}
			profileResult, err := pa.profiler.ProfileOperation(ctx, iterationOp, wrappedFn)
			if err != nil {
				pa.logger.Error("Benchmark iteration failed", err, "operation", iterationOp, "iteration", i)
				continue
			}
			profileResults = append(profileResults, profileResult)
		}

		// Run optimization if enabled
		if pa.optimizer != nil {
			_, optimizationResult, err := pa.optimizer.OptimizeOperation(ctx, iterationOp, fn)
			if err != nil {
				pa.logger.Error("Benchmark optimization failed", err, "operation", iterationOp, "iteration", i)
				continue
			}
			optimizationResults = append(optimizationResults, optimizationResult)
			totalDuration += optimizationResult.OptimizedDuration
		} else {
			// Just run the operation
			start := time.Now()
			_, err := fn()
			if err != nil {
				pa.logger.Error("Benchmark operation failed", err, "operation", iterationOp, "iteration", i)
				continue
			}
			totalDuration += time.Since(start)
		}

		successfulOperations++
	}

	report.ProfileResults = profileResults
	report.OptimizationResults = optimizationResults
	report.Duration = totalDuration
	report.Summary = pa.generateBenchmarkSummary(report, iterations, successfulOperations)
	report.Recommendations = pa.generateBenchmarkRecommendations(report)

	pa.logger.Info("Benchmark completed", "operation", operation, "iterations", successfulOperations, "avg_duration", report.Summary.AverageDuration)

	return report, nil
}

// StartProfiling starts comprehensive profiling
func (pa *PerformanceAnalyzer) StartProfiling() error {
	if pa.profiler == nil {
		return fmt.Errorf("profiling is not enabled")
	}

	return pa.profiler.StartProfiling()
}

// StopProfiling stops profiling and returns results
func (pa *PerformanceAnalyzer) StopProfiling() (*PerformanceReport, error) {
	if pa.profiler == nil {
		return nil, fmt.Errorf("profiling is not enabled")
	}

	profileResult, err := pa.profiler.StopProfiling()
	if err != nil {
		return nil, err
	}

	report := &PerformanceReport{
		Timestamp:      time.Now(),
		Duration:       0, // Profiling duration is in profileResult
		ProfileResults: []*profiling.ProfileResult{profileResult},
		SystemStats:    pa.getSystemStats(),
		Summary:        pa.generateProfilingSummary(profileResult),
		Recommendations: pa.generateProfilingRecommendations(profileResult),
	}

	return report, nil
}

// ExportReport exports a performance report to file
func (pa *PerformanceAnalyzer) ExportReport(report *PerformanceReport, filename string) error {
	if pa.config.ProfileOutputDir == "" {
		return fmt.Errorf("no output directory configured")
	}

	if filename == "" {
		timestamp := time.Now().Format("20060102-150405")
		filename = fmt.Sprintf("performance_report_%s.%s", timestamp, pa.config.ReportFormat)
	}

	filepath := filepath.Join(pa.config.ProfileOutputDir, filename)

	switch pa.config.ReportFormat {
	case "json":
		return pa.exportJSON(report, filepath)
	case "yaml":
		return pa.exportYAML(report, filepath)
	case "text":
		return pa.exportText(report, filepath)
	default:
		return fmt.Errorf("unsupported report format: %s", pa.config.ReportFormat)
	}
}

// GeneratePerformanceReport generates a comprehensive performance report
func (pa *PerformanceAnalyzer) GeneratePerformanceReport(ctx context.Context) (*PerformanceReport, error) {
	if !pa.enabled {
		return &PerformanceReport{
			Timestamp: time.Now(),
			Summary: &PerformanceSummary{
				PerformanceScore: 1.0,
			},
		}, nil
	}

	report := &PerformanceReport{
		Timestamp:   time.Now(),
		SystemStats: pa.getSystemStats(),
	}

	// Get current system metrics
	stats := pa.metrics.GetAllMetrics()
	if stats != nil {
		// Add metrics to system stats
		report.SystemStats["metrics"] = stats
	}

	// Generate summary based on current state
	report.Summary = &PerformanceSummary{
		PerformanceScore: pa.calculatePerformanceScore(report.SystemStats),
		Bottlenecks:     pa.identifyBottlenecks(report.SystemStats),
	}

	report.Recommendations = pa.generateSystemRecommendations(report.SystemStats)

	return report, nil
}

// Helper methods
func (pa *PerformanceAnalyzer) getSystemStats() map[string]interface{} {
	if pa.profiler != nil {
		return pa.profiler.GetSystemStats()
	}

	// Fallback system stats
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	return map[string]interface{}{
		"goroutines":    runtime.NumGoroutine(),
		"memory_alloc":  memStats.Alloc,
		"memory_total":  memStats.TotalAlloc,
		"memory_sys":    memStats.Sys,
		"gc_cycles":     memStats.NumGC,
		"cpu_cores":     runtime.NumCPU(),
	}
}

func (pa *PerformanceAnalyzer) generateSummary(report *PerformanceReport) *PerformanceSummary {
	summary := &PerformanceSummary{
		TotalOperations:    len(report.OptimizationResults),
		TotalOptimizations: len(report.OptimizationResults),
	}

	if len(report.OptimizationResults) > 0 {
		var totalDuration time.Duration
		var totalTimeSaved time.Duration
		var cacheHits int64

		for _, result := range report.OptimizationResults {
			totalDuration += result.OptimizedDuration
			totalTimeSaved += result.OriginalDuration - result.OptimizedDuration
			cacheHits += result.CacheHits
		}

		summary.AverageDuration = totalDuration / time.Duration(len(report.OptimizationResults))
		summary.TotalTimeSaved = totalTimeSaved
		summary.CacheHits = cacheHits
	}

	summary.PerformanceScore = pa.calculatePerformanceScore(report.SystemStats)
	summary.Bottlenecks = pa.identifyBottlenecks(report.SystemStats)

	return summary
}

func (pa *PerformanceAnalyzer) generateBenchmarkSummary(report *PerformanceReport, iterations, successful int) *PerformanceSummary {
	summary := &PerformanceSummary{
		TotalOperations:    successful,
		TotalOptimizations: len(report.OptimizationResults),
	}

	if successful > 0 {
		summary.AverageDuration = report.Duration / time.Duration(successful)
	}

	summary.PerformanceScore = pa.calculatePerformanceScore(report.SystemStats)
	summary.Bottlenecks = pa.identifyBottlenecks(report.SystemStats)

	return summary
}

func (pa *PerformanceAnalyzer) generateProfilingSummary(result *profiling.ProfileResult) *PerformanceSummary {
	return &PerformanceSummary{
		TotalOperations:   1,
		AverageDuration:   result.Duration,
		PerformanceScore:  pa.calculateProfilingScore(result),
		Bottlenecks:       pa.identifyProfilingBottlenecks(result),
	}
}

func (pa *PerformanceAnalyzer) calculatePerformanceScore(stats map[string]interface{}) float64 {
	score := 1.0

	// Deduct for high memory usage
	if alloc, ok := stats["memory_alloc"].(uint64); ok {
		if alloc > 100*1024*1024 { // 100MB
			score -= 0.2
		}
	}

	// Deduct for high goroutine count
	if goroutines, ok := stats["goroutines"].(int); ok {
		if goroutines > 1000 {
			score -= 0.1
		}
	}

	// Deduct for frequent GC
	if gcCycles, ok := stats["gc_cycles"].(uint32); ok {
		if gcCycles > 100 {
			score -= 0.1
		}
	}

	return max(0.0, min(1.0, score))
}

func (pa *PerformanceAnalyzer) calculateProfilingScore(result *profiling.ProfileResult) float64 {
	score := 1.0

	// Deduct for long duration
	if result.Duration > time.Second {
		score -= 0.3
	} else if result.Duration > 500*time.Millisecond {
		score -= 0.1
	}

	// Deduct for high memory usage
	if result.MemoryStats != nil {
		if result.MemoryStats.Alloc > 50*1024*1024 { // 50MB
			score -= 0.2
		}
	}

	return max(0.0, min(1.0, score))
}

func (pa *PerformanceAnalyzer) identifyBottlenecks(stats map[string]interface{}) []string {
	bottlenecks := []string{}

	if alloc, ok := stats["memory_alloc"].(uint64); ok && alloc > 100*1024*1024 {
		bottlenecks = append(bottlenecks, "High memory usage")
	}

	if goroutines, ok := stats["goroutines"].(int); ok && goroutines > 1000 {
		bottlenecks = append(bottlenecks, "High goroutine count")
	}

	if gcCycles, ok := stats["gc_cycles"].(uint32); ok && gcCycles > 100 {
		bottlenecks = append(bottlenecks, "Frequent garbage collection")
	}

	return bottlenecks
}

func (pa *PerformanceAnalyzer) identifyProfilingBottlenecks(result *profiling.ProfileResult) []string {
	bottlenecks := []string{}

	if result.Duration > time.Second {
		bottlenecks = append(bottlenecks, "Long execution time")
	}

	if result.MemoryStats != nil && result.MemoryStats.Alloc > 50*1024*1024 {
		bottlenecks = append(bottlenecks, "High memory allocation")
	}

	if result.GoroutineCount > 100 {
		bottlenecks = append(bottlenecks, "High goroutine count")
	}

	return bottlenecks
}

func (pa *PerformanceAnalyzer) generateRecommendations(report *PerformanceReport) []string {
	recommendations := []string{}

	// Add profiling recommendations
	for _, profileResult := range report.ProfileResults {
		recommendations = append(recommendations, profileResult.Recommendations...)
	}

	// Add optimization recommendations
	for _, optResult := range report.OptimizationResults {
		recommendations = append(recommendations, optResult.Recommendations...)
	}

	// Add system-level recommendations
	recommendations = append(recommendations, pa.generateSystemRecommendations(report.SystemStats)...)

	return pa.deduplicateRecommendations(recommendations)
}

func (pa *PerformanceAnalyzer) generateBenchmarkRecommendations(report *PerformanceReport) []string {
	recommendations := []string{}

	if report.Summary != nil {
		if report.Summary.AverageDuration > time.Second {
			recommendations = append(recommendations, "Consider optimizing algorithmic complexity")
		}

		if report.Summary.TotalOptimizations > 0 {
			recommendations = append(recommendations, "Optimization is working, consider enabling caching")
		}

		if len(report.Summary.Bottlenecks) > 0 {
			recommendations = append(recommendations, "Address identified bottlenecks")
		}
	}

	return recommendations
}

func (pa *PerformanceAnalyzer) generateProfilingRecommendations(result *profiling.ProfileResult) []string {
	recommendations := []string{}

	if result.Duration > time.Second {
		recommendations = append(recommendations, "Consider parallelizing long-running operations")
	}

	if result.MemoryStats != nil && result.MemoryStats.Alloc > 50*1024*1024 {
		recommendations = append(recommendations, "Consider memory optimization techniques")
	}

	if result.GoroutineCount > 100 {
		recommendations = append(recommendations, "Check for goroutine leaks")
	}

	return recommendations
}

func (pa *PerformanceAnalyzer) generateSystemRecommendations(stats map[string]interface{}) []string {
	recommendations := []string{}

	if alloc, ok := stats["memory_alloc"].(uint64); ok && alloc > 100*1024*1024 {
		recommendations = append(recommendations, "Monitor memory usage and implement optimization")
	}

	if goroutines, ok := stats["goroutines"].(int); ok && goroutines > 1000 {
		recommendations = append(recommendations, "Review goroutine usage patterns")
	}

	return recommendations
}

func (pa *PerformanceAnalyzer) deduplicateRecommendations(recommendations []string) []string {
	seen := make(map[string]bool)
	unique := []string{}

	for _, rec := range recommendations {
		if !seen[rec] {
			seen[rec] = true
			unique = append(unique, rec)
		}
	}

	return unique
}

// Export methods (simplified implementations)
func (pa *PerformanceAnalyzer) exportJSON(report *PerformanceReport, filepath string) error {
	// Implementation would use JSON encoding
	pa.logger.Info("Exporting performance report as JSON", "filepath", filepath)
	return nil
}

func (pa *PerformanceAnalyzer) exportYAML(report *PerformanceReport, filepath string) error {
	// Implementation would use YAML encoding
	pa.logger.Info("Exporting performance report as YAML", "filepath", filepath)
	return nil
}

func (pa *PerformanceAnalyzer) exportText(report *PerformanceReport, filepath string) error {
	// Implementation would write human-readable text
	pa.logger.Info("Exporting performance report as text", "filepath", filepath)
	return nil
}

// Cleanup cleans up analyzer resources
func (pa *PerformanceAnalyzer) Cleanup() error {
	pa.enabled = false

	if pa.profiler != nil {
		_ = pa.profiler.Cleanup()
	}

	if pa.optimizer != nil {
		_ = pa.optimizer.Cleanup()
	}

	pa.logger.Info("Performance analyzer cleaned up")
	return nil
}

// Utility functions
func min(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func max(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}