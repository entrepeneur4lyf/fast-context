package unit

import (
	"testing"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestErrorHandling tests error handling functionality
func TestErrorHandling(t *testing.T) {
	t.Run("FastContextError", func(t *testing.T) {
		err := fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "test error")
		require.NotNil(t, err)
		assert.Equal(t, fastcontext.ErrInvalidInput, err.Code)
		assert.Equal(t, "test error", err.Message)
		assert.Contains(t, err.Error(), "FastContextError")

		errWithCause := fastcontext.NewFastContextErrorWithCause(fastcontext.ErrAnalysisFailed, "analysis failed", err)
		require.NotNil(t, errWithCause)
		assert.Equal(t, fastcontext.ErrAnalysisFailed, errWithCause.Code)
		assert.Equal(t, err, errWithCause.Unwrap())
	})

	t.Run("ErrorIs", func(t *testing.T) {
		err1 := fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "input error")
		err2 := fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "another input error")

		assert.True(t, err1.Is(err2))
		assert.False(t, err1.Is(fastcontext.NewFastContextError(fastcontext.ErrAnalysisFailed, "analysis error")))
	})
}

// TestSymbolKindString tests SymbolKind string conversion
func TestSymbolKindString(t *testing.T) {
	testCases := []struct {
		kind    fastcontext.SymbolKind
		expect string
	}{
		{fastcontext.SymbolKindFunction, "function"},
		{fastcontext.SymbolKindMethod, "method"},
		{fastcontext.SymbolKindClass, "class"},
		{fastcontext.SymbolKindInterface, "interface"},
		{fastcontext.SymbolKindStruct, "struct"},
		{fastcontext.SymbolKindEnum, "enum"},
		{fastcontext.SymbolKindVariable, "variable"},
		{fastcontext.SymbolKindConstant, "constant"},
		{fastcontext.SymbolKindParameter, "parameter"},
		{fastcontext.SymbolKindModule, "module"},
		{fastcontext.SymbolKindPackage, "package"},
		{fastcontext.SymbolKindType, "type"},
		{fastcontext.SymbolKindField, "field"},
		{fastcontext.SymbolKindProperty, "property"},
		{fastcontext.SymbolKindConstructor, "constructor"},
		{fastcontext.SymbolKindDestructor, "destructor"},
		{fastcontext.SymbolKindOperator, "operator"},
		{fastcontext.SymbolKindMacro, "macro"},
		{fastcontext.SymbolKindAnnotation, "annotation"},
		{fastcontext.SymbolKindUnknown, ""},
		{fastcontext.SymbolKind(999), ""}, // Invalid kind
	}

	for _, tc := range testCases {
		t.Run(tc.expect, func(t *testing.T) {
			analyzer := &fastcontext.Analyzer{}
			result := analyzer.SymbolKindToString(tc.kind)
			assert.Equal(t, tc.expect, result)
		})
	}
}

// TestConfiguration tests configuration functionality
func TestConfiguration(t *testing.T) {
	t.Run("DefaultConfig", func(t *testing.T) {
		cfg, err := config.NewConfig()
		require.NoError(t, err)
		require.NotNil(t, cfg)
		assert.Equal(t, ".", cfg.ProjectRoot)
		assert.NotEmpty(t, cfg.Languages)
		assert.Greater(t, cfg.Performance.TimeoutSeconds, 0)
	})

	t.Run("ConfigWithOptions", func(t *testing.T) {
		cfg, err := config.NewConfig(
			config.WithProjectRoot("/test/project"),
			config.WithLanguages([]string{"Go", "Rust"}),
			config.WithTimeoutSeconds(60),
		)
		require.NoError(t, err)
		require.NotNil(t, cfg)
		assert.Equal(t, "/test/project", cfg.ProjectRoot)
		assert.Contains(t, cfg.Languages, "Go")
		assert.Contains(t, cfg.Languages, "Rust")
		assert.Equal(t, 60, cfg.Performance.TimeoutSeconds)
	})

	t.Run("PresetConfigs", func(t *testing.T) {
		fastCfg, err := config.NewConfig(config.WithPreset(config.PresetFast))
		require.NoError(t, err)
		require.NotNil(t, fastCfg)
		assert.Equal(t, config.PresetFast, fastCfg.Preset)

		balancedCfg, err := config.NewConfig(config.WithPreset(config.PresetBalanced))
		require.NoError(t, err)
		require.NotNil(t, balancedCfg)
		assert.Equal(t, config.PresetBalanced, balancedCfg.Preset)

		thoroughCfg, err := config.NewConfig(config.WithPreset(config.PresetThorough))
		require.NoError(t, err)
		require.NotNil(t, thoroughCfg)
		assert.Equal(t, config.PresetThorough, thoroughCfg.Preset)
	})

	t.Run("ConfigValidation", func(t *testing.T) {
		// Valid config
		cfg, err := config.NewConfig(config.WithProjectRoot("/valid"))
		require.NoError(t, err)
		err = cfg.Validate()
		assert.NoError(t, err)

		// Invalid config (empty project root)
		invalidCfg, err := config.NewConfig(config.WithProjectRoot(""))
		require.NoError(t, err)
		err = invalidCfg.Validate()
		assert.Error(t, err)
	})
}

// TestAnalyzerBasic tests basic analyzer functionality
func TestAnalyzerBasic(t *testing.T) {
	t.Run("NewAnalyzer", func(t *testing.T) {
		analyzer, err := fastcontext.NewAnalyzer()
		require.NoError(t, err)
		require.NotNil(t, analyzer)
		assert.NotNil(t, analyzer.GetConfig())
		assert.NotNil(t, analyzer.GetVersion())
	})

	t.Run("NewAnalyzerWithConfig", func(t *testing.T) {
		cfg, err := config.NewConfig(config.WithProjectRoot("/test"))
		require.NoError(t, err)
		analyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)
		require.NoError(t, err)
		require.NotNil(t, analyzer)
		assert.Equal(t, cfg, analyzer.GetConfig())
	})

	t.Run("UpdateConfig", func(t *testing.T) {
		analyzer, err := fastcontext.NewAnalyzer()
		require.NoError(t, err)

		err = analyzer.UpdateConfig(config.WithProjectRoot("/new/project"))
		require.NoError(t, err)
		assert.Equal(t, "/new/project", analyzer.GetConfig().ProjectRoot)
	})

	t.Run("GetSupportedLanguages", func(t *testing.T) {
		languages := fastcontext.GetSupportedLanguages()
		assert.NotEmpty(t, languages)
		assert.Contains(t, languages, "Go")
		assert.Contains(t, languages, "Rust")
		assert.Contains(t, languages, "Python")
		assert.Contains(t, languages, "JavaScript")
	})
}

// TestAnalysisPhase tests analysis phase functionality
func TestAnalysisPhase(t *testing.T) {
	phases := []fastcontext.AnalysisPhase{
		fastcontext.PhaseDiscovery,
		fastcontext.PhaseParsing,
		fastcontext.PhaseSymbolExtraction,
		fastcontext.PhaseDependencyAnalysis,
		fastcontext.PhaseGraphConstruction,
		fastcontext.PhaseOptimization,
		fastcontext.PhaseComplete,
	}

	for _, phase := range phases {
		assert.NotEqual(t, 0, int(phase))
		assert.Greater(t, int(phase), 0)
		assert.Less(t, int(phase), 100) // Reasonable upper bound
	}
}

// TestDependencyType tests dependency type functionality
func TestDependencyType(t *testing.T) {
	types := []fastcontext.DependencyType{
		fastcontext.DepTypeUnknown,
		fastcontext.DepTypeImports,
		fastcontext.DepTypeCalls,
		fastcontext.DepTypeInherits,
		fastcontext.DepTypeImplements,
		fastcontext.DepTypeReferences,
		fastcontext.DepTypeInstantiates,
		fastcontext.DepTypeContains,
		fastcontext.DepTypeOverrides,
		fastcontext.DepTypeDecorates,
	}

	for _, depType := range types {
		assert.NotEqual(t, 0, int(depType))
		assert.Greater(t, int(depType), 0)
		assert.Less(t, int(depType), 100) // Reasonable upper bound
	}
}

// TestFileChangeEvent tests file change event functionality
func TestFileChangeEvent(t *testing.T) {
	event := &fastcontext.FileChangeEvent{
		Type:      "created",
		Path:      "/test/file.go",
		Timestamp: 1234567890,
		Size:      1024,
	}

	assert.Equal(t, "created", event.Type)
	assert.Equal(t, "/test/file.go", event.Path)
	assert.Equal(t, int64(1234567890), event.Timestamp)
	assert.Equal(t, int64(1024), event.Size)
}

// TestPerformanceMetrics tests performance metrics functionality
func TestPerformanceMetrics(t *testing.T) {
	metrics := &fastcontext.PerformanceMetrics{
		TotalTimeMs:     1000,
		ParsingTimeMs:   500,
		AnalysisTimeMs:  400,
		MemoryPeakBytes: 1024 * 1024,
		CacheHitRate:    0.85,
		FilesPerSecond:  10.5,
		SymbolsPerFile:  25.0,
	}

	assert.Equal(t, int64(1000), metrics.TotalTimeMs)
	assert.Equal(t, int64(500), metrics.ParsingTimeMs)
	assert.Equal(t, int64(400), metrics.AnalysisTimeMs)
	assert.Equal(t, int64(1024*1024), metrics.MemoryPeakBytes)
	assert.Equal(t, 0.85, metrics.CacheHitRate)
	assert.Equal(t, 10.5, metrics.FilesPerSecond)
	assert.Equal(t, 25.0, metrics.SymbolsPerFile)
}

// TestPredefinedErrors tests predefined error constants
func TestPredefinedErrors(t *testing.T) {
	require.NotNil(t, fastcontext.ErrInvalidProjectRoot)
	require.NotNil(t, fastcontext.ErrInvalidConfig)
	require.NotNil(t, fastcontext.ErrAnalysisTimeout)
	require.NotNil(t, fastcontext.ErrAnalysisCancelled)
	require.NotNil(t, fastcontext.ErrMemoryLimitExceeded)

	assert.Equal(t, fastcontext.ErrProjectNotFound, fastcontext.ErrInvalidProjectRoot.Code)
	assert.Equal(t, fastcontext.ErrInvalidConfiguration, fastcontext.ErrInvalidConfig.Code)
	assert.Equal(t, fastcontext.ErrTimeout, fastcontext.ErrAnalysisTimeout.Code)
	assert.Equal(t, fastcontext.ErrCancelled, fastcontext.ErrAnalysisCancelled.Code)
	assert.Equal(t, fastcontext.ErrOutOfMemory, fastcontext.ErrMemoryLimitExceeded.Code)
}