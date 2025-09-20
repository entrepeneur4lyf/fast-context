package logging

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestLogLevelString tests LogLevel string conversion
func TestLogLevelString(t *testing.T) {
	assert.Equal(t, "DEBUG", LevelDebug.String())
	assert.Equal(t, "INFO", LevelInfo.String())
	assert.Equal(t, "WARN", LevelWarn.String())
	assert.Equal(t, "ERROR", LevelError.String())
}

// TestParseLogLevel tests log level parsing
func TestParseLogLevel(t *testing.T) {
	testCases := []struct {
		input    string
		expected LogLevel
	}{
		{"debug", LevelDebug},
		{"info", LevelInfo},
		{"warn", LevelWarn},
		{"warning", LevelWarn},
		{"error", LevelError},
		{"invalid", LevelInfo},
		{"", LevelInfo},
	}

	for _, tc := range testCases {
		t.Run(tc.input, func(t *testing.T) {
			result := ParseLogLevel(tc.input)
			assert.Equal(t, tc.expected, result)
		})
	}
}

// TestNewStructuredLogger tests structured logger creation
func TestNewStructuredLogger(t *testing.T) {
	logger := NewStructuredLogger(LevelInfo)
	assert.NotNil(t, logger)
	assert.Equal(t, LevelInfo, logger.level)
	assert.NotNil(t, logger.fields)
	assert.NotNil(t, logger.handlers)
}

// TestStructuredLoggerWithOptions tests logger with options
func TestNewStructuredLoggerWithOptions(t *testing.T) {
	logger := NewStructuredLogger(LevelDebug,
		WithConsole(false),
		WithJSON(true),
		WithColor(false),
	)

	assert.NotNil(t, logger)
	assert.Equal(t, LevelDebug, logger.level)
}

// TestLoggerMethods tests various logger methods
func TestLoggerMethods(t *testing.T) {
	logger := NewStructuredLogger(LevelDebug)

	// These should not panic
	logger.Debug("Debug message", "key", "value")
	logger.Info("Info message", "key", "value")
	logger.Warn("Warn message", "key", "value")
	logger.Error("Error message", assert.AnError, "key", "value")
}

// TestLoggerWithFields tests logger with additional fields
func TestLoggerWithFields(t *testing.T) {
	logger := NewStructuredLogger(LevelInfo)

	// Create logger with fields
	loggerWithFields := logger.With("context", "test", "user", "123")
	assert.NotNil(t, loggerWithFields)

	// This should include the fields (we can't easily test this without mocking)
	loggerWithFields.Info("Message with fields")
}

// TestLoggerLevelFiltering tests log level filtering
func TestLoggerLevelFiltering(t *testing.T) {
	// Create a logger at WARN level
	logger := NewStructuredLogger(LevelWarn)

	// These should not be logged (level below WARN)
	logger.Debug("Debug message")
	logger.Info("Info message")

	// These should be logged
	logger.Warn("Warn message")
	logger.Error("Error message", assert.AnError)
}

// TestConsoleHandler tests console handler
func TestConsoleHandler(t *testing.T) {
	handler := &ConsoleHandler{enableColor: false}
	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     LevelInfo,
		Message:   "Test message",
	}

	err := handler.Handle(event)
	assert.NoError(t, err)
}

// TestFileHandler tests file handler
func TestFileHandler(t *testing.T) {
	tempDir := t.TempDir()
	logFile := filepath.Join(tempDir, "test.log")

	handler := &FileHandler{filePath: logFile}
	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     LevelInfo,
		Message:   "Test message",
	}

	err := handler.Handle(event)
	assert.NoError(t, err)
	assert.FileExists(t, logFile)

	// Verify file content
	content, err := os.ReadFile(logFile)
	require.NoError(t, err)
	assert.Contains(t, string(content), "Test message")
}

// TestJSONHandler tests JSON handler
func TestJSONHandler(t *testing.T) {
	var buf bytes.Buffer
	handler := &JSONHandler{output: &buf}

	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     LevelInfo,
		Message:   "Test message",
		Fields: map[string]interface{}{
			"key": "value",
		},
	}

	err := handler.Handle(event)
	assert.NoError(t, err)

	// Parse JSON to verify it's valid
	var parsed map[string]interface{}
	err = json.Unmarshal(buf.Bytes(), &parsed)
	require.NoError(t, err)
	assert.Equal(t, "Test message", parsed["message"])
	assert.Equal(t, "INFO", parsed["level"])
	assert.Equal(t, "value", parsed["fields"].(map[string]interface{})["key"])
}

// TestMetricsHandler tests metrics handler
func TestMetricsHandler(t *testing.T) {
	metrics := NewMetricsCollector()
	handler := &MetricsHandler{metrics: metrics}

	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     LevelInfo,
		Message:   "Test message",
	}

	err := handler.Handle(event)
	assert.NoError(t, err)

	// Verify metrics were recorded
	value, exists := metrics.GetCounter("log_events_total", "level", "INFO")
	assert.True(t, exists)
	assert.Equal(t, int64(1), value)
}

// TestContextLogger tests context logger
func TestContextLogger(t *testing.T) {
	baseLogger := NewStructuredLogger(LevelInfo)
	ctx := map[string]interface{}{
		"request_id": "123",
		"user":       "testuser",
	}

	contextLogger := NewContextLogger(baseLogger, ctx)
	assert.NotNil(t, contextLogger)

	// This should include context fields
	contextLogger.Info("Message with context")
}

// TestMetricsCollector tests metrics collector
func TestMetricsCollector(t *testing.T) {
	metrics := NewMetricsCollector()
	assert.NotNil(t, metrics)

	// Test counter
	metrics.IncrementCounter("test_counter", "tag1", "value1")
	value, exists := metrics.GetCounter("test_counter", "tag1", "value1")
	assert.True(t, exists)
	assert.Equal(t, int64(1), value)

	// Test gauge
	metrics.SetGauge("test_gauge", 42.0, "tag1", "value1")
	gaugeValue, exists := metrics.GetGauge("test_gauge", "tag1", "value1")
	assert.True(t, exists)
	assert.Equal(t, 42.0, gaugeValue)

	// Test histogram
	metrics.ObserveHistogram("test_histogram", 1.5, "tag1", "value1")
	histogram, exists := metrics.GetHistogram("test_histogram", "tag1", "value1")
	assert.True(t, exists)
	assert.Contains(t, histogram.Values, 1.5)

	// Test timer
	duration := 100 * time.Millisecond
	metrics.RecordTimer("test_timer", duration, "tag1", "value1")
	timer, exists := metrics.GetTimer("test_timer", "tag1", "value1")
	assert.True(t, exists)
	assert.Contains(t, timer.Durations, duration)
}

// TestMetricsCollectorExport tests metrics export
func TestMetricsCollectorExport(t *testing.T) {
	metrics := NewMetricsCollector()

	// Add some metrics
	metrics.IncrementCounter("test_counter", "tag1", "value1")
	metrics.SetGauge("test_gauge", 42.0)
	metrics.ObserveHistogram("test_histogram", 1.5)
	metrics.RecordTimer("test_timer", 100*time.Millisecond)

	// Export metrics
	data, err := metrics.ExportMetrics()
	require.NoError(t, err)

	// Verify JSON is valid
	var parsed map[string]interface{}
	err = json.Unmarshal(data, &parsed)
	require.NoError(t, err)

	// Verify structure
	assert.Contains(t, parsed, "counters")
	assert.Contains(t, parsed, "gauges")
	assert.Contains(t, parsed, "histograms")
	assert.Contains(t, parsed, "timers")
}

// TestMetricsCollectorReset tests metrics reset
func TestMetricsCollectorReset(t *testing.T) {
	metrics := NewMetricsCollector()

	// Add some metrics
	metrics.IncrementCounter("test_counter")
	metrics.SetGauge("test_gauge", 42.0)

	// Verify metrics exist
	value, exists := metrics.GetCounter("test_counter")
	assert.True(t, exists)
	assert.Equal(t, int64(1), value)

	// Reset metrics
	metrics.Reset()

	// Verify metrics are gone
	_, exists = metrics.GetCounter("test_counter")
	assert.False(t, exists)
}

// TestHistogramStats tests histogram statistics
func TestHistogramStats(t *testing.T) {
	histogram := &Histogram{
		Values: []float64{1.0, 2.0, 3.0, 4.0, 5.0},
	}

	stats := histogram.CalculateStats()
	assert.NotNil(t, stats)
	assert.Equal(t, int64(5), stats.Count)
	assert.Equal(t, 15.0, stats.Sum)
	assert.Equal(t, 3.0, stats.Average)
	assert.Equal(t, 1.0, stats.Min)
	assert.Equal(t, 5.0, stats.Max)
}

// TestTimerStats tests timer statistics
func TestTimerStats(t *testing.T) {
	durations := []time.Duration{
		100 * time.Millisecond,
		200 * time.Millisecond,
		300 * time.Millisecond,
	}

	timer := &Timer{Durations: durations}
	stats := timer.CalculateStats()

	assert.NotNil(t, stats)
	assert.Equal(t, int64(3), stats.Count)
	assert.Equal(t, 600*time.Millisecond, stats.Sum)
	assert.Equal(t, 200*time.Millisecond, stats.Average)
	assert.Equal(t, 100*time.Millisecond, stats.Min)
	assert.Equal(t, 300*time.Millisecond, stats.Max)
}

// TestOperationTimer tests operation timer
func TestOperationTimer(t *testing.T) {
	metrics := NewMetricsCollector()

	timer := NewTimer("test_operation", "tag1", "value1")
	assert.NotNil(t, timer)

	// Simulate some work
	time.Sleep(10 * time.Millisecond)

	duration := timer.Stop()
	assert.Greater(t, duration, time.Duration(0))

	// Verify timer was recorded
	timerMetric, exists := metrics.GetTimer("test_operation", "tag1", "value1")
	assert.True(t, exists)
	assert.Len(t, timerMetric.Durations, 1)
}

// TestTimeFunction tests function timing
func TestTimeFunction(t *testing.T) {
	metrics := NewMetricsCollector()

	duration := TimeFunction("timed_function", func() {
		time.Sleep(10 * time.Millisecond)
	}, "tag1", "value1")

	assert.Greater(t, duration, time.Duration(0))

	// Verify timer was recorded
	timerMetric, exists := metrics.GetTimer("timed_function", "tag1", "value1")
	assert.True(t, exists)
	assert.Len(t, timerMetric.Durations, 1)
}

// TestTimeFunctionWithError tests function timing with error
func TestTimeFunctionWithError(t *testing.T) {
	metrics := NewMetricsCollector()

	duration, err := TimeFunctionWithError("timed_function", func() error {
		time.Sleep(10 * time.Millisecond)
		return assert.AnError
	}, "tag1", "value1")

	assert.Greater(t, duration, time.Duration(0))
	assert.Error(t, err)

	// Verify timer was recorded
	timerMetric, exists := metrics.GetTimer("timed_function", "tag1", "value1")
	assert.True(t, exists)
	assert.Len(t, timerMetric.Durations, 1)
}

// TestGlobalLogger tests global logger functions
func TestGlobalLogger(t *testing.T) {
	// Reset global logger
	defaultLogger = nil

	// These should initialize the default logger
	Debug("Debug message")
	Info("Info message")
	Warn("Warn message")
	Error("Error message", assert.AnError)

	// Test With function
	loggerWithFields := With("key", "value")
	assert.NotNil(t, loggerWithFields)
	loggerWithFields.Info("Message with fields")
}

// TestGlobalMetrics tests global metrics functions
func TestGlobalMetrics(t *testing.T) {
	// Reset global metrics collector
	defaultMetricsCollector = nil

	// These should initialize the default metrics collector
	IncrementCounter("global_counter", "tag1", "value1")
	SetGauge("global_gauge", 42.0, "tag1", "value1")
	ObserveHistogram("global_histogram", 1.5, "tag1", "value1")
	RecordTimer("global_timer", 100*time.Millisecond, "tag1", "value1")

	// Verify metrics were recorded
	metrics := GetMetricsCollector()
	assert.NotNil(t, metrics)

	value, exists := metrics.GetCounter("global_counter", "tag1", "value1")
	assert.True(t, exists)
	assert.Equal(t, int64(1), value)
}

// TestMetricsCollectorWithSameTags tests metrics collector with same tags
func TestMetricsCollectorWithSameTags(t *testing.T) {
	metrics := NewMetricsCollector()

	// Add multiple metrics with same name and tags
	metrics.IncrementCounter("test_counter", "tag1", "value1")
	metrics.IncrementCounter("test_counter", "tag1", "value1")

	value, exists := metrics.GetCounter("test_counter", "tag1", "value1")
	assert.True(t, exists)
	assert.Equal(t, int64(2), value)
}

// TestMetricsCollectorWithDifferentTags tests metrics collector with different tags
func TestMetricsCollectorWithDifferentTags(t *testing.T) {
	metrics := NewMetricsCollector()

	// Add metrics with same name but different tags
	metrics.IncrementCounter("test_counter", "tag1", "value1")
	metrics.IncrementCounter("test_counter", "tag1", "value2")

	value1, exists1 := metrics.GetCounter("test_counter", "tag1", "value1")
	value2, exists2 := metrics.GetCounter("test_counter", "tag1", "value2")

	assert.True(t, exists1)
	assert.True(t, exists2)
	assert.Equal(t, int64(1), value1)
	assert.Equal(t, int64(1), value2)
}

// TestMetricsCollectorGetAllMetrics tests getting all metrics
func TestMetricsCollectorGetAllMetrics(t *testing.T) {
	metrics := NewMetricsCollector()

	// Add various metrics
	metrics.IncrementCounter("counter1")
	metrics.SetGauge("gauge1", 42.0)
	metrics.ObserveHistogram("histogram1", 1.5)
	metrics.RecordTimer("timer1", 100*time.Millisecond)

	allMetrics := metrics.GetAllMetrics()
	assert.NotNil(t, allMetrics)

	// Verify structure
	assert.Contains(t, allMetrics, "counters")
	assert.Contains(t, allMetrics, "gauges")
	assert.Contains(t, allMetrics, "histograms")
	assert.Contains(t, allMetrics, "timers")
}

// TestLoggerFlush tests logger flush functionality
func TestLoggerFlush(t *testing.T) {
	logger := NewStructuredLogger(LevelInfo)
	err := logger.Flush()
	assert.NoError(t, err)
}

// TestLogEventSerialization tests log event JSON serialization
func TestLogEventSerialization(t *testing.T) {
	event := &LogEvent{
		Timestamp: time.Now(),
		Level:     LevelInfo,
		Message:   "Test message",
		Fields: map[string]interface{}{
			"key": "value",
		},
	}

	data, err := json.Marshal(event)
	require.NoError(t, err)

	// Parse back to verify
	var parsed LogEvent
	err = json.Unmarshal(data, &parsed)
	require.NoError(t, err)
	assert.Equal(t, event.Message, parsed.Message)
	assert.Equal(t, event.Level, parsed.Level)
}

// TestMetricsCollectorEmpty tests metrics collector with empty metrics
func TestMetricsCollectorGetEmpty(t *testing.T) {
	metrics := NewMetricsCollector()

	// Test getting non-existent metrics
	value, exists := metrics.GetCounter("non_existent")
	assert.False(t, exists)
	assert.Equal(t, int64(0), value)

	gaugeValue, exists := metrics.GetGauge("non_existent")
	assert.False(t, exists)
	assert.Equal(t, 0.0, gaugeValue)

	_, exists = metrics.GetHistogram("non_existent")
	assert.False(t, exists)

	_, exists = metrics.GetTimer("non_existent")
	assert.False(t, exists)
}