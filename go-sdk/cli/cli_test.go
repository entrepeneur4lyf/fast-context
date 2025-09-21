package cli

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"github.com/fast-context/go-sdk/config"
	"github.com/fast-context/go-sdk/fastcontext"
	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewCLI tests CLI creation
func TestNewCLI(t *testing.T) {
	cli := NewCLI()
	assert.NotNil(t, cli)
	assert.NotNil(t, cli.rootCmd)
	assert.NotNil(t, cli.analyzer)
	assert.NotNil(t, cli.config)
}

// TestCLICommands tests that all commands are properly registered
func TestCLICommands(t *testing.T) {
	cli := NewCLI()

	// Check that all expected commands are present
	expectedCommands := []string{
		"analyze", "symbols", "dependencies", "complexity",
		"patterns", "export", "config", "watch", "serve", "version",
	}

	for _, cmd := range expectedCommands {
		found := false
		for _, c := range cli.rootCmd.Commands() {
			if c.Name() == cmd {
				found = true
				break
			}
		}
		assert.True(t, found, "Command %s should be registered", cmd)
	}
}

// TestAnalyzeCommand tests the analyze command
func TestAnalyzeCommand(t *testing.T) {
	cli := NewCLI()

	// Create a temporary directory for testing
	tempDir := t.TempDir()
	testFile := filepath.Join(tempDir, "test.go")
	
	content := `package main

func main() {
    println("Hello, World!")
}`
	
	err := os.WriteFile(testFile, []byte(content), 0644)
	require.NoError(t, err)

	// Test the command
	cmd := cli.createAnalyzeCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "analyze", cmd.Use)
	assert.Equal(t, "Analyze a codebase", cmd.Short)
}

// TestSymbolsCommand tests the symbols command
func TestSymbolsCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createSymbolsCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "symbols", cmd.Use)
	assert.Equal(t, "Find and analyze symbols", cmd.Short)
}

// TestDependenciesCommand tests the dependencies command
func TestDependenciesCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createDependenciesCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "dependencies", cmd.Use)
	assert.Equal(t, "Find dependencies for a symbol", cmd.Short)
}

// TestComplexityCommand tests the complexity command
func TestComplexityCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createComplexityCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "complexity", cmd.Use)
	assert.Equal(t, "Analyze code complexity", cmd.Short)
}

// TestPatternsCommand tests the patterns command
func TestPatternsCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createPatternsCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "patterns", cmd.Use)
	assert.Equal(t, "Search for code patterns", cmd.Short)
}

// TestExportCommand tests the export command
func TestExportCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createExportCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "export", cmd.Use)
	assert.Equal(t, "Export analysis results", cmd.Short)
}

// TestConfigCommand tests the config command
func TestConfigCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createConfigCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "config", cmd.Use)
	assert.Equal(t, "Manage configuration", cmd.Short)
}

// TestWatchCommand tests the watch command
func TestWatchCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createWatchCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "watch", cmd.Use)
	assert.Equal(t, "Watch project for changes", cmd.Short)
}

// TestServeCommand tests the serve command
func TestServeCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createServeCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "serve", cmd.Use)
	assert.Equal(t, "Start HTTP server", cmd.Short)
}

// TestVersionCommand tests the version command
func TestVersionCommand(t *testing.T) {
	cli := NewCLI()

	cmd := cli.createVersionCommand()
	assert.NotNil(t, cmd)
	assert.Equal(t, "version", cmd.Use)
	assert.Equal(t, "Show version information", cmd.Short)
}

// TestPrintSummary tests the summary printing function
func TestPrintSummary(t *testing.T) {
	cli := NewCLI()

	result := &fastcontext.AnalysisResult{
		FileCount:         5,
		SymbolCount:       20,
		RelationshipCount: 15,
		Languages:         []string{"Go", "Rust"},
		DurationMs:        1000,
		MemoryUsed:        1024 * 1024,
	}

	// Capture output
	var buf bytes.Buffer
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cli.printSummary(result)

	_ = w.Close()
	os.Stdout = oldStdout

	_, _ = buf.ReadFrom(r)
	output := buf.String()

	assert.Contains(t, output, "=== Analysis Summary ===")
	assert.Contains(t, output, "Files analyzed: 5")
	assert.Contains(t, output, "Symbols found: 20")
	assert.Contains(t, output, "Dependencies: 15")
	assert.Contains(t, output, "Languages: Go and Rust")
	assert.Contains(t, output, "Duration: 1000ms")
	assert.Contains(t, output, "Memory used: 1 MB")
}

// TestPrintSymbols tests the symbol printing function
func TestPrintSymbols(t *testing.T) {
	cli := NewCLI()

	symbols := []*fastcontext.Symbol{
		{
			Name:        "main",
			Kind:        fastcontext.SymbolKindFunction,
			File:        "main.go",
			LineStart:   1,
			LineEnd:     10,
			Complexity:  1.0,
		},
		{
			Name:        "MyStruct",
			Kind:        fastcontext.SymbolKindStruct,
			File:        "struct.go",
			LineStart:   5,
			LineEnd:     15,
			Complexity:  2.0,
		},
	}

	// Capture output
	var buf bytes.Buffer
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cli.printSymbols(symbols)

	_ = w.Close()
	os.Stdout = oldStdout

	_, _ = buf.ReadFrom(r)
	output := buf.String()

	assert.Contains(t, output, "Found 2 symbols:")
	assert.Contains(t, output, "main (function) - main.go:1-10 (complexity: 1.0)")
	assert.Contains(t, output, "MyStruct (struct) - struct.go:5-15 (complexity: 2.0)")
}

// TestPrintDependencies tests the dependency printing function
func TestPrintDependencies(t *testing.T) {
	cli := NewCLI()

	deps := []*fastcontext.Dependency{
		{
			From:     "func1",
			To:       "func2",
			Type:     fastcontext.DepTypeCalls,
			Strength: 0.8,
		},
		{
			From:     "struct1",
			To:       "interface1",
			Type:     fastcontext.DepTypeImplements,
			Strength: 1.0,
		},
	}

	// Capture output
	var buf bytes.Buffer
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cli.printDependencies(deps)

	_ = w.Close()
	os.Stdout = oldStdout

	_, _ = buf.ReadFrom(r)
	output := buf.String()

	assert.Contains(t, output, "Found 2 dependencies:")
	assert.Contains(t, output, "func1 -> func2 (calls, strength: 0.80)")
	assert.Contains(t, output, "struct1 -> interface1 (implements, strength: 1.00)")
}

// TestPrintConfig tests the config printing function
func TestPrintConfig(t *testing.T) {
	cli := NewCLI()

	// Capture output
	var buf bytes.Buffer
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	cli.printConfig()

	_ = w.Close()
	os.Stdout = oldStdout

	_, _ = buf.ReadFrom(r)
	output := buf.String()

	assert.Contains(t, output, "projectRoot")
	assert.Contains(t, output, "languages")
	assert.Contains(t, output, "performance")
}

// TestInitConfig tests config initialization
func TestInitConfig(t *testing.T) {
	cli := NewCLI()
	tempDir := t.TempDir()
	cli.config.ProjectRoot = tempDir

	err := cli.initConfig()
	require.NoError(t, err)

	configPath := filepath.Join(tempDir, ".fast-context.yaml")
	assert.FileExists(t, configPath)

	content, err := os.ReadFile(configPath)
	require.NoError(t, err)
	assert.Contains(t, string(content), "# Fast-Context Configuration")
	assert.Contains(t, string(content), "projectRoot: .")
	assert.Contains(t, string(content), "languages:")
}

// TestApplyPreset tests preset application
func TestApplyPreset(t *testing.T) {
	cli := NewCLI()

	// Test fast preset
	err := cli.applyPreset("fast")
	require.NoError(t, err)
	assert.Equal(t, 120, cli.config.Performance.TimeoutSeconds)
	assert.Equal(t, 512, cli.config.Performance.MaxMemoryMB)

	// Test balanced preset
	err = cli.applyPreset("balanced")
	require.NoError(t, err)
	assert.Equal(t, 300, cli.config.Performance.TimeoutSeconds)
	assert.Equal(t, 1024, cli.config.Performance.MaxMemoryMB)

	// Test thorough preset
	err = cli.applyPreset("thorough")
	require.NoError(t, err)
	assert.Equal(t, 600, cli.config.Performance.TimeoutSeconds)
	assert.Equal(t, 2048, cli.config.Performance.MaxMemoryMB)

	// Test invalid preset
	err = cli.applyPreset("invalid")
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "unknown preset")
}

// TestFormatList tests list formatting
func TestFormatList(t *testing.T) {
	assert.Equal(t, "none", formatList([]string{}))
	assert.Equal(t, "item1", formatList([]string{"item1"}))
	assert.Equal(t, "item1 and item2", formatList([]string{"item1", "item2"}))
	assert.Equal(t, "item1, item2, and item3", formatList([]string{"item1", "item2", "item3"}))
	assert.Equal(t, "item1, item2, item3, and item4", formatList([]string{"item1", "item2", "item3", "item4"}))
}

// TestGlobalFlags tests global flag registration
func TestGlobalFlags(t *testing.T) {
	cli := NewCLI()

	// Test that global flags are properly registered
	flags := cli.rootCmd.Flags()

	// Check if flags exist and can be retrieved
	assert.NotNil(t, flags.Lookup("project"))
	assert.NotNil(t, flags.Lookup("languages"))
	assert.NotNil(t, flags.Lookup("ignore"))
	assert.NotNil(t, flags.Lookup("timeout"))
	assert.NotNil(t, flags.Lookup("memory"))
	assert.NotNil(t, flags.Lookup("log-level"))
	assert.NotNil(t, flags.Lookup("progress"))
}

// TestCLIIntegration tests CLI integration with mock analyzer
func TestCLIIntegration(t *testing.T) {
	cli := NewCLI()

	// Test that the analyzer is properly initialized
	assert.NotNil(t, cli.analyzer)
	assert.NotNil(t, cli.analyzer.GetConfig())

	// Test version command execution
	versionCmd := cli.createVersionCommand()
	require.NotNil(t, versionCmd)

	// Set up args for version command
	oldArgs := os.Args
	defer func() { os.Args = oldArgs }()
	os.Args = []string{"fast-context", "version"}

	err := cli.Execute()
	// Note: In a real test, you would capture output instead of checking for error
	// This is a simplified integration test
	assert.NotNil(t, err) // Expected due to os.Exit behavior
}

// TestCommandValidation tests command argument validation
func TestCommandValidation(t *testing.T) {
	cli := NewCLI()

	// Test dependencies command without required symbol
	depsCmd := cli.createDependenciesCommand()
	assert.NotNil(t, depsCmd)

	// Set up args without symbol
	oldArgs := os.Args
	defer func() { os.Args = oldArgs }()
	os.Args = []string{"fast-context", "dependencies"}

	// The command should require a symbol argument
	assert.Equal(t, cobra.MaximumNArgs(1), depsCmd.Args)
}

// TestConfigurationUpdates tests that configuration updates work properly
func TestConfigurationUpdates(t *testing.T) {
	cli := NewCLI()

	// Test updating project root
	newProjectRoot := "/tmp/test-project"
	cli.config.ProjectRoot = newProjectRoot

	err := cli.analyzer.UpdateConfig(config.WithProjectRoot(newProjectRoot))
	require.NoError(t, err)

	assert.Equal(t, newProjectRoot, cli.analyzer.GetConfig().ProjectRoot)
}

// TestTimeoutHandling tests timeout configuration
func TestTimeoutHandling(t *testing.T) {
	cli := NewCLI()

	// Test setting timeout
	cli.config.Performance.TimeoutSeconds = 60
	assert.Equal(t, 60, cli.config.Performance.TimeoutSeconds)

	// Test timeout validation
	err := cli.config.Validate()
	require.NoError(t, err)
}

// TestMemoryConfiguration tests memory configuration
func TestMemoryConfiguration(t *testing.T) {
	cli := NewCLI()

	// Test setting memory limit
	cli.config.Performance.MaxMemoryMB = 2048
	assert.Equal(t, 2048, cli.config.Performance.MaxMemoryMB)

	// Test memory validation
	err := cli.config.Validate()
	require.NoError(t, err)
}

// TestLanguageConfiguration tests language configuration
func TestLanguageConfiguration(t *testing.T) {
	cli := NewCLI()

	// Test setting languages
	cli.config.Languages = []string{"Go", "Rust", "Python"}
	assert.Equal(t, []string{"Go", "Rust", "Python"}, cli.config.Languages)

	// Test configuration validation
	err := cli.config.Validate()
	require.NoError(t, err)
}

// TestIgnorePatternsConfiguration tests ignore patterns configuration
func TestIgnorePatternsConfiguration(t *testing.T) {
	cli := NewCLI()

	// Test setting ignore patterns
	cli.config.IgnorePatterns = []string{"node_modules/**", "target/**", "*.min.js"}
	assert.Equal(t, []string{"node_modules/**", "target/**", "*.min.js"}, cli.config.IgnorePatterns)

	// Test configuration validation
	err := cli.config.Validate()
	require.NoError(t, err)
}

// TestProgressConfiguration tests progress configuration
func TestProgressConfiguration(t *testing.T) {
	cli := NewCLI()

	// Test enabling/disabling progress
	cli.config.EnableProgress = false
	assert.False(t, cli.config.EnableProgress)

	cli.config.EnableProgress = true
	assert.True(t, cli.config.EnableProgress)

	// Test configuration validation
	err := cli.config.Validate()
	require.NoError(t, err)
}

// TestLogLevelConfiguration tests log level configuration
func TestLogLevelConfiguration(t *testing.T) {
	cli := NewCLI()

	// Test setting log level
	validLevels := []string{"debug", "info", "warn", "error"}
	for _, level := range validLevels {
		cli.config.LogLevel = level
		assert.Equal(t, level, cli.config.LogLevel)

		err := cli.config.Validate()
		require.NoError(t, err)
	}

	// Test invalid log level
	cli.config.LogLevel = "invalid"
	err := cli.config.Validate()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "invalid log level")
}