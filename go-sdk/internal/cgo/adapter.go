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
	"unsafe"
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
type Adapter struct{}

// NewAdapter creates a new CGO adapter
func NewAdapter() *Adapter {
	return &Adapter{}
}

// Analyze calls the Rust analyze function
func (a *Adapter) Analyze(projectRoot string, configJSON []byte) (*Result, error) {
	cProjectRoot := C.CString(projectRoot)
	defer C.free(unsafe.Pointer(cProjectRoot))

	var cConfigJSON *C.char
	if len(configJSON) > 0 {
		cConfigJSON = C.CString(string(configJSON))
		defer C.free(unsafe.Pointer(cConfigJSON))
	}

	cResult := C.fast_context_analyze(cProjectRoot, cConfigJSON)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_analyze")
	}
	defer C.fast_context_free_result(cResult)

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		result.JSONData = C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// FindSymbols calls the Rust find_symbols function
func (a *Adapter) FindSymbols(projectRoot string, symbolKind string) (*Result, error) {
	cProjectRoot := C.CString(projectRoot)
	defer C.free(unsafe.Pointer(cProjectRoot))

	cSymbolKind := C.CString(symbolKind)
	defer C.free(unsafe.Pointer(cSymbolKind))

	cResult := C.fast_context_find_symbols(cProjectRoot, cSymbolKind)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_find_symbols")
	}
	defer C.fast_context_free_result(cResult)

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		result.JSONData = C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// FindDependencies calls the Rust find_dependencies function
func (a *Adapter) FindDependencies(projectRoot string, symbolName string) (*Result, error) {
	cProjectRoot := C.CString(projectRoot)
	defer C.free(unsafe.Pointer(cProjectRoot))

	cSymbolName := C.CString(symbolName)
	defer C.free(unsafe.Pointer(cSymbolName))

	cResult := C.fast_context_find_dependencies(cProjectRoot, cSymbolName)
	if cResult == nil {
		return nil, errors.New("failed to call fast_context_find_dependencies")
	}
	defer C.fast_context_free_result(cResult)

	result := &Result{
		ErrorCode: int(cResult.error_code),
	}

	if cResult.json_data != nil && cResult.json_len > 0 {
		result.JSONData = C.GoBytes(unsafe.Pointer(cResult.json_data), C.int(cResult.json_len))
	}

	if cResult.error_message != nil {
		result.ErrorMessage = C.GoString(cResult.error_message)
	}

	return result, nil
}

// StartWatching starts watching the project for file changes
func (a *Adapter) StartWatching(projectRoot string, callback func(*Progress)) error {
	cProjectRoot := C.CString(projectRoot)
	defer C.free(unsafe.Pointer(cProjectRoot))

	// Convert Go callback to C function pointer
	// Note: This is a simplified implementation. In practice, you'd need to handle
	// the callback marshaling more carefully, possibly using cgo's callback support
	C.fast_context_start_watching(cProjectRoot, nil)
	return nil
}

// StopWatching stops watching the project
func (a *Adapter) StopWatching() {
	C.fast_context_stop_watching()
}

// GetVersion returns the Rust library version
func (a *Adapter) GetVersion() string {
	cVersion := C.fast_context_get_version()
	if cVersion == nil {
		return "unknown"
	}
	defer C.fast_context_free_string(cVersion)
	return C.GoString(cVersion)
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