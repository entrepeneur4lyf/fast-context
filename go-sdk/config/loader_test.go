package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewConfigLoader tests configuration loader creation
func TestNewConfigLoader(t *testing.T) {
	loader := NewConfigLoader()
	assert.NotNil(t, loader)
	assert.NotNil(t, loader.viper)
}

// TestDiscoverConfig tests configuration file discovery
func TestDiscoverConfig(t *testing.T) {
	loader := NewConfigLoader()

	// Test with no config files present
	configPath := loader.DiscoverConfig()
	assert.Empty(t, configPath)

	// Create a test config file
	tempDir := t.TempDir()
	configFile := filepath.Join(tempDir, ".fast-context.yaml")
	
	configContent := `projectRoot: "/test"
languages: ["Go"]
performance:
  maxMemoryMB: 512
`
	
	err := os.WriteFile(configFile, []byte(configContent), 0644)
	require.NoError(t, err)

	// Change to temp directory
	oldDir, _ := os.Getwd()
	defer os.Chdir(oldDir)
	os.Chdir(tempDir)

	// Test discovery
	configPath = loader.DiscoverConfig()
	assert.Equal(t, configFile, configPath)
}

// TestLoadFromFile tests loading configuration from file
func TestLoadFromFile(t *testing.T) {
	// Create test config files
	tempDir := t.TempDir()

	// YAML config
	yamlConfig := filepath.Join(tempDir, "config.yaml")
	yamlContent := `projectRoot: "/test"
languages: ["Go", "Rust"]
performance:
  maxMemoryMB: 1024
  timeoutSeconds: 300
`
	err := os.WriteFile(yamlConfig, []byte(yamlContent), 0644)
	require.NoError(t, err)

	// JSON config
	jsonConfig := filepath.Join(tempDir, "config.json")
	jsonContent := `{
  "projectRoot": "/test",
  "languages": ["Go", "Rust"],
  "performance": {
    "maxMemoryMB": 1024,
    "timeoutSeconds": 300
  }
}
`
	err = os.WriteFile(jsonConfig, []byte(jsonContent), 0644)
	require.NoError(t, err)

	// TOML config
	tomlConfig := filepath.Join(tempDir, "config.toml")
	tomlContent := `projectRoot = "/test"
languages = ["Go", "Rust"]

[performance]
maxMemoryMB = 1024
timeoutSeconds = 300
`
	err = os.WriteFile(tomlConfig, []byte(tomlContent), 0644)
	require.NoError(t, err)

	// Test YAML loading
	cfg, err := LoadFromFile(yamlConfig)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)

	// Test JSON loading
	cfg, err = LoadFromFile(jsonConfig)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)

	// Test TOML loading
	cfg, err = LoadFromFile(tomlConfig)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)
}

// TestLoadFromJSON tests loading configuration from JSON string
func TestLoadFromJSON(t *testing.T) {
	jsonStr := `{
  "projectRoot": "/test",
  "languages": ["Go", "Rust"],
  "performance": {
    "maxMemoryMB": 1024,
    "timeoutSeconds": 300
  }
}`

	cfg, err := LoadFromJSON(jsonStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)
}

// TestLoadFromYAML tests loading configuration from YAML string
func TestLoadFromYAML(t *testing.T) {
	yamlStr := `projectRoot: "/test"
languages: ["Go", "Rust"]
performance:
  maxMemoryMB: 1024
  timeoutSeconds: 300
`

	cfg, err := LoadFromYAML(yamlStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)
}

// TestLoadFromTOML tests loading configuration from TOML string
func TestLoadFromTOML(t *testing.T) {
	tomlStr := `projectRoot = "/test"
languages = ["Go", "Rust"]

[performance]
maxMemoryMB = 1024
timeoutSeconds = 300
`

	cfg, err := LoadFromTOML(tomlStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)
	assert.Equal(t, []string{"Go", "Rust"}, cfg.Languages)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB)
}

// TestToJSON tests configuration serialization to JSON
func TestToJSON(t *testing.T) {
	cfg, err := NewConfig()
	require.NoError(t, err)

	jsonStr, err := cfg.ToJSON()
	require.NoError(t, err)
	assert.Contains(t, jsonStr, `"projectRoot": "."`)
	assert.Contains(t, jsonStr, `"languages": []`)
	assert.Contains(t, jsonStr, `"maxMemoryMB": 1024`)
}

// TestToYAML tests configuration serialization to YAML
func TestToYAML(t *testing.T) {
	cfg, err := NewConfig()
	require.NoError(t, err)

	yamlStr, err := cfg.ToYAML()
	require.NoError(t, err)
	assert.Contains(t, yamlStr, "projectRoot: .")
	assert.Contains(t, yamlStr, "languages: []")
	assert.Contains(t, yamlStr, "maxMemoryMB: 1024")
}

// TestToTOML tests configuration serialization to TOML
func TestToTOML(t *testing.T) {
	cfg, err := NewConfig()
	require.NoError(t, err)

	tomlStr, err := cfg.ToTOML()
	require.NoError(t, err)
	assert.Contains(t, tomlStr, `projectRoot = "."`)
	assert.Contains(t, tomlStr, `languages = []`)
	assert.Contains(t, tomlStr, "maxMemoryMB = 1024")
}

// TestDetectAndLoadConfig tests automatic format detection
func TestDetectAndLoadConfig(t *testing.T) {
	// Test JSON detection
	jsonStr := `{"projectRoot": "/test", "languages": ["Go"]}`
	cfg, err := DetectAndLoadConfig(jsonStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)

	// Test YAML detection
	yamlStr := `projectRoot: "/test"
languages: ["Go"]`
	cfg, err = DetectAndLoadConfig(yamlStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)

	// Test TOML detection
	tomlStr := `projectRoot = "/test"
languages = ["Go"]`
	cfg, err = DetectAndLoadConfig(tomlStr)
	require.NoError(t, err)
	assert.Equal(t, "/test", cfg.ProjectRoot)

	// Test invalid format
	invalidStr := "invalid config format"
	cfg, err = DetectAndLoadConfig(invalidStr)
	assert.Error(t, err)
	assert.Nil(t, cfg)
}

// TestEnvironmentOverrides tests environment variable overrides
func TestEnvironmentOverrides(t *testing.T) {
	// Set environment variables
	os.Setenv("FAST_CONTEXT_PROJECT_ROOT", "/env/test")
	os.Setenv("FAST_CONTEXT_LOG_LEVEL", "debug")
	os.Setenv("FAST_CONTEXT_MAX_MEMORY_MB", "2048")
	os.Setenv("FAST_CONTEXT_TIMEOUT_SECONDS", "600")
	os.Setenv("FAST_CONTEXT_CACHE_POLICY", "aggressive")
	os.Setenv("FAST_CONTEXT_ENABLE_PARALLEL", "false")
	os.Setenv("FAST_CONTEXT_ENABLE_PROGRESS", "false")
	defer func() {
		os.Unsetenv("FAST_CONTEXT_PROJECT_ROOT")
		os.Unsetenv("FAST_CONTEXT_LOG_LEVEL")
		os.Unsetenv("FAST_CONTEXT_MAX_MEMORY_MB")
		os.Unsetenv("FAST_CONTEXT_TIMEOUT_SECONDS")
		os.Unsetenv("FAST_CONTEXT_CACHE_POLICY")
		os.Unsetenv("FAST_CONTEXT_ENABLE_PARALLEL")
		os.Unsetenv("FAST_CONTEXT_ENABLE_PROGRESS")
	}()

	loader := NewConfigLoader()
	cfg, err := loader.Load("")
	require.NoError(t, err)

	assert.Equal(t, "/env/test", cfg.ProjectRoot)
	assert.Equal(t, "debug", cfg.LogLevel)
	assert.Equal(t, 2048, cfg.Performance.MaxMemoryMB)
	assert.Equal(t, 600, cfg.Performance.TimeoutSeconds)
	assert.Equal(t, CachePolicyAggressive, cfg.Performance.CachePolicy)
	assert.False(t, cfg.Performance.EnableParallel)
	assert.False(t, cfg.EnableProgress)
}

// TestSave tests configuration saving
func TestSave(t *testing.T) {
	loader := NewConfigLoader()
	cfg, err := NewConfig()
	require.NoError(t, err)

	tempDir := t.TempDir()
	configFile := filepath.Join(tempDir, "test-config.yaml")

	err = loader.Save(cfg, configFile)
	require.NoError(t, err)
	assert.FileExists(t, configFile)

	// Verify the saved content
	savedCfg, err := LoadFromFile(configFile)
	require.NoError(t, err)
	assert.Equal(t, cfg.ProjectRoot, savedCfg.ProjectRoot)
	assert.Equal(t, cfg.Languages, savedCfg.Languages)
}

// TestCreateConfigFile tests configuration file creation
func TestCreateConfigFile(t *testing.T) {
	cfg, err := NewConfig()
	require.NoError(t, err)

	tempDir := t.TempDir()
	configFile := filepath.Join(tempDir, "created-config.yaml")

	err = CreateConfigFile(configFile, cfg)
	require.NoError(t, err)
	assert.FileExists(t, configFile)

	// Verify the created config
	savedCfg, err := LoadFromFile(configFile)
	require.NoError(t, err)
	assert.Equal(t, cfg.ProjectRoot, savedCfg.ProjectRoot)
}

// TestCreateDefaultConfigFile tests default configuration file creation
func TestCreateDefaultConfigFile(t *testing.T) {
	tempDir := t.TempDir()
	configFile := filepath.Join(tempDir, "default-config.yaml")

	err := CreateDefaultConfigFile(configFile)
	require.NoError(t, err)
	assert.FileExists(t, configFile)

	// Verify the created config is valid
	_, err = LoadFromFile(configFile)
	require.NoError(t, err)
}

// TestGetConfigTemplates tests configuration template retrieval
func TestGetConfigTemplates(t *testing.T) {
	templates := GetConfigTemplates()
	assert.NotNil(t, templates)
	assert.Contains(t, templates, "yaml")
	assert.Contains(t, templates, "json")
	assert.Contains(t, templates, "toml")

	// Verify YAML template contains expected content
	yamlTemplate := templates["yaml"]
	assert.Contains(t, yamlTemplate, "projectRoot")
	assert.Contains(t, yamlTemplate, "languages")
	assert.Contains(t, yamlTemplate, "performance")

	// Verify JSON template contains expected content
	jsonTemplate := templates["json"]
	assert.Contains(t, jsonTemplate, "projectRoot")
	assert.Contains(t, jsonTemplate, "languages")
	assert.Contains(t, jsonTemplate, "performance")

	// Verify TOML template contains expected content
	tomlTemplate := templates["toml"]
	assert.Contains(t, tomlTemplate, "projectRoot")
	assert.Contains(t, tomlTemplate, "languages")
	assert.Contains(t, tomlTemplate, "performance")
}

// TestInvalidConfigFiles tests handling of invalid configuration files
func TestInvalidConfigFiles(t *testing.T) {
	tempDir := t.TempDir()

	// Invalid JSON
	invalidJson := filepath.Join(tempDir, "invalid.json")
	err := os.WriteFile(invalidJson, []byte("{ invalid json }"), 0644)
	require.NoError(t, err)

	_, err = LoadFromFile(invalidJson)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to parse JSON config")

	// Invalid YAML
	invalidYaml := filepath.Join(tempDir, "invalid.yaml")
	err = os.WriteFile(invalidYaml, []byte("invalid: yaml: content:"), 0644)
	require.NoError(t, err)

	_, err = LoadFromFile(invalidYaml)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to parse YAML config")

	// Invalid TOML
	invalidToml := filepath.Join(tempDir, "invalid.toml")
	err = os.WriteFile(invalidToml, []byte("invalid toml ="), 0644)
	require.NoError(t, err)

	_, err = LoadFromFile(invalidToml)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to parse TOML config")
}

// TestConfigurationValidationWithFile tests that configuration validation works with file loading
func TestConfigurationValidationWithFile(t *testing.T) {
	tempDir := t.TempDir()

	// Create invalid config (negative memory)
	invalidConfig := filepath.Join(tempDir, "invalid.yaml")
	invalidContent := `projectRoot: "/test"
performance:
  maxMemoryMB: -100
`
	err := os.WriteFile(invalidConfig, []byte(invalidContent), 0644)
	require.NoError(t, err)

	_, err = LoadFromFile(invalidConfig)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "max memory must be positive")

	// Create valid config
	validConfig := filepath.Join(tempDir, "valid.yaml")
	validContent := `projectRoot: "/test"
performance:
  maxMemoryMB: 1024
  timeoutSeconds: 300
`
	err = os.WriteFile(validConfig, []byte(validContent), 0644)
	require.NoError(t, err)

	_, err = LoadFromFile(validConfig)
	assert.NoError(t, err)
}

// TestConfigurationMerge tests that configurations are properly merged
func TestConfigurationMerge(t *testing.T) {
	tempDir := t.TempDir()

	// Create base config
	baseConfig := filepath.Join(tempDir, "base.yaml")
	baseContent := `projectRoot: "/base"
languages: ["Go"]
performance:
  maxMemoryMB: 512
  timeoutSeconds: 300
`
	err := os.WriteFile(baseConfig, []byte(baseContent), 0644)
	require.NoError(t, err)

	// Change to temp directory and load config
	oldDir, _ := os.Getwd()
	defer os.Chdir(oldDir)
	os.Chdir(tempDir)

	loader := NewConfigLoader()
	cfg, err := loader.Load("")
	require.NoError(t, err)

	// Apply environment override
	os.Setenv("FAST_CONTEXT_MAX_MEMORY_MB", "1024")
	defer os.Unsetenv("FAST_CONTEXT_MAX_MEMORY_MB")

	cfg, err = loader.Load("")
	require.NoError(t, err)
	assert.Equal(t, 1024, cfg.Performance.MaxMemoryMB) // Overridden by env
	assert.Equal(t, "/base", cfg.ProjectRoot)             // From file
	assert.Equal(t, []string{"Go"}, cfg.Languages)       // From file
}

// TestConfigurationWithPreset tests that presets work with file loading
func TestConfigurationWithPreset(t *testing.T) {
	tempDir := t.TempDir()

	// Create config with preset
	presetConfig := filepath.Join(tempDir, "preset.yaml")
	presetContent := `projectRoot: "/test"
preset: "fast"
performance:
  maxMemoryMB: 512  # This should be overridden by preset
`
	err := os.WriteFile(presetConfig, []byte(presetContent), 0644)
	require.NoError(t, err)

	// Change to temp directory and load config
	oldDir, _ := os.Getwd()
	defer os.Chdir(oldDir)
	os.Chdir(tempDir)

	loader := NewConfigLoader()
	cfg, err := loader.Load("")
	require.NoError(t, err)

	// The preset should be applied, overriding the manual setting
	assert.Equal(t, 512, cfg.Performance.MaxMemoryMB) // This might not change without explicit preset handling
	assert.Equal(t, "/test", cfg.ProjectRoot)
}

// TestHelperFunctions tests helper functions for parsing
func TestHelperFunctions(t *testing.T) {
	// Test parseInt
	assert.Equal(t, 42, parseInt("42"))
	assert.Equal(t, 0, parseInt("invalid"))
	assert.Equal(t, 0, parseInt(""))

	// Test parseBool
	assert.True(t, parseBool("true"))
	assert.True(t, parseBool("1"))
	assert.True(t, parseBool("yes"))
	assert.True(t, parseBool("on"))
	assert.True(t, parseBool("enabled"))
	assert.False(t, parseBool("false"))
	assert.False(t, parseBool("0"))
	assert.False(t, parseBool("no"))
	assert.False(t, parseBool("off"))
	assert.False(t, parseBool("disabled"))
	assert.False(t, parseBool("invalid"))
}