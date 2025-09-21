package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/export"
	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/fast-context/go-sdk/filewatch"
	"github.com/spf13/cobra"
)

// CLI represents the command-line interface
type CLI struct {
	rootCmd  *cobra.Command
	analyzer *fastcontext.Analyzer
	config   *config.Config
}

// NewCLI creates a new CLI instance
func NewCLI() *CLI {
	cli := &CLI{
		rootCmd: &cobra.Command{
			Use:   "fast-context",
			Short: "Fast-Context - Intelligent codebase analysis engine",
			Long: `Fast-Context is an intelligent codebase analysis engine that provides
comprehensive code comprehension through graph-powered dependency analysis
and multi-language symbol extraction.`,
			Version: "1.0.0",
		},
	}

	// Initialize with default config
	cfg, err := config.NewConfig()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error creating default config: %v\n", err)
		os.Exit(1)
	}
	cli.config = cfg

	// Create analyzer
	analyzer, err := fastcontext.NewAnalyzerWithConfig(cfg)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error creating analyzer: %v\n", err)
		os.Exit(1)
	}
	cli.analyzer = analyzer

	cli.setupCommands()
	return cli
}

// setupCommands sets up all CLI commands
func (cli *CLI) setupCommands() {
	// Add global flags
	cli.rootCmd.PersistentFlags().StringVarP(&cli.config.ProjectRoot, "project", "p", ".", "Project root directory")
	cli.rootCmd.PersistentFlags().StringSliceVarP(&cli.config.Languages, "languages", "l", []string{}, "Languages to analyze")
	cli.rootCmd.PersistentFlags().StringSliceVar(&cli.config.IgnorePatterns, "ignore", []string{}, "File patterns to ignore")
	cli.rootCmd.PersistentFlags().IntVar(&cli.config.Performance.TimeoutSeconds, "timeout", 300, "Analysis timeout in seconds")
	cli.rootCmd.PersistentFlags().IntVar(&cli.config.Performance.MaxMemoryMB, "memory", 1024, "Maximum memory usage in MB")
	cli.rootCmd.PersistentFlags().StringVar(&cli.config.LogLevel, "log-level", "info", "Log level (debug, info, warn, error)")
	cli.rootCmd.PersistentFlags().BoolVarP(&cli.config.EnableProgress, "progress", "v", true, "Show progress")

	// Add commands
	cli.rootCmd.AddCommand(cli.createAnalyzeCommand())
	cli.rootCmd.AddCommand(cli.createSymbolsCommand())
	cli.rootCmd.AddCommand(cli.createDependenciesCommand())
	cli.rootCmd.AddCommand(cli.createComplexityCommand())
	cli.rootCmd.AddCommand(cli.createPatternsCommand())
	cli.rootCmd.AddCommand(cli.createExportCommand())
	cli.rootCmd.AddCommand(cli.createConfigCommand())
	cli.rootCmd.AddCommand(cli.createWatchCommand())
	cli.rootCmd.AddCommand(cli.createServeCommand())
	cli.rootCmd.AddCommand(cli.createVersionCommand())
}

// createAnalyzeCommand creates the analyze command
func (cli *CLI) createAnalyzeCommand() *cobra.Command {
	var outputFile string

	cmd := &cobra.Command{
		Use:   "analyze [path]",
		Short: "Analyze a codebase",
		Long:  `Perform comprehensive analysis of a codebase, extracting symbols, dependencies, and metrics.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				cli.config.ProjectRoot = args[0]
			}

			// Update analyzer config
			if err := cli.analyzer.UpdateConfig(config.WithProjectRoot(cli.config.ProjectRoot)); err != nil {
				return fmt.Errorf("failed to update config: %w", err)
			}

			fmt.Printf("Analyzing project: %s\n", cli.config.ProjectRoot)

			result, err := cli.analyzer.Analyze()
			if err != nil {
				return fmt.Errorf("analysis failed: %w", err)
			}

			// Output results
			if outputFile != "" {
				data, err := json.MarshalIndent(result, "", "  ")
				if err != nil {
					return fmt.Errorf("failed to marshal results: %w", err)
				}
				if err := os.WriteFile(outputFile, data, 0644); err != nil {
					return fmt.Errorf("failed to write output file: %w", err)
				}
				fmt.Printf("Results written to: %s\n", outputFile)
			} else {
				data, err := json.MarshalIndent(result, "", "  ")
				if err != nil {
					return fmt.Errorf("failed to marshal results: %w", err)
				}
				fmt.Println(string(data))
			}

			cli.printSummary(result)
			return nil
		},
	}

	cmd.Flags().StringVarP(&outputFile, "output", "o", "", "Output file path")
	return cmd
}

// createSymbolsCommand creates the symbols command
func (cli *CLI) createSymbolsCommand() *cobra.Command {
	var kind, file, pattern string
	var threshold float64

	cmd := &cobra.Command{
		Use:   "symbols",
		Short: "Find and analyze symbols",
		Long:  `Search for symbols in the codebase with various filtering options.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var symbols []*fastcontext.Symbol
			var err error

			if kind != "" {
				symKind := fastcontext.SymbolKindFunction // Default
				// Parse kind string to enum (simplified)
				symbols, err = cli.analyzer.FindSymbolsByKind(symKind)
			} else if file != "" {
				symbols, err = cli.analyzer.FindSymbolsInFile(file)
			} else if pattern != "" {
				symbols, err = cli.analyzer.FindSymbolsByPattern(pattern)
			} else if threshold > 0 {
				symbols, err = cli.analyzer.FindComplexSymbols(threshold)
			} else {
				// Get all symbols
				symbols, err = cli.analyzer.FindSymbolsByKind(fastcontext.SymbolKindUnknown)
			}

			if err != nil {
				return fmt.Errorf("failed to find symbols: %w", err)
			}

			cli.printSymbols(symbols)
			return nil
		},
	}

	cmd.Flags().StringVar(&kind, "kind", "", "Symbol kind (function, class, struct, etc.)")
	cmd.Flags().StringVar(&file, "file", "", "File to analyze")
	cmd.Flags().StringVar(&pattern, "pattern", "", "Regex pattern to match")
	cmd.Flags().Float64Var(&threshold, "complexity", 0, "Minimum complexity threshold")
	return cmd
}

// createDependenciesCommand creates the dependencies command
func (cli *CLI) createDependenciesCommand() *cobra.Command {
	var symbolName string

	cmd := &cobra.Command{
		Use:   "dependencies [symbol]",
		Short: "Find dependencies for a symbol",
		Long:  `Show all dependencies for a specific symbol.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				symbolName = args[0]
			}

			if symbolName == "" {
				return fmt.Errorf("symbol name is required")
			}

			deps, err := cli.analyzer.FindDependencies(symbolName)
			if err != nil {
				return fmt.Errorf("failed to find dependencies: %w", err)
			}

			cli.printDependencies(deps)
			return nil
		},
	}

	return cmd
}

// createComplexityCommand creates the complexity command
func (cli *CLI) createComplexityCommand() *cobra.Command {
	var threshold float64

	cmd := &cobra.Command{
		Use:   "complexity [threshold]",
		Short: "Analyze code complexity",
		Long:  `Find symbols with complexity above the specified threshold.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				threshold, _ = strconv.ParseFloat(args[0], 64)
			}

			if threshold <= 0 {
				threshold = 10.0
			}

			symbols, err := cli.analyzer.FindComplexSymbols(threshold)
			if err != nil {
				return fmt.Errorf("failed to find complex symbols: %w", err)
			}

			fmt.Printf("Symbols with complexity >= %.1f:\n", threshold)
			cli.printSymbols(symbols)
			return nil
		},
	}

	return cmd
}

// createPatternsCommand creates the patterns command
func (cli *CLI) createPatternsCommand() *cobra.Command {
	var pattern string

	cmd := &cobra.Command{
		Use:   "patterns [pattern]",
		Short: "Search for code patterns",
		Long:  `Find symbols matching a specific regex pattern.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				pattern = args[0]
			}

			if pattern == "" {
				return fmt.Errorf("pattern is required")
			}

			symbols, err := cli.analyzer.FindSymbolsByPattern(pattern)
			if err != nil {
				return fmt.Errorf("failed to find symbols by pattern: %w", err)
			}

			cli.printSymbols(symbols)
			return nil
		},
	}

	return cmd
}

// createExportCommand creates the export command
func (cli *CLI) createExportCommand() *cobra.Command {
	var format, outputFile string
	var indent bool

	cmd := &cobra.Command{
		Use:   "export [format]",
		Short: "Export analysis results",
		Long:  `Export analysis results to various formats (json, yaml, xml, graphml, dot, csv, markdown).`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				format = args[0]
			}

			if format == "" {
				format = "json"
			}

			exportFormat, err := export.ParseFormat(format)
			if err != nil {
				return fmt.Errorf("invalid format: %w", err)
			}

			// Perform analysis
			result, err := cli.analyzer.Analyze()
			if err != nil {
				return fmt.Errorf("analysis failed: %w", err)
			}

			// Create exporter
			exporter := export.NewExporter(cli.analyzer,
				export.WithFormat(exportFormat),
				export.WithOutputFile(outputFile),
				export.WithIndent(indent),
			)

			if outputFile != "" {
				err = exporter.ExportToFile(result)
			} else {
				data, err := exporter.Export(result)
				if err != nil {
					return fmt.Errorf("export failed: %w", err)
				}
				fmt.Println(string(data))
				return nil
			}

			if err != nil {
				return fmt.Errorf("export failed: %w", err)
			}

			fmt.Printf("Exported to %s format", format)
			if outputFile != "" {
				fmt.Printf(": %s", outputFile)
			}
			fmt.Println()

			return nil
		},
	}

	cmd.Flags().StringVarP(&outputFile, "output", "o", "", "Output file path")
	cmd.Flags().BoolVar(&indent, "indent", true, "Enable indentation")
	return cmd
}

// createConfigCommand creates the config command
func (cli *CLI) createConfigCommand() *cobra.Command {
	var show, validate, init bool
	var preset string

	cmd := &cobra.Command{
		Use:   "config",
		Short: "Manage configuration",
		Long:  `View, validate, or initialize configuration.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if show {
				cli.printConfig()
				return nil
			}

			if validate {
				if err := cli.config.Validate(); err != nil {
					return fmt.Errorf("configuration validation failed: %w", err)
				}
				fmt.Println("Configuration is valid")
				return nil
			}

			if init {
				return cli.initConfig()
			}

			if preset != "" {
				return cli.applyPreset(preset)
			}

			return cmd.Help()
		},
	}

	cmd.Flags().BoolVar(&show, "show", false, "Show current configuration")
	cmd.Flags().BoolVar(&validate, "validate", false, "Validate configuration")
	cmd.Flags().BoolVar(&init, "init", false, "Initialize configuration file")
	cmd.Flags().StringVar(&preset, "preset", "", "Apply preset (fast, balanced, thorough)")
	return cmd
}

// createWatchCommand creates the watch command
func (cli *CLI) createWatchCommand() *cobra.Command {
	var duration int

	cmd := &cobra.Command{
		Use:   "watch [duration]",
		Short: "Watch project for changes",
		Long:  `Monitor the project for file changes and trigger analysis automatically.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				_, _ = fmt.Sscanf(args[0], "%d", &duration)
			}

			if duration <= 0 {
				duration = 60 // Default 1 minute
			}

			fmt.Printf("Watching project: %s\n", cli.config.ProjectRoot)
			fmt.Printf("Press Ctrl+C to stop watching\n")

			// Create file watcher
			watcher, _ := filewatch.NewWatcher(cli.config.ProjectRoot, &filewatch.WatchOptions{
			IgnorePatterns: cli.config.IgnorePatterns,
		})

			// Set up progress callback
			progressCallback := func(p *fastcontext.Progress) {
				fmt.Printf("\rProgress: %.1f%% - %s", p.Percentage, p.Message)
			}

			// Start watching
			if err := cli.analyzer.StartWatching(progressCallback); err != nil {
				return fmt.Errorf("failed to start watching: %w", err)
			}
			defer func() { _ = cli.analyzer.StopWatching() }()

			// Start file system watcher
			eventChan := make(chan *filewatch.FileEvent, 100)
			if err := watcher.StartWatching(); err != nil {
				return fmt.Errorf("failed to start file watcher: %w", err)
			}
			defer func() { _ = watcher.StopWatching() }()

			// Watch for events
			timeout := time.After(time.Duration(duration) * time.Second)
			for {
				select {
				case event := <-eventChan:
					fmt.Printf("\nFile %s: %s\n", event.Type, event.Path)
					// Trigger re-analysis
					if _, err := cli.analyzer.Analyze(); err != nil {
						fmt.Printf("Analysis failed: %v\n", err)
					}
				case <-timeout:
					fmt.Println("\nWatch duration completed")
					return nil
				}
			}
		},
	}

	return cmd
}

// createServeCommand creates the serve command
func (cli *CLI) createServeCommand() *cobra.Command {
	var port int

	cmd := &cobra.Command{
		Use:   "serve [port]",
		Short: "Start HTTP server",
		Long:  `Start an HTTP server for web-based analysis.`,
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			if len(args) > 0 {
				_, _ = fmt.Sscanf(args[0], "%d", &port)
			}

			if port <= 0 {
				port = 8080
			}

			fmt.Printf("Starting HTTP server on port %d\n", port)
			fmt.Printf("Project root: %s\n", cli.config.ProjectRoot)
			fmt.Println("Press Ctrl+C to stop the server")

			// Start HTTP server (simplified implementation)
			server := &HTTPServer{
				port:     port,
				analyzer: cli.analyzer,
			}

			return server.Start()
		},
	}

	return cmd
}

// createVersionCommand creates the version command
func (cli *CLI) createVersionCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "version",
		Short: "Show version information",
		Long:  `Display version and build information.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Printf("Fast-Context CLI v%s\n", cli.rootCmd.Version)
			fmt.Printf("Analyzer version: %s\n", cli.analyzer.GetVersion())
			fmt.Printf("Supported languages: %d\n", len(fastcontext.GetSupportedLanguages()))
			return nil
		},
	}

	return cmd
}

// Execute runs the CLI
func (cli *CLI) Execute() error {
	return cli.rootCmd.Execute()
}

// Helper methods for output formatting

func (cli *CLI) printSummary(result *fastcontext.AnalysisResult) {
	fmt.Printf("\n=== Analysis Summary ===\n")
	fmt.Printf("Files analyzed: %d\n", result.FileCount)
	fmt.Printf("Symbols found: %d\n", result.SymbolCount)
	fmt.Printf("Dependencies: %d\n", result.RelationshipCount)
	fmt.Printf("Languages: %s\n", formatList(result.Languages))
	fmt.Printf("Duration: %dms\n", result.DurationMs)
	fmt.Printf("Memory used: %d MB\n", result.MemoryUsed/(1024*1024))
}

func (cli *CLI) printSymbols(symbols []*fastcontext.Symbol) {
	fmt.Printf("Found %d symbols:\n", len(symbols))
	for _, symbol := range symbols {
		fmt.Printf("  %s (%s) - %s:%d-%d (complexity: %.1f)\n",
			symbol.Name, symbol.Kind, symbol.File, symbol.LineStart, symbol.LineEnd, symbol.Complexity)
	}
}

func (cli *CLI) printDependencies(deps []*fastcontext.Dependency) {
	fmt.Printf("Found %d dependencies:\n", len(deps))
	for _, dep := range deps {
		fmt.Printf("  %s -> %s (%s, strength: %.2f)\n",
			dep.From, dep.To, dep.Type, dep.Strength)
	}
}

func (cli *CLI) printConfig() {
	data, err := json.MarshalIndent(cli.config, "", "  ")
	if err != nil {
		fmt.Printf("Error marshaling config: %v\n", err)
		return
	}
	fmt.Println(string(data))
}

func (cli *CLI) initConfig() error {
	configPath := filepath.Join(cli.config.ProjectRoot, ".fast-context.yaml")
	configContent := `# Fast-Context Configuration
projectRoot: .
languages:
  - Go
  - Rust
  - Python
  - JavaScript
ignorePatterns:
  - node_modules/**
  - target/**
  - build/**
  - dist/**
  - vendor/**
  - __pycache__/**
  - .git/**
performance:
  maxMemoryMB: 1024
  maxConcurrentFiles: 50
  timeoutSeconds: 300
  cachePolicy: balanced
  enableParallel: true
  enableStreaming: true
  enableWatching: false
  analysisDepth: 3
enableProgress: true
enableMetrics: true
logLevel: info
maxFileSizeKB: 1024
maxFiles: 10000
`

	if err := os.WriteFile(configPath, []byte(configContent), 0644); err != nil {
		return fmt.Errorf("failed to create config file: %w", err)
	}

	fmt.Printf("Configuration file created: %s\n", configPath)
	return nil
}

func (cli *CLI) applyPreset(preset string) error {
	var err error
	switch preset {
	case "fast":
		cli.config, err = config.FastConfig(cli.config.ProjectRoot)
	case "balanced":
		cli.config, err = config.BalancedConfig(cli.config.ProjectRoot)
	case "thorough":
		cli.config, err = config.ThoroughConfig(cli.config.ProjectRoot)
	default:
		return fmt.Errorf("unknown preset: %s", preset)
	}

	if err != nil {
		return fmt.Errorf("failed to apply preset: %w", err)
	}

	// Update analyzer with new config
	if err := cli.analyzer.UpdateConfig(); err != nil {
		return fmt.Errorf("failed to update analyzer config: %w", err)
	}

	fmt.Printf("Applied preset: %s\n", preset)
	return nil
}

func formatList(items []string) string {
	if len(items) == 0 {
		return "none"
	}
	if len(items) == 1 {
		return items[0]
	}
	if len(items) == 2 {
		return items[0] + " and " + items[1]
	}
	result := ""
	for i, item := range items {
		if i > 0 {
			result += ", "
		}
		if i == len(items)-1 {
			result += "and "
		}
		result += item
	}
	return result
}

// HTTPServer is a simple HTTP server for web-based analysis
type HTTPServer struct {
	port     int
	analyzer *fastcontext.Analyzer
}

func (s *HTTPServer) Start() error {
	// This is a simplified implementation
	// In a real implementation, you would use a proper HTTP server like net/http
	fmt.Printf("HTTP server not implemented in this version\n")
	fmt.Printf("Would start server on port %d\n", s.port)
	return nil
}