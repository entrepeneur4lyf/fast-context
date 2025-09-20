//go:build !rustlib
// +build !rustlib

package cgo

// Mock implementation for development without Rust library
// This allows the Go SDK to be developed and tested independently

import (
	"encoding/json"
	"fmt"
	"time"
)

// Result represents the result of a mock function call
type Result struct {
	JSONData     []byte
	ErrorCode    int
	ErrorMessage string
}

// Progress represents analysis progress from mock
type Progress struct {
	Phase       int
	Current     int
	Total       int
	Percentage  float64
	Message     string
	CurrentFile string
}

// Adapter provides a mock interface for development
type Adapter struct{}

// NewAdapter creates a new mock adapter
func NewAdapter() *Adapter {
	return &Adapter{}
}

// Analyze performs mock analysis
func (a *Adapter) Analyze(projectRoot string, configJSON []byte) (*Result, error) {
	// Mock analysis result
	result := &AnalysisResult{
		FileCount:         42,
		SymbolCount:       156,
		RelationshipCount: 89,
		Languages:         []string{"Go", "Rust", "JavaScript"},
		DurationMs:        time.Now().UnixMilli(),
	}

	jsonData, err := json.Marshal(result)
	if err != nil {
		return &Result{
			ErrorCode:    10, // Internal error
			ErrorMessage: fmt.Sprintf("Failed to marshal result: %v", err),
		}, nil
	}

	return &Result{
		JSONData:  jsonData,
		ErrorCode: 0, // Success
	}, nil
}

// FindSymbols performs mock symbol search
func (a *Adapter) FindSymbols(projectRoot string, symbolKind string) (*Result, error) {
	// Mock symbols with realistic data
	symbols := []*Symbol{
		{
			ID:         "main",
			Name:       "main",
			Kind:       SymbolKindFunction,
			Language:   "Go",
			File:       "main.go",
			LineStart:  10,
			LineEnd:    25,
			Complexity: 3.5,
			IsPublic:   true,
			IsExported: false,
		},
		{
			ID:         "calculate",
			Name:       "calculate",
			Kind:       SymbolKindFunction,
			Language:   "Go",
			File:       "utils.go",
			LineStart:  5,
			LineEnd:    15,
			Complexity: 7.2,
			IsPublic:   true,
			IsExported: false,
		},
		{
			ID:         "Config",
			Name:       "Config",
			Kind:       SymbolKindStruct,
			Language:   "Go",
			File:       "config.go",
			LineStart:  1,
			LineEnd:    30,
			Complexity: 2.1,
			IsPublic:   true,
			IsExported: true,
		},
		{
			ID:         "processData",
			Name:       "processData",
			Kind:       SymbolKindMethod,
			Language:   "Go",
			File:       "processor.go",
			LineStart:  20,
			LineEnd:    45,
			Complexity: 12.8,
			IsPublic:   true,
			IsExported: false,
		},
	}

	jsonData, err := json.Marshal(symbols)
	if err != nil {
		return &Result{
			ErrorCode:    10,
			ErrorMessage: fmt.Sprintf("Failed to marshal symbols: %v", err),
		}, nil
	}

	return &Result{
		JSONData:  jsonData,
		ErrorCode: 0,
	}, nil
}

// FindDependencies performs mock dependency search
func (a *Adapter) FindDependencies(projectRoot string, symbolName string) (*Result, error) {
	// Mock dependencies
	dependencies := []*Dependency{
		{
			From:     symbolName,
			To:       "fmt",
			Type:     DepTypeImports,
			Strength: 1.0,
		},
	}

	jsonData, err := json.Marshal(dependencies)
	if err != nil {
		return &Result{
			ErrorCode:    10,
			ErrorMessage: fmt.Sprintf("Failed to marshal dependencies: %v", err),
		}, nil
	}

	return &Result{
		JSONData:  jsonData,
		ErrorCode: 0,
	}, nil
}

// StartWatching performs mock watching setup
func (a *Adapter) StartWatching(projectRoot string, callback func(*Progress)) error {
	// Mock implementation - just log the call
	fmt.Printf("[MOCK] Starting to watch project: %s\n", projectRoot)
	return nil
}

// StopWatching performs mock watching stop
func (a *Adapter) StopWatching() {
	fmt.Println("[MOCK] Stopped watching project")
}

// GetVersion returns mock version
func (a *Adapter) GetVersion() string {
	return "0.1.0-mock"
}

// Mock types for compatibility
type AnalysisResult struct {
	FileCount         int       `json:"fileCount"`
	SymbolCount       int       `json:"symbolCount"`
	RelationshipCount int       `json:"relationshipCount"`
	Languages         []string  `json:"languages"`
	DurationMs        int64      `json:"durationMs"`
}

type Symbol struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Kind        SymbolKind `json:"kind"`
	Language    string    `json:"language"`
	File        string    `json:"file"`
	LineStart   int       `json:"lineStart"`
	LineEnd     int       `json:"lineEnd"`
	Complexity  float64   `json:"complexity"`
	IsPublic    bool      `json:"isPublic"`
	IsExported  bool      `json:"isExported"`
}

type SymbolKind int

const (
	SymbolKindUnknown SymbolKind = iota
	SymbolKindFunction
	SymbolKindMethod
	SymbolKindClass
	SymbolKindInterface
	SymbolKindStruct
	SymbolKindEnum
	SymbolKindVariable
	SymbolKindConstant
	SymbolKindParameter
	SymbolKindModule
	SymbolKindPackage
	SymbolKindType
	SymbolKindField
	SymbolKindProperty
	SymbolKindConstructor
	SymbolKindDestructor
	SymbolKindOperator
	SymbolKindMacro
	SymbolKindAnnotation
)

type Dependency struct {
	From     string        `json:"from"`
	To       string        `json:"to"`
	Type     DependencyType `json:"type"`
	Strength float64       `json:"strength"`
}

type DependencyType int

const (
	DepTypeUnknown DependencyType = iota
	DepTypeImports
	DepTypeCalls
	DepTypeInherits
	DepTypeImplements
	DepTypeReferences
	DepTypeInstantiates
	DepTypeContains
	DepTypeOverrides
	DepTypeDecorates
)

// UnmarshalJSON safely unmarshals JSON data from the result
func (r *Result) UnmarshalJSON(v interface{}) error {
	if len(r.JSONData) == 0 {
		return fmt.Errorf("no JSON data to unmarshal")
	}
	return json.Unmarshal(r.JSONData, v)
}

// Error returns the result as an error if it has one
func (r *Result) Error() error {
	if r.ErrorCode != 0 {
		return fmt.Errorf("mock error %d: %s", r.ErrorCode, r.ErrorMessage)
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