package logging

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"log/slog"
	"os"
	"strings"
	"time"
)

// LogLevel represents the logging level
type LogLevel int

const (
	LevelDebug LogLevel = iota
	LevelInfo
	LevelWarn
	LevelError
)

func (l LogLevel) String() string {
	switch l {
	case LevelDebug:
		return "DEBUG"
	case LevelInfo:
		return "INFO"
	case LevelWarn:
		return "WARN"
	case LevelError:
		return "ERROR"
	default:
		return "UNKNOWN"
	}
}

// ParseLogLevel converts string to LogLevel
func ParseLogLevel(level string) LogLevel {
	switch strings.ToLower(level) {
	case "debug":
		return LevelDebug
	case "info":
		return LevelInfo
	case "warn", "warning":
		return LevelWarn
	case "error":
		return LevelError
	default:
		return LevelInfo
	}
}

// LogEvent represents a structured log event
type LogEvent struct {
	Timestamp time.Time              `json:"timestamp"`
	Level     LogLevel               `json:"level"`
	Message   string                 `json:"message"`
	Fields    map[string]interface{} `json:"fields,omitempty"`
	Error     error                  `json:"error,omitempty"`
}

// Logger interface defines the logging contract
type Logger interface {
	Debug(msg string, fields ...interface{})
	Info(msg string, fields ...interface{})
	Warn(msg string, fields ...interface{})
	Error(msg string, err error, fields ...interface{})
	With(fields ...interface{}) Logger
	Flush() error
}

// StructuredLogger implements structured logging
type StructuredLogger struct {
	logger   *slog.Logger
	level    LogLevel
	fields   map[string]interface{}
	handlers []logHandler
}

// logHandler interface for custom log handlers
type logHandler interface {
	Handle(event *LogEvent) error
}

// ConsoleHandler outputs logs to console
type ConsoleHandler struct {
	enableColor bool
}

// FileHandler outputs logs to file
type FileHandler struct {
	filePath string
	file     *os.File
}

// JSONHandler outputs logs in JSON format
type JSONHandler struct {
	output io.Writer
}

// MetricsHandler collects metrics from logs
type MetricsHandler struct {
	metrics *MetricsCollector
}

// NewStructuredLogger creates a new structured logger
func NewStructuredLogger(level LogLevel, opts ...LoggerOption) *StructuredLogger {
	// Default options
	options := &loggerOptions{
		enableConsole: true,
		enableJSON:    false,
		enableFile:    false,
		enableColor:   true,
		enableMetrics: true,
	}

	// Apply options
	for _, opt := range opts {
		opt(options)
	}

	// Create handlers
	var handlers []logHandler

	if options.enableConsole {
		handlers = append(handlers, &ConsoleHandler{
			enableColor: options.enableColor,
		})
	}

	if options.enableFile {
		handlers = append(handlers, &FileHandler{
			filePath: options.filePath,
		})
	}

	if options.enableJSON {
		output := os.Stdout
		if options.enableFile && options.filePath != "" {
			file, err := os.OpenFile(options.filePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
			if err == nil {
				output = file
			}
		}
		handlers = append(handlers, &JSONHandler{output: output})
	}

	if options.enableMetrics {
		handlers = append(handlers, &MetricsHandler{
			metrics: NewMetricsCollector(),
		})
	}

	return &StructuredLogger{
		level:    level,
		fields:   make(map[string]interface{}),
		handlers: handlers,
	}
}

// LoggerOption defines options for logger configuration
type LoggerOption func(*loggerOptions)

type loggerOptions struct {
	enableConsole bool
	enableJSON    bool
	enableFile    bool
	enableColor   bool
	enableMetrics bool
	filePath      string
}

// WithConsole enables console logging
func WithConsole(enable bool) LoggerOption {
	return func(o *loggerOptions) {
		o.enableConsole = enable
	}
}

// WithJSON enables JSON logging
func WithJSON(enable bool) LoggerOption {
	return func(o *loggerOptions) {
		o.enableJSON = enable
	}
}

// WithFile enables file logging
func WithFile(filePath string) LoggerOption {
	return func(o *loggerOptions) {
		o.enableFile = true
		o.filePath = filePath
	}
}

// WithColor enables colored console output
func WithColor(enable bool) LoggerOption {
	return func(o *loggerOptions) {
		o.enableColor = enable
	}
}

// WithMetrics enables metrics collection
func WithMetrics(enable bool) LoggerOption {
	return func(o *loggerOptions) {
		o.enableMetrics = enable
	}
}

// Debug logs a debug message
func (l *StructuredLogger) Debug(msg string, fields ...interface{}) {
	if l.level <= LevelDebug {
		l.log(LevelDebug, msg, fields...)
	}
}

// Info logs an info message
func (l *StructuredLogger) Info(msg string, fields ...interface{}) {
	if l.level <= LevelInfo {
		l.log(LevelInfo, msg, fields...)
	}
}

// Warn logs a warning message
func (l *StructuredLogger) Warn(msg string, fields ...interface{}) {
	if l.level <= LevelWarn {
		l.log(LevelWarn, msg, fields...)
	}
}

// Error logs an error message
func (l *StructuredLogger) Error(msg string, err error, fields ...interface{}) {
	if l.level <= LevelError {
		l.log(LevelError, msg, append(fields, "error", err)...)
	}
}

// With creates a new logger with additional fields
func (l *StructuredLogger) With(fields ...interface{}) Logger {
	newLogger := &StructuredLogger{
		level:    l.level,
		fields:   make(map[string]interface{}),
		handlers: l.handlers,
	}

	// Copy existing fields
	for k, v := range l.fields {
		newLogger.fields[k] = v
	}

	// Add new fields
	if len(fields) > 0 {
		if len(fields)%2 != 0 {
			fields = append(fields, "missing_value")
		}
		for i := 0; i < len(fields); i += 2 {
			if key, ok := fields[i].(string); ok {
				newLogger.fields[key] = fields[i+1]
			}
		}
	}

	return newLogger
}

// Flush ensures all logs are written
func (l *StructuredLogger) Flush() error {
	for _, handler := range l.handlers {
		if flusher, ok := handler.(interface{ Flush() error }); ok {
			if err := flusher.Flush(); err != nil {
				return err
			}
		}
	}
	return nil
}

// log internal logging method
func (l *StructuredLogger) log(level LogLevel, msg string, fields ...interface{}) {
	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     level,
		Message:   msg,
		Fields:    make(map[string]interface{}),
	}

	// Add logger fields
	for k, v := range l.fields {
		event.Fields[k] = v
	}

	// Add event fields
	if len(fields) > 0 {
		if len(fields)%2 != 0 {
			fields = append(fields, "missing_value")
		}
		for i := 0; i < len(fields); i += 2 {
			if key, ok := fields[i].(string); ok {
				event.Fields[key] = fields[i+1]
			}
		}
	}

	// Send to all handlers
	for _, handler := range l.handlers {
		if err := handler.Handle(event); err != nil {
			log.Printf("Error in log handler: %v", err)
		}
	}
}

// ConsoleHandler implementation
func (h *ConsoleHandler) Handle(event *LogEvent) error {
	var colorReset = "\033[0m"
	var colorRed = "\033[31m"
	var colorYellow = "\033[33m"
	var colorBlue = "\033[34m"
	var colorGreen = "\033[32m"

	var color string
	switch event.Level {
	case LevelError:
		color = colorRed
	case LevelWarn:
		color = colorYellow
	case LevelInfo:
		color = colorGreen
	case LevelDebug:
		color = colorBlue
	}

	if h.enableColor {
		log.Printf("%s[%s] %s%s %s\n", color, event.Level, event.Timestamp.Format("2006-01-02 15:04:05"), colorReset, event.Message)
	} else {
		log.Printf("[%s] %s %s\n", event.Level, event.Timestamp.Format("2006-01-02 15:04:05"), event.Message)
	}

	return nil
}

// FileHandler implementation
func (h *FileHandler) Handle(event *LogEvent) error {
	if h.file == nil {
		file, err := os.OpenFile(h.filePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			return err
		}
		h.file = file
	}

	logLine := fmt.Sprintf("[%s] %s %s\n", event.Level, event.Timestamp.Format("2006-01-02 15:04:05"), event.Message)
	_, err := h.file.WriteString(logLine)
	return err
}

// JSONHandler implementation
func (h *JSONHandler) Handle(event *LogEvent) error {
	jsonData, err := json.Marshal(event)
	if err != nil {
		return err
	}
	_, err = h.output.Write(append(jsonData, '\n'))
	return err
}

// MetricsHandler implementation
func (h *MetricsHandler) Handle(event *LogEvent) error {
	if h.metrics != nil {
		h.metrics.IncrementCounter("log_events_total", "level", event.Level.String())
		h.metrics.ObserveHistogram("log_events_size", float64(len(event.Message)), "level", event.Level.String())
	}
	return nil
}

// Global logger instance
var defaultLogger Logger

// InitializeDefaultLogger initializes the default logger
func InitializeDefaultLogger(level LogLevel) {
	defaultLogger = NewStructuredLogger(level)
}

// GetLogger returns the default logger
func GetLogger() Logger {
	if defaultLogger == nil {
		InitializeDefaultLogger(LevelInfo)
	}
	return defaultLogger
}

// Convenience functions for global logging
func Debug(msg string, fields ...interface{}) {
	GetLogger().Debug(msg, fields...)
}

func Info(msg string, fields ...interface{}) {
	GetLogger().Info(msg, fields...)
}

func Warn(msg string, fields ...interface{}) {
	GetLogger().Warn(msg, fields...)
}

func Error(msg string, err error, fields ...interface{}) {
	GetLogger().Error(msg, err, fields...)
}

func With(fields ...interface{}) Logger {
	return GetLogger().With(fields...)
}

// ContextLogger provides context-aware logging
type ContextLogger struct {
	logger Logger
	ctx    map[string]interface{}
}

// NewContextLogger creates a new context logger
func NewContextLogger(logger Logger, ctx map[string]interface{}) *ContextLogger {
	return &ContextLogger{
		logger: logger,
		ctx:    ctx,
	}
}

// Debug logs with context
func (cl *ContextLogger) Debug(msg string, fields ...interface{}) {
	allFields := make([]interface{}, 0)
	for k, v := range cl.ctx {
		allFields = append(allFields, k, v)
	}
	allFields = append(allFields, fields...)
	cl.logger.Debug(msg, allFields...)
}

// Info logs with context
func (cl *ContextLogger) Info(msg string, fields ...interface{}) {
	allFields := make([]interface{}, 0)
	for k, v := range cl.ctx {
		allFields = append(allFields, k, v)
	}
	allFields = append(allFields, fields...)
	cl.logger.Info(msg, allFields...)
}

// Warn logs with context
func (cl *ContextLogger) Warn(msg string, fields ...interface{}) {
	allFields := make([]interface{}, 0)
	for k, v := range cl.ctx {
		allFields = append(allFields, k, v)
	}
	allFields = append(allFields, fields...)
	cl.logger.Warn(msg, allFields...)
}

// Error logs with context
func (cl *ContextLogger) Error(msg string, err error, fields ...interface{}) {
	allFields := make([]interface{}, 0)
	for k, v := range cl.ctx {
		allFields = append(allFields, k, v)
	}
	allFields = append(allFields, fields...)
	cl.logger.Error(msg, err, allFields...)
}

// With creates a new context logger with additional fields
func (cl *ContextLogger) With(fields ...interface{}) Logger {
	newCtx := make(map[string]interface{})
	for k, v := range cl.ctx {
		newCtx[k] = v
	}

	if len(fields) > 0 {
		if len(fields)%2 != 0 {
			fields = append(fields, "missing_value")
		}
		for i := 0; i < len(fields); i += 2 {
			if key, ok := fields[i].(string); ok {
				newCtx[key] = fields[i+1]
			}
		}
	}

	return NewContextLogger(cl.logger, newCtx)
}

// Flush ensures all logs are written
func (cl *ContextLogger) Flush() error {
	return cl.logger.Flush()
}