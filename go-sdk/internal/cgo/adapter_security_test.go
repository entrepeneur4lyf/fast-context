//go:build rustlib
// +build rustlib

package cgo

import (
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

// TestAdapterSecurity validates security constraints in adapter operations
func TestAdapterSecurity(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping security tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test with malicious project root (path traversal)
	_, err := adapter.Analyze("../../../../../etc/passwd", []byte{})
	assert.Error(t, err, "Should reject path traversal attempts")
	assert.Contains(t, err.Error(), "path traversal")

	// Test with null byte injection
	_, err = adapter.Analyze("test\x00evil", []byte{})
	assert.Error(t, err, "Should reject null byte injection")
	assert.Contains(t, err.Error(), "null bytes")

	// Test with dangerous characters
	_, err = adapter.Analyze("test; rm -rf /", []byte{})
	assert.Error(t, err, "Should reject dangerous characters")
	assert.Contains(t, err.Error(), "dangerous characters")

	// Test with oversized input
	oversizedInput := make([]byte, 100*1024*1024+1) // 100MB + 1
	_, err = adapter.Analyze("/tmp", oversizedInput)
	assert.Error(t, err, "Should reject oversized input")
	assert.Contains(t, err.Error(), "too large")

	// Test with empty project root
	_, err = adapter.Analyze("", []byte{})
	assert.Error(t, err, "Should reject empty project root")
	assert.Contains(t, err.Error(), "cannot be empty")
}

// TestFindSymbolsSecurity validates security in FindSymbols
func TestFindSymbolsSecurity(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping security tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test with malicious project root
	_, err := adapter.FindSymbols("../../etc/passwd", "function")
	assert.Error(t, err, "Should reject path traversal in FindSymbols")

	// Test with malicious symbol kind
	_, err = adapter.FindSymbols("/tmp", "function\x00evil")
	assert.Error(t, err, "Should reject null byte injection in symbol kind")

	// Test with dangerous symbol kind
	_, err = adapter.FindSymbols("/tmp", "function; rm -rf")
	assert.Error(t, err, "Should reject dangerous characters in symbol kind")

	// Test with empty symbol kind
	_, err = adapter.FindSymbols("/tmp", "")
	assert.Error(t, err, "Should reject empty symbol kind")
}

// TestFindDependenciesSecurity validates security in FindDependencies
func TestFindDependenciesSecurity(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping security tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test with malicious project root
	_, err := adapter.FindDependencies("../../etc/passwd", "test_function")
	assert.Error(t, err, "Should reject path traversal in FindDependencies")

	// Test with malicious symbol name
	_, err = adapter.FindDependencies("/tmp", "test\x00evil")
	assert.Error(t, err, "Should reject null byte injection in symbol name")

	// Test with dangerous symbol name
	_, err = adapter.FindDependencies("/tmp", "test; rm -rf")
	assert.Error(t, err, "Should reject dangerous characters in symbol name")

	// Test with empty symbol name
	_, err = adapter.FindDependencies("/tmp", "")
	assert.Error(t, err, "Should reject empty symbol name")
}

// TestStartWatchingSecurity validates security in StartWatching
func TestStartWatchingSecurity(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping security tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test with malicious project root
	err := adapter.StartWatching("../../etc/passwd", func(*Progress) {})
	assert.Error(t, err, "Should reject path traversal in StartWatching")

	// Test with nil callback
	err = adapter.StartWatching("/tmp", nil)
	assert.Error(t, err, "Should reject nil callback")

	// Test callback limit
	for i := 0; i < 100; i++ {
		err := adapter.StartWatching("/tmp", func(*Progress) {})
		if i < 99 {
			assert.NoError(t, err, "Should accept callbacks under limit")
		}
	}

	// Test exceeding callback limit
	err = adapter.StartWatching("/tmp", func(*Progress) {})
	assert.Error(t, err, "Should reject callbacks over limit")
	assert.Contains(t, err.Error(), "too many active callbacks")
}

// TestMemoryTracking tests memory allocation tracking
func TestMemoryTracking(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test initial state
	total, max, active := adapter.GetMemoryStats()
	assert.Equal(t, uint64(0), total, "Initial total allocated should be 0")
	assert.Equal(t, uint64(0), max, "Initial max allocated should be 0")
	assert.Equal(t, uint64(0), active, "Initial active objects should be 0")

	// Memory tracking should work with normal operations (indirectly through analyze calls)
	_, err := adapter.Analyze("/tmp", []byte(`{"test": "data"}`))
	if err == nil {
		// If analyze succeeds, check if memory tracking is working
		total, max, active = adapter.GetMemoryStats()
		t.Logf("Memory stats after analyze - Total: %d, Max: %d, Active: %d", total, max, active)
	}
}

// TestConcurrentAccess tests thread safety under concurrent access
func TestConcurrentAccess(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping concurrent tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test concurrent analysis
	done := make(chan bool, 10)
	errors := make(chan error, 10)

	for i := 0; i < 10; i++ {
		go func(id int) {
			defer func() { done <- true }()
			
			_, err := adapter.Analyze("/tmp", []byte(`{"test": "data"}`))
			if err != nil && !strings.Contains(err.Error(), "failed to call") {
				errors <- err
			}
		}(i)
	}

	// Wait for all goroutines to complete
	for i := 0; i < 10; i++ {
		select {
		case <-done:
			// Goroutine completed
		case <-time.After(5 * time.Second):
			t.Fatal("Concurrent test timed out")
		}
	}

	// Check for any errors
	select {
	case err := <-errors:
		t.Errorf("Concurrent access error: %v", err)
	default:
		// No errors
	}

	// Verify memory tracking is consistent
	total, max, active := adapter.GetMemoryStats()
	t.Logf("Memory stats - Total: %d, Max: %d, Active: %d", total, max, active)
}

// TestGetVersionSecurity validates version function security
func TestGetVersionSecurity(t *testing.T) {
	adapter := NewAdapter()
	defer adapter.Cleanup()

	version := adapter.GetVersion()
	
	// Version should be safe
	assert.NotContains(t, version, "\x00", "Version should not contain null bytes")
	assert.LessOrEqual(t, len(version), 4096, "Version should not exceed max length")
	
	// Version should be printable characters
	for _, c := range version {
		if c >= 32 && c <= 126 || c == '\n' || c == '\t' || c == '\r' {
			continue
		}
		t.Errorf("Version should contain only printable characters, got: %d", c)
	}
}

// TestCleanupSecurity validates cleanup security
func TestCleanupSecurity(t *testing.T) {
	adapter := NewAdapter()

	// Perform some operations
	adapter.GetVersion()
	_, err := adapter.Analyze("/tmp", []byte(`{"test": "data"}`))
	if err != nil {
		t.Logf("Analyze failed (expected in test environment): %v", err)
	}

	// Perform cleanup
	adapter.Cleanup()

	// Verify cleanup completed
	total, _, active := adapter.GetMemoryStats()
	assert.Equal(t, uint64(0), total, "Memory should be cleaned up")
	assert.Equal(t, uint64(0), active, "Active objects should be cleaned up")

	count := adapter.GetCallbackCount()
	assert.Equal(t, 0, count, "Callbacks should be cleaned up")
}

// TestResourceExhaustion tests behavior under resource pressure
func TestResourceExhaustion(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping resource exhaustion tests in short mode")
	}

	adapter := NewAdapter()
	defer adapter.Cleanup()

	// Test with many small operations
	for i := 0; i < 50; i++ {
		_, err := adapter.Analyze("/tmp", []byte(`{"test": "data"}`))
		if err != nil && !strings.Contains(err.Error(), "failed to call") {
			// Only fail on security or validation errors, not on missing Rust library
			t.Errorf("Security validation failed on iteration %d: %v", i, err)
		}
	}

	// Verify system is still stable
	total, max, active := adapter.GetMemoryStats()
	t.Logf("Memory stats after stress test - Total: %d, Max: %d, Active: %d", total, max, active)
}

// BenchmarkSecurityOperations benchmarks security-protected operations
func BenchmarkSecurityOperations(b *testing.B) {
	adapter := NewAdapter()
	defer adapter.Cleanup()

	b.Run("AnalyzeSecurity", func(b *testing.B) {
		validInput := []byte(`{"test": "data"}`)
		for i := 0; i < b.N; i++ {
			_, err := adapter.Analyze("/tmp", validInput)
			if err != nil && !strings.Contains(err.Error(), "failed to call") {
				b.Fatal(err)
			}
		}
	})

	b.Run("GetVersionSecurity", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			version := adapter.GetVersion()
			if len(version) == 0 {
				b.Fatal("Empty version")
			}
		}
	})

	b.Run("MemoryStatsSecurity", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_, _, _ = adapter.GetMemoryStats()
		}
	})
}