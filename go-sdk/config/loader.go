package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/viper"
	"gopkg.in/yaml.v3"
	"github.com/pelletier/go-toml/v2"
)

// ConfigLoader handles loading configuration from various sources
type ConfigLoader struct {
	viper *viper.Viper
}

// NewConfigLoader creates a new configuration loader
func NewConfigLoader() *ConfigLoader {
	v := viper.New()
	v.SetConfigName(".fast-context")
	v.SetConfigType("yaml")

	// Set default search paths
	v.AddConfigPath(".")
	v.AddConfigPath("$HOME")
	v.AddConfigPath("$HOME/.config")
	v.AddConfigPath("/etc/fast-context")

	return &ConfigLoader{viper: v}
}

// Load loads configuration from file with the specified path
func (cl *ConfigLoader) Load(configPath string) (*Config, error) {
	if configPath != "" {
		// Load from specific file
		ext := filepath.Ext(configPath)
		if ext != "" {
			ext = ext[1:] // Remove dot
		}

		cl.viper.SetConfigFile(configPath)
		if ext != "" {
			cl.viper.SetConfigType(ext)
		}
	} else {
		// Search for config file in default locations
		if err := cl.viper.ReadInConfig(); err != nil {
			if _, ok := err.(viper.ConfigFileNotFoundError); ok {
				// Config file not found, return default config
				return NewConfig()
			}
			return nil, fmt.Errorf("failed to read config file: %w", err)
		}
	}

	// Read the config file
	if err := cl.viper.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	// Unmarshal into our Config struct
	var cfg Config
	if err := cl.viper.Unmarshal(&cfg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}

	// Apply environment variables
	cl.applyEnvironmentOverrides(&cfg)

	// Validate the configuration
	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("configuration validation failed: %w", err)
	}

	return &cfg, nil
}

// applyEnvironmentOverrides applies environment variable overrides
func (cl *ConfigLoader) applyEnvironmentOverrides(cfg *Config) {
	// Project root
	if root := os.Getenv("FAST_CONTEXT_PROJECT_ROOT"); root != "" {
		cfg.ProjectRoot = root
	}

	// Log level
	if level := os.Getenv("FAST_CONTEXT_LOG_LEVEL"); level != "" {
		cfg.LogLevel = strings.ToLower(level)
	}

	// Performance settings
	if mem := os.Getenv("FAST_CONTEXT_MAX_MEMORY_MB"); mem != "" {
		if val := parseInt(mem); val > 0 {
			cfg.Performance.MaxMemoryMB = val
		}
	}

	if timeout := os.Getenv("FAST_CONTEXT_TIMEOUT_SECONDS"); timeout != "" {
		if val := parseInt(timeout); val > 0 {
			cfg.Performance.TimeoutSeconds = val
		}
	}

	if concurrent := os.Getenv("FAST_CONTEXT_MAX_CONCURRENT_FILES"); concurrent != "" {
		if val := parseInt(concurrent); val > 0 {
			cfg.Performance.MaxConcurrentFiles = val
		}
	}

	// Cache policy
	if policy := os.Getenv("FAST_CONTEXT_CACHE_POLICY"); policy != "" {
		switch strings.ToLower(policy) {
		case "none":
			cfg.Performance.CachePolicy = CachePolicyNone
		case "minimal":
			cfg.Performance.CachePolicy = CachePolicyMinimal
		case "balanced":
			cfg.Performance.CachePolicy = CachePolicyBalanced
		case "aggressive":
			cfg.Performance.CachePolicy = CachePolicyAggressive
		case "persistent":
			cfg.Performance.CachePolicy = CachePolicyPersistent
		}
	}

	// Boolean flags
	if parallel := os.Getenv("FAST_CONTEXT_ENABLE_PARALLEL"); parallel != "" {
		cfg.Performance.EnableParallel = parseBool(parallel)
	}

	if streaming := os.Getenv("FAST_CONTEXT_ENABLE_STREAMING"); streaming != "" {
		cfg.Performance.EnableStreaming = parseBool(streaming)
	}

	if watching := os.Getenv("FAST_CONTEXT_ENABLE_WATCHING"); watching != "" {
		cfg.Performance.EnableWatching = parseBool(watching)
	}

	if progress := os.Getenv("FAST_CONTEXT_ENABLE_PROGRESS"); progress != "" {
		cfg.EnableProgress = parseBool(progress)
	}

	if metrics := os.Getenv("FAST_CONTEXT_ENABLE_METRICS"); metrics != "" {
		cfg.EnableMetrics = parseBool(metrics)
	}

	// File size and count limits
	if maxSize := os.Getenv("FAST_CONTEXT_MAX_FILE_SIZE_KB"); maxSize != "" {
		if val := parseInt(maxSize); val > 0 {
			cfg.MaxFileSizeKB = val
		}
	}

	if maxFiles := os.Getenv("FAST_CONTEXT_MAX_FILES"); maxFiles != "" {
		if val := parseInt(maxFiles); val > 0 {
			cfg.MaxFiles = val
		}
	}

	// Analysis depth
	if depth := os.Getenv("FAST_CONTEXT_ANALYSIS_DEPTH"); depth != "" {
		if val := parseInt(depth); val > 0 {
			cfg.Performance.AnalysisDepth = val
		}
	}
}

// DiscoverConfig searches for configuration files in common locations
func (cl *ConfigLoader) DiscoverConfig() string {
	// List of possible config file names
	configNames := []string{
		".fast-context.yaml",
		".fast-context.yml",
		".fast-context.json",
		".fast-context.toml",
		"fast-context.yaml",
		"fast-context.yml",
		"fast-context.json",
		"fast-context.toml",
	}

	// Search paths
	paths := []string{
		".",
		"$HOME",
		"$HOME/.config",
		"$HOME/.config/fast-context",
		"/etc/fast-context",
		"/etc",
	}

	for _, path := range paths {
		for _, name := range configNames {
			configPath := filepath.Join(path, name)
			if _, err := os.Stat(configPath); err == nil {
				return configPath
			}
		}
	}

	return ""
}

// Save saves the configuration to a file
func (cl *ConfigLoader) Save(cfg *Config, filePath string) error {
	var data []byte
	var err error

	ext := filepath.Ext(filePath)
	switch strings.ToLower(ext) {
	case ".yaml", ".yml":
		data, err = yaml.Marshal(cfg)
	case ".json":
		data, err = json.MarshalIndent(cfg, "", "  ")
	case ".toml":
		data, err = toml.Marshal(cfg)
	default:
		// Default to YAML
		data, err = yaml.Marshal(cfg)
	}

	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	// Ensure directory exists
	dir := filepath.Dir(filePath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	if err := os.WriteFile(filePath, data, 0644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

// LoadFromJSON loads configuration from JSON string
func LoadFromJSON(jsonStr string) (*Config, error) {
	var cfg Config
	if err := json.Unmarshal([]byte(jsonStr), &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse JSON config: %w", err)
	}

	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("configuration validation failed: %w", err)
	}

	return &cfg, nil
}

// LoadFromYAML loads configuration from YAML string
func LoadFromYAML(yamlStr string) (*Config, error) {
	var cfg Config
	if err := yaml.Unmarshal([]byte(yamlStr), &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse YAML config: %w", err)
	}

	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("configuration validation failed: %w", err)
	}

	return &cfg, nil
}

// LoadFromTOML loads configuration from TOML string
func LoadFromTOML(tomlStr string) (*Config, error) {
	var cfg Config
	if err := toml.Unmarshal([]byte(tomlStr), &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse TOML config: %w", err)
	}

	if err := cfg.Validate(); err != nil {
		return nil, fmt.Errorf("configuration validation failed: %w", err)
	}

	return &cfg, nil
}

// ToJSON converts configuration to JSON string
func (c *Config) ToJSON() (string, error) {
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal config to JSON: %w", err)
	}
	return string(data), nil
}

// ToYAML converts configuration to YAML string
func (c *Config) ToYAML() (string, error) {
	data, err := yaml.Marshal(c)
	if err != nil {
		return "", fmt.Errorf("failed to marshal config to YAML: %w", err)
	}
	return string(data), nil
}

// ToTOML converts configuration to TOML string
func (c *Config) ToTOML() (string, error) {
	data, err := toml.Marshal(c)
	if err != nil {
		return "", fmt.Errorf("failed to marshal config to TOML: %w", err)
	}
	return string(data), nil
}

// LoadFromFile loads configuration from a file with automatic format detection
func LoadFromFile(filePath string) (*Config, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	ext := filepath.Ext(filePath)
	switch strings.ToLower(ext) {
	case ".json":
		return LoadFromJSON(string(data))
	case ".yaml", ".yml":
		return LoadFromYAML(string(data))
	case ".toml":
		return LoadFromTOML(string(data))
	default:
		// Try to detect format from content
		return DetectAndLoadConfig(string(data))
	}
}

// DetectAndLoadConfig automatically detects format and loads configuration
func DetectAndLoadConfig(configStr string) (*Config, error) {
	// Try JSON first
	if json.Valid([]byte(configStr)) {
		return LoadFromJSON(configStr)
	}

	// Try YAML
	var yamlTest interface{}
	if err := yaml.Unmarshal([]byte(configStr), &yamlTest); err == nil {
		return LoadFromYAML(configStr)
	}

	// Try TOML
	var tomlTest interface{}
	if err := toml.Unmarshal([]byte(configStr), &tomlTest); err == nil {
		return LoadFromTOML(configStr)
	}

	return nil, fmt.Errorf("could not detect configuration format")
}

// Helper functions for parsing environment variables

func parseInt(s string) int {
	var val int
	_, err := fmt.Sscanf(s, "%d", &val)
	if err != nil {
		return 0
	}
	return val
}

func parseBool(s string) bool {
	switch strings.ToLower(s) {
	case "true", "1", "yes", "on", "enabled":
		return true
	case "false", "0", "no", "off", "disabled":
		return false
	default:
		return false
	}
}

// CreateConfigFile creates a configuration file with the specified format
func CreateConfigFile(filePath string, cfg *Config) error {
	loader := NewConfigLoader()
	return loader.Save(cfg, filePath)
}

// CreateDefaultConfigFile creates a default configuration file
func CreateDefaultConfigFile(filePath string) error {
	cfg, err := NewConfig()
	if err != nil {
		return err
	}

	return CreateConfigFile(filePath, cfg)
}

// GetConfigTemplates returns configuration templates for different formats
func GetConfigTemplates() map[string]string {
	return map[string]string{
		"yaml": `# Fast-Context Configuration File
# This file contains all configuration options for the Fast-Context analyzer

# Project configuration
projectRoot: "."
languages:
  - "Go"
  - "Rust"
  - "Python"
  - "JavaScript"
  - "TypeScript"
  - "Java"

# File patterns to ignore (supports glob patterns)
ignorePatterns:
  - "node_modules/**"
  - "target/**"
  - "build/**"
  - "dist/**"
  - "vendor/**"
  - "*.min.js"
  - "*.min.css"
  - "__pycache__/**"
  - ".git/**"
  - ".idea/**"
  - ".vscode/**"

# File patterns to include (optional, overrides ignore patterns)
#includePatterns:
#  - "src/**"
#  - "lib/**"

# Performance configuration
performance:
  # Maximum memory usage in MB
  maxMemoryMB: 1024
  
  # Maximum number of files to analyze concurrently
  maxConcurrentFiles: 50
  
  # Analysis timeout in seconds
  timeoutSeconds: 300
  
  # Cache policy: none, minimal, balanced, aggressive, persistent
  cachePolicy: "balanced"
  
  # Enable parallel processing
  enableParallel: true
  
  # Enable streaming analysis for large projects
  enableStreaming: true
  
  # Enable file watching
  enableWatching: false
  
  # Analysis depth (1-10)
  analysisDepth: 3

# Feature flags
enableProgress: true
enableMetrics: true
logLevel: "info"

# File analysis limits
maxFileSizeKB: 1024
maxFiles: 10000

# Configuration preset (optional: fast, balanced, thorough)
# preset: "balanced"
`,
		"json": `{
  "projectRoot": ".",
  "languages": ["Go", "Rust", "Python", "JavaScript", "TypeScript", "Java"],
  "ignorePatterns": [
    "node_modules/**",
    "target/**",
    "build/**",
    "dist/**",
    "vendor/**",
    "*.min.js",
    "*.min.css",
    "__pycache__/**",
    ".git/**"
  ],
  "performance": {
    "maxMemoryMB": 1024,
    "maxConcurrentFiles": 50,
    "timeoutSeconds": 300,
    "cachePolicy": "balanced",
    "enableParallel": true,
    "enableStreaming": true,
    "enableWatching": false,
    "analysisDepth": 3
  },
  "enableProgress": true,
  "enableMetrics": true,
  "logLevel": "info",
  "maxFileSizeKB": 1024,
  "maxFiles": 10000
}`,
		"toml": `# Fast-Context Configuration File

projectRoot = "."
languages = ["Go", "Rust", "Python", "JavaScript", "TypeScript", "Java"]

ignorePatterns = [
  "node_modules/**",
  "target/**", 
  "build/**",
  "dist/**",
  "vendor/**",
  "*.min.js",
  "*.min.css",
  "__pycache__/**",
  ".git/**"
]

[performance]
maxMemoryMB = 1024
maxConcurrentFiles = 50
timeoutSeconds = 300
cachePolicy = "balanced"
enableParallel = true
enableStreaming = true
enableWatching = false
analysisDepth = 3

enableProgress = true
enableMetrics = true
logLevel = "info"
maxFileSizeKB = 1024
maxFiles = 10000
`,
	}
}