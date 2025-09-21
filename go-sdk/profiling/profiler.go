package profiling

import (
	"context"
	"fmt"
	"os"
	"runtime"
	"runtime/pprof"
	"sync"
	"time"

	"github.com/fast-context/go-sdk/logging"
)

// Profiler manages performance profiling and optimization
type Profiler struct {
	cpuProfile     *os.File
	memProfile     *os.File
	mutexProfile   *os.File
	blockProfile   *os.File
	logger         *logging.StructuredLogger
	metrics        *logging.MetricsCollector
	enabled        bool
	profilingTypes map[string]bool
	mutex          sync.RWMutex
}

// ProfilingConfig configures profiling behavior
type ProfilingConfig struct {
	EnableCPUProfiling     bool
	EnableMemoryProfiling  bool
	EnableMutexProfiling   bool
	EnableBlockProfiling   bool
	ProfileOutputDir       string
	ProfileDuration        time.Duration
	AutoStartProfiling     bool
	ProfileThreshold       time.Duration // Operations longer than this trigger profiling
	MaxProfileFileSize     int64
}

// ProfileResult contains profiling results and analysis
type ProfileResult struct {
	Operation      string        `json:"operation"`
	Duration       time.Duration `json:"duration"`
	CPUProfilePath  string        `json:"cpuProfilePath,omitempty"`
	MemProfilePath  string        `json:"memProfilePath,omitempty"`
	MutexProfilePath string      `json:"mutexProfilePath,omitempty"`
	BlockProfilePath string      `json:"blockProfilePath,omitempty"`
	MemoryStats     *MemoryStats  `json:"memoryStats"`
	GoroutineCount  int           `json:"goroutineCount"`
	CPUUsage        float64       `json:"cpuUsage"`
	Recommendations []string      `json:"recommendations"`
}

// MemoryStats captures memory usage statistics
type MemoryStats struct {
	Alloc      uint64 `json:"alloc"`
	TotalAlloc uint64 `json:"totalAlloc"`
	Sys        uint64 `json:"sys"`
	NumGC      uint32 `json:"numGC"`
	GCPause    uint64 `json:"gcPause"`
	HeapAlloc  uint64 `json:"heapAlloc"`
	HeapSys    uint64 `json:"heapSys"`
}

// NewProfiler creates a new performance profiler
func NewProfiler(config *ProfilingConfig, logger *logging.StructuredLogger) (*Profiler, error) {
	if config == nil {
		config = &ProfilingConfig{
			EnableCPUProfiling:    true,
			EnableMemoryProfiling: true,
			ProfileOutputDir:      "./profiles",
			ProfileDuration:       5 * time.Minute,
			AutoStartProfiling:    false,
			ProfileThreshold:      100 * time.Millisecond,
			MaxProfileFileSize:    100 * 1024 * 1024, // 100MB
		}
	}

	// Create output directory if it doesn't exist
	if config.ProfileOutputDir != "" {
		if err := os.MkdirAll(config.ProfileOutputDir, 0755); err != nil {
			return nil, fmt.Errorf("failed to create profile output directory: %w", err)
		}
	}

	profiler := &Profiler{
		logger:         logger,
		metrics:        logging.GetMetricsCollector(),
		enabled:        true,
		profilingTypes: make(map[string]bool),
	}

	// Configure profiling types
	if config.EnableCPUProfiling {
		profiler.profilingTypes["cpu"] = true
	}
	if config.EnableMemoryProfiling {
		profiler.profilingTypes["memory"] = true
	}
	if config.EnableMutexProfiling {
		profiler.profilingTypes["mutex"] = true
	}
	if config.EnableBlockProfiling {
		profiler.profilingTypes["block"] = true
	}

	// Auto-start profiling if configured
	if config.AutoStartProfiling {
		if err := profiler.StartProfiling(); err != nil {
			return nil, fmt.Errorf("failed to auto-start profiling: %w", err)
		}
	}

	return profiler, nil
}

// StartProfiling starts all configured profilers
func (p *Profiler) StartProfiling() error {
	p.mutex.Lock()
	defer p.mutex.Unlock()

	if !p.enabled {
		return fmt.Errorf("profiler is disabled")
	}

	timestamp := time.Now().Format("20060102-150405")
	basePath := fmt.Sprintf("./profiles/profile-%s", timestamp)

	var err error

	// Start CPU profiling
	if p.profilingTypes["cpu"] {
		cpuPath := basePath + ".cpu"
		p.cpuProfile, err = os.Create(cpuPath)
		if err != nil {
			return fmt.Errorf("failed to create CPU profile file: %w", err)
		}
		if err := pprof.StartCPUProfile(p.cpuProfile); err != nil {
			_ = p.cpuProfile.Close()
			return fmt.Errorf("failed to start CPU profiling: %w", err)
		}
		p.logger.Info("CPU profiling started", "profile_path", cpuPath)
	}

	// Enable mutex profiling
	if p.profilingTypes["mutex"] {
		runtime.SetMutexProfileFraction(1)
		p.logger.Info("Mutex profiling enabled")
	}

	// Enable block profiling
	if p.profilingTypes["block"] {
		runtime.SetBlockProfileRate(1)
		p.logger.Info("Block profiling enabled")
	}

	p.logger.Info("Performance profiling started", "timestamp", timestamp)
	return nil
}

// StopProfiling stops all profilers and saves results
func (p *Profiler) StopProfiling() (*ProfileResult, error) {
	p.mutex.Lock()
	defer p.mutex.Unlock()

	if !p.enabled {
		return nil, fmt.Errorf("profiler is disabled")
	}

	result := &ProfileResult{
		Operation: "general_profiling",
		Duration:  0,
		MemoryStats: p.getMemoryStats(),
		GoroutineCount: runtime.NumGoroutine(),
		CPUUsage:      p.getCPUUsage(),
	}

	timestamp := time.Now().Format("20060102-150405")
	basePath := fmt.Sprintf("./profiles/profile-%s", timestamp)

	// Stop CPU profiling
	if p.cpuProfile != nil {
		pprof.StopCPUProfile()
		_ = p.cpuProfile.Close()
		result.CPUProfilePath = basePath + ".cpu"
		p.logger.Info("CPU profiling stopped", "profile_path", result.CPUProfilePath)
	}

	// Save memory profile
	if p.profilingTypes["memory"] {
		memPath := basePath + ".mem"
		memProfile, err := os.Create(memPath)
		if err == nil {
			runtime.GC() // Get up-to-date memory information
			if err := pprof.WriteHeapProfile(memProfile); err == nil {
				result.MemProfilePath = memPath
				p.logger.Info("Memory profile saved", "profile_path", memPath)
			}
			_ = memProfile.Close()
		}
	}

	// Save mutex profile
	if p.profilingTypes["mutex"] {
		mutexPath := basePath + ".mutex"
		mutexProfile, err := os.Create(mutexPath)
		if err == nil {
			if err := pprof.Lookup("mutex").WriteTo(mutexProfile, 0); err == nil {
				result.MutexProfilePath = mutexPath
				p.logger.Info("Mutex profile saved", "profile_path", mutexPath)
			}
			_ = mutexProfile.Close()
		}
	}

	// Save block profile
	if p.profilingTypes["block"] {
		blockPath := basePath + ".block"
		blockProfile, err := os.Create(blockPath)
		if err == nil {
			if err := pprof.Lookup("block").WriteTo(blockProfile, 0); err == nil {
				result.BlockProfilePath = blockPath
				p.logger.Info("Block profile saved", "profile_path", blockPath)
			}
			_ = blockProfile.Close()
		}
	}

	// Generate recommendations
	result.Recommendations = p.generateRecommendations(result)

	p.logger.Info("Performance profiling completed", "result", result)
	return result, nil
}

// ProfileOperation profiles a specific operation
func (p *Profiler) ProfileOperation(ctx context.Context, operation string, fn func() error) (*ProfileResult, error) {
	start := time.Now()
	
	// Set up goroutine and memory stats before operation
	initialGoroutines := runtime.NumGoroutine()
	initialMemStats := p.getMemoryStats()

	// Execute the operation
	err := fn()
	duration := time.Since(start)

	if err != nil {
		return nil, fmt.Errorf("operation failed: %w", err)
	}

	// Only profile if operation took longer than threshold
	if duration < 100*time.Millisecond {
		return &ProfileResult{
			Operation:      operation,
			Duration:       duration,
			MemoryStats:    p.getMemoryStats(),
			GoroutineCount: runtime.NumGoroutine(),
			CPUUsage:       p.getCPUUsage(),
			Recommendations: []string{"Operation completed quickly, no profiling needed"},
		}, nil
	}

	// Capture post-operation stats
	finalGoroutines := runtime.NumGoroutine()
	finalMemStats := p.getMemoryStats()

	result := &ProfileResult{
		Operation:      operation,
		Duration:       duration,
		MemoryStats:    finalMemStats,
		GoroutineCount: finalGoroutines,
		CPUUsage:       p.getCPUUsage(),
	}

	// Generate specific recommendations for this operation
	recommendations := []string{}

	// Check for goroutine leaks
	if finalGoroutines > initialGoroutines+10 {
		recommendations = append(recommendations, 
			fmt.Sprintf("Potential goroutine leak: %d goroutines created, %d cleaned up", 
				finalGoroutines-initialGoroutines, finalGoroutines-initialGoroutines))
	}

	// Check for memory leaks
	if finalMemStats.Alloc > initialMemStats.Alloc+1024*1024 { // 1MB threshold
		recommendations = append(recommendations,
			fmt.Sprintf("High memory allocation: %.2f MB allocated during operation",
				float64(finalMemStats.Alloc-initialMemStats.Alloc)/1024/1024))
	}

	// Check for long duration
	if duration > time.Second {
		recommendations = append(recommendations,
			fmt.Sprintf("Operation took %.2f seconds, consider optimization or parallelization",
				duration.Seconds()))
	}

	result.Recommendations = recommendations

	p.metrics.ObserveHistogram("operation_duration", duration.Seconds(), "operation", operation)
	p.logger.Info("Operation profiled", "operation", operation, "duration", duration, "recommendations", len(recommendations))

	return result, nil
}

// getMemoryStats captures current memory statistics
func (p *Profiler) getMemoryStats() *MemoryStats {
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	return &MemoryStats{
		Alloc:      memStats.Alloc,
		TotalAlloc: memStats.TotalAlloc,
		Sys:        memStats.Sys,
		NumGC:      memStats.NumGC,
		GCPause:    memStats.PauseTotalNs,
		HeapAlloc:  memStats.HeapAlloc,
		HeapSys:    memStats.HeapSys,
	}
}

// getCPUUsage estimates current CPU usage (simplified)
func (p *Profiler) getCPUUsage() float64 {
	// This is a simplified CPU usage estimation
	// In a real implementation, you'd use system-specific APIs
	return 0.0 // Placeholder
}

// generateRecommendations analyzes profile results and generates optimization suggestions
func (p *Profiler) generateRecommendations(result *ProfileResult) []string {
	recommendations := []string{}

	// Memory recommendations
	if result.MemoryStats != nil {
		allocMB := float64(result.MemoryStats.Alloc) / 1024 / 1024
		if allocMB > 100 {
			recommendations = append(recommendations, 
				fmt.Sprintf("High memory usage: %.2f MB allocated, consider memory optimization", allocMB))
		}

		if result.MemoryStats.NumGC > 100 {
			recommendations = append(recommendations, 
				fmt.Sprintf("Frequent garbage collection: %d GC cycles, consider reducing allocations", result.MemoryStats.NumGC))
		}
	}

	// Goroutine recommendations
	if result.GoroutineCount > 1000 {
		recommendations = append(recommendations, 
			fmt.Sprintf("High goroutine count: %d goroutines, check for leaks", result.GoroutineCount))
	}

	// Duration recommendations
	if result.Duration > time.Second {
		recommendations = append(recommendations, 
			fmt.Sprintf("Long operation duration: %v, consider optimization or parallelization", result.Duration))
	}

	if len(recommendations) == 0 {
		recommendations = append(recommendations, "No significant performance issues detected")
	}

	return recommendations
}

// Cleanup cleans up profiler resources
func (p *Profiler) Cleanup() error {
	p.mutex.Lock()
	defer p.mutex.Unlock()

	p.enabled = false

	// Stop any active profiling
	if p.cpuProfile != nil {
		pprof.StopCPUProfile()
		_ = p.cpuProfile.Close()
		p.cpuProfile = nil
	}

	if p.memProfile != nil {
		_ = p.memProfile.Close()
		p.memProfile = nil
	}

	if p.mutexProfile != nil {
		_ = p.mutexProfile.Close()
		p.mutexProfile = nil
	}

	if p.blockProfile != nil {
		_ = p.blockProfile.Close()
		p.blockProfile = nil
	}

	p.logger.Info("Profiler cleaned up")
	return nil
}

// GetSystemStats returns current system performance statistics
func (p *Profiler) GetSystemStats() map[string]interface{} {
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	return map[string]interface{}{
		"goroutines":    runtime.NumGoroutine(),
		"memory_alloc":  memStats.Alloc,
		"memory_total":  memStats.TotalAlloc,
		"memory_sys":    memStats.Sys,
		"gc_cycles":     memStats.NumGC,
		"gc_pause_ns":   memStats.PauseTotalNs,
		"heap_alloc":    memStats.HeapAlloc,
		"heap_sys":      memStats.HeapSys,
		"cpu_cores":     runtime.NumCPU(),
		"cgocalls":      runtime.NumCgoCall(),
	}
}

// BenchmarkOperation benchmarks an operation multiple times
func (p *Profiler) BenchmarkOperation(ctx context.Context, operation string, fn func() error, iterations int) (*ProfileResult, error) {
	if iterations <= 0 {
		iterations = 10
	}

	var totalDuration time.Duration
	var results []*ProfileResult
	var errors []error

	for i := 0; i < iterations; i++ {
		result, err := p.ProfileOperation(ctx, fmt.Sprintf("%s_iteration_%d", operation, i), fn)
		if err != nil {
			errors = append(errors, err)
			continue
		}
		results = append(results, result)
		totalDuration += result.Duration
	}

	if len(errors) > 0 {
		return nil, fmt.Errorf("%d/%d iterations failed: %v", len(errors), iterations, errors[0])
	}

	avgDuration := totalDuration / time.Duration(len(results))
	
	// Create summary result
	summary := &ProfileResult{
		Operation:      fmt.Sprintf("%s_benchmark_%d_iterations", operation, iterations),
		Duration:       avgDuration,
		MemoryStats:    p.getMemoryStats(),
		GoroutineCount: runtime.NumGoroutine(),
		CPUUsage:       p.getCPUUsage(),
		Recommendations: []string{
			fmt.Sprintf("Average duration over %d iterations: %v", len(results), avgDuration),
			fmt.Sprintf("Total benchmark time: %v", totalDuration),
		},
	}

	p.logger.Info("Benchmark completed", "operation", operation, "iterations", len(results), "avg_duration", avgDuration)

	return summary, nil
}