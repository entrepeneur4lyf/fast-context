package fastcontext

import (
	"testing"

	"github.com/fast-context/go-sdk/config"
)

func TestNewAnalyzer(t *testing.T) {
	// Test with minimal configuration
	analyzer, err := NewAnalyzer(
		config.WithProjectRoot("."),
	)
	if err != nil {
		t.Fatalf("Failed to create analyzer: %v", err)
	}

	if analyzer == nil {
		t.Fatal("Analyzer should not be nil")
	}

	if analyzer.GetConfig() == nil {
		t.Fatal("Analyzer config should not be nil")
	}
}

func TestNewAnalyzerWithConfig(t *testing.T) {
	cfg, err := config.NewConfig(
		config.WithProjectRoot("."),
	)
	if err != nil {
		t.Fatalf("Failed to create config: %v", err)
	}

	analyzer, err := NewAnalyzerWithConfig(cfg)
	if err != nil {
		t.Fatalf("Failed to create analyzer with config: %v", err)
	}

	if analyzer == nil {
		t.Fatal("Analyzer should not be nil")
	}
}

func TestGetSupportedLanguages(t *testing.T) {
	languages := GetSupportedLanguages()

	if len(languages) == 0 {
		t.Fatal("Should return supported languages")
	}

	// Check for some common languages
	found := false
	for _, lang := range languages {
		if lang == "Go" || lang == "Rust" || lang == "Python" {
			found = true
			break
		}
	}

	if !found {
		t.Fatal("Should include common languages like Go, Rust, or Python")
	}
}

func TestErrorCodes(t *testing.T) {
	// Test that predefined errors are properly created
	err := ErrInvalidProjectRoot
	if err == nil {
		t.Fatal("Predefined error should not be nil")
	}

	if err.Code != ErrProjectNotFound {
		t.Fatalf("Expected error code %d, got %d", ErrProjectNotFound, err.Code)
	}
}

func TestConfigOptions(t *testing.T) {
	// Test various configuration options
	analyzer, err := NewAnalyzer(
		config.WithProjectRoot("."),
		config.WithMaxMemory(512),
		config.WithTimeout(120),
		config.WithParallelProcessing(true),
		config.WithProgress(false),
		config.WithLogLevel("debug"),
	)
	if err != nil {
		t.Fatalf("Failed to create analyzer with options: %v", err)
	}

	cfg := analyzer.GetConfig()
	if cfg.Performance.MaxMemoryMB != 512 {
		t.Fatalf("Expected max memory 512MB, got %d", cfg.Performance.MaxMemoryMB)
	}

	if cfg.Performance.TimeoutSeconds != 120 {
		t.Fatalf("Expected timeout 120s, got %d", cfg.Performance.TimeoutSeconds)
	}

	if cfg.Performance.EnableParallel != true {
		t.Fatal("Expected parallel processing to be enabled")
	}

	if cfg.EnableProgress != false {
		t.Fatal("Expected progress to be disabled")
	}

	if cfg.LogLevel != "debug" {
		t.Fatalf("Expected log level 'debug', got '%s'", cfg.LogLevel)
	}
}

func TestPresetConfigurations(t *testing.T) {
	// Test fast configuration
	fastCfg, err := config.FastConfig(".")
	if err != nil {
		t.Fatalf("Failed to create fast config: %v", err)
	}

	if fastCfg.Performance.CachePolicy != config.CachePolicyMinimal {
		t.Fatalf("Fast config should use minimal cache policy")
	}

	// Test balanced configuration
	balancedCfg, err := config.BalancedConfig(".")
	if err != nil {
		t.Fatalf("Failed to create balanced config: %v", err)
	}

	if balancedCfg.Performance.CachePolicy != config.CachePolicyBalanced {
		t.Fatalf("Balanced config should use balanced cache policy")
	}

	// Test thorough configuration
	thoroughCfg, err := config.ThoroughConfig(".")
	if err != nil {
		t.Fatalf("Failed to create thorough config: %v", err)
	}

	if thoroughCfg.Performance.CachePolicy != config.CachePolicyAggressive {
		t.Fatalf("Thorough config should use aggressive cache policy")
	}
}