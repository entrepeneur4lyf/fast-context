package main

import (
	"fmt"
	"log"
	"os"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: basic <project-path>")
		os.Exit(1)
	}

	projectPath := os.Args[1]

	// Create analyzer with balanced configuration
	analyzer, err := fastcontext.NewAnalyzer(
		config.WithProjectRoot(projectPath),
		config.WithProgress(true),
		config.WithMetrics(true),
	)
	if err != nil {
		log.Fatalf("Failed to create analyzer: %v", err)
	}

	fmt.Printf("Analyzing project: %s\n", projectPath)

	// Perform analysis
	result, err := analyzer.Analyze()
	if err != nil {
		log.Fatalf("Analysis failed: %v", err)
	}

	// Print results
	fmt.Printf("\n=== Analysis Results ===\n")
	fmt.Printf("Files analyzed: %d\n", result.FileCount)
	fmt.Printf("Symbols found: %d\n", result.SymbolCount)
	fmt.Printf("Dependencies: %d\n", result.RelationshipCount)
	fmt.Printf("Languages detected: %v\n", result.Languages)
	fmt.Printf("Analysis time: %dms\n", result.DurationMs)

	// Find all functions
	fmt.Printf("\n=== Functions ===\n")
	functions, err := analyzer.FindSymbolsByKind(fastcontext.SymbolKindFunction)
	if err != nil {
		log.Printf("Failed to find functions: %v", err)
	} else {
		for i, fn := range functions {
			if i >= 10 { // Limit output
				fmt.Printf("... and %d more functions\n", len(functions)-10)
				break
			}
			fmt.Printf("- %s (%s:%d)\n", fn.Name, fn.File, fn.LineStart)
		}
	}

	// Find dependencies for first symbol
	if len(result.Symbols) > 0 {
		firstSymbol := result.Symbols[0]
		fmt.Printf("\n=== Dependencies for %s ===\n", firstSymbol.Name)
		deps, err := analyzer.FindDependencies(firstSymbol.Name)
		if err != nil {
			log.Printf("Failed to find dependencies: %v", err)
		} else {
			for _, dep := range deps {
				fmt.Printf("- %s (%s)\n", dep.To, dependencyTypeToString(dep.Type))
			}
		}
	}

	fmt.Printf("\nAnalysis completed successfully!\n")
}

func dependencyTypeToString(depType fastcontext.DependencyType) string {
	switch depType {
	case fastcontext.DepTypeImports:
		return "imports"
	case fastcontext.DepTypeCalls:
		return "calls"
	case fastcontext.DepTypeInherits:
		return "inherits"
	case fastcontext.DepTypeImplements:
		return "implements"
	default:
		return "unknown"
	}
}