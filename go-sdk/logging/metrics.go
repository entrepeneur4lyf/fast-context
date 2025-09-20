package logging

import (
	"encoding/json"
	"fmt"
	"math"
	"sort"
	"sync"
	"time"
)

// MetricsCollector collects and manages metrics
type MetricsCollector struct {
	mu         sync.RWMutex
	counters   map[string]*Counter
	gauges     map[string]*Gauge
	histograms map[string]*Histogram
	timers     map[string]*Timer
}

// Counter represents a counter metric
type Counter struct {
	Value int64
	Tags  map[string]string
}

// Gauge represents a gauge metric
type Gauge struct {
	Value float64
	Tags  map[string]string
}

// Histogram represents a histogram metric
type Histogram struct {
	Values []float64
	Tags   map[string]string
}

// Timer represents a timer metric
type Timer struct {
	Durations []time.Duration
	Tags      map[string]string
}

// NewMetricsCollector creates a new metrics collector
func NewMetricsCollector() *MetricsCollector {
	return &MetricsCollector{
		counters:   make(map[string]*Counter),
		gauges:     make(map[string]*Gauge),
		histograms: make(map[string]*Histogram),
		timers:     make(map[string]*Timer),
	}
}

// IncrementCounter increments a counter metric
func (mc *MetricsCollector) IncrementCounter(name string, tags ...string) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	key := mc.buildKey(name, tags...)
	if _, exists := mc.counters[key]; !exists {
		mc.counters[key] = &Counter{
			Tags: mc.buildTags(tags...),
		}
	}
	mc.counters[key].Value++
}

// SetGauge sets a gauge metric
func (mc *MetricsCollector) SetGauge(name string, value float64, tags ...string) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	key := mc.buildKey(name, tags...)
	if _, exists := mc.gauges[key]; !exists {
		mc.gauges[key] = &Gauge{
			Tags: mc.buildTags(tags...),
		}
	}
	mc.gauges[key].Value = value
}

// ObserveHistogram observes a histogram value
func (mc *MetricsCollector) ObserveHistogram(name string, value float64, tags ...string) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	key := mc.buildKey(name, tags...)
	if _, exists := mc.histograms[key]; !exists {
		mc.histograms[key] = &Histogram{
			Tags: mc.buildTags(tags...),
		}
	}
	mc.histograms[key].Values = append(mc.histograms[key].Values, value)
}

// RecordTimer records a timer duration
func (mc *MetricsCollector) RecordTimer(name string, duration time.Duration, tags ...string) {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	key := mc.buildKey(name, tags...)
	if _, exists := mc.timers[key]; !exists {
		mc.timers[key] = &Timer{
			Tags: mc.buildTags(tags...),
		}
	}
	mc.timers[key].Durations = append(mc.timers[key].Durations, duration)
}

// GetCounter returns a counter metric
func (mc *MetricsCollector) GetCounter(name string, tags ...string) (int64, bool) {
	mc.mu.RLock()
	defer mc.mu.RUnlock()

	key := mc.buildKey(name, tags...)
	if counter, exists := mc.counters[key]; exists {
		return counter.Value, true
	}
	return 0, false
}

// GetGauge returns a gauge metric
func (mc *MetricsCollector) GetGauge(name string, tags ...string) (float64, bool) {
	mc.mu.RLock()
	defer mc.mu.RUnlock()

	key := mc.buildKey(name, tags...)
	if gauge, exists := mc.gauges[key]; exists {
		return gauge.Value, true
	}
	return 0, false
}

// GetHistogram returns a histogram metric
func (mc *MetricsCollector) GetHistogram(name string, tags ...string) (*Histogram, bool) {
	mc.mu.RLock()
	defer mc.mu.RUnlock()

	key := mc.buildKey(name, tags...)
	if histogram, exists := mc.histograms[key]; exists {
		return histogram, true
	}
	return nil, false
}

// GetTimer returns a timer metric
func (mc *MetricsCollector) GetTimer(name string, tags ...string) (*Timer, bool) {
	mc.mu.RLock()
	defer mc.mu.RUnlock()

	key := mc.buildKey(name, tags...)
	if timer, exists := mc.timers[key]; exists {
		return timer, true
	}
	return nil, false
}

// GetAllMetrics returns all metrics as a map
func (mc *MetricsCollector) GetAllMetrics() map[string]interface{} {
	mc.mu.RLock()
	defer mc.mu.RUnlock()

	result := make(map[string]interface{})

	// Add counters
	counters := make(map[string]interface{})
	for key, counter := range mc.counters {
		counters[key] = counter.Value
	}
	result["counters"] = counters

	// Add gauges
	gauges := make(map[string]interface{})
	for key, gauge := range mc.gauges {
		gauges[key] = gauge.Value
	}
	result["gauges"] = gauges

	// Add histograms
	histograms := make(map[string]interface{})
	for key, histogram := range mc.histograms {
		histograms[key] = map[string]interface{}{
			"values": histogram.Values,
			"count":  len(histogram.Values),
			"sum":    mc.sum(histogram.Values),
			"avg":    mc.avg(histogram.Values),
			"min":    mc.min(histogram.Values),
			"max":    mc.max(histogram.Values),
		}
	}
	result["histograms"] = histograms

	// Add timers
	timers := make(map[string]interface{})
	for key, timer := range mc.timers {
		durations := make([]float64, len(timer.Durations))
		for i, d := range timer.Durations {
			durations[i] = d.Seconds()
		}
		timers[key] = map[string]interface{}{
			"durations": durations,
			"count":     len(timer.Durations),
			"sum":       mc.sum(durations),
			"avg":       mc.avg(durations),
			"min":       mc.min(durations),
			"max":       mc.max(durations),
		}
	}
	result["timers"] = timers

	return result
}

// ExportMetrics exports metrics in JSON format
func (mc *MetricsCollector) ExportMetrics() ([]byte, error) {
	metrics := mc.GetAllMetrics()
	return json.MarshalIndent(metrics, "", "  ")
}

// Reset resets all metrics
func (mc *MetricsCollector) Reset() {
	mc.mu.Lock()
	defer mc.mu.Unlock()

	mc.counters = make(map[string]*Counter)
	mc.gauges = make(map[string]*Gauge)
	mc.histograms = make(map[string]*Histogram)
	mc.timers = make(map[string]*Timer)
}

// Helper methods

func (mc *MetricsCollector) buildKey(name string, tags ...string) string {
	if len(tags) == 0 {
		return name
	}
	return fmt.Sprintf("%s|%v", name, tags)
}

func (mc *MetricsCollector) buildTags(tags ...string) map[string]string {
	result := make(map[string]string)
	for i := 0; i < len(tags); i += 2 {
		if i+1 < len(tags) {
			result[tags[i]] = tags[i+1]
		}
	}
	return result
}

func (mc *MetricsCollector) sum(values []float64) float64 {
	var sum float64
	for _, v := range values {
		sum += v
	}
	return sum
}

func (mc *MetricsCollector) avg(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	return mc.sum(values) / float64(len(values))
}

func (mc *MetricsCollector) min(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	min := values[0]
	for _, v := range values[1:] {
		if v < min {
			min = v
		}
	}
	return min
}

func (mc *MetricsCollector) max(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	max := values[0]
	for _, v := range values[1:] {
		if v > max {
			max = v
		}
	}
	return max
}

// HistogramStats provides statistical information about a histogram
type HistogramStats struct {
	Count     int64   `json:"count"`
	Sum       float64 `json:"sum"`
	Average   float64 `json:"average"`
	Min       float64 `json:"min"`
	Max       float64 `json:"max"`
	P50       float64 `json:"p50"`
	P95       float64 `json:"p95"`
	P99       float64 `json:"p99"`
	StdDev    float64 `json:"stdDev"`
	Variance  float64 `json:"variance"`
}

// CalculateStats calculates statistics for a histogram
func (h *Histogram) CalculateStats() *HistogramStats {
	if len(h.Values) == 0 {
		return &HistogramStats{}
	}

	values := make([]float64, len(h.Values))
	copy(values, h.Values)

	sort.Float64s(values)

	stats := &HistogramStats{
		Count: int64(len(values)),
		Sum:   h.sum(),
		Min:   values[0],
		Max:   values[len(values)-1],
	}

	stats.Average = stats.Sum / float64(stats.Count)

	// Calculate percentiles
	stats.P50 = h.percentile(values, 50)
	stats.P95 = h.percentile(values, 95)
	stats.P99 = h.percentile(values, 99)

	// Calculate standard deviation
	stats.Variance = h.variance(values, stats.Average)
	stats.StdDev = math.Sqrt(stats.Variance)

	return stats
}

func (h *Histogram) sum() float64 {
	var sum float64
	for _, v := range h.Values {
		sum += v
	}
	return sum
}

func (h *Histogram) percentile(values []float64, p float64) float64 {
	if len(values) == 0 {
		return 0
	}
	
	index := int(float64(len(values)-1) * p / 100)
	if index >= len(values) {
		index = len(values) - 1
	}
	return values[index]
}

func (h *Histogram) variance(values []float64, mean float64) float64 {
	if len(values) == 0 {
		return 0
	}
	
	var sum float64
	for _, v := range values {
		diff := v - mean
		sum += diff * diff
	}
	return sum / float64(len(values))
}

// TimerStats provides statistical information about a timer
type TimerStats struct {
	Count     int64         `json:"count"`
	Sum       time.Duration `json:"sum"`
	Average   time.Duration `json:"average"`
	Min       time.Duration `json:"min"`
	Max       time.Duration `json:"max"`
	P50       time.Duration `json:"p50"`
	P95       time.Duration `json:"p95"`
	P99       time.Duration `json:"p99"`
}

// CalculateStats calculates statistics for a timer
func (t *Timer) CalculateStats() *TimerStats {
	if len(t.Durations) == 0 {
		return &TimerStats{}
	}

	durations := make([]time.Duration, len(t.Durations))
	copy(durations, t.Durations)

	sort.Slice(durations, func(i, j int) bool {
		return durations[i] < durations[j]
	})

	stats := &TimerStats{
		Count: int64(len(durations)),
		Min:   durations[0],
		Max:   durations[len(durations)-1],
	}

	// Calculate sum and average
	var sum time.Duration
	for _, d := range durations {
		sum += d
	}
	stats.Sum = sum
	stats.Average = time.Duration(int64(sum) / stats.Count)

	// Calculate percentiles
	stats.P50 = t.percentile(durations, 50)
	stats.P95 = t.percentile(durations, 95)
	stats.P99 = t.percentile(durations, 99)

	return stats
}

func (t *Timer) percentile(durations []time.Duration, p float64) time.Duration {
	if len(durations) == 0 {
		return 0
	}
	
	index := int(float64(len(durations)-1) * p / 100)
	if index >= len(durations) {
		index = len(durations) - 1
	}
	return durations[index]
}

// Global metrics collector instance
var defaultMetricsCollector *MetricsCollector

// InitializeDefaultMetricsCollector initializes the default metrics collector
func InitializeDefaultMetricsCollector() {
	defaultMetricsCollector = NewMetricsCollector()
}

// GetMetricsCollector returns the default metrics collector
func GetMetricsCollector() *MetricsCollector {
	if defaultMetricsCollector == nil {
		InitializeDefaultMetricsCollector()
	}
	return defaultMetricsCollector
}

// Convenience functions for global metrics collection

func IncrementCounter(name string, tags ...string) {
	GetMetricsCollector().IncrementCounter(name, tags...)
}

func SetGauge(name string, value float64, tags ...string) {
	GetMetricsCollector().SetGauge(name, value, tags...)
}

func ObserveHistogram(name string, value float64, tags ...string) {
	GetMetricsCollector().ObserveHistogram(name, value, tags...)
}

func RecordTimer(name string, duration time.Duration, tags ...string) {
	GetMetricsCollector().RecordTimer(name, duration, tags...)
}

// Timer provides a convenient way to time operations
type OperationTimer struct {
	name string
	tags []string
	start time.Time
}

// NewTimer creates a new operation timer
func NewTimer(name string, tags ...string) *OperationTimer {
	return &OperationTimer{
		name:  name,
		tags:  tags,
		start: time.Now(),
	}
}

// Stop stops the timer and records the duration
func (t *OperationTimer) Stop() time.Duration {
	duration := time.Since(t.start)
	RecordTimer(t.name, duration, t.tags...)
	return duration
}

// TimeFunction times a function execution
func TimeFunction(name string, fn func(), tags ...string) time.Duration {
	timer := NewTimer(name, tags...)
	defer timer.Stop()
	fn()
	return timer.Stop()
}

// TimeFunctionWithError times a function that returns an error
func TimeFunctionWithError(name string, fn func() error, tags ...string) (time.Duration, error) {
	timer := NewTimer(name, tags...)
	defer timer.Stop()
	err := fn()
	return timer.Stop(), err
}