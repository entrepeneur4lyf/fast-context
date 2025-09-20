package optimization

import (
	"context"
	"fmt"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/fast-context/go-sdk/logging"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewOptimizer tests optimizer creation
func TestNewOptimizer(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching:           true,
		EnableParallelization:   true,
		EnableMemoryOptimization: true,
		MaxCacheSize:           1024 * 1024,
		MaxWorkerCount:         4,
		MemoryPressureThreshold: 0.8,
		OptimizationLevel:      "balanced",
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)
	assert.NotNil(t, optimizer)
	assert.True(t, optimizer.enabled)
	assert.NotNil(t, optimizer.cache)
	assert.NotNil(t, optimizer.parallelPool)
	assert.NotNil(t, optimizer.memoryManager)

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestNewOptimizerWithDefaultConfig tests optimizer with default configuration
func TestNewOptimizerWithDefaultConfig(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	optimizer, err := NewOptimizer(nil, logger)
	require.NoError(t, err)
	assert.NotNil(t, optimizer)
	assert.True(t, optimizer.enabled)

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestNewOptimizerDisabledFeatures tests optimizer with disabled features
func TestNewOptimizerDisabledFeatures(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching:           false,
		EnableParallelization:   false,
		EnableMemoryOptimization: false,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)
	assert.NotNil(t, optimizer)
	assert.True(t, optimizer.enabled)
	assert.Nil(t, optimizer.cache)
	assert.Nil(t, optimizer.parallelPool)
	assert.Nil(t, optimizer.memoryManager)

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizeOperation tests basic operation optimization
func TestOptimizeOperation(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching:           true,
		EnableParallelization:   true,
		EnableMemoryOptimization: true,
		MaxCacheSize:           1024 * 1024,
		MaxWorkerCount:         2,
		MemoryPressureThreshold: 0.8,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	
	// Test simple operation
	result, optResult, err := optimizer.OptimizeOperation(ctx, "test_operation", func() (interface{}, error) {
		time.Sleep(50 * time.Millisecond)
		return "test_result", nil
	})

	require.NoError(t, err)
	assert.Equal(t, "test_result", result)
	assert.NotNil(t, optResult)
	assert.Greater(t, optResult.OptimizedDuration, time.Duration(0))
	assert.NotEmpty(t, optResult.TechniquesUsed)

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizeOperationWithError tests optimization with operation errors
func TestOptimizeOperationWithError(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching: true,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	
	// Test operation that returns an error
	result, optResult, err := optimizer.OptimizeOperation(ctx, "error_operation", func() (interface{}, error) {
		return nil, assert.AnError
	})

	assert.Error(t, err)
	assert.Nil(t, result)
	assert.Nil(t, optResult)

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizeOperationDisabled tests behavior when optimizer is disabled
func TestOptimizeOperationDisabled(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching: true,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	// Disable optimizer
	optimizer.enabled = false

	ctx := context.Background()
	
	result, optResult, err := optimizer.OptimizeOperation(ctx, "test_operation", func() (interface{}, error) {
		time.Sleep(50 * time.Millisecond)
		return "test_result", nil
	})

	require.NoError(t, err)
	assert.Equal(t, "test_result", result)
	assert.NotNil(t, optResult)
	assert.Equal(t, optResult.OriginalDuration, optResult.OptimizedDuration)
	assert.Contains(t, optResult.TechniquesUsed, "none")

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizationCache tests the optimization cache
func TestOptimizationCache(t *testing.T) {
	cache := NewOptimizationCache(1024) // 1KB cache

	// Test cache set and get
	cache.Set("key1", "value1", 100)
	value, found := cache.Get("key1")
	assert.True(t, found)
	assert.Equal(t, "value1", value)

	// Test cache miss
	_, found = cache.Get("nonexistent")
	assert.False(t, found)

	// Test cache overwrite
	cache.Set("key1", "new_value", 150)
	value, found = cache.Get("key1")
	assert.True(t, found)
	assert.Equal(t, "new_value", value)

	// Test cache eviction with small size
	smallCache := NewOptimizationCache(200) // 200 bytes
	
	smallCache.Set("key1", "value1", 100)
	smallCache.Set("key2", "value2", 100)
	smallCache.Set("key3", "value3", 100) // Should evict key1

	_, found = smallCache.Get("key1")
	assert.False(t, found)

	_, found = smallCache.Get("key2")
	assert.True(t, found)

	_, found = smallCache.Get("key3")
	assert.True(t, found)
}

// TestOptimizationCacheLRU tests cache LRU eviction
func TestOptimizationCacheLRU(t *testing.T) {
	cache := NewOptimizationCache(300) // 300 bytes

	// Add items
	cache.Set("key1", "value1", 100)
	cache.Set("key2", "value2", 100)
	cache.Set("key3", "value3", 100)

	// Access key1 to make it most recently used
	cache.Get("key1")

	// Add item that should evict key2 (LRU)
	cache.Set("key4", "value4", 100)

	// Check eviction
	_, found := cache.Get("key1") // Should exist (accessed recently)
	assert.True(t, found)

	_, found = cache.Get("key2") // Should be evicted (LRU)
	assert.False(t, found)

	_, found = cache.Get("key3") // Should exist
	assert.True(t, found)

	_, found = cache.Get("key4") // Should exist
	assert.True(t, found)
}

// TestWorkerPool tests the worker pool
func TestWorkerPool(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	pool := NewWorkerPool(ctx, 2)
	assert.NotNil(t, pool)
	assert.Equal(t, 2, pool.maxWorkers)

	// Test task submission
	resultCh := make(chan taskResult, 1)
	task := task{
		id:       1,
		fn:       func() (interface{}, error) { return "task_result", nil },
		resultCh: resultCh,
	}

	err := pool.Submit(task)
	require.NoError(t, err)

	// Wait for result
	select {
	case result := <-resultCh:
		assert.Equal(t, 1, result.id)
		assert.Equal(t, "task_result", result.result)
		assert.NoError(t, result.err)
	case <-time.After(1 * time.Second):
		t.Fatal("Task timeout")
	}

	// Test context cancellation
	cancel()
	
	err = pool.Submit(task)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "shutting down")
}

// TestWorkerPoolConcurrency tests worker pool concurrency
func TestWorkerPoolConcurrency(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	pool := NewWorkerPool(ctx, 4)
	
	var wg sync.WaitGroup
	results := make(chan string, 10)

	// Submit multiple tasks
	for i := 0; i < 10; i++ {
		wg.Add(1)
		taskID := i
		
		resultCh := make(chan taskResult, 1)
		task := task{
			id: taskID,
			fn: func() (interface{}, error) {
				time.Sleep(10 * time.Millisecond)
				return fmt.Sprintf("result_%d", taskID), nil
			},
			resultCh: resultCh,
		}

		go func() {
			defer wg.Done()
			if err := pool.Submit(task); err == nil {
				if result := <-resultCh; result.err == nil {
					results <- result.result.(string)
				}
			}
		}()
	}

	// Wait for all tasks
	wg.Wait()
	close(results)

	// Collect results
	var collectedResults []string
	for result := range results {
		collectedResults = append(collectedResults, result)
	}

	assert.Len(t, collectedResults, 10)
}

// TestMemoryManager tests the memory manager
func TestMemoryManager(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	mm := NewMemoryManager(0.9, logger) // High threshold to avoid GC
	assert.NotNil(t, mm)

	// Test memory optimization
	saved := mm.OptimizeMemoryUsage()
	assert.GreaterOrEqual(t, saved, int64(0))

	// Test stop
	mm.Stop()
}

// TestMemoryManagerPressure tests memory pressure detection
func TestMemoryManagerPressure(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	// Use low threshold to trigger pressure detection
	mm := NewMemoryManager(0.1, logger)
	assert.NotNil(t, mm)

	// Allocate some memory to trigger pressure
	data := make([]byte, 1024*1024) // 1MB
	for i := range data {
		data[i] = byte(i % 256)
	}

	// Let the monitor run briefly
	time.Sleep(100 * time.Millisecond)

	mm.Stop()
}

// TestOptimizeOperationWithCache tests optimization with caching
func TestOptimizeOperationWithCache(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching: true,
		MaxCacheSize:  1024 * 1024,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	
	// Execute operation multiple times - should cache after first time
	for i := 0; i < 3; i++ {
		result, optResult, err := optimizer.OptimizeOperation(ctx, "cached_operation", func() (interface{}, error) {
			time.Sleep(50 * time.Millisecond)
			return "cached_result", nil
		})

		require.NoError(t, err)
		assert.Equal(t, "cached_result", result)
		assert.NotNil(t, optResult)
		
		// First call should not be cached, subsequent calls should be
		if i == 0 {
			assert.NotContains(t, optResult.TechniquesUsed, "caching")
		} else {
			assert.Contains(t, optResult.TechniquesUsed, "caching")
		}
	}

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizeOperationWithParallelization tests optimization with parallelization
func TestOptimizeOperationWithParallelization(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableParallelization: true,
		MaxWorkerCount:        2,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	
	result, optResult, err := optimizer.OptimizeOperation(ctx, "parallel_operation", func() (interface{}, error) {
		time.Sleep(100 * time.Millisecond)
		return "parallel_result", nil
	})

	require.NoError(t, err)
	assert.Equal(t, "parallel_result", result)
	assert.NotNil(t, optResult)
	assert.Contains(t, optResult.TechniquesUsed, "parallelization")

	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizerConcurrentAccess tests concurrent access to optimizer
func TestOptimizerConcurrentAccess(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching:         true,
		EnableParallelization: true,
		MaxWorkerCount:        4,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	ctx := context.Background()
	
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			
			result, _, err := optimizer.OptimizeOperation(ctx, fmt.Sprintf("concurrent_%d", id), func() (interface{}, error) {
				time.Sleep(20 * time.Millisecond)
				return fmt.Sprintf("result_%d", id), nil
			})
			
			assert.NoError(t, err)
			assert.Equal(t, fmt.Sprintf("result_%d", id), result)
		}(i)
	}

	wg.Wait()
	err = optimizer.Cleanup()
	require.NoError(t, err)
}

// TestOptimizerCleanup tests optimizer cleanup
func TestOptimizerCleanup(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)
	
	config := &OptimizationConfig{
		EnableCaching:           true,
		EnableParallelization:   true,
		EnableMemoryOptimization: true,
		MaxCacheSize:           1024 * 1024,
		MaxWorkerCount:         2,
		MemoryPressureThreshold: 0.8,
	}

	optimizer, err := NewOptimizer(config, logger)
	require.NoError(t, err)

	// Use the optimizer
	ctx := context.Background()
	_, _, err = optimizer.OptimizeOperation(ctx, "cleanup_test", func() (interface{}, error) {
		return "test", nil
	})
	require.NoError(t, err)

	// Cleanup should stop all components
	err = optimizer.Cleanup()
	require.NoError(t, err)
	assert.False(t, optimizer.enabled)
	assert.Nil(t, optimizer.cache)
	assert.Nil(t, optimizer.parallelPool)
	assert.Nil(t, optimizer.memoryManager)
}

// TestOptimizerConfigLevels tests different optimization levels
func TestOptimizerConfigLevels(t *testing.T) {
	logger := logging.NewStructuredLogger(logging.LevelInfo)

	// Test fast level
	fastConfig := &OptimizationConfig{
		OptimizationLevel: "fast",
		MaxWorkerCount:    2,
	}
	fastOptimizer, err := NewOptimizer(fastConfig, logger)
	require.NoError(t, err)
	assert.Equal(t, "fast", fastOptimizer.config.OptimizationLevel)
	fastOptimizer.Cleanup()

	// Test balanced level
	balancedConfig := &OptimizationConfig{
		OptimizationLevel: "balanced",
		MaxWorkerCount:    4,
	}
	balancedOptimizer, err := NewOptimizer(balancedConfig, logger)
	require.NoError(t, err)
	assert.Equal(t, "balanced", balancedOptimizer.config.OptimizationLevel)
	balancedOptimizer.Cleanup()

	// Test thorough level
	thoroughConfig := &OptimizationConfig{
		OptimizationLevel: "thorough",
		MaxWorkerCount:    runtime.NumCPU(),
	}
	thoroughOptimizer, err := NewOptimizer(thoroughConfig, logger)
	require.NoError(t, err)
	assert.Equal(t, "thorough", thoroughOptimizer.config.OptimizationLevel)
	thoroughOptimizer.Cleanup()
}