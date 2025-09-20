package fastcontext

import (
	"errors"
	"fmt"
)

// ErrorCode represents different types of errors that can occur
type ErrorCode int

const (
	ErrNone ErrorCode = iota
	ErrInvalidConfiguration
	ErrProjectNotFound
	ErrAnalysisFailed
	ErrTimeout
	ErrCancelled
	ErrOutOfMemory
	ErrPermissionDenied
	ErrUnsupportedLanguage
	ErrInvalidInput
	ErrInternal
)

// FastContextError represents an error from the Fast-Context SDK
type FastContextError struct {
	Code    ErrorCode
	Message string
	Cause   error
}

// Error implements the error interface
func (e *FastContextError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf("FastContextError[%d]: %s (cause: %v)", e.Code, e.Message, e.Cause)
	}
	return fmt.Sprintf("FastContextError[%d]: %s", e.Code, e.Message)
}

// Unwrap returns the underlying cause
func (e *FastContextError) Unwrap() error {
	return e.Cause
}

// Is checks if the error matches a target error
func (e *FastContextError) Is(target error) bool {
	var fcErr *FastContextError
	if errors.As(target, &fcErr) {
		return e.Code == fcErr.Code
	}
	return false
}

// NewFastContextError creates a new FastContextError
func NewFastContextError(code ErrorCode, message string) *FastContextError {
	return &FastContextError{
		Code:    code,
		Message: message,
	}
}

// NewFastContextErrorWithCause creates a new FastContextError with a cause
func NewFastContextErrorWithCause(code ErrorCode, message string, cause error) *FastContextError {
	return &FastContextError{
		Code:    code,
		Message: message,
		Cause:   cause,
	}
}

// Predefined errors for common scenarios
var (
	ErrInvalidProjectRoot   = NewFastContextError(ErrProjectNotFound, "project root does not exist")
	ErrInvalidConfig        = NewFastContextError(ErrInvalidConfiguration, "invalid configuration")
	ErrAnalysisTimeout      = NewFastContextError(ErrTimeout, "analysis timed out")
	ErrAnalysisCancelled    = NewFastContextError(ErrCancelled, "analysis cancelled")
	ErrMemoryLimitExceeded  = NewFastContextError(ErrOutOfMemory, "memory limit exceeded")
)