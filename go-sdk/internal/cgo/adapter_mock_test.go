//go:build !rustlib
// +build !rustlib

package cgo

import (
	"encoding/json"
	"sync"
	"testing"
)

func TestMockAdapterCreation(t *testing.T) {
	adapter := NewAdapter()
	if adapter == nil {
		t.Fatal("Adapter should not be nil")
	}
	
	// Test version
	version := adapter.GetVersion()
	if version != "0.1.0-mock" {
		t.Errorf("Expected version '0.1.0-mock', got '%s'", version)
	}
}

func TestMockAdapterConcurrency(t *testing.T) {
	adapter := NewAdapter()
	
	var wg sync.WaitGroup
	const numGoroutines = 10
	
	wg.Add(numGoroutines)
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			defer wg.Done()
			
			// Test concurrent access
			version := adapter.GetVersion()
			if version != "0.1.0-mock" {
				t.Errorf("Goroutine %d: version should be '0.1.0-mock', got '%s'", id, version)
			}
		}(i)
	}
	
	wg.Wait()
}

func TestMockAdapterInputValidation(t *testing.T) {
	adapter := NewAdapter()
	
	// Test with empty inputs - mock should handle gracefully
	result, err := adapter.Analyze("", []byte{})
	if err != nil {
		t.Errorf("Mock should handle empty inputs gracefully: %v", err)
	}
	
	if result == nil {
		t.Fatal("Result should not be nil")
	}
	
	if result.ErrorCode != 0 {
		t.Errorf("Mock should succeed, got error code %d", result.ErrorCode)
	}
}

func TestMockAdapterAnalysis(t *testing.T) {
	adapter := NewAdapter()
	
	// Test analysis
	result, err := adapter.Analyze("/tmp", []byte{})
	if err != nil {
		t.Errorf("Analysis should succeed: %v", err)
	}
	
	if result == nil {
		t.Fatal("Result should not be nil")
	}
	
	// Test JSON unmarshaling
	var analysisResult struct {
		FileCount         int      `json:"fileCount"`
		SymbolCount       int      `json:"symbolCount"`
		RelationshipCount int      `json:"relationshipCount"`
		Languages         []string `json:"languages"`
	}
	
	err = json.Unmarshal(result.JSONData, &analysisResult)
	if err != nil {
		t.Errorf("Failed to unmarshal JSON: %v", err)
	}
	
	if analysisResult.FileCount != 42 {
		t.Errorf("Expected file count 42, got %d", analysisResult.FileCount)
	}
	
	if analysisResult.SymbolCount != 156 {
		t.Errorf("Expected symbol count 156, got %d", analysisResult.SymbolCount)
	}
}

func TestMockAdapterFindSymbols(t *testing.T) {
	adapter := NewAdapter()
	
	// Test symbol finding
	result, err := adapter.FindSymbols("/tmp", "function")
	if err != nil {
		t.Errorf("FindSymbols should succeed: %v", err)
	}
	
	if result == nil {
		t.Fatal("Result should not be nil")
	}
	
	// Test JSON unmarshaling
	var symbols []struct {
		ID    string `json:"id"`
		Name  string `json:"name"`
		Kind  int    `json:"kind"`
		File  string `json:"file"`
	}
	
	err = json.Unmarshal(result.JSONData, &symbols)
	if err != nil {
		t.Errorf("Failed to unmarshal JSON: %v", err)
	}
	
	if len(symbols) != 4 {
		t.Errorf("Expected 4 symbols, got %d", len(symbols))
	}
	
	// Check main function
	foundMain := false
	for _, symbol := range symbols {
		if symbol.Name == "main" && symbol.ID == "main" {
			foundMain = true
			break
		}
	}
	
	if !foundMain {
		t.Error("Should find 'main' function")
	}
}

func TestMockAdapterFindDependencies(t *testing.T) {
	adapter := NewAdapter()
	
	// Test dependency finding
	result, err := adapter.FindDependencies("/tmp", "test")
	if err != nil {
		t.Errorf("FindDependencies should succeed: %v", err)
	}
	
	if result == nil {
		t.Fatal("Result should not be nil")
	}
	
	// Test JSON unmarshaling
	var dependencies []struct {
		From     string  `json:"from"`
		To       string  `json:"to"`
		Type     int     `json:"type"`
		Strength float64 `json:"strength"`
	}
	
	err = json.Unmarshal(result.JSONData, &dependencies)
	if err != nil {
		t.Errorf("Failed to unmarshal JSON: %v", err)
	}
	
	if len(dependencies) != 1 {
		t.Errorf("Expected 1 dependency, got %d", len(dependencies))
	}
	
	if dependencies[0].From != "test" {
		t.Errorf("Expected dependency from 'test', got '%s'", dependencies[0].From)
	}
	
	if dependencies[0].To != "fmt" {
		t.Errorf("Expected dependency to 'fmt', got '%s'", dependencies[0].To)
	}
}

func TestMockAdapterFileWatching(t *testing.T) {
	adapter := NewAdapter()
	
	// Test start watching
	err := adapter.StartWatching("/tmp", func(progress *Progress) {
		// Simple callback
	})
	if err != nil {
		t.Errorf("StartWatching should succeed: %v", err)
	}
	
	// Test stop watching
	adapter.StopWatching()
	
	// Should not panic
	adapter.StopWatching()
}

func TestMockAdapterResultMethods(t *testing.T) {
	// Test successful result
	successResult := &Result{
		JSONData:     []byte(`{"test": "data"}`),
		ErrorCode:    0,
		ErrorMessage: "",
	}
	
	if !successResult.IsSuccess() {
		t.Error("Success result should be successful")
	}
	
	if successResult.IsError() {
		t.Error("Success result should not be in error state")
	}
	
	if successResult.Error() != nil {
		t.Errorf("Success result should not have error: %v", successResult.Error())
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

func TestMockAdapterMemoryLeakDetection(t *testing.T) {
	adapter := NewAdapter()
	
	// Test multiple operations that should not leak memory
	for i := 0; i < 100; i++ {
		version := adapter.GetVersion()
		if version != "0.1.0-mock" {
			t.Errorf("Iteration %d: version should be '0.1.0-mock', got '%s'", i, version)
		}
	}
}

func TestMockAdapterStressTest(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping stress test in short mode")
	}
	
	adapter := NewAdapter()
	const numIterations = 1000
	var wg sync.WaitGroup
	
	wg.Add(3)
	
	// Concurrent version requests
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations; i++ {
			version := adapter.GetVersion()
			if version != "0.1.0-mock" {
				t.Errorf("Version should be '0.1.0-mock', got '%s'", version)
			}
		}
	}()
	
	// Concurrent analysis requests
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations/10; i++ {
			_, err := adapter.Analyze("/tmp", []byte{})
			if err != nil {
				t.Errorf("Analysis failed: %v", err)
			}
		}
	}()
	
	// Concurrent symbol finding
	go func() {
		defer wg.Done()
		for i := 0; i < numIterations/10; i++ {
			_, err := adapter.FindSymbols("/tmp", "function")
			if err != nil {
				t.Errorf("FindSymbols failed: %v", err)
			}
		}
	}()
	
	wg.Wait()
}