package export

import (
	"encoding/csv"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
	"gopkg.in/yaml.v3"
)

// Format represents the supported export formats
type Format int

const (
	FormatJSON Format = iota
	FormatYAML
	FormatXML
	FormatGraphML
	FormatDOT
	FormatCSV
	FormatMarkdown
)

func (f Format) String() string {
	switch f {
	case FormatJSON:
		return "json"
	case FormatYAML:
		return "yaml"
	case FormatXML:
		return "xml"
	case FormatGraphML:
		return "graphml"
	case FormatDOT:
		return "dot"
	case FormatCSV:
		return "csv"
	case FormatMarkdown:
		return "markdown"
	default:
		return "unknown"
	}
}

// ParseFormat converts a string to Format
func ParseFormat(format string) (Format, error) {
	switch strings.ToLower(format) {
	case "json":
		return FormatJSON, nil
	case "yaml", "yml":
		return FormatYAML, nil
	case "xml":
		return FormatXML, nil
	case "graphml":
		return FormatGraphML, nil
	case "dot":
		return FormatDOT, nil
	case "csv":
		return FormatCSV, nil
	case "markdown", "md":
		return FormatMarkdown, nil
	default:
		return FormatJSON, fmt.Errorf("unsupported format: %s", format)
	}
}

// FilterOptions defines filtering options for exports
type FilterOptions struct {
	SymbolKinds    []fastcontext.SymbolKind `json:"symbolKinds"`
	DependencyTypes []fastcontext.DependencyType `json:"dependencyTypes"`
	LanguageFilter []string               `json:"languageFilter"`
	FilePattern    string                 `json:"filePattern"`
	MinComplexity  float64                `json:"minComplexity"`
	MaxComplexity  float64                `json:"maxComplexity"`
	IncludeTests   bool                   `json:"includeTests"`
	IncludePrivate bool                   `json:"includePrivate"`
}

// Options defines export options
type Options struct {
	Format         Format         `json:"format"`
	OutputFile     string         `json:"outputFile"`
	Indent         bool           `json:"indent"`
	PrettyPrint    bool           `json:"prettyPrint"`
	IncludeMetrics bool           `json:"includeMetrics"`
	IncludeProgress bool          `json:"includeProgress"`
	Filters        *FilterOptions `json:"filters"`
}

// DefaultOptions returns default export options
func DefaultOptions() *Options {
	return &Options{
		Format:         FormatJSON,
		OutputFile:     "",
		Indent:         true,
		PrettyPrint:    true,
		IncludeMetrics: true,
		IncludeProgress: true,
		Filters: &FilterOptions{
			IncludeTests:   true,
			IncludePrivate: true,
		},
	}
}

// Exporter handles exporting analysis results to various formats
type Exporter struct {
	analyzer *fastcontext.Analyzer
	options  *Options
}

// NewExporter creates a new exporter
func NewExporter(analyzer *fastcontext.Analyzer, opts ...func(*Options)) *Exporter {
	options := DefaultOptions()
	for _, opt := range opts {
		opt(options)
	}

	return &Exporter{
		analyzer: analyzer,
		options:  options,
	}
}

// Export exports the analysis result to the specified format
func (e *Exporter) Export(result *fastcontext.AnalysisResult) ([]byte, error) {
	// Apply filters if specified
	if e.options.Filters != nil {
		result = e.applyFilters(result)
	}

	switch e.options.Format {
	case FormatJSON:
		return e.exportJSON(result)
	case FormatYAML:
		return e.exportYAML(result)
	case FormatXML:
		return e.exportXML(result)
	case FormatGraphML:
		return e.exportGraphML(result)
	case FormatDOT:
		return e.exportDOT(result)
	case FormatCSV:
		return e.exportCSV(result)
	case FormatMarkdown:
		return e.exportMarkdown(result)
	default:
		return nil, fmt.Errorf("unsupported export format: %s", e.options.Format)
	}
}

// ExportToFile exports the analysis result to a file
func (e *Exporter) ExportToFile(result *fastcontext.AnalysisResult) error {
	data, err := e.Export(result)
	if err != nil {
		return err
	}

	if e.options.OutputFile == "" {
		// Default to stdout
		_, err = os.Stdout.Write(data)
		return err
	}

	return os.WriteFile(e.options.OutputFile, data, 0644)
}

// applyFilters applies the specified filters to the analysis result
func (e *Exporter) applyFilters(result *fastcontext.AnalysisResult) *fastcontext.AnalysisResult {
	filtered := &fastcontext.AnalysisResult{
		FileCount:         result.FileCount,
		SymbolCount:       result.SymbolCount,
		RelationshipCount: result.RelationshipCount,
		Languages:         result.Languages,
		DurationMs:        result.DurationMs,
		MemoryUsed:        result.MemoryUsed,
		Metadata:          result.Metadata,
	}

	// Filter symbols
	if len(e.options.Filters.SymbolKinds) > 0 || len(e.options.Filters.LanguageFilter) > 0 ||
		e.options.Filters.MinComplexity > 0 || e.options.Filters.MaxComplexity > 0 ||
		!e.options.Filters.IncludeTests || !e.options.Filters.IncludePrivate {
		filtered.Symbols = e.filterSymbols(result.Symbols)
	} else {
		filtered.Symbols = result.Symbols
	}

	// Filter dependencies
	if len(e.options.Filters.DependencyTypes) > 0 {
		filtered.Dependencies = e.filterDependencies(result.Dependencies)
	} else {
		filtered.Dependencies = result.Dependencies
	}

	// Recalculate counts
	filtered.SymbolCount = len(filtered.Symbols)
	filtered.RelationshipCount = len(filtered.Dependencies)

	return filtered
}

// filterSymbols filters symbols based on the filter options
func (e *Exporter) filterSymbols(symbols []*fastcontext.Symbol) []*fastcontext.Symbol {
	var filtered []*fastcontext

	for _, symbol := range symbols {
		// Check symbol kind filter
		if len(e.options.Filters.SymbolKinds) > 0 {
			found := false
			for _, kind := range e.options.Filters.SymbolKinds {
				if symbol.Kind == kind {
					found = true
					break
				}
			}
			if !found {
				continue
			}
		}

		// Check language filter
		if len(e.options.Filters.LanguageFilter) > 0 {
			found := false
			for _, lang := range e.options.Filters.LanguageFilter {
				if symbol.Language == lang {
					found = true
					break
				}
			}
			if !found {
				continue
			}
		}

		// Check complexity range
		if e.options.Filters.MinComplexity > 0 && symbol.Complexity < e.options.Filters.MinComplexity {
			continue
		}
		if e.options.Filters.MaxComplexity > 0 && symbol.Complexity > e.options.Filters.MaxComplexity {
			continue
		}

		// Check test inclusion
		if !e.options.Filters.IncludeTests && symbol.IsTest {
			continue
		}

		// Check private inclusion
		if !e.options.Filters.IncludePrivate && !symbol.IsPublic {
			continue
		}

		// Check file pattern
		if e.options.Filters.FilePattern != "" {
			if !strings.Contains(symbol.File, e.options.Filters.FilePattern) {
				continue
			}
		}

		filtered = append(filtered, symbol)
	}

	return filtered
}

// filterDependencies filters dependencies based on the filter options
func (e *Exporter) filterDependencies(dependencies []*fastcontext.Dependency) []*fastcontext.Dependency {
	var filtered []*fastcontext.Dependency

	for _, dep := range dependencies {
		if len(e.options.Filters.DependencyTypes) > 0 {
			found := false
			for _, depType := range e.options.Filters.DependencyTypes {
				if dep.Type == depType {
					found = true
					break
				}
			}
			if !found {
				continue
			}
		}

		filtered = append(filtered, dep)
	}

	return filtered
}

// exportJSON exports to JSON format
func (e *Exporter) exportJSON(result *fastcontext.AnalysisResult) ([]byte, error) {
	if e.options.Indent {
		return json.MarshalIndent(result, "", "  ")
	}
	return json.Marshal(result)
}

// exportYAML exports to YAML format
func (e *Exporter) exportYAML(result *fastcontext.AnalysisResult) ([]byte, error) {
	return yaml.Marshal(result)
}

// exportXML exports to XML format
func (e *Exporter) exportXML(result *fastcontext.AnalysisResult) ([]byte, error) {
	output, err := xml.MarshalIndent(result, "", "  ")
	if err != nil {
		return nil, err
	}
	return []byte(xml.Header + string(output)), nil
}

// exportGraphML exports to GraphML format
func (e *Exporter) exportGraphML(result *fastcontext.AnalysisResult) ([]byte, error) {
	var builder strings.Builder

	builder.WriteString(`<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns
         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">
  <key id="label" for="node" attr.name="label" attr.type="string"/>
  <key id="kind" for="node" attr.name="kind" attr.type="string"/>
  <key id="language" for="node" attr.name="language" attr.type="string"/>
  <key id="file" for="node" attr.name="file" attr.type="string"/>
  <key id="complexity" for="node" attr.name="complexity" attr.type="double"/>
  <key id="weight" for="edge" attr.name="weight" attr.type="double"/>
  <key id="type" for="edge" attr.name="type" attr.type="string"/>
  <graph id="G" edgedefault="directed">
`)

	// Add nodes (symbols)
	for _, symbol := range result.Symbols {
		builder.WriteString(fmt.Sprintf(`    <node id="%s">
      <data key="label">%s</data>
      <data key="kind">%s</data>
      <data key="language">%s</data>
      <data key="file">%s</data>
      <data key="complexity">%.2f</data>
    </node>
`, symbol.ID, symbol.Name, symbol.Kind, symbol.Language, symbol.File, symbol.Complexity))
	}

	// Add edges (dependencies)
	for _, dep := range result.Dependencies {
		builder.WriteString(fmt.Sprintf(`    <edge source="%s" target="%s">
      <data key="weight">%.2f</data>
      <data key="type">%s</data>
    </edge>
`, dep.From, dep.To, dep.Strength, dep.Type))
	}

	builder.WriteString(`  </graph>
</graphml>`)

	return []byte(builder.String()), nil
}

// exportDOT exports to DOT format
func (e *Exporter) exportDOT(result *fastcontext.AnalysisResult) ([]byte, error) {
	var builder strings.Builder

	builder.WriteString("digraph G {\n")
	builder.WriteString("  node [shape=box];\n")
	builder.WriteString("  edge [fontsize=10];\n\n")

	// Add nodes
	for _, symbol := range result.Symbols {
		color := "lightblue"
		switch symbol.Kind {
		case fastcontext.SymbolKindFunction, fastcontext.SymbolKindMethod:
			color = "lightgreen"
		case fastcontext.SymbolKindClass, fastcontext.SymbolKindStruct:
			color = "lightyellow"
		case fastcontext.SymbolKindInterface:
			color = "lightpink"
		}

		builder.WriteString(fmt.Sprintf(`  "%s" [label="%s\n(%s)" fillcolor="%s" style=filled];
`, symbol.ID, symbol.Name, symbol.Kind, color))
	}

	// Add edges
	for _, dep := range result.Dependencies {
		style := "solid"
		weight := 1.0
		if dep.Strength > 0.7 {
			weight = 3.0
			style = "bold"
		} else if dep.Strength < 0.3 {
			weight = 0.5
			style = "dashed"
		}

		builder.WriteString(fmt.Sprintf(`  "%s" -> "%s" [weight=%.1f style=%s label="%s"];
`, dep.From, dep.To, weight, style, dep.Type))
	}

	builder.WriteString("}\n")

	return []byte(builder.String()), nil
}

// exportCSV exports to CSV format
func (e *Exporter) exportCSV(result *fastcontext.AnalysisResult) ([]byte, error) {
	var builder strings.Builder
	writer := csv.NewWriter(&builder)

	// Write symbols header
	writer.Write([]string{"Type", "ID", "Name", "Kind", "Language", "File", "LineStart", "LineEnd", "Complexity", "IsPublic", "IsExported", "IsTest"})

	// Write symbols
	for _, symbol := range result.Symbols {
		writer.Write([]string{
			"Symbol",
			symbol.ID,
			symbol.Name,
			symbol.Kind.String(),
			symbol.Language,
			symbol.File,
			fmt.Sprintf("%d", symbol.LineStart),
			fmt.Sprintf("%d", symbol.LineEnd),
			fmt.Sprintf("%.2f", symbol.Complexity),
			fmt.Sprintf("%t", symbol.IsPublic),
			fmt.Sprintf("%t", symbol.IsExported),
			fmt.Sprintf("%t", symbol.IsTest),
		})
	}

	// Write dependencies header
	writer.Write([]string{"Type", "From", "To", "DependencyType", "Strength", "Context"})

	// Write dependencies
	for _, dep := range result.Dependencies {
		writer.Write([]string{
			"Dependency",
			dep.From,
			dep.To,
			dep.Type.String(),
			fmt.Sprintf("%.2f", dep.Strength),
			dep.Context,
		})
	}

	writer.Flush()
	return []byte(builder.String()), nil
}

// exportMarkdown exports to Markdown format
func (e *Exporter) exportMarkdown(result *fastcontext.AnalysisResult) ([]byte, error) {
	var builder strings.Builder

	builder.WriteString("# Fast-Context Analysis Report\n\n")
	builder.WriteString(fmt.Sprintf("Generated: %s\n\n", time.Now().Format("2006-01-02 15:04:05")))
	builder.WriteString(fmt.Sprintf("**Project:** %s\n\n", e.analyzer.GetConfig().ProjectRoot))

	// Summary
	builder.WriteString("## Summary\n\n")
	builder.WriteString(fmt.Sprintf("- **Files Analyzed:** %d\n", result.FileCount))
	builder.WriteString(fmt.Sprintf("- **Symbols Found:** %d\n", result.SymbolCount))
	builder.WriteString(fmt.Sprintf("- **Dependencies:** %d\n", result.RelationshipCount))
	builder.WriteString(fmt.Sprintf("- **Languages:** %s\n", strings.Join(result.Languages, ", ")))
	builder.WriteString(fmt.Sprintf("- **Analysis Time:** %dms\n", result.DurationMs))
	builder.WriteString(fmt.Sprintf("- **Memory Used:** %d bytes\n", result.MemoryUsed))

	// Language breakdown
	builder.WriteString("\n## Language Breakdown\n\n")
	langCount := make(map[string]int)
	for _, symbol := range result.Symbols {
		langCount[symbol.Language]++
	}
	for lang, count := range langCount {
		builder.WriteString(fmt.Sprintf("- **%s:** %d symbols\n", lang, count))
	}

	// Symbol kinds
	builder.WriteString("\n## Symbol Types\n\n")
	kindCount := make(map[fastcontext.SymbolKind]int)
	for _, symbol := range result.Symbols {
		kindCount[symbol.Kind]++
	}
	for kind, count := range kindCount {
		builder.WriteString(fmt.Sprintf("- **%s:** %d\n", kind, count))
	}

	// Complexity analysis
	builder.WriteString("\n## Complexity Analysis\n\n")
	if len(result.Symbols) > 0 {
		var totalComplexity float64
		var maxComplexity float64
		var complexSymbols []*fastcontext.Symbol

		for _, symbol := range result.Symbols {
			totalComplexity += symbol.Complexity
			if symbol.Complexity > maxComplexity {
				maxComplexity = symbol.Complexity
			}
			if symbol.Complexity > 10.0 {
				complexSymbols = append(complexSymbols, symbol)
			}
		}

		avgComplexity := totalComplexity / float64(len(result.Symbols))
		builder.WriteString(fmt.Sprintf("- **Average Complexity:** %.2f\n", avgComplexity))
		builder.WriteString(fmt.Sprintf("- **Max Complexity:** %.2f\n", maxComplexity))
		builder.WriteString(fmt.Sprintf("- **High Complexity Symbols (>10.0):** %d\n", len(complexSymbols)))

		if len(complexSymbols) > 0 {
			builder.WriteString("\n### High Complexity Symbols\n\n")
			for _, symbol := range complexSymbols {
				builder.WriteString(fmt.Sprintf("- **%s** (%s:%d-%d) - Complexity: %.2f\n",
					symbol.Name, symbol.File, symbol.LineStart, symbol.LineEnd, symbol.Complexity))
			}
		}
	}

	// Top symbols by dependencies
	builder.WriteString("\n## Most Connected Symbols\n\n")
	symbolDeps := make(map[string]int)
	for _, dep := range result.Dependencies {
		symbolDeps[dep.From]++
		symbolDeps[dep.To]++
	}

	// Sort by dependency count
	type symbolDep struct {
		name     string
		count    int
		symbol   *fastcontext.Symbol
	}
	var sortedDeps []symbolDep
	for id, count := range symbolDeps {
		for _, symbol := range result.Symbols {
			if symbol.ID == id {
				sortedDeps = append(sortedDeps, symbolDep{name: symbol.Name, count: count, symbol: symbol})
				break
			}
		}
	}

	// Simple sort (top 10)
	for i := 0; i < len(sortedDeps) && i < 10; i++ {
		for j := i + 1; j < len(sortedDeps); j++ {
			if sortedDeps[j].count > sortedDeps[i].count {
				sortedDeps[i], sortedDeps[j] = sortedDeps[j], sortedDeps[i]
			}
		}
	}

	for i := 0; i < len(sortedDeps) && i < 10; i++ {
		builder.WriteString(fmt.Sprintf("%d. **%s** (%s:%d) - %d connections\n",
			i+1, sortedDeps[i].name, sortedDeps[i].symbol.File, sortedDeps[i].symbol.LineStart, sortedDeps[i].count))
	}

	return []byte(builder.String()), nil
}

// Option functions for configuring the exporter

// WithFormat sets the export format
func WithFormat(format Format) func(*Options) {
	return func(o *Options) {
		o.Format = format
	}
}

// WithOutputFile sets the output file path
func WithOutputFile(file string) func(*Options) {
	return func(o *Options) {
		o.OutputFile = file
	}
}

// WithIndent enables/disables JSON indentation
func WithIndent(indent bool) func(*Options) {
	return func(o *Options) {
		o.Indent = indent
	}
}

// WithPrettyPrint enables/disables pretty printing
func WithPrettyPrint(pretty bool) func(*Options) {
	return func(o *Options) {
		o.PrettyPrint = pretty
	}
}

// WithIncludeMetrics enables/disables metrics inclusion
func WithIncludeMetrics(include bool) func(*Options) {
	return func(o *Options) {
		o.IncludeMetrics = include
	}
}

// WithIncludeProgress enables/disables progress inclusion
func WithIncludeProgress(include bool) func(*Options) {
	return func(o *Options) {
		o.IncludeProgress = include
	}
}

// WithFilters sets the filter options
func WithFilters(filters *FilterOptions) func(*Options) {
	return func(o *Options) {
		o.Filters = filters
	}
}