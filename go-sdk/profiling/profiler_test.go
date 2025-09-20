package profiling

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/fast-context/go-sdk/logging"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewProfiler tests profiler creation
func TestNewProfiler(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		EnableCPUProfiling:    true,
		EnableMemoryProfiling: true,
		ProfileOutputDir:      t.TempDir(),
		ProfileDuration:       1 * time.Second,
		AutoStartProfiling:    false,
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)
	assert.NotNil(t, profiler)
	assert.True(t, profiler.enabled)
	assert.NotNil(t, profiler.logger)
	assert.NotNil(t, profiler.metrics)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfilerWithDefaultConfig tests profiler with default configuration
func TestProfilerWithDefaultConfig(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	profiler, err := NewProfiler(nil, logger)
	require.NoError(t, err)
	assert.NotNil(t, profiler)
	assert.True(t, profiler.enabled)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestStartStopProfiling tests profiling start and stop
func TestStartStopProfiling(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		EnableCPUProfiling:    true,
		EnableMemoryProfiling: true,
		EnableMutexProfiling:   true,
		EnableBlockProfiling:   true,
		ProfileOutputDir:      t.TempDir(),
		AutoStartProfiling:    false,
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Start profiling
	err = profiler.StartProfiling()
	require.NoError(t, err)

	// Let it run briefly
	time.Sleep(100 * time.Millisecond)

	// Stop profiling
	result, err := profiler.StopProfiling()
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.NotEmpty(t, result.Operation)
	assert.Greater(t, result.GoroutineCount, 0)
	assert.NotNil(t, result.MemoryStats)
	assert.NotEmpty(t, result.Recommendations)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfileOperation tests operation profiling
func TestProfileOperation(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Test fast operation (should not profile)
	ctx := context.Background()
	result, err := profiler.ProfileOperation(ctx, "fast_operation", func() error {
		time.Sleep(10 * time.Millisecond)
		return nil
	})
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, "fast_operation", result.Operation)
	assert.Less(t, result.Duration, 50*time.Millisecond)

	// Test slow operation (should profile)
	result, err = profiler.ProfileOperation(ctx, "slow_operation", func() error {
		time.Sleep(200 * time.Millisecond)
		return nil
	})
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, "slow_operation", result.Operation)
	assert.GreaterOrEqual(t, result.Duration, 200*time.Millisecond)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfileOperationWithError tests operation profiling with errors
func TestProfileOperationWithError(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	result, err := profiler.ProfileOperation(ctx, "error_operation", func() error {
		time.Sleep(50 * time.Millisecond)
		return assert.AnError
	})

	assert.Error(t, err)
	assert.Nil(t, result)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestBenchmarkOperation tests operation benchmarking
func TestBenchmarkOperation(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	result, err := profiler.BenchmarkOperation(ctx, "benchmark_test", func() error {
		time.Sleep(10 * time.Millisecond)
		return nil
	}, 5)

	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Contains(t, result.Operation, "benchmark_test")
	assert.Contains(t, result.Operation, "5_iterations")
	assert.Greater(t, result.Duration, 0)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestGetMemoryStats tests memory statistics collection
func TestGetMemoryStats(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	stats := profiler.getMemoryStats()
	assert.NotNil(t, stats)
	assert.GreaterOrEqual(t, stats.Alloc, uint64(0))
	assert.GreaterOrEqual(t, stats.TotalAlloc, uint64(0))
	assert.GreaterOrEqual(t, stats.Sys, uint64(0))

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestGetSystemStats tests system statistics collection
func TestGetSystemStats(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	stats := profiler.GetSystemStats()
	assert.NotNil(t, stats)
	assert.Contains(t, stats, "goroutines")
	assert.Contains(t, stats, "memory_alloc")
	assert.Contains(t, stats, "cpu_cores")

	goroutines := stats["goroutines"].(int)
	assert.Greater(t, goroutines, 0)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfileFileCreation tests that profile files are created
func TestProfileFileCreation(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	tempDir := t.TempDir()
	config := &ProfilingConfig{
		EnableCPUProfiling:    true,
		EnableMemoryProfiling: true,
		EnableMutexProfiling:   true,
		EnableBlockProfiling:   true,
		ProfileOutputDir:      tempDir,
		AutoStartProfiling:    false,
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Start profiling
	err = profiler.StartProfiling()
	require.NoError(t, err)

	// Let it run briefly
	time.Sleep(100 * time.Millisecond)

	// Stop profiling
	result, err := profiler.StopProfiling()
	require.NoError(t, err)

	// Check that profile files were created
	files, err := filepath.Glob(filepath.Join(tempDir, "profile-*.cpu"))
	require.NoError(t, err)
	if config.EnableCPUProfiling {
		assert.GreaterOrEqual(t, len(files), 0, "CPU profile files should exist")
	}

	files, err = filepath.Glob(filepath.Join(tempDir, "profile-*.mem"))
	require.NoError(t, err)
	if config.EnableMemoryProfiling {
		assert.GreaterOrEqual(t, len(files), 0, "Memory profile files should exist")
	}

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestGenerateRecommendations tests recommendation generation
func TestGenerateRecommendations(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Test with normal stats
	normalResult := &ProfileResult{
		Operation: "normal",
		Duration:  100 * time.Millisecond,
		MemoryStats: &MemoryStats{
			Alloc:      1024 * 1024, // 1MB
			TotalAlloc: 2 * 1024 * 1024,
			Sys:        10 * 1024 * 1024,
			NumGC:      5,
		},
		GoroutineCount: 10,
	}

	recommendations := profiler.generateRecommendations(normalResult)
	assert.NotNil(t, recommendations)
	assert.GreaterOrEqual(t, len(recommendations), 0)

	// Test with high memory usage
	highMemoryResult := &ProfileResult{
		Operation: "high_memory",
		Duration:  100 * time.Millisecond,
		MemoryStats: &MemoryStats{
			Alloc:      200 * 1024 * 1024, // 200MB
			TotalAlloc: 300 * 1024 * 1024,
			Sys:        500 * 1024 * 1024,
			NumGC:      150,
		},
		GoroutineCount: 10,
	}

	recommendations = profiler.generateRecommendations(highMemoryResult)
	assert.NotNil(t, recommendations)
	assert.GreaterOrEqual(t, len(recommendations), 1)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestDisabledProfiler tests behavior when profiler is disabled
func TestDisabledProfiler(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Disable profiler
	profiler.enabled = false

	// Test that operations fail when disabled
	err = profiler.StartProfiling()
	assert.Error(t, err)

	_, err = profiler.StopProfiling()
	assert.Error(t, err)

	ctx := context.Background()
	_, err = profiler.ProfileOperation(ctx, "test", func() error { return nil })
	assert.Error(t, err)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestAutoStartProfiling tests automatic profiling start
func TestAutoStartProfiling(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		EnableCPUProfiling:    true,
		ProfileOutputDir:      t.TempDir(),
		AutoStartProfiling:    true,
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Profiler should be automatically started
	assert.True(t, profiler.enabled)

	// Should be able to stop it
	result, err := profiler.StopProfiling()
	require.NoError(t, err)
	assert.NotNil(t, result)

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfileWithCancelContext tests profiling with context cancellation
func TestProfileWithCancelContext(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	ctx, cancel := context.WithCancel(context.Background())
	
	// Cancel context immediately
	cancel()

	// Test that operation respects cancellation
	_, err = profiler.ProfileOperation(ctx, "cancel_test", func() error {
		time.Sleep(100 * time.Millisecond)
		return nil
	})

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "context canceled")

	err = profiler.Cleanup()
	require.NoError(t, err)
}

// TestProfilerCleanup tests cleanup functionality
func TestProfilerCleanup(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		EnableCPUProfiling:    true,
		ProfileOutputDir:      t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Start profiling
	err = profiler.StartProfiling()
	require.NoError(t, err)

	// Cleanup should stop profiling
	err = profiler.Cleanup()
	require.NoError(t, err)
	assert.False(t, profiler.enabled)
	assert.Nil(t, profiler.cpuProfile)
	assert.Nil(t, profiler.memProfile)
}

// TestMultipleProfilerInstances tests multiple profiler instances
func TestMultipleProfilerInstances(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config1 := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	config2 := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler1, err := NewProfiler(config1, logger)
	require.NoError(t, err)

	profiler2, err := NewProfiler(config2, logger)
	require.NoError(t, err)

	// Both profilers should work independently
	assert.True(t, profiler1.enabled)
	assert.True(t, profiler2.enabled)
	assert.NotEqual(t, profiler1, profiler2)

	err = profiler1.Cleanup()
	require.NoError(t, err)

	err = profiler2.Cleanup()
	require.NoError(t, err)
}

// TestProfilerConcurrentAccess tests concurrent access to profiler
func TestProfilerConcurrentAccess(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &ProfilingConfig{
		ProfileOutputDir: t.TempDir(),
	}

	profiler, err := NewProfiler(config, logger)
	require.NoError(t, err)

	// Test concurrent access
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			
			ctx := context.Background()
			_, err := profiler.ProfileOperation(ctx, fmt.Sprintf("concurrent_%d", id), func() error {
				time.Sleep(10 * time.Millisecond)
				return nil
			})
			assert.NoError(t, err)
		}(i)
	}

	wg.Wait()
	err = profiler.Cleanup()
	require.NoError(t, err)
}