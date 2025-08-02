// TypeScript usage example for Fast-Context
// This demonstrates the auto-generated TypeScript types in action

// Import the runtime exports
import {
    FastContextAnalyzer,
    getSupportedLanguages,
    detectLanguage,
    checkConfiguration,
    getVersion
} from '../index.js';

// Import types for TypeScript checking (these are compile-time only)
import type {
    AnalyzerConfig,
    AnalysisResultJs,
    SymbolInfoJs,
    FileChangeBatchJs,
    ExportOptionsJs
} from '../index.js';

// Example 1: Basic Configuration with Type Safety
function createAnalyzer(): FastContextAnalyzer {
    console.log('🚀 Fast-Context TypeScript Example\n');

    // Show version and supported languages with type safety
    console.log(`📦 Version: ${getVersion()}`);
    console.log(`🔧 Supported Languages: ${getSupportedLanguages().join(', ')}\n`);

    const config: AnalyzerConfig = {
        projectRoot: process.cwd(),
        languages: ['javascript', 'typescript', 'rust'],
        ignorePatterns: ['node_modules/**', 'target/**', '.git/**'],
        enableCaching: true,
        cachePolicy: 'adaptive',
        enableWatching: false, // Disable for this example
        maxFiles: 1000,
        parallelProcessing: true
    };

    console.log('⚙️ Configuration:');
    console.log(JSON.stringify(config, null, 2));

    // Type-safe configuration validation
    try {
        const isValid = checkConfiguration();
        console.log(`✅ Configuration check: ${isValid}`);
    } catch (error) {
        console.error(`❌ Configuration error: ${error}`);
        throw error;
    }

    return new FastContextAnalyzer(config);
}

// Example 2: Type-Safe Language Detection
function demonstrateLanguageDetection(): void {
    console.log('\n🔍 Language Detection Examples:');

    const testFiles: string[] = [
        'src/main.rs',
        'src/index.js',
        'src/app.ts',
        'src/main.py',
        'src/Main.java',
        'src/main.go',
        'README.md',
        'package.json'
    ];

    testFiles.forEach((file: string) => {
        try {
            const language: string | null = detectLanguage(file);
            console.log(`  ${file} -> ${language || 'unknown'}`);
        } catch (error) {
            console.log(`  ${file} -> Error: ${error}`);
        }
    });
}

// Example 3: Type-Safe Analysis Result Processing
function processAnalysisResult(result: AnalysisResultJs): void {
    console.log(`📊 Analysis Results:`);
    console.log(`  Files: ${result.fileCount}`);
    console.log(`  Symbols: ${result.symbolCount}`);
    console.log(`  Relationships: ${result.relationshipCount}`);
    console.log(`  Languages: ${result.languages.join(', ')}`);
    console.log(`  Duration: ${result.durationMs}ms`);

    if (result.memoryUsageMb) {
        console.log(`  Memory: ${result.memoryUsageMb}MB`);
    }
}

// Example 3: Type-Safe Symbol Processing
function analyzeSymbols(symbols: SymbolInfoJs[]): void {
    const functionSymbols = symbols.filter(s => s.kind === 'function');
    const complexSymbols = symbols.filter(s => s.complexity > 10);
    
    console.log(`🔍 Symbol Analysis:`);
    console.log(`  Functions: ${functionSymbols.length}`);
    console.log(`  Complex symbols (>10): ${complexSymbols.length}`);
    
    // Find most complex symbol
    const mostComplex = symbols.reduce((max, current) => 
        current.complexity > max.complexity ? current : max
    );
    
    console.log(`  Most complex: ${mostComplex.name} (${mostComplex.complexity})`);
    console.log(`    File: ${mostComplex.filePath}`);
    console.log(`    Lines: ${mostComplex.startLine}-${mostComplex.endLine}`);
    
    if (mostComplex.signature) {
        console.log(`    Signature: ${mostComplex.signature}`);
    }
}

// Example 4: Type-Safe File Watching
function setupFileWatching(analyzer: FastContextAnalyzer): void {
    analyzer.startWatching((changeBatch: FileChangeBatchJs) => {
        console.log(`📁 File Changes Detected:`);
        console.log(`  Count: ${changeBatch.changeCount}`);
        console.log(`  Impact: ${changeBatch.impactLevel}`);
        console.log(`  Needs Reanalysis: ${changeBatch.requiresReanalysis}`);
        
        changeBatch.changes.forEach((change, index) => {
            console.log(`  ${index + 1}. ${change.changeType.toUpperCase()}: ${change.filePath}`);
            
            if (change.language) {
                console.log(`     Language: ${change.language}`);
            }
            
            if (change.oldPath) {
                console.log(`     From: ${change.oldPath}`);
            }
        });
        
        // Type-safe reanalysis trigger
        if (changeBatch.requiresReanalysis && changeBatch.impactLevel !== 'low') {
            console.log('🔄 Triggering reanalysis...');
            const newResult = analyzer.analyze();
            processAnalysisResult(newResult);
        }
    });
}


// Example 6: Export Configuration with Type Safety
function configureExport(): ExportOptionsJs {
    return {
        prettyPrint: true,
        includeDetails: true,
        includeRelationships: true,
        maxSymbols: 50000,
        format: 'json',
        streaming: true
    };
}

// Example 7: Error Handling with TypeScript
async function safeAnalysis(): Promise<AnalysisResultJs | null> {
    try {
        const analyzer = createAnalyzer();
        
        // TypeScript ensures we pass the right type
        const result = analyzer.analyze();
        
        // Type checking ensures we handle all required fields
        if (result.fileCount === 0) {
            console.warn('⚠️ No files found in project');
            return null;
        }
        
        return result;
    } catch (error) {
        if (error instanceof Error) {
            console.error('❌ Analysis failed:', error.message);
        } else {
            console.error('❌ Unknown analysis error:', error);
        }
        return null;
    }
}

// Example 8: Complex Type Combinations
interface ProjectStats {
    analysis: AnalysisResultJs;
    topSymbols: SymbolInfoJs[];
    complexityReport: {
        averageComplexity: number;
        highComplexityCount: number;
        fileComplexityMap: Map<string, number>;
    };
}

function generateProjectReport(analyzer: FastContextAnalyzer): ProjectStats | null {
    const analysis = analyzer.analyze();
    
    if (!analysis) return null;
    
    // This would require additional query methods to be implemented
    // but shows how TypeScript types enable rich data modeling
    const topSymbols: SymbolInfoJs[] = []; // Would come from analyzer.findTopSymbols()
    
    const complexityReport = {
        averageComplexity: 0, // Would be calculated from symbols
        highComplexityCount: topSymbols.filter(s => s.complexity > 15).length,
        fileComplexityMap: new Map<string, number>()
    };
    
    return {
        analysis,
        topSymbols,
        complexityReport
    };
}

// Example 9: Main Demo Function
async function runTypeScriptDemo(): Promise<void> {
    try {
        // Create analyzer with type safety
        const _analyzer = createAnalyzer();

        // Demonstrate language detection
        demonstrateLanguageDetection();

        // Show export configuration
        const exportConfig = configureExport();
        console.log('\n📤 Export Configuration:');
        console.log(JSON.stringify(exportConfig, null, 2));

        // Demonstrate error handling
        console.log('\n🛡️ Error Handling Examples:');
        const result = await safeAnalysis();
        if (result) {
            processAnalysisResult(result);
        }

        console.log('\n✅ TypeScript Demo completed successfully!');
        console.log('🎯 This example demonstrates type-safe usage of Fast-Context');

    } catch (error) {
        console.error(`\n💥 Demo failed: ${error}`);
        if (error instanceof Error) {
            console.error(error.stack);
        }
    }
}

// Example 10: Type Guards for Runtime Safety
function isValidAnalysisResult(result: any): result is AnalysisResultJs {
    return (
        typeof result === 'object' &&
        typeof result.fileCount === 'number' &&
        typeof result.symbolCount === 'number' &&
        typeof result.relationshipCount === 'number' &&
        Array.isArray(result.languages) &&
        typeof result.durationMs === 'number'
    );
}

function isValidSymbol(symbol: any): symbol is SymbolInfoJs {
    return (
        typeof symbol === 'object' &&
        typeof symbol.name === 'string' &&
        typeof symbol.qualifiedName === 'string' &&
        typeof symbol.kind === 'string' &&
        typeof symbol.filePath === 'string' &&
        typeof symbol.language === 'string' &&
        typeof symbol.startLine === 'number' &&
        typeof symbol.endLine === 'number' &&
        typeof symbol.complexity === 'number' &&
        Array.isArray(symbol.dependencies) &&
        Array.isArray(symbol.dependents) &&
        Array.isArray(symbol.modifiers)
    );
}

// Export for use in other modules
export {
    createAnalyzer,
    processAnalysisResult,
    analyzeSymbols,
    setupFileWatching,
    configureExport,
    safeAnalysis,
    generateProjectReport,
    runTypeScriptDemo,
    isValidAnalysisResult,
    isValidSymbol
};

// Run demo if this file is executed directly
if (require.main === module) {
    runTypeScriptDemo().catch(console.error);
}