package config

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// CachePolicy defines how caching should be handled
type CachePolicy int

const (
	CachePolicyNone CachePolicy = iota
	CachePolicyMinimal
	CachePolicyBalanced
	CachePolicyAggressive
	CachePolicyPersistent
)

func (c CachePolicy) String() string {
	switch c {
	case CachePolicyNone:
		return "none"
	case CachePolicyMinimal:
		return "minimal"
	case CachePolicyBalanced:
		return "balanced"
	case CachePolicyAggressive:
		return "aggressive"
	case CachePolicyPersistent:
		return "persistent"
	default:
		return "unknown"
	}
}

// PerformanceConfig contains performance-related settings
type PerformanceConfig struct {
	MaxMemoryMB           int     `json:"maxMemoryMB" yaml:"maxMemoryMB"`
	MaxConcurrentFiles    int     `json:"maxConcurrentFiles" yaml:"maxConcurrentFiles"`
	TimeoutSeconds        int     `json:"timeoutSeconds" yaml:"timeoutSeconds"`
	CachePolicy           CachePolicy `json:"cachePolicy" yaml:"cachePolicy"`
	EnableParallel        bool    `json:"enableParallel" yaml:"enableParallel"`
	EnableStreaming       bool    `json:"enableStreaming" yaml:"enableStreaming"`
	EnableWatching        bool    `json:"enableWatching" yaml:"enableWatching"`
	AnalysisDepth         int     `json:"analysisDepth" yaml:"analysisDepth"`
}

// Config contains all configuration for the Fast-Context analyzer
type Config struct {
	ProjectRoot       string            `json:"projectRoot" yaml:"projectRoot"`
	Languages         []string          `json:"languages" yaml:"languages"`
	IgnorePatterns    []string          `json:"ignorePatterns" yaml:"ignorePatterns"`
	IncludePatterns   []string          `json:"includePatterns" yaml:"includePatterns"`
	Performance       PerformanceConfig `json:"performance" yaml:"performance"`
	EnableProgress    bool              `json:"enableProgress" yaml:"enableProgress"`
	EnableMetrics     bool              `json:"enableMetrics" yaml:"enableMetrics"`
	LogLevel          string            `json:"logLevel" yaml:"logLevel"`
	MaxFileSizeKB     int               `json:"maxFileSizeKB" yaml:"maxFileSizeKB"`
	MaxFiles          int               `json:"maxFiles" yaml:"maxFiles"`
	Preset            ConfigPreset      `json:"preset" yaml:"preset"`
}

// ConfigOption is a functional option for configuring the analyzer
type ConfigOption func(*Config) error

// NewConfig creates a new configuration with default values
func NewConfig(opts ...ConfigOption) (*Config, error) {
	config := &Config{
		ProjectRoot: ".",
		Languages:   []string{},
		IgnorePatterns: []string{
			"node_modules/**",
			"target/**",
			"build/**",
			"dist/**",
			"vendor/**",
			"*.min.js",
			"*.min.css",
			"__pycache__/**",
			".git/**",
		},
		Performance: PerformanceConfig{
			MaxMemoryMB:        1024,
			MaxConcurrentFiles: 50,
			TimeoutSeconds:     300,
			CachePolicy:        CachePolicyBalanced,
			EnableParallel:     true,
			EnableStreaming:    true,
			EnableWatching:     false,
			AnalysisDepth:      3,
		},
		EnableProgress: true,
		EnableMetrics:  true,
		LogLevel:       "info",
		MaxFileSizeKB:  1024,
		MaxFiles:       10000,
	}

	for _, opt := range opts {
		if err := opt(config); err != nil {
			return nil, err
		}
	}

	if err := config.Validate(); err != nil {
		return nil, err
	}

	return config, nil
}

// Validate validates the configuration
func (c *Config) Validate() error {
	if c.ProjectRoot == "" {
		return fmt.Errorf("project root cannot be empty")
	}

	// Check if project root exists
	if _, err := os.Stat(c.ProjectRoot); os.IsNotExist(err) {
		return fmt.Errorf("project root does not exist: %s", c.ProjectRoot)
	}

	// Resolve to absolute path
	absPath, err := filepath.Abs(c.ProjectRoot)
	if err != nil {
		return fmt.Errorf("cannot resolve project root path: %w", err)
	}
	c.ProjectRoot = absPath

	// Validate performance settings
	if c.Performance.MaxMemoryMB <= 0 {
		return fmt.Errorf("max memory must be positive")
	}

	if c.Performance.MaxConcurrentFiles <= 0 {
		return fmt.Errorf("max concurrent files must be positive")
	}

	if c.Performance.TimeoutSeconds <= 0 {
		return fmt.Errorf("timeout must be positive")
	}

	if c.MaxFileSizeKB <= 0 {
		return fmt.Errorf("max file size must be positive")
	}

	if c.MaxFiles <= 0 {
		return fmt.Errorf("max files must be positive")
	}

	// Validate log level
	validLogLevels := map[string]bool{
		"debug": true, "info": true, "warn": true, "error": true,
	}
	if !validLogLevels[c.LogLevel] {
		return fmt.Errorf("invalid log level: %s", c.LogLevel)
	}

	return nil
}

// ConfigOption functions

// WithProjectRoot sets the project root directory
func WithProjectRoot(root string) ConfigOption {
	return func(c *Config) error {
		c.ProjectRoot = root
		return nil
	}
}

// WithLanguages sets the languages to analyze
func WithLanguages(languages []string) ConfigOption {
	return func(c *Config) error {
		c.Languages = languages
		return nil
	}
}

// WithIgnorePatterns sets file patterns to ignore
func WithIgnorePatterns(patterns []string) ConfigOption {
	return func(c *Config) error {
		c.IgnorePatterns = patterns
		return nil
	}
}

// WithIncludePatterns sets file patterns to include
func WithIncludePatterns(patterns []string) ConfigOption {
	return func(c *Config) error {
		c.IncludePatterns = patterns
		return nil
	}
}

// WithPerformanceConfig sets performance configuration
func WithPerformanceConfig(perf PerformanceConfig) ConfigOption {
	return func(c *Config) error {
		c.Performance = perf
		return nil
	}
}

// WithCachePolicy sets the caching policy
func WithCachePolicy(policy CachePolicy) ConfigOption {
	return func(c *Config) error {
		c.Performance.CachePolicy = policy
		return nil
	}
}

// WithMaxMemory sets the maximum memory usage in MB
func WithMaxMemory(mb int) ConfigOption {
	return func(c *Config) error {
		c.Performance.MaxMemoryMB = mb
		return nil
	}
}

// WithTimeout sets the analysis timeout in seconds
func WithTimeout(seconds int) ConfigOption {
	return func(c *Config) error {
		c.Performance.TimeoutSeconds = seconds
		return nil
	}
}

// WithTimeoutSeconds sets the analysis timeout in seconds (alias for WithTimeout)
func WithTimeoutSeconds(seconds int) ConfigOption {
	return WithTimeout(seconds)
}

// WithParallelProcessing enables or disables parallel processing
func WithParallelProcessing(enabled bool) ConfigOption {
	return func(c *Config) error {
		c.Performance.EnableParallel = enabled
		return nil
	}
}

// WithProgress enables or disables progress reporting
func WithProgress(enabled bool) ConfigOption {
	return func(c *Config) error {
		c.EnableProgress = enabled
		return nil
	}
}

// WithMetrics enables or disables metrics collection
func WithMetrics(enabled bool) ConfigOption {
	return func(c *Config) error {
		c.EnableMetrics = enabled
		return nil
	}
}

// WithLogLevel sets the logging level
func WithLogLevel(level string) ConfigOption {
	return func(c *Config) error {
		c.LogLevel = strings.ToLower(level)
		return nil
	}
}

// Preset configurations

// FastConfig returns a configuration optimized for speed
func FastConfig(projectRoot string) (*Config, error) {
	return NewConfig(
		WithProjectRoot(projectRoot),
		WithCachePolicy(CachePolicyMinimal),
		WithMaxMemory(512),
		WithTimeout(120),
		WithParallelProcessing(true),
		WithProgress(false),
		WithMetrics(false),
	)
}

// BalancedConfig returns a balanced configuration
func BalancedConfig(projectRoot string) (*Config, error) {
	return NewConfig(
		WithProjectRoot(projectRoot),
		WithCachePolicy(CachePolicyBalanced),
		WithMaxMemory(1024),
		WithTimeout(300),
		WithParallelProcessing(true),
		WithProgress(true),
		WithMetrics(true),
	)
}

// ThoroughConfig returns a configuration for thorough analysis
func ThoroughConfig(projectRoot string) (*Config, error) {
	return NewConfig(
		WithProjectRoot(projectRoot),
		WithCachePolicy(CachePolicyAggressive),
		WithMaxMemory(2048),
		WithTimeout(600),
		WithParallelProcessing(false),
		WithProgress(true),
		WithMetrics(true),
	)
}

// ConfigPreset represents predefined configuration presets
type ConfigPreset int

const (
	PresetDefault ConfigPreset = iota
	PresetFast
	PresetBalanced
	PresetThorough
)

// WithPreset applies a predefined configuration preset
func WithPreset(preset ConfigPreset) ConfigOption {
	return func(c *Config) error {
		c.Preset = preset
		
		switch preset {
		case PresetFast:
			c.Performance.TimeoutSeconds = 120
			c.Performance.MaxMemoryMB = 512
			c.Performance.CachePolicy = CachePolicyMinimal
			c.EnableProgress = false
			c.EnableMetrics = false
		case PresetBalanced:
			c.Performance.TimeoutSeconds = 300
			c.Performance.MaxMemoryMB = 1024
			c.Performance.CachePolicy = CachePolicyBalanced
			c.EnableProgress = true
			c.EnableMetrics = true
		case PresetThorough:
			c.Performance.TimeoutSeconds = 600
			c.Performance.MaxMemoryMB = 2048
			c.Performance.CachePolicy = CachePolicyAggressive
			c.EnableProgress = true
			c.EnableMetrics = true
			c.Performance.EnableParallel = false
		}
		
		return nil
	}
}