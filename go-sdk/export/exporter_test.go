package export

import (
	"testing"

	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestFormatString tests Format string conversion
func TestFormatString(t *testing.T) {
	testCases := []struct {
		format   Format
		expected string
	}{
		{FormatJSON, "json"},
		{FormatYAML, "yaml"},
		{FormatXML, "xml"},
		{FormatGraphML, "graphml"},
		{FormatDOT, "dot"},
		{FormatCSV, "csv"},
		{FormatMarkdown, "markdown"},
		{Format(999), "unknown"},
	}

	for _, tc := range testCases {
		t.Run(tc.expected, func(t *testing.T) {
			result := tc.format.String()
			assert.Equal(t, tc.expected, result)
		})
	}
}

// TestParseFormat tests format parsing
func TestParseFormat(t *testing.T) {
	testCases := []struct {
		input    string
		expected Format
		err      bool
	}{
		{"json", FormatJSON, false},
		{"yaml", FormatYAML, false},
		{"yml", FormatYAML, false},
		{"xml", FormatXML, false},
		{"graphml", FormatGraphML, false},
		{"dot", FormatDOT, false},
		{"csv", FormatCSV, false},
		{"markdown", FormatMarkdown, false},
		{"md", FormatMarkdown, false},
		{"invalid", FormatJSON, true},
	}

	for _, tc := range testCases {
		t.Run(tc.input, func(t *testing.T) {
			result, err := ParseFormat(tc.input)
			if tc.err {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tc.expected, result)
			}
		})
	}
}

// TestDefaultOptions tests default options
func TestDefaultOptions(t *testing.T) {
	opts := DefaultOptions()
	assert.NotNil(t, opts)
	assert.Equal(t, FormatJSON, opts.Format)
	assert.Equal(t, "", opts.OutputFile)
	assert.True(t, opts.Indent)
	assert.True(t, opts.PrettyPrint)
	assert.True(t, opts.IncludeMetrics)
	assert.True(t, opts.IncludeProgress)
	assert.NotNil(t, opts.Filters)
	assert.True(t, opts.Filters.IncludeTests)
	assert.True(t, opts.Filters.IncludePrivate)
}

// TestNewExporter tests exporter creation
func TestNewExporter(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)
	require.NotNil(t, analyzer)

	exporter := NewExporter(analyzer)
	assert.NotNil(t, exporter)
	assert.Equal(t, analyzer, exporter.analyzer)
	assert.NotNil(t, exporter.options)
}

// TestNewExporterWithOptions tests exporter creation with options
func TestNewExporterWithOptions(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)
	require.NotNil(t, analyzer)

	exporter := NewExporter(analyzer,
		WithFormat(FormatYAML),
		WithOutputFile("test.yaml"),
		WithIndent(false),
	)

	assert.NotNil(t, exporter)
	assert.Equal(t, FormatYAML, exporter.options.Format)
	assert.Equal(t, "test.yaml", exporter.options.OutputFile)
	assert.False(t, exporter.options.Indent)
}

// TestExportJSON tests JSON export
func TestExportJSON(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatJSON))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), `"symbols":`)
	assert.Contains(t, string(data), `"dependencies":`)
}

// TestExportYAML tests YAML export
func TestExportYAML(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatYAML))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "symbols:")
	assert.Contains(t, string(data), "dependencies:")
}

// TestExportXML tests XML export
func TestExportXML(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatXML))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "<?xml version=")
	assert.Contains(t, string(data), "<AnalysisResult>")
}

// TestExportGraphML tests GraphML export
func TestExportGraphML(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatGraphML))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "<?xml version=")
	assert.Contains(t, string(data), "<graphml")
	assert.Contains(t, string(data), "<graph id=")
}

// TestExportDOT tests DOT export
func TestExportDOT(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatDOT))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "digraph G {")
	assert.Contains(t, string(data), "node [shape=box];")
}

// TestExportCSV tests CSV export
func TestExportCSV(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatCSV))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "Type,ID,Name,Kind")
	assert.Contains(t, string(data), "Symbol,")
	assert.Contains(t, string(data), "Dependency,")
}

// TestExportMarkdown tests Markdown export
func TestExportMarkdown(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithFormat(FormatMarkdown))
	result := createTestAnalysisResult()

	data, err := exporter.Export(result)
	require.NoError(t, err)
	assert.Contains(t, string(data), "# Fast-Context Analysis Report")
	assert.Contains(t, string(data), "## Summary")
}

// TestFilterOptions tests filtering functionality
func TestFilterOptions(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		SymbolKinds:    []fastcontext.SymbolKind{fastcontext.SymbolKindFunction},
		LanguageFilter: []string{"Go"},
		MinComplexity:  5.0,
		MaxComplexity:  15.0,
		IncludeTests:   false,
		IncludePrivate: false,
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	// Should have fewer symbols after filtering
	assert.Less(t, len(filtered.Symbols), len(result.Symbols))
}

// TestFilterBySymbolKind tests symbol kind filtering
func TestFilterBySymbolKind(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		SymbolKinds: []fastcontext.SymbolKind{fastcontext.SymbolKindFunction},
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, symbol := range filtered.Symbols {
		assert.Equal(t, fastcontext.SymbolKindFunction, symbol.Kind)
	}
}

// TestFilterByLanguage tests language filtering
func TestFilterByLanguage(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		LanguageFilter: []string{"Go"},
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, symbol := range filtered.Symbols {
		assert.Equal(t, "Go", symbol.Language)
	}
}

// TestFilterByComplexity tests complexity filtering
func TestFilterByComplexity(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		MinComplexity: 5.0,
		MaxComplexity: 15.0,
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, symbol := range filtered.Symbols {
		assert.GreaterOrEqual(t, symbol.Complexity, 5.0)
		assert.LessOrEqual(t, symbol.Complexity, 15.0)
	}
}

// TestFilterTests tests test filtering
func TestFilterTests(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		IncludeTests: false,
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, symbol := range filtered.Symbols {
		assert.False(t, symbol.IsTest)
	}
}

// TestFilterPrivate tests private symbol filtering
func TestFilterPrivate(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		IncludePrivate: false,
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, symbol := range filtered.Symbols {
		assert.True(t, symbol.IsPublic)
	}
}

// TestFilterDependencies tests dependency filtering
func TestFilterDependencies(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	filters := &FilterOptions{
		DependencyTypes: []fastcontext.DependencyType{fastcontext.DepTypeCalls},
	}

	exporter := NewExporter(analyzer, WithFilters(filters))
	result := createTestAnalysisResult()

	filtered := exporter.applyFilters(result)

	for _, dep := range filtered.Dependencies {
		assert.Equal(t, fastcontext.DepTypeCalls, dep.Type)
	}
}

// TestExportToFile tests file export
func TestExportToFile(t *testing.T) {
	analyzer, err := fastcontext.NewAnalyzer()
	require.NoError(t, err)

	exporter := NewExporter(analyzer, WithOutputFile("/tmp/test_export.json"))
	result := createTestAnalysisResult()

	err = exporter.ExportToFile(result)
	require.NoError(t, err)
}

// Helper function to create test analysis result
func createTestAnalysisResult() *fastcontext.AnalysisResult {
	return &fastcontext.AnalysisResult{
		FileCount:         5,
		SymbolCount:       20,
		RelationshipCount: 15,
		Symbols: []*fastcontext.Symbol{
			{
				ID:          "func1",
				Name:        "TestFunction",
				Kind:        fastcontext.SymbolKindFunction,
				Language:    "Go",
				File:        "test.go",
				LineStart:   10,
				LineEnd:     20,
				Complexity:  5.0,
				IsPublic:    true,
				IsExported:  true,
				IsTest:      false,
			},
			{
				ID:          "struct1",
				Name:        "TestStruct",
				Kind:        fastcontext.SymbolKindStruct,
				Language:    "Go",
				File:        "test.go",
				LineStart:   25,
				LineEnd:     30,
				Complexity:  2.0,
				IsPublic:    true,
				IsExported:  true,
				IsTest:      false,
			},
			{
				ID:          "test_func",
				Name:        "TestUnitTest",
				Kind:        fastcontext.SymbolKindFunction,
				Language:    "Go",
				File:        "test_test.go",
				LineStart:   5,
				LineEnd:     15,
				Complexity:  1.0,
				IsPublic:    false,
				IsExported:  false,
				IsTest:      true,
			},
		},
		Dependencies: []*fastcontext.Dependency{
			{
				From:     "func1",
				To:       "struct1",
				Type:     fastcontext.DepTypeCalls,
				Strength: 0.8,
			},
		},
		Languages:  []string{"Go"},
		DurationMs: 1000,
		MemoryUsed: 1024 * 1024,
	}
}