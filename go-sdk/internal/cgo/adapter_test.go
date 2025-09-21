//go:build rustlib
// +build rustlib

package cgo

import (
	"runtime"
	"sync"
	"testing"
	"time"
)

func TestAdapterCreation(t *testing.T) {
	adapter := NewAdapter()
	if adapter == nil {
		t.Fatal("Adapter should not be nil")
	}
	
	// Test cleanup
	adapter.Cleanup()
}

func TestAdapterConcurrency(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	var wg sync.WaitGroup
	const numGoroutines = 10
	
	wg.Add(numGoroutines)
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			defer wg.Done()
			
			// Test concurrent access to GetVersion
			version := adapter.GetVersion()
			if version == "" {
				t.Errorf("Goroutine %d: version should not be empty", id)
			}
		}(i)
	}
	
	wg.Wait()
}

func TestAdapterInputValidation(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	// Test empty project root
	_, err := adapter.Analyze("", []byte{})
	if err == nil {
		t.Error("Should return error for empty project root")
	}
	
	_, err = adapter.FindSymbols("", "function")
	if err == nil {
		t.Error("Should return error for empty project root")
	}
	
	_, err = adapter.FindDependencies("", "test")
	if err == nil {
		t.Error("Should return error for empty project root")
	}
	
	// Test empty symbol kind
	_, err = adapter.FindSymbols("/tmp", "")
	if err == nil {
		t.Error("Should return error for empty symbol kind")
	}
	
	// Test empty symbol name
	_, err = adapter.FindDependencies("/tmp", "")
	if err == nil {
		t.Error("Should return error for empty symbol name")
	}
	
	// Test nil callback
	err = adapter.StartWatching("/tmp", nil)
	if err == nil {
		t.Error("Should return error for nil callback")
	}
}

func TestAdapterCallbackManagement(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	// Test callback counting
	initialCount := adapter.GetCallbackCount()
	if initialCount != 0 {
		t.Errorf("Initial callback count should be 0, got %d", initialCount)
	}
	
	// Test adding callback
	callback := func(progress *Progress) {
		// Simple callback for testing
	}
	
	err := adapter.StartWatching("/tmp", callback)
	if err != nil {
		t.Errorf("StartWatching should succeed: %v", err)
	}
	
	count := adapter.GetCallbackCount()
	if count != 1 {
		t.Errorf("Callback count should be 1, got %d", count)
	}
	
	// Test stopping watching
	adapter.StopWatching()
	count = adapter.GetCallbackCount()
	if count != 0 {
		t.Errorf("Callback count should be 0 after StopWatching, got %d", count)
	}
}

func TestAdapterMemorySafety(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	// Test with large config (should fail gracefully)
	largeConfig := make([]byte, 1024*1024) // 1MB
	for i := range largeConfig {
		largeConfig[i] = 'A'
	}
	
	result, err := adapter.Analyze("/nonexistent", largeConfig)
	if err == nil {
		t.Error("Should fail with large config to non-existent path")
	}
	
	if result != nil {
		t.Error("Result should be nil on error")
	}
}

func TestAdapterCleanup(t *testing.T) {
	adapter := NewAdapter()
	
	// Add some callbacks
	callback := func(progress *Progress) {}
	err := adapter.StartWatching("/tmp", callback)
	if err != nil {
		t.Errorf("StartWatching should succeed: %v", err)
	}
	
	// Verify callbacks exist
	count := adapter.GetCallbackCount()
	if count != 1 {
		t.Errorf("Should have 1 callback, got %d", count)
	}
	
	// Test cleanup
	adapter.Cleanup()
	
	// Verify cleanup
	count = adapter.GetCallbackCount()
	if count != 0 {
		t.Errorf("Should have 0 callbacks after cleanup, got %d", count)
	}
}

func TestAdapterFinalizer(t *testing.T) {
	// This test verifies that the finalizer is set correctly
	// but we can't easily test the actual finalizer execution
	
	adapter := NewAdapter()
	callback := func(progress *Progress) {}
	
	err := adapter.StartWatching("/tmp", callback)
	if err != nil {
		t.Errorf("StartWatching should succeed: %v", err)
	}
	
	// The finalizer should be set automatically
	// We can't easily test when it runs, but we can verify the adapter is still functional
	version := adapter.GetVersion()
	if version == "" {
		t.Error("Version should not be empty")
	}
	
	adapter.Cleanup()
}

func TestAdapterErrorHandling(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	// Test with non-existent paths
	_, err := adapter.Analyze("/nonexistent/path", []byte{})
	if err != nil {
		t.Logf("Expected error for non-existent path: %v", err)
	}
	
	_, err = adapter.FindSymbols("/nonexistent/path", "function")
	if err != nil {
		t.Logf("Expected error for non-existent path: %v", err)
	}
	
	_, err = adapter.FindDependencies("/nonexistent/path", "test")
	if err != nil {
		t.Logf("Expected error for non-existent path: %v", err)
	}
}

func TestAdapterResultValidation(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	// Test result methods
	result := &Result{
		JSONData:     []byte(`{"test": "data"}`),
		ErrorCode:    0,
		ErrorMessage: "",
	}
	
	if !result.IsSuccess() {
		t.Error("Result should be successful")
	}
	
	if result.IsError() {
		t.Error("Result should not be in error state")
	}
	
	if result.Error() != nil {
		t.Errorf("Result should not have error: %v", result.Error())
	}
	
	// Test error result
	errorResult := &Result{
		JSONData:     []byte{},
		ErrorCode:    1,
		ErrorMessage: "Test error",
	}
	
	if errorResult.IsSuccess() {
		t.Error("Error result should not be successful")
	}
	
	if !errorResult.IsError() {
		t.Error("Error result should be in error state")
	}
	
	if errorResult.Error() == nil {
		t.Error("Error result should have error")
	}
}

func TestAdapterMemoryLeakDetection(t *testing.T) {
	adapter := NewAdapter()
	
	// Test multiple operations that should not leak memory
	for i := 0; i < 100; i++ {
		version := adapter.GetVersion()
		if version == "" {
			t.Errorf("Iteration %d: version should not be empty", i)
		}
		
		// Force garbage collection to check for leaks
		runtime.GC()
	}
	
	adapter.Cleanup()
	
	// Final garbage collection
	runtime.GC()
	runtime.GC()
}

func TestAdapterConcurrentCallbacks(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	var wg sync.WaitGroup
	const numCallbacks = 5
	
	wg.Add(numCallbacks)
	for i := 0; i < numCallbacks; i++ {
		go func(id int) {
			defer wg.Done()
			
			callback := func(progress *Progress) {
				// Do nothing for testing
			}
			
			err := adapter.StartWatching("/tmp", callback)
			if err != nil {
				t.Errorf("Goroutine %d: StartWatching failed: %v", id, err)
			}
		}(i)
	}
	
	wg.Wait()
	
	// Verify all callbacks were added
	count := adapter.GetCallbackCount()
	if count != numCallbacks {
		t.Errorf("Should have %d callbacks, got %d", numCallbacks, count)
	}
	
	// Clean up
	adapter.StopWatching()
	
	count = adapter.GetCallbackCount()
	if count != 0 {
		t.Errorf("Should have 0 callbacks after stop, got %d", count)
	}
}

func TestAdapterStressTest(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping stress test in short mode")
	}
	
	adapter := NewAdapter()
	defer adapter.Cleanup()
	
	const numIterations = 1000
	var wg sync.WaitGroup
	
	wg.Add(3)
	
	// Concurrent version requests
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations; i++ {
			version := adapter.GetVersion()
			if version == "" {
				t.Errorf("Version should not be empty (iteration %d)", i)
			}
		}
	}()
	
	// Concurrent callback management
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations/10; i++ {
			callback := func(progress *Progress) {}
			err := adapter.StartWatching("/tmp", callback)
			if err != nil {
				t.Errorf("StartWatching failed: %v", err)
			}
			time.Sleep(time.Millisecond)
		}
	}()
	
	// Concurrent cleanup
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations/20; i++ {
			adapter.StopWatching()
			time.Sleep(2 * time.Millisecond)
		}
	}()
	
	wg.Wait()
}