"use strict";
// TypeScript usage example for Fast-Context
// This demonstrates the auto-generated TypeScript types in action
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAnalyzer = createAnalyzer;
exports.processAnalysisResult = processAnalysisResult;
exports.analyzeSymbols = analyzeSymbols;
exports.setupFileWatching = setupFileWatching;
exports.configureExport = configureExport;
exports.safeAnalysis = safeAnalysis;
exports.generateProjectReport = generateProjectReport;
exports.runTypeScriptDemo = runTypeScriptDemo;
exports.isValidAnalysisResult = isValidAnalysisResult;
exports.isValidSymbol = isValidSymbol;
// Import the runtime exports
const index_js_1 = require("../index.js");
// Example 1: Basic Configuration with Type Safety
function createAnalyzer() {
    console.log('🚀 Fast-Context TypeScript Example\n');
    // Show version and supported languages with type safety
    console.log(`📦 Version: ${(0, index_js_1.getVersion)()}`);
    console.log(`🔧 Supported Languages: ${(0, index_js_1.getSupportedLanguages)().join(', ')}\n`);
    const config = {
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
        const isValid = (0, index_js_1.checkConfiguration)();
        console.log(`✅ Configuration check: ${isValid}`);
    }
    catch (error) {
        console.error(`❌ Configuration error: ${error}`);
        throw error;
    }
    return new index_js_1.FastContextAnalyzer(config);
}
// Example 2: Type-Safe Language Detection
function demonstrateLanguageDetection() {
    console.log('\n🔍 Language Detection Examples:');
    const testFiles = [
        'src/main.rs',
        'src/index.js',
        'src/app.ts',
        'src/main.py',
        'src/Main.java',
        'src/main.go',
        'README.md',
        'package.json'
    ];
    testFiles.forEach((file) => {
        try {
            const language = (0, index_js_1.detectLanguage)(file);
            console.log(`  ${file} -> ${language || 'unknown'}`);
        }
        catch (error) {
            console.log(`  ${file} -> Error: ${error}`);
        }
    });
}
// Example 3: Type-Safe Analysis Result Processing
function processAnalysisResult(result) {
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
function analyzeSymbols(symbols) {
    const functionSymbols = symbols.filter(s => s.kind === 'function');
    const classSymbols = symbols.filter(s => s.kind === 'class');
    console.log(`🔍 Symbol Analysis:`);
    console.log(`  Functions: ${functionSymbols.length}`);
    console.log(`  Classes: ${classSymbols.length}`);
    console.log(`  Total symbols: ${symbols.length}`);
    // Find symbol with longest name (as a proxy for complexity)
    const longestNameSymbol = symbols.reduce((max, current) => current.name.length > max.name.length ? current : max);
    console.log(`  Longest name: ${longestNameSymbol.name} (${longestNameSymbol.name.length} chars)`);
    console.log(`    File: ${longestNameSymbol.filePath}`);
    console.log(`    Position: Line ${longestNameSymbol.line}, Column ${longestNameSymbol.column}`);
    console.log(`    Language: ${longestNameSymbol.language}`);
    console.log(`    Scope: ${longestNameSymbol.scope}`);
    if (longestNameSymbol.signature) {
        console.log(`    Signature: ${longestNameSymbol.signature}`);
    }
    if (longestNameSymbol.documentation) {
        console.log(`    Documentation: ${longestNameSymbol.documentation.substring(0, 100)}...`);
    }
}
// Example 4: Type-Safe File Watching
function setupFileWatching(analyzer) {
    try {
        console.log('📁 Starting file watcher...');
        analyzer.startWatching();
        console.log('✅ File watcher started successfully');
        // Note: The actual file watching events would be handled internally
        // This is a simplified example showing how to start/stop watching
        // Simulate stopping the watcher after some time
        setTimeout(() => {
            try {
                analyzer.stopWatching();
                console.log('🛑 File watcher stopped');
            }
            catch {
                console.log('ℹ️ Watcher was not running or already stopped');
            }
        }, 5000);
    }
    catch (error) {
        console.error('❌ Failed to start file watcher:', error);
    }
}
// Example 6: Export Configuration with Type Safety
function configureExport() {
    return {
        format: 'json',
        outputPath: './analysis-output.json',
        includeSource: true,
        includeDocs: true,
        minify: false
    };
}
// Example 7: Error Handling with TypeScript
async function safeAnalysis() {
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
    }
    catch (error) {
        if (error instanceof Error) {
            console.error('❌ Analysis failed:', error.message);
        }
        else {
            console.error('❌ Unknown analysis error:', error);
        }
        return null;
    }
}
function generateProjectReport(analyzer) {
    const analysis = analyzer.analyze();
    if (!analysis)
        return null;
    // This would require additional query methods to be implemented
    // but shows how TypeScript types enable rich data modeling
    const topSymbols = []; // Would come from analyzer.findTopSymbols()
    const complexityReport = {
        averageComplexity: 0, // Would be calculated from symbols
        highComplexityCount: topSymbols.filter(s => s.name.length > 15).length, // Using name length as proxy
        fileComplexityMap: new Map()
    };
    return {
        analysis,
        topSymbols,
        complexityReport
    };
}
// Example 9: Main Demo Function
async function runTypeScriptDemo() {
    try {
        // Create analyzer with type safety
        const analyzer = createAnalyzer();
        console.log(`✅ Analyzer created for project: ${analyzer.constructor.name}`);
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
    }
    catch (error) {
        console.error(`\n💥 Demo failed: ${error}`);
        if (error instanceof Error) {
            console.error(error.stack);
        }
    }
}
// Example 10: Type Guards for Runtime Safety
function isValidAnalysisResult(result) {
    return (typeof result === 'object' &&
        typeof result.fileCount === 'number' &&
        typeof result.symbolCount === 'number' &&
        typeof result.relationshipCount === 'number' &&
        Array.isArray(result.languages) &&
        typeof result.durationMs === 'number');
}
function isValidSymbol(symbol) {
    return (typeof symbol === 'object' &&
        typeof symbol.name === 'string' &&
        typeof symbol.kind === 'string' &&
        typeof symbol.filePath === 'string' &&
        typeof symbol.line === 'number' &&
        typeof symbol.column === 'number' &&
        typeof symbol.scope === 'string' &&
        typeof symbol.language === 'string' &&
        (symbol.documentation === undefined || typeof symbol.documentation === 'string') &&
        (symbol.signature === undefined || typeof symbol.signature === 'string'));
}
// Run demo if this file is executed directly
if (require.main === module) {
    runTypeScriptDemo().catch(console.error);
}
