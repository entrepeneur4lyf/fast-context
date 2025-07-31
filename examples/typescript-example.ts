// TypeScript usage example for Fast-Context
// This demonstrates the auto-generated TypeScript types in action

import { 
    FastContextAnalyzer, 
    AnalyzerConfig, 
    AnalysisResultJs, 
    SymbolInfoJs,
    FileChangeBatchJs,
    QueryResultJs,
    StreamingOptionsJs,
    QueryChunkJs,
    ExportOptionsJs
} from '../index';

// Example 1: Basic Configuration with Type Safety
function createAnalyzer(): FastContextAnalyzer {
    const config: AnalyzerConfig = {
        projectRoot: process.cwd(),
        languages: ['javascript', 'typescript', 'rust'],
        ignorePatterns: ['node_modules/**', 'target/**', '.git/**'],
        enableCaching: true,
        cachePolicy: 'adaptive',
        enableWatching: true,
        maxFiles: 10000,
        parallelProcessing: true
    };

    return new FastContextAnalyzer(config);
}

// Example 2: Type-Safe Analysis Result Processing
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

// Example 5: Type-Safe Streaming Query (with proper callback typing)
function performStreamingQuery(analyzer: FastContextAnalyzer): void {
    const streamingOptions: StreamingOptionsJs = {
        enabled: true,
        chunkSize: 1000,
        includeProgress: true,
        chunkTimeoutMs: 5000
    };
    
    // Note: This method doesn't exist in current implementation 
    // but shows how it would work with TypeScript types
    // analyzer.findSymbolsStreaming('.*', streamingOptions, (chunk: QueryChunkJs) => {
    //     console.log(`📦 Chunk ${chunk.chunkIndex + 1}/${chunk.totalChunks}:`);
    //     console.log(`  Symbols: ${chunk.symbols.length}`);
    //     console.log(`  Progress: ${chunk.progress.toFixed(1)}%`);
    //     console.log(`  Processing Time: ${chunk.processingTimeMs}ms`);
    //     
    //     if (chunk.isLast) {
    //         console.log('✅ Streaming query completed');
    //     }
    // });
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
    console.log('🚀 Fast-Context TypeScript Demo\n');
    
    // Create analyzer with type safety
    const analyzer = createAnalyzer();
    
    // Perform analysis
    const result = await safeAnalysis();
    if (!result) {
        console.error('Failed to analyze project');
        return;
    }
    
    // Process results with type safety
    processAnalysisResult(result);
    
    // Set up file watching
    setupFileWatching(analyzer);
    
    // Configure export options
    const exportConfig = configureExport();
    console.log('\n📤 Export Configuration:', exportConfig);
    
    console.log('\n✅ Demo completed! File watching is active...');
    
    // Clean shutdown after demo
    setTimeout(() => {
        analyzer.stopWatching();
        console.log('🛑 File watching stopped');
    }, 30000); // 30 seconds
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