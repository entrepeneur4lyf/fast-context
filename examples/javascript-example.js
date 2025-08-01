// JavaScript usage example for Fast-Context
// This demonstrates the complete functionality in pure JavaScript

const { 
    FastContextAnalyzer, 
    getSupportedLanguages, 
    detectLanguage, 
    checkConfiguration, 
    getVersion 
} = require('../index.js');

// Example 1: Basic Setup and Information
function showBasicInfo() {
    console.log('🚀 Fast-Context JavaScript Example\n');
    
    // Show version and capabilities
    console.log(`📦 Version: ${getVersion()}`);
    console.log(`🔧 Supported Languages: ${getSupportedLanguages().join(', ')}\n`);
    
    // Check system configuration
    try {
        const configStatus = checkConfiguration();
        console.log(`✅ System Configuration: ${configStatus}\n`);
    } catch (error) {
        console.error(`❌ Configuration Error: ${error.message}\n`);
    }
}

// Example 2: Language Detection Demonstration
function demonstrateLanguageDetection() {
    console.log('🔍 Language Detection Examples:');
    
    const testFiles = [
        'src/main.rs',
        'src/index.js', 
        'src/app.ts',
        'src/main.py',
        'src/Main.java',
        'src/main.go',
        'src/main.cs',
        'src/main.swift',
        'src/main.php',
        'src/main.rb',
        'README.md',
        'package.json',
        'Cargo.toml',
        'requirements.txt',
        'pom.xml',
        'go.mod'
    ];
    
    testFiles.forEach(file => {
        try {
            const language = detectLanguage(file);
            console.log(`  ${file.padEnd(20)} -> ${language || 'unknown'}`);
        } catch (error) {
            console.log(`  ${file.padEnd(20)} -> Error: ${error.message}`);
        }
    });
    console.log();
}

// Example 3: Configuration Examples
function demonstrateConfigurations() {
    console.log('⚙️ Configuration Examples:\n');
    
    // Basic configuration
    const basicConfig = {
        projectRoot: process.cwd(),
        languages: ['javascript', 'typescript'],
        ignorePatterns: ['node_modules/**', '.git/**'],
        enableCaching: true,
        enableWatching: false,
        maxFiles: 1000,
        parallelProcessing: true
    };
    
    console.log('📋 Basic Configuration:');
    console.log(JSON.stringify(basicConfig, null, 2));
    
    // Advanced configuration
    const advancedConfig = {
        projectRoot: process.cwd(),
        languages: ['rust', 'javascript', 'typescript', 'python', 'java'],
        ignorePatterns: [
            'node_modules/**', 
            'target/**', 
            '.git/**',
            'dist/**',
            'build/**',
            '**/*.test.*',
            '**/*.spec.*'
        ],
        enableCaching: true,
        cachePolicy: 'adaptive',
        enableWatching: false,
        maxFiles: 10000,
        parallelProcessing: true,
        memoryLimit: 512, // MB
        analysisTimeout: 30000 // 30 seconds
    };
    
    console.log('\n🔧 Advanced Configuration:');
    console.log(JSON.stringify(advancedConfig, null, 2));
    
    return { basicConfig, advancedConfig };
}

// Example 4: Analyzer Creation and Basic Usage
function createAndTestAnalyzer(config) {
    console.log('\n🏗️ Creating Analyzer:');
    
    try {
        const analyzer = new FastContextAnalyzer(config);
        console.log('✅ FastContextAnalyzer created successfully');
        
        // Test basic analyzer properties
        console.log('📊 Analyzer Information:');
        console.log(`  Project Root: ${config.projectRoot}`);
        console.log(`  Languages: ${config.languages.join(', ')}`);
        console.log(`  Caching: ${config.enableCaching ? 'Enabled' : 'Disabled'}`);
        console.log(`  Max Files: ${config.maxFiles}`);
        
        return analyzer;
        
    } catch (error) {
        console.error(`❌ Failed to create analyzer: ${error.message}`);
        console.error(`   Stack: ${error.stack}`);
        return null;
    }
}

// Example 5: Project Analysis
function performProjectAnalysis(analyzer) {
    console.log('\n📈 Project Analysis:');
    
    if (!analyzer) {
        console.log('❌ No analyzer available for analysis');
        return null;
    }
    
    try {
        console.log('🔄 Analyzing project...');
        const startTime = Date.now();
        
        const result = analyzer.analyze();
        const endTime = Date.now();
        
        console.log('✅ Analysis completed!');
        console.log('\n📊 Analysis Results:');
        console.log(`  📁 Files Processed: ${result.fileCount}`);
        console.log(`  🔍 Symbols Found: ${result.symbolCount}`);
        console.log(`  🔗 Relationships: ${result.relationshipCount}`);
        console.log(`  🌐 Languages Detected: ${result.languages.join(', ')}`);
        console.log(`  ⏱️  Analysis Duration: ${result.durationMs}ms`);
        console.log(`  ⏱️  Total Time: ${endTime - startTime}ms`);
        
        if (result.memoryUsageMb) {
            console.log(`  💾 Memory Usage: ${result.memoryUsageMb}MB`);
        }
        
        if (result.cacheHitRate !== undefined) {
            console.log(`  🎯 Cache Hit Rate: ${(result.cacheHitRate * 100).toFixed(1)}%`);
        }
        
        return result;
        
    } catch (error) {
        console.error(`❌ Analysis failed: ${error.message}`);
        console.error(`   Stack: ${error.stack}`);
        return null;
    }
}

// Example 6: Symbol Analysis
function analyzeSymbols(analyzer) {
    console.log('\n🔍 Symbol Analysis:');
    
    if (!analyzer) {
        console.log('❌ No analyzer available for symbol analysis');
        return;
    }
    
    try {
        console.log('🔄 Searching for symbols...');

        // Search for different symbol types
        const symbolTypes = ['function', 'class', 'interface', 'module', 'variable'];
        
        symbolTypes.forEach(symbolType => {
            try {
                console.log(`\n  📋 Searching for ${symbolType}s...`);
                const symbols = analyzer.findSymbolsByKind(symbolType);
                console.log(`    ✅ Found ${symbols.symbols.length} ${symbolType}s`);

                if (symbols.symbols.length > 0) {
                    // Show first few examples
                    const examples = symbols.symbols.slice(0, 3);
                    examples.forEach((symbol, index) => {
                        console.log(`      ${index + 1}. ${symbol.name} (${symbol.filePath}:${symbol.startLine})`);
                    });
                    if (symbols.symbols.length > 3) {
                        console.log(`      ... and ${symbols.symbols.length - 3} more`);
                    }
                }
            } catch (error) {
                console.log(`    ❌ Error searching ${symbolType}s: ${error.message}`);
            }
        });
        
    } catch (error) {
        console.error(`❌ Symbol analysis failed: ${error.message}`);
    }
}

// Example 7: Error Handling Demonstration
function demonstrateErrorHandling() {
    console.log('\n🛡️ Error Handling Examples:');
    
    const invalidConfigs = [
        {
            name: 'Empty Project Root',
            config: { projectRoot: '' }
        },
        {
            name: 'Non-existent Path',
            config: { projectRoot: '/nonexistent/path/that/does/not/exist' }
        },
        {
            name: 'No Languages',
            config: { 
                projectRoot: process.cwd(),
                languages: [] 
            }
        },
        {
            name: 'Invalid Max Files',
            config: { 
                projectRoot: process.cwd(),
                maxFiles: -1 
            }
        },
        {
            name: 'Invalid Cache Policy',
            config: { 
                projectRoot: process.cwd(),
                cachePolicy: 'invalid_policy'
            }
        }
    ];
    
    invalidConfigs.forEach((test, index) => {
        console.log(`\n  ${index + 1}. Testing: ${test.name}`);
        try {
            const _analyzer = new FastContextAnalyzer(test.config);
            console.log(`    ⚠️ Unexpectedly succeeded (analyzer created)`);
            
            // Try to use it
            try {
                _analyzer.analyze();
                console.log(`    ⚠️ Analysis also succeeded unexpectedly`);
            } catch (analysisError) {
                console.log(`    ✅ Analysis correctly failed: ${analysisError.message}`);
            }
            
        } catch (error) {
            console.log(`    ✅ Correctly caught error: ${error.message}`);
        }
    });
}

// Example 8: Performance Benchmarking
function performanceBenchmark(analyzer) {
    console.log('\n⚡ Performance Benchmark:');
    
    if (!analyzer) {
        console.log('❌ No analyzer available for benchmarking');
        return;
    }
    
    const iterations = 3;
    const times = [];
    
    console.log(`🔄 Running ${iterations} analysis iterations...`);
    
    for (let i = 0; i < iterations; i++) {
        try {
            const startTime = Date.now();
            const result = analyzer.analyze();
            const endTime = Date.now();
            
            const duration = endTime - startTime;
            times.push(duration);
            
            console.log(`  Iteration ${i + 1}: ${duration}ms (${result.symbolCount} symbols)`);
            
        } catch (error) {
            console.log(`  Iteration ${i + 1}: Failed - ${error.message}`);
        }
    }
    
    if (times.length > 0) {
        const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
        const minTime = Math.min(...times);
        const maxTime = Math.max(...times);
        
        console.log('\n📊 Performance Summary:');
        console.log(`  Average: ${avgTime.toFixed(1)}ms`);
        console.log(`  Fastest: ${minTime}ms`);
        console.log(`  Slowest: ${maxTime}ms`);
        console.log(`  Variance: ${(maxTime - minTime)}ms`);
    }
}

// Example 9: Export Configuration
function demonstrateExportOptions() {
    console.log('\n📤 Export Configuration Examples:');

    const exportConfigs = [
        {
            name: 'Basic JSON Export',
            config: {
                format: 'json',
                prettyPrint: true,
                includeDetails: false
            }
        },
        {
            name: 'Detailed JSON Export',
            config: {
                format: 'json',
                prettyPrint: true,
                includeDetails: true,
                includeRelationships: true,
                maxSymbols: 10000
            }
        },
        {
            name: 'Streaming Export',
            config: {
                format: 'json',
                streaming: true,
                chunkSize: 1000,
                includeProgress: true
            }
        },
        {
            name: 'LSP Format Export',
            config: {
                format: 'lsp',
                includeDetails: true,
                includeRelationships: false
            }
        }
    ];

    exportConfigs.forEach(example => {
        console.log(`\n  📋 ${example.name}:`);
        console.log(JSON.stringify(example.config, null, 4));
    });
}

// Example 10: Main Demo Function
async function runJavaScriptDemo() {
    try {
        console.log('=' .repeat(60));
        console.log('🚀 FAST-CONTEXT JAVASCRIPT COMPREHENSIVE DEMO');
        console.log('=' .repeat(60));

        // 1. Basic information
        showBasicInfo();

        // 2. Language detection
        demonstrateLanguageDetection();

        // 3. Configuration examples
        const { basicConfig, advancedConfig } = demonstrateConfigurations();

        // 4. Create analyzer
        const analyzer = createAndTestAnalyzer(advancedConfig);

        // 5. Perform analysis
        const analysisResult = performProjectAnalysis(analyzer);

        // 6. Symbol analysis
        analyzeSymbols(analyzer);

        // 7. Error handling
        demonstrateErrorHandling();

        // 8. Performance benchmark
        if (analyzer && analysisResult) {
            performanceBenchmark(analyzer);
        }

        // 9. Export options
        demonstrateExportOptions();

        console.log('\n' + '=' .repeat(60));
        console.log('✅ JavaScript Demo completed successfully!');
        console.log('🎯 This example demonstrates comprehensive Fast-Context usage');
        console.log('=' .repeat(60));

        return {
            analyzer,
            analysisResult,
            success: true
        };

    } catch (error) {
        console.error('\n' + '=' .repeat(60));
        console.error('💥 Demo failed with error:');
        console.error(`   ${error.message}`);
        console.error(`   Stack: ${error.stack}`);
        console.error('=' .repeat(60));

        return {
            analyzer: null,
            analysisResult: null,
            success: false,
            error: error.message
        };
    }
}

// Export functions for use in other modules
module.exports = {
    showBasicInfo,
    demonstrateLanguageDetection,
    demonstrateConfigurations,
    createAndTestAnalyzer,
    performProjectAnalysis,
    analyzeSymbols,
    demonstrateErrorHandling,
    performanceBenchmark,
    demonstrateExportOptions,
    runJavaScriptDemo
};

// Run demo if this file is executed directly
if (require.main === module) {
    runJavaScriptDemo()
        .then(result => {
            if (result.success) {
                console.log('\n🎉 Demo execution completed successfully!');
                process.exit(0);
            } else {
                console.log('\n❌ Demo execution failed!');
                process.exit(1);
            }
        })
        .catch(error => {
            console.error('\n💥 Unexpected demo error:', error);
            process.exit(1);
        });
}
