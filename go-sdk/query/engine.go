package query

import (
	"context"
	"regexp"
	"strings"

	"github.com/fast-context/go-sdk/fastcontext"
)

// Engine provides semantic search and query capabilities for code analysis
type Engine struct {
	analyzer *fastcontext.Analyzer
}

// NewEngine creates a new query engine
func NewEngine(analyzer *fastcontext.Analyzer) *Engine {
	return &Engine{
		analyzer: analyzer,
	}
}

// SemanticQuery represents a semantic search query
type SemanticQuery struct {
	Query         string            `json:"query"`
	Filters       map[string]string `json:"filters,omitempty"`
	SortBy        string            `json:"sortBy,omitempty"`        // "name", "complexity", "size"
	SortOrder     string            `json:"sortOrder,omitempty"`     // "asc", "desc"
	Limit         int               `json:"limit,omitempty"`
	Offset        int               `json:"offset,omitempty"`
	Highlight     bool              `json:"highlight,omitempty"`
	SearchType    string            `json:"searchType,omitempty"`    // "fuzzy", "exact", "regex"
	Language      string            `json:"language,omitempty"`
	FilePattern   string            `json:"filePattern,omitempty"`
}

// QueryResult represents the result of a semantic query
type QueryResult struct {
	Symbols      []*fastcontext.Symbol `json:"symbols"`
	TotalCount   int                   `json:"totalCount"`
	QueryTimeMs  int64                 `json:"queryTimeMs"`
	Highlights   map[string][]string   `json:"highlights,omitempty"`
	Suggestions  []string              `json:"suggestions,omitempty"`
	Facets       map[string]interface{} `json:"facets,omitempty"`
}

// DependencyOptions defines options for dependency queries
type DependencyOptions struct {
	Direction     string  `json:"direction,omitempty"`     // "incoming", "outgoing", "both"
	MaxDepth      int     `json:"maxDepth,omitempty"`
	MinStrength   float64 `json:"minStrength,omitempty"`
	IncludeTypes  []string `json:"includeTypes,omitempty"`
	ExcludeTypes  []string `json:"excludeTypes,omitempty"`
}

// DependencyGraph represents a dependency graph query result
type DependencyGraph struct {
	Nodes      []*DependencyNode    `json:"nodes"`
	Edges      []*DependencyEdge    `json:"edges"`
	Stats      DependencyStats       `json:"stats"`
	Paths      [][]string           `json:"paths,omitempty"`
	Cycles     [][]string           `json:"cycles,omitempty"`
}

// DependencyNode represents a node in the dependency graph
type DependencyNode struct {
	ID         string                `json:"id"`
	Name       string                `json:"name"`
	Kind       fastcontext.SymbolKind `json:"kind"`
	File       string                `json:"file"`
	Complexity float64               `json:"complexity"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
}

// DependencyEdge represents an edge in the dependency graph
type DependencyEdge struct {
	From      string                   `json:"from"`
	To        string                   `json:"to"`
	Type      fastcontext.DependencyType `json:"type"`
	Strength  float64                  `json:"strength"`
	Context   string                   `json:"context,omitempty"`
}

// DependencyStats contains statistics about the dependency graph
type DependencyStats struct {
	NodeCount    int     `json:"nodeCount"`
	EdgeCount    int     `json:"edgeCount"`
	AvgDegree    float64 `json:"avgDegree"`
	MaxDegree    int     `json:"maxDegree"`
	Density      float64 `json:"density"`
	Connected    bool    `json:"connected"`
}

// ComplexityOptions defines options for complexity analysis
type ComplexityOptions struct {
	Threshold      float64 `json:"threshold,omitempty"`
	IncludeTests   bool    `json:"includeTests,omitempty"`
	GroupByFile    bool    `json:"groupByFile,omitempty"`
	GroupByType    bool    `json:"groupByType,omitempty"`
	Metrics        []string `json:"metrics,omitempty"` // "cyclomatic", "cognitive", "maintainability"
}

// ComplexityResult represents the result of complexity analysis
type ComplexityResult struct {
	Symbols      []*fastcontext.Symbol `json:"symbols"`
	Average      float64               `json:"average"`
	Max          float64               `json:"max"`
	Min          float64               `json:"min"`
	Distribution map[string]int        `json:"distribution"` // complexity range -> count
	FileStats    map[string]*FileComplexityStats `json:"fileStats,omitempty"`
	TypeStats    map[string]*TypeComplexityStats `json:"typeStats,omitempty"`
}

// FileComplexityStats contains complexity stats for a file
type FileComplexityStats struct {
	FilePath        string  `json:"filePath"`
	SymbolCount     int     `json:"symbolCount"`
	TotalComplexity float64 `json:"totalComplexity"`
	AverageComplexity float64 `json:"averageComplexity"`
	MaxComplexity   float64 `json:"maxComplexity"`
}

// TypeComplexityStats contains complexity stats for symbol types
type TypeComplexityStats struct {
	Type           string  `json:"type"`
	Count          int     `json:"count"`
	TotalComplexity float64 `json:"totalComplexity"`
	AverageComplexity float64 `json:"averageComplexity"`
}

// FindSymbols performs semantic search for symbols
func (e *Engine) FindSymbols(ctx context.Context, query *SemanticQuery) (*QueryResult, error) {
	if query.Query == "" {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "query cannot be empty")
	}

	startTime := makeTimestamp()

	// Get all symbols as base for filtering
	symbols, err := e.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	// Apply filters
	filteredSymbols := e.applyFilters(symbols, query)

	// Apply search
	searchResults := e.applySearch(filteredSymbols, query)

	// Sort results
	sortedResults := e.sortResults(searchResults, query)

	// Apply pagination
	paginatedResults := e.paginateResults(sortedResults, query)

	// Generate highlights if requested
	highlights := map[string][]string{}
	if query.Highlight {
		highlights = e.generateHighlights(paginatedResults, query.Query)
	}

	// Calculate facets
	facets := e.calculateFacets(paginatedResults)

	// Generate suggestions
	suggestions := e.generateSuggestions(query.Query, symbols)

	return &QueryResult{
		Symbols:     paginatedResults,
		TotalCount:  len(searchResults),
		QueryTimeMs: makeTimestamp() - startTime,
		Highlights:  highlights,
		Suggestions: suggestions,
		Facets:      facets,
	}, nil
}

// GetSymbolDependencies retrieves the dependency graph for a symbol
func (e *Engine) GetSymbolDependencies(ctx context.Context, symbolName string, opts *DependencyOptions) (*DependencyGraph, error) {
	if symbolName == "" {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "symbol name cannot be empty")
	}

	// Get dependencies for the symbol
	dependencies, err := e.analyzer.FindDependencies(symbolName)
	if err != nil {
		return nil, err
	}

	// Build dependency graph
	graph := e.buildDependencyGraph(symbolName, dependencies, opts)

	return graph, nil
}

// GetSymbolUsages finds all usages of a symbol
func (e *Engine) GetSymbolUsages(ctx context.Context, symbolName string) ([]*fastcontext.Dependency, error) {
	if symbolName == "" {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "symbol name cannot be empty")
	}

	// Get all symbols to find usages
	allSymbols, err := e.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	var usages []*fastcontext.Dependency
	for _, symbol := range allSymbols {
		// Get dependencies for each symbol
		deps, err := e.analyzer.FindDependencies(symbol.Name)
		if err != nil {
			continue // Skip errors for now
		}

		// Check if any dependency matches our target symbol
		for _, dep := range deps {
			if dep.To == symbolName {
				usages = append(usages, dep)
			}
		}
	}

	return usages, nil
}

// DetectPatterns detects common code patterns and anti-patterns
func (e *Engine) DetectPatterns(ctx context.Context, patternTypes []string) (map[string]interface{}, error) {
	if len(patternTypes) == 0 {
		patternTypes = []string{"complexity", "duplication", "coupling", "cohesion"}
	}

	// Get all symbols for analysis
	symbols, err := e.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	results := make(map[string]interface{})

	for _, patternType := range patternTypes {
		switch patternType {
		case "complexity":
			results["complexity"] = e.detectComplexityPatterns(symbols)
		case "duplication":
			results["duplication"] = e.detectDuplicationPatterns(symbols)
		case "coupling":
			results["coupling"] = e.detectCouplingPatterns(symbols)
		case "cohesion":
			results["cohesion"] = e.detectCohesionPatterns(symbols)
		}
	}

	return results, nil
}

// AnalyzeComplexity performs comprehensive complexity analysis
func (e *Engine) AnalyzeComplexity(ctx context.Context, opts *ComplexityOptions) (*ComplexityResult, error) {
	if opts == nil {
		opts = &ComplexityOptions{}
	}

	// Get all symbols
	symbols, err := e.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	// Filter by threshold
	var filteredSymbols []*fastcontext.Symbol
	for _, symbol := range symbols {
		if opts.Threshold <= 0 || symbol.Complexity >= opts.Threshold {
			if !symbol.IsTest || opts.IncludeTests {
				filteredSymbols = append(filteredSymbols, symbol)
			}
		}
	}

	// Calculate statistics
	result := &ComplexityResult{
		Symbols: filteredSymbols,
		Distribution: make(map[string]int),
	}

	if len(filteredSymbols) > 0 {
		result.Max = filteredSymbols[0].Complexity
		result.Min = filteredSymbols[0].Complexity

		totalComplexity := 0.0
		for _, symbol := range filteredSymbols {
			totalComplexity += symbol.Complexity

			if symbol.Complexity > result.Max {
				result.Max = symbol.Complexity
			}
			if symbol.Complexity < result.Min {
				result.Min = symbol.Complexity
			}

			// Update distribution
			bucket := e.getComplexityBucket(symbol.Complexity)
			result.Distribution[bucket]++
		}

		result.Average = totalComplexity / float64(len(filteredSymbols))
	}

	// Group by file if requested
	if opts.GroupByFile {
		result.FileStats = e.calculateFileComplexityStats(filteredSymbols)
	}

	// Group by type if requested
	if opts.GroupByType {
		result.TypeStats = e.calculateTypeComplexityStats(filteredSymbols)
	}

	return result, nil
}

// FindSimilarCode finds code that is similar to a given symbol or code snippet
func (e *Engine) FindSimilarCode(ctx context.Context, reference string, threshold float64) ([]*fastcontext.Symbol, error) {
	if reference == "" {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "reference cannot be empty")
	}

	// Get all symbols
	symbols, err := e.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
	if err != nil {
		return nil, err
	}

	// For now, use simple name-based similarity
	// In production, this would use more sophisticated algorithms
	var similarSymbols []*fastcontext.Symbol
	for _, symbol := range symbols {
		similarity := e.calculateSimilarity(reference, symbol.Name)
		if similarity >= threshold {
			similarSymbols = append(similarSymbols, symbol)
		}
	}

	return similarSymbols, nil
}

// Helper functions

func (e *Engine) applyFilters(symbols []*fastcontext.Symbol, query *SemanticQuery) []*fastcontext.Symbol {
	var filtered []*fastcontext.Symbol

	for _, symbol := range symbols {
		include := true

		// Language filter
		if query.Language != "" && symbol.Language != query.Language {
			include = false
		}

		// File pattern filter
		if query.FilePattern != "" {
			matched, err := regexp.MatchString(query.FilePattern, symbol.File)
			if err != nil || !matched {
				include = false
			}
		}

		// Additional filters
		for key, value := range query.Filters {
			switch key {
			case "kind":
				if symbol.Kind.String() != value {
					include = false
				}
			case "file":
				if !strings.Contains(symbol.File, value) {
					include = false
				}
			case "isPublic":
				if symbol.IsPublic != (value == "true") {
					include = false
				}
			}
		}

		if include {
			filtered = append(filtered, symbol)
		}
	}

	return filtered
}

func (e *Engine) applySearch(symbols []*fastcontext.Symbol, query *SemanticQuery) []*fastcontext.Symbol {
	var results []*fastcontext.Symbol

	for _, symbol := range symbols {
		matched := false

		switch query.SearchType {
		case "exact":
			matched = strings.Contains(strings.ToLower(symbol.Name), strings.ToLower(query.Query))
		case "regex":
			matched, _ = regexp.MatchString(query.Query, symbol.Name)
		case "fuzzy":
			fallthrough
		default:
			matched = e.fuzzyMatch(query.Query, symbol.Name)
		}

		if matched {
			results = append(results, symbol)
		}
	}

	return results
}

func (e *Engine) sortResults(symbols []*fastcontext.Symbol, query *SemanticQuery) []*fastcontext.Symbol {
	if query.SortBy == "" {
		return symbols
	}

	// Simple implementation - in production would use more efficient sorting
	switch query.SortBy {
	case "name":
		if query.SortOrder == "desc" {
			// Reverse sort by name
			for i, j := 0, len(symbols)-1; i < j; i, j = i+1, j-1 {
				symbols[i], symbols[j] = symbols[j], symbols[i]
			}
		}
	case "complexity":
		// Sort by complexity (implementation omitted for brevity)
	case "size":
		// Sort by symbol size (implementation omitted for brevity)
	}

	return symbols
}

func (e *Engine) paginateResults(symbols []*fastcontext.Symbol, query *SemanticQuery) []*fastcontext.Symbol {
	if query.Limit <= 0 {
		return symbols
	}

	start := query.Offset
	if start >= len(symbols) {
		return []*fastcontext.Symbol{}
	}

	end := start + query.Limit
	if end > len(symbols) {
		end = len(symbols)
	}

	return symbols[start:end]
}

func (e *Engine) generateHighlights(symbols []*fastcontext.Symbol, query string) map[string][]string {
	highlights := make(map[string][]string)

	for _, symbol := range symbols {
		if strings.Contains(strings.ToLower(symbol.Name), strings.ToLower(query)) {
			highlights[symbol.ID] = []string{query}
		}
	}

	return highlights
}

func (e *Engine) calculateFacets(symbols []*fastcontext.Symbol) map[string]interface{} {
	facets := make(map[string]interface{})

	// Language facet
	languages := make(map[string]int)
	for _, symbol := range symbols {
		languages[symbol.Language]++
	}
	facets["languages"] = languages

	// Kind facet
	kinds := make(map[string]int)
	for _, symbol := range symbols {
		kinds[symbol.Kind.String()]++
	}
	facets["kinds"] = kinds

	// File facet
	files := make(map[string]int)
	for _, symbol := range symbols {
		files[symbol.File]++
	}
	facets["files"] = files

	return facets
}

func (e *Engine) generateSuggestions(query string, symbols []*fastcontext.Symbol) []string {
	// Simple suggestion algorithm - find similar symbol names
	suggestions := []string{}
	queryLower := strings.ToLower(query)

	for _, symbol := range symbols {
		nameLower := strings.ToLower(symbol.Name)
		if strings.HasPrefix(nameLower, queryLower) && nameLower != queryLower {
			suggestions = append(suggestions, symbol.Name)
		}
		if len(suggestions) >= 5 {
			break
		}
	}

	return suggestions
}

func (e *Engine) buildDependencyGraph(symbolName string, dependencies []*fastcontext.Dependency, opts *DependencyOptions) *DependencyGraph {
	// Simplified implementation - would build actual graph structure
	nodes := []*DependencyNode{
		{ID: symbolName, Name: symbolName, Kind: fastcontext.SymbolKindUnknown},
	}

	edges := []*DependencyEdge{}
	for _, dep := range dependencies {
		edges = append(edges, &DependencyEdge{
			From:     symbolName,
			To:       dep.To,
			Type:     dep.Type,
			Strength: dep.Strength,
			Context:  dep.Context,
		})
	}

	return &DependencyGraph{
		Nodes: nodes,
		Edges: edges,
		Stats: DependencyStats{
			NodeCount: len(nodes),
			EdgeCount: len(edges),
		},
	}
}

func (e *Engine) detectComplexityPatterns(symbols []*fastcontext.Symbol) map[string]interface{} {
	highComplexity := []*fastcontext.Symbol{}
	for _, symbol := range symbols {
		if symbol.Complexity > 10.0 {
			highComplexity = append(highComplexity, symbol)
		}
	}

	return map[string]interface{}{
		"highComplexitySymbols": highComplexity,
		"count":                 len(highComplexity),
	}
}

func (e *Engine) detectDuplicationPatterns(symbols []*fastcontext.Symbol) map[string]interface{} {
	// Simplified implementation
	return map[string]interface{}{
		"potentialDuplicates": []string{},
		"count":               0,
	}
}

func (e *Engine) detectCouplingPatterns(symbols []*fastcontext.Symbol) map[string]interface{} {
	// Simplified implementation
	return map[string]interface{}{
		"highlyCoupled": []string{},
		"count":         0,
	}
}

func (e *Engine) detectCohesionPatterns(symbols []*fastcontext.Symbol) map[string]interface{} {
	// Simplified implementation
	return map[string]interface{}{
		"lowCohesion": []string{},
		"count":       0,
	}
}

func (e *Engine) calculateFileComplexityStats(symbols []*fastcontext.Symbol) map[string]*FileComplexityStats {
	stats := make(map[string]*FileComplexityStats)

	for _, symbol := range symbols {
		if stats[symbol.File] == nil {
			stats[symbol.File] = &FileComplexityStats{
				FilePath: symbol.File,
			}
		}

		fileStats := stats[symbol.File]
		fileStats.SymbolCount++
		fileStats.TotalComplexity += symbol.Complexity
		if symbol.Complexity > fileStats.MaxComplexity {
			fileStats.MaxComplexity = symbol.Complexity
		}
	}

	// Calculate averages
	for _, fileStats := range stats {
		if fileStats.SymbolCount > 0 {
			fileStats.AverageComplexity = fileStats.TotalComplexity / float64(fileStats.SymbolCount)
		}
	}

	return stats
}

func (e *Engine) calculateTypeComplexityStats(symbols []*fastcontext.Symbol) map[string]*TypeComplexityStats {
	stats := make(map[string]*TypeComplexityStats)

	for _, symbol := range symbols {
		typeName := symbol.Kind.String()
		if stats[typeName] == nil {
			stats[typeName] = &TypeComplexityStats{
				Type: typeName,
			}
		}

		typeStats := stats[typeName]
		typeStats.Count++
		typeStats.TotalComplexity += symbol.Complexity
	}

	// Calculate averages
	for _, typeStats := range stats {
		if typeStats.Count > 0 {
			typeStats.AverageComplexity = typeStats.TotalComplexity / float64(typeStats.Count)
		}
	}

	return stats
}

func (e *Engine) calculateSimilarity(ref, target string) float64 {
	// Simple similarity calculation based on common prefix
	refLower := strings.ToLower(ref)
	targetLower := strings.ToLower(target)

	maxLen := len(refLower)
	if len(targetLower) > maxLen {
		maxLen = len(targetLower)
	}

	if maxLen == 0 {
		return 0.0
	}

	// Calculate common prefix length
	common := 0
	for i := 0; i < len(refLower) && i < len(targetLower); i++ {
		if refLower[i] == targetLower[i] {
			common++
		} else {
			break
		}
	}

	return float64(common) / float64(maxLen)
}

func (e *Engine) fuzzyMatch(pattern, target string) bool {
	patternLower := strings.ToLower(pattern)
	targetLower := strings.ToLower(target)
	
	// Simple fuzzy matching - check if all pattern characters appear in order in target
	patternIndex := 0
	for _, char := range targetLower {
		if patternIndex < len(patternLower) && char == rune(patternLower[patternIndex]) {
			patternIndex++
		}
	}
	
	return patternIndex == len(patternLower)
}

func (e *Engine) getComplexityBucket(complexity float64) string {
	switch {
	case complexity < 5:
		return "low"
	case complexity < 10:
		return "medium"
	case complexity < 20:
		return "high"
	default:
		return "very-high"
	}
}

func makeTimestamp() int64 {
	return 0 // Simplified - would use actual timestamp
}

