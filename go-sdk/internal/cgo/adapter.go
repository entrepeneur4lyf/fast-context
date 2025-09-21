//go:build rustlib
// +build rustlib

package cgo

/*
#cgo CFLAGS: -I../../src
#cgo LDFLAGS: -L../../target/release -lfast_context -ldl -lm

#include "fast_context.h"
*/
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
	"runtime"
	"strings"
	"sync"
	"syscall"
	"unsafe"
)

// Security constants
const (
	MAX_JSON_SIZE      = 100 * 1024 * 1024 // 100MB
	MAX_PATH_LENGTH     = 4096             // PATH_MAX
	MAX_STRING_LENGTH   = 4096             // Maximum string length
	MAX_CALLBACKS       = 100              // Maximum concurrent callbacks
	MAX_TOTAL_ALLOCATED = 1024 * 1024 * 1024 // 1GB total memory limit
)

// Result represents the result of a Rust function call
type Result struct {
	JSONData     []byte
	ErrorCode    int
	ErrorMessage string
}

// Progress represents analysis progress from Rust
type Progress struct {
	Phase       int
	Current     int
	Total       int
	Percentage  float64
	Message     string
	CurrentFile string
}

// Adapter provides a high-level interface to the CGO functions
type Adapter struct {
	mutex       sync.RWMutex
	callbacks   map[uintptr]func(*Progress)
	callbackID  uintptr
	cgoActive   bool
	memoryStats struct {
		totalAllocated uint64
		maxAllocated   uint64
		activeObjects  uint64
	}
}

// NewAdapter creates a new CGO adapter
func NewAdapter() *Adapter {
	return &Adapter{
		callbacks: make(map[uintptr]func(*Progress)),
		memoryStats: struct {
			totalAllocated uint64
			maxAllocated   uint64
			activeObjects  uint64
		}{},
	}
}

// validateString validates input string for security
func validateString(input string, name string) error {
	if input == "" {
		return fmt.Errorf("%s cannot be empty", name)
	}
	if len(input) > MAX_STRING_LENGTH {
		return fmt.Errorf("%s too long (max %d bytes)", name, MAX_STRING_LENGTH)
	}
	if strings.ContainsAny(input, "\x00") {
		return fmt.Errorf("%s contains null bytes", name)
	}
	return nil
}

// validatePath validates file path for security
func validatePath(path string, name string) error {
	if err := validateString(path, name); err != nil {
		return err
	}
	
	// Check for path traversal
	if strings.Contains(path, "..") {
		return fmt.Errorf("%s contains path traversal sequences", name)
	}
	
	// Check for suspicious characters
	if strings.ContainsAny(path, "|&;<>()$`\"'\\") {
		return fmt.Errorf("%s contains potentially dangerous characters", name)
	}
	
	return nil
}

// validateCResult validates C function result for security
func validateCResult(cResult *C.FastContextResult, operation string) error {
	if cResult == nil {
		return fmt.Errorf("%s returned null result", operation)
	}
	
	// Validate error code range
	if cResult.error_code < 0 || cResult.error_code > 10 {
		return fmt.Errorf("%s returned invalid error code: %d", operation, cResult.error_code)
	}
	
	// Validate JSON data integrity
	if cResult.json_data != nil && cResult.json_len > 0 {
		if uint64(cResult.json_len) > MAX_JSON_SIZE {
			return fmt.Errorf("%s returned excessive data size: %d", operation, cResult.json_len)
		}
	}
	
	return nil
}

// validateResourceLimits checks system resource limits
func validateResourceLimits() error {
	var rLimit syscall.Rlimit
	if err := syscall.Getrlimit(syscall.RLIMIT_NOFILE, &rLimit); err != nil {
		return fmt.Errorf("failed to get file descriptor limit: %w", err)
	}
	
	if rLimit.Cur < 1024 {
		return fmt.Errorf("insufficient file descriptor limit: %d", rLimit.Cur)
	}
	
	// Check memory limits
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)
	
	if memStats.Alloc > MAX_TOTAL_ALLOCATED {
		return fmt.Errorf("memory usage too high: %d bytes", memStats.Alloc)
	}
	
	return nil
}

// Analyze calls the Rust analyze function
func (a *Adapter) Analyze(projectRoot string, configJSON []byte) (*Result, error) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	// Validate resource limits
	if err := validateResourceLimits(); err != nil {
		return nil, fmt.Errorf("resource limit validation failed: %w", err)
	}
	
	// Validate inputs
	if err := validatePath(projectRoot, "project root"); err != nil {
		return nil, err
	}
	
	cProjectRoot := C.CString(projectRoot)
	if cProjectRoot == nil {
		return nil, errors.New("failed to convert project root to CString")
	}
	defer C.free(unsafe.Pointer(cProjectRoot))

	var cConfigJSON *C.char
	if len(configJSON) > 0 {
		if uint64(len(configJSON)) > MAX_JSON_SIZE {
			return nil, fmt.Errorf("config JSON too large: %d bytes", len(configJSON))
		}
		cConfigJSON = C.CString(string(configJSON))
		if cConfigJSON == nil {
			return nil, errors.New("failed to convert config JSON to CString")
		}
		defer C.free(unsafe.Pointer(cConfigJSON))
	}

	cResult := C.fast_context_analyze(cProjectRoot, cConfigJSON)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_analyze")
	}
	
	// Validate C result
	if err := validateCResult(cResult, "fast_context_analyze"); err != nil {
		C.fast_context_free_result(cResult)
		return nil, err
	}
	defer C.fast_context_free_result(cResult)

	// Validate result bounds with overflow protection
	if cResult.json_len < 0 || uint64(cResult.json_len) > MAX_JSON_SIZE {
		return nil, fmt.Errorf("invalid JSON data length: %d", cResult.json_len)
	}

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		// Validate pointer before copying
		if uintptr(unsafe.Pointer(cResult.json_data)) == 0 {
			return nil, errors.New("invalid JSON data pointer")
		}
		
		// Safe copy with bounds checking
		result.JSONData = make([]byte, cResult.json_len)
		copied := C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
		if copied == nil {
			return nil, errors.New("failed to copy JSON data")
		}
		copy(result.JSONData, copied)
		
		// Track memory allocation
		a.trackAllocation(uint64(len(result.JSONData)))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// FindSymbols calls the Rust find_symbols function
func (a *Adapter) FindSymbols(projectRoot string, symbolKind string) (*Result, error) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	// Validate resource limits
	if err := validateResourceLimits(); err != nil {
		return nil, fmt.Errorf("resource limit validation failed: %w", err)
	}
	
	// Validate inputs
	if err := validatePath(projectRoot, "project root"); err != nil {
		return nil, err
	}
	if err := validateString(symbolKind, "symbol kind"); err != nil {
		return nil, err
	}
	
	cProjectRoot := C.CString(projectRoot)
	if cProjectRoot == nil {
		return nil, errors.New("failed to convert project root to CString")
	}
	defer C.free(unsafe.Pointer(cProjectRoot))

	cSymbolKind := C.CString(symbolKind)
	if cSymbolKind == nil {
		return nil, errors.New("failed to convert symbol kind to CString")
	}
	defer C.free(unsafe.Pointer(cSymbolKind))

	cResult := C.fast_context_find_symbols(cProjectRoot, cSymbolKind)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_find_symbols")
	}
	
	// Validate C result
	if err := validateCResult(cResult, "fast_context_find_symbols"); err != nil {
		C.fast_context_free_result(cResult)
		return nil, err
	}
	defer C.fast_context_free_result(cResult)

	// Validate result bounds with overflow protection
	if cResult.json_len < 0 || uint64(cResult.json_len) > MAX_JSON_SIZE {
		return nil, fmt.Errorf("invalid JSON data length: %d", cResult.json_len)
	}

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		// Validate pointer before copying
		if uintptr(unsafe.Pointer(cResult.json_data)) == 0 {
			return nil, errors.New("invalid JSON data pointer")
		}
		
		// Safe copy with bounds checking
		result.JSONData = make([]byte, cResult.json_len)
		copied := C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
		if copied == nil {
			return nil, errors.New("failed to copy JSON data")
		}
		copy(result.JSONData, copied)
		
		// Track memory allocation
		a.trackAllocation(uint64(len(result.JSONData)))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// FindDependencies calls the Rust find_dependencies function
func (a *Adapter) FindDependencies(projectRoot string, symbolName string) (*Result, error) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	// Validate resource limits
	if err := validateResourceLimits(); err != nil {
		return nil, fmt.Errorf("resource limit validation failed: %w", err)
	}
	
	// Validate inputs
	if err := validatePath(projectRoot, "project root"); err != nil {
		return nil, err
	}
	if err := validateString(symbolName, "symbol name"); err != nil {
		return nil, err
	}
	
	cProjectRoot := C.CString(projectRoot)
	if cProjectRoot == nil {
		return nil, errors.New("failed to convert project root to CString")
	}
	defer C.free(unsafe.Pointer(cProjectRoot))

	cSymbolName := C.CString(symbolName)
	if cSymbolName == nil {
		return nil, errors.New("failed to convert symbol name to CString")
	}
	defer C.free(unsafe.Pointer(cSymbolName))

	cResult := C.fast_context_find_dependencies(cProjectRoot, cSymbolName)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_find_dependencies")
	}
	
	// Validate C result
	if err := validateCResult(cResult, "fast_context_find_dependencies"); err != nil {
		C.fast_context_free_result(cResult)
		return nil, err
	}
	defer C.fast_context_free_result(cResult)

	// Validate result bounds with overflow protection
	if cResult.json_len < 0 || uint64(cResult.json_len) > MAX_JSON_SIZE {
		return nil, fmt.Errorf("invalid JSON data length: %d", cResult.json_len)
	}

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		// Validate pointer before copying
		if uintptr(unsafe.Pointer(cResult.json_data)) == 0 {
			return nil, errors.New("invalid JSON data pointer")
		}
		
		// Safe copy with bounds checking
		result.JSONData = make([]byte, cResult.json_len)
		copied := C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
		if copied == nil {
			return nil, errors.New("failed to copy JSON data")
		}
		copy(result.JSONData, copied)
		
		// Track memory allocation
		a.trackAllocation(uint64(len(result.JSONData)))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// trackAllocation tracks memory allocation for security monitoring
func (a *Adapter) trackAllocation(size uint64) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	a.memoryStats.totalAllocated += size
	a.memoryStats.activeObjects++
	
	if a.memoryStats.totalAllocated > a.memoryStats.maxAllocated {
		a.memoryStats.maxAllocated = a.memoryStats.totalAllocated
	}
	
	// Check for memory exhaustion
	if a.memoryStats.totalAllocated > MAX_TOTAL_ALLOCATED {
		fmt.Printf("Warning: Total allocated memory exceeds limit: %d bytes\n", a.memoryStats.totalAllocated)
	}
}

// untrackAllocation untracks memory allocation
func (a *Adapter) untrackAllocation(size uint64) {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	if a.memoryStats.totalAllocated >= size {
		a.memoryStats.totalAllocated -= size
	} else {
		a.memoryStats.totalAllocated = 0
	}
	
	if a.memoryStats.activeObjects > 0 {
		a.memoryStats.activeObjects--
	}
}

// StartWatching starts watching the project for file changes
func (a *Adapter) StartWatching(projectRoot string, callback func(*Progress)) error {
	// Validate inputs
	if err := validatePath(projectRoot, "project root"); err != nil {
		return err
	}
	if callback == nil {
		return errors.New("callback cannot be nil")
	}
	
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	if a.cgoActive {
		return errors.New("CGO watching already active")
	}
	
	if len(a.callbacks) >= MAX_CALLBACKS {
		return fmt.Errorf("too many active callbacks (max %d)", MAX_CALLBACKS)
	}
	
	// Safe callback registration
	callbackID := a.callbackID
	a.callbackID++
	a.callbacks[callbackID] = callback
	
	// Set up proper cleanup
	runtime.SetFinalizer(&callbackID, func(id *uintptr) {
		a.mutex.Lock()
		defer a.mutex.Unlock()
		delete(a.callbacks, *id)
	})
	
	// Mark CGO as active
	a.cgoActive = true
	
	cProjectRoot := C.CString(projectRoot)
	if cProjectRoot == nil {
		return errors.New("failed to convert project root to CString")
	}
	defer C.free(unsafe.Pointer(cProjectRoot))
	
	// Call C function with error handling
	C.fast_context_start_watching(cProjectRoot, nil)
	return nil
}

// StopWatching stops watching the project
func (a *Adapter) StopWatching() {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	// Clean up all callbacks
	for id := range a.callbacks {
		delete(a.callbacks, id)
	}
	
	// Mark CGO as inactive
	a.cgoActive = false
	
	C.fast_context_stop_watching()
}

// GetVersion returns the Rust library version
func (a *Adapter) GetVersion() string {
	a.mutex.RLock()
	defer a.mutex.RUnlock()
	
	cVersion := C.fast_context_get_version()
	if cVersion == nil {
		return "unknown"
	}
	defer C.fast_context_free_string(cVersion)
	
	version := C.GoString(cVersion)
	if len(version) > MAX_STRING_LENGTH {
		return "version too long"
	}
	
	return version
}

// Cleanup releases all resources and callbacks
func (a *Adapter) Cleanup() {
	a.mutex.Lock()
	defer a.mutex.Unlock()
	
	// Clean up all callbacks
	for id := range a.callbacks {
		delete(a.callbacks, id)
	}
	
	// Mark CGO as inactive
	a.cgoActive = false
	
	// Reset memory tracking
	a.memoryStats.totalAllocated = 0
	a.memoryStats.activeObjects = 0
	
	// Force garbage collection to clean up any remaining resources
	runtime.GC()
	runtime.GC() // Run twice to ensure cleanup
}

// GetMemoryStats returns memory usage statistics for monitoring
func (a *Adapter) GetMemoryStats() (totalAllocated uint64, maxAllocated uint64, activeObjects uint64) {
	a.mutex.RLock()
	defer a.mutex.RUnlock()
	
	return a.memoryStats.totalAllocated, a.memoryStats.maxAllocated, a.memoryStats.activeObjects
}

// GetCallbackCount returns the number of active callbacks (for testing)
func (a *Adapter) GetCallbackCount() int {
	a.mutex.RLock()
	defer a.mutex.RUnlock()
	return len(a.callbacks)
}

// UnmarshalJSON safely unmarshals JSON data from the result
func (r *Result) UnmarshalJSON(v interface{}) error {
	if len(r.JSONData) == 0 {
		return errors.New("no JSON data to unmarshal")
	}
	return json.Unmarshal(r.JSONData, v)
}

// Error returns the result as an error if it has one
func (r *Result) Error() error {
	if r.ErrorCode != 0 {
		return fmt.Errorf("Rust error %d: %s", r.ErrorCode, r.ErrorMessage)
	}
	return nil
}

// IsSuccess checks if the result represents success
func (r *Result) IsSuccess() bool {
	return r.ErrorCode == 0
}

// IsError checks if the result represents an error
func (r *Result) IsError() bool {
	return r.ErrorCode != 0
}