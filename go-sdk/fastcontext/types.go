package fastcontext

// SymbolKind represents the type of symbol found in code
type SymbolKind int

const (
	SymbolKindUnknown SymbolKind = iota
	SymbolKindFunction
	SymbolKindMethod
	SymbolKindClass
	SymbolKindInterface
	SymbolKindStruct
	SymbolKindEnum
	SymbolKindVariable
	SymbolKindConstant
	SymbolKindParameter
	SymbolKindModule
	SymbolKindPackage
	SymbolKindType
	SymbolKindField
	SymbolKindProperty
	SymbolKindConstructor
	SymbolKindDestructor
	SymbolKindOperator
	SymbolKindMacro
	SymbolKindAnnotation
)

// String returns the string representation of SymbolKind
func (sk SymbolKind) String() string {
	switch sk {
	case SymbolKindUnknown:
		return "unknown"
	case SymbolKindFunction:
		return "function"
	case SymbolKindMethod:
		return "method"
	case SymbolKindClass:
		return "class"
	case SymbolKindInterface:
		return "interface"
	case SymbolKindStruct:
		return "struct"
	case SymbolKindEnum:
		return "enum"
	case SymbolKindVariable:
		return "variable"
	case SymbolKindConstant:
		return "constant"
	case SymbolKindParameter:
		return "parameter"
	case SymbolKindModule:
		return "module"
	case SymbolKindPackage:
		return "package"
	case SymbolKindType:
		return "type"
	case SymbolKindField:
		return "field"
	case SymbolKindProperty:
		return "property"
	case SymbolKindConstructor:
		return "constructor"
	case SymbolKindDestructor:
		return "destructor"
	case SymbolKindOperator:
		return "operator"
	case SymbolKindMacro:
		return "macro"
	case SymbolKindAnnotation:
		return "annotation"
	default:
		return "unknown"
	}
}

// DependencyType represents the type of dependency relationship
type DependencyType int

const (
	DepTypeUnknown DependencyType = iota
	DepTypeImports
	DepTypeCalls
	DepTypeInherits
	DepTypeImplements
	DepTypeReferences
	DepTypeInstantiates
	DepTypeContains
	DepTypeOverrides
	DepTypeDecorates
)

// AnalysisPhase represents the current phase of analysis
type AnalysisPhase int

const (
	PhaseDiscovery AnalysisPhase = iota
	PhaseParsing
	PhaseSymbolExtraction
	PhaseDependencyAnalysis
	PhaseGraphConstruction
	PhaseOptimization
	PhaseComplete
)

// Progress represents analysis progress information
type Progress struct {
	Phase       AnalysisPhase
	Current     int
	Total       int
	Percentage  float64
	Message     string
	CurrentFile string
}

// Symbol represents a code symbol with metadata
type Symbol struct {
	ID          string      `json:"id"`
	Name        string      `json:"name"`
	Kind        SymbolKind  `json:"kind"`
	Language    string      `json:"language"`
	File        string      `json:"file"`
	LineStart   int         `json:"lineStart"`
	LineEnd     int         `json:"lineEnd"`
	ColumnStart int         `json:"columnStart"`
	ColumnEnd   int         `json:"columnEnd"`
	Documentation string    `json:"documentation,omitempty"`
	Children    []*Symbol   `json:"children,omitempty"`
	Properties  map[string]interface{} `json:"properties,omitempty"`
	Complexity  float64     `json:"complexity"`
	IsPublic    bool        `json:"isPublic"`
	IsExported  bool        `json:"isExported"`
	IsTest      bool        `json:"isTest"`
	IsDeprecated bool       `json:"isDeprecated"`
	Tags        []string    `json:"tags,omitempty"`
}

// Dependency represents a relationship between symbols
type Dependency struct {
	From      string         `json:"from"`
	To        string         `json:"to"`
	Type      DependencyType `json:"type"`
	Strength  float64        `json:"strength"`
	Context   string         `json:"context,omitempty"`
	Location  *Symbol        `json:"location,omitempty"`
}

// AnalysisResult contains the results of codebase analysis
type AnalysisResult struct {
	FileCount         int           `json:"fileCount"`
	SymbolCount       int           `json:"symbolCount"`
	RelationshipCount int           `json:"relationshipCount"`
	Symbols           []*Symbol     `json:"symbols"`
	Dependencies      []*Dependency `json:"dependencies"`
	Languages         []string      `json:"languages"`
	DurationMs        int64         `json:"durationMs"`
	MemoryUsed        int64         `json:"memoryUsed"`
	Progress          []Progress    `json:"progress,omitempty"`
	Metadata          map[string]interface{} `json:"metadata,omitempty"`
}

// PerformanceMetrics contains performance information
type PerformanceMetrics struct {
	TotalTimeMs     int64   `json:"totalTimeMs"`
	ParsingTimeMs   int64   `json:"parsingTimeMs"`
	AnalysisTimeMs  int64   `json:"analysisTimeMs"`
	MemoryPeakBytes int64   `json:"memoryPeakBytes"`
	CacheHitRate    float64 `json:"cacheHitRate"`
	FilesPerSecond  float64 `json:"filesPerSecond"`
	SymbolsPerFile  float64 `json:"symbolsPerFile"`
}

// FileChangeEvent represents a file system change event
type FileChangeEvent struct {
	Type      string `json:"type"`      // "created", "modified", "deleted", "renamed"
	Path      string `json:"path"`
	OldPath   string `json:"oldPath,omitempty"` // for rename events
	Timestamp int64  `json:"timestamp"`
	Size      int64  `json:"size,omitempty"`
}

// SymbolMetrics contains detailed metrics for a specific symbol
type SymbolMetrics struct {
	Name               string  `json:"name"`
	Complexity         float64 `json:"complexity"`
	LinesOfCode        int     `json:"linesOfCode"`
	Dependencies       int     `json:"dependencies"`
	Dependents         int     `json:"dependents"`
	CyclomaticComplexity int   `json:"cyclomaticComplexity"`
	CognitiveComplexity float64 `json:"cognitiveComplexity"`
	MaintainabilityIndex float64 `json:"maintainabilityIndex"`
	NumberOfParameters  int     `json:"numberOfParameters"`
	DepthOfNesting     int     `json:"depthOfNesting"`
	NumberOfReturns    int     `json:"numberOfReturns"`
}

// FileMetrics contains metrics for a specific file
type FileMetrics struct {
	FilePath        string  `json:"filePath"`
	SymbolCount     int     `json:"symbolCount"`
	TotalLines      int     `json:"totalLines"`
	CodeLines       int     `json:"codeLines"`
	CommentLines    int     `json:"commentLines"`
	BlankLines      int     `json:"blankLines"`
	Complexity      float64 `json:"complexity"`
	AverageComplexity float64 `json:"averageComplexity"`
	MaxComplexity   float64 `json:"maxComplexity"`
	MaintainabilityIndex float64 `json:"maintainabilityIndex"`
	Languages       []string `json:"languages,omitempty"`
}

