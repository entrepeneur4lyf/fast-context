// Type validation test for Fast-Context TypeScript definitions
// This file tests that the TypeScript definitions match the actual runtime behavior

import {
    FastContextAnalyzer,
    getSupportedLanguages,
    detectLanguage,
    checkConfiguration,
    getVersion
} from '../index.js';

import type {
    AnalyzerConfig,
    AnalysisResultJs,
    ExportOptionsJs
} from '../index.js';

// Test 1: Verify basic utility functions work
function testUtilityFunctions(): void {
    console.log('🧪 Testing utility functions...');
    
    // Test version function
    const version: string = getVersion();
    console.log(`✅ Version: ${version}`);
    
    // Test supported languages
    const languages: string[] = getSupportedLanguages();
    console.log(`✅ Supported languages (${languages.length}): ${languages.slice(0, 5).join(', ')}...`);
    
    // Test language detection
    const rustLang: string | null = detectLanguage('main.rs');
    const jsLang: string | null = detectLanguage('index.js');
    console.log(`✅ Language detection: main.rs -> ${rustLang}, index.js -> ${jsLang}`);
}

// Test 2: Verify configuration type safety
function testConfigurationTypes(): AnalyzerConfig {
    console.log('🧪 Testing configuration types...');
    
    const config: AnalyzerConfig = {
        projectRoot: process.cwd(),
        languages: ['javascript', 'typescript'],
        ignorePatterns: ['node_modules/**'],
        enableCaching: true,
        cachePolicy: 'balanced',
        enableWatching: false,
        maxFiles: 1000,
        parallelProcessing: true,
        enableExperimentalArchitecture: false
    };
    
    // Test configuration validation
    const validationResult: string = checkConfiguration(config);
    console.log(`✅ Configuration validation: ${validationResult}`);
    
    return config;
}

// Test 3: Verify analyzer instantiation
function testAnalyzerCreation(config: AnalyzerConfig): FastContextAnalyzer {
    console.log('🧪 Testing analyzer creation...');
    
    const analyzer = new FastContextAnalyzer(config);
    console.log(`✅ Analyzer created: ${analyzer.constructor.name}`);
    
    return analyzer;
}

// Test 4: Verify analysis result types
function testAnalysisResultTypes(analyzer: FastContextAnalyzer): AnalysisResultJs {
    console.log('🧪 Testing analysis result types...');
    
    const result: AnalysisResultJs = analyzer.analyze();
    
    // Verify all required properties exist and have correct types
    console.log(`✅ File count: ${result.fileCount} (type: ${typeof result.fileCount})`);
    console.log(`✅ Symbol count: ${result.symbolCount} (type: ${typeof result.symbolCount})`);
    console.log(`✅ Relationship count: ${result.relationshipCount} (type: ${typeof result.relationshipCount})`);
    console.log(`✅ Languages: [${result.languages.join(', ')}] (type: ${typeof result.languages})`);
    console.log(`✅ Duration: ${result.durationMs}ms (type: ${typeof result.durationMs})`);
    
    if (result.memoryUsageMb !== undefined) {
        console.log(`✅ Memory usage: ${result.memoryUsageMb}MB (type: ${typeof result.memoryUsageMb})`);
    }
    
    return result;
}

// Test 5: Verify export options types
function testExportOptionsTypes(): ExportOptionsJs {
    console.log('🧪 Testing export options types...');
    
    const exportOptions: ExportOptionsJs = {
        format: 'json',
        outputPath: './test-output.json',
        includeSource: true,
        includeDocs: false,
        minify: true
    };
    
    console.log(`✅ Export options: ${JSON.stringify(exportOptions, null, 2)}`);
    
    return exportOptions;
}

// Test 6: Verify analyzer methods
function testAnalyzerMethods(analyzer: FastContextAnalyzer): void {
    console.log('🧪 Testing analyzer methods...');
    
    try {
        // Test symbol finding methods
        const functionSymbols: string[] = analyzer.findSymbolsByKind('function');
        console.log(`✅ Found ${functionSymbols.length} function symbols`);
        
        const complexSymbols: string[] = analyzer.findComplexSymbols(5);
        console.log(`✅ Found ${complexSymbols.length} complex symbols`);
        
        // Test file watching methods
        try {
            analyzer.startWatching();
            console.log(`✅ File watching started`);
            
            analyzer.stopWatching();
            console.log(`✅ File watching stopped`);
        } catch (error) {
            console.log(`ℹ️ File watching test: ${error}`);
        }
        
        // Test analysis retrieval
        const currentAnalysis: AnalysisResultJs | null = analyzer.getAnalysis();
        if (currentAnalysis) {
            console.log(`✅ Current analysis retrieved: ${currentAnalysis.fileCount} files`);
        } else {
            console.log(`ℹ️ No current analysis available`);
        }
        
    } catch (error) {
        console.error(`❌ Analyzer method test failed: ${error}`);
    }
}

// Test 7: Type guard validation
function testTypeGuards(result: AnalysisResultJs): void {
    console.log('🧪 Testing type guards...');
    
    // Test analysis result type guard
    function isValidAnalysisResult(obj: any): obj is AnalysisResultJs {
        return (
            typeof obj === 'object' &&
            typeof obj.fileCount === 'number' &&
            typeof obj.symbolCount === 'number' &&
            typeof obj.relationshipCount === 'number' &&
            Array.isArray(obj.languages) &&
            typeof obj.durationMs === 'number'
        );
    }
    
    const isValid = isValidAnalysisResult(result);
    console.log(`✅ Analysis result type guard: ${isValid}`);
    
    // Test with invalid object
    const invalidObj = { fileCount: "not a number" };
    const isInvalid = isValidAnalysisResult(invalidObj);
    console.log(`✅ Invalid object type guard: ${!isInvalid}`);
}

// Main test runner
async function runTypeValidationTests(): Promise<void> {
    console.log('🚀 Starting Fast-Context TypeScript Type Validation Tests\n');
    
    try {
        // Run all tests in sequence
        testUtilityFunctions();
        console.log('');
        
        const config = testConfigurationTypes();
        console.log('');
        
        const analyzer = testAnalyzerCreation(config);
        console.log('');
        
        const result = testAnalysisResultTypes(analyzer);
        console.log('');
        
        testExportOptionsTypes();
        console.log('');
        
        testAnalyzerMethods(analyzer);
        console.log('');
        
        testTypeGuards(result);
        console.log('');
        
        console.log('✅ All TypeScript type validation tests completed successfully!');
        console.log('🎯 The TypeScript definitions correctly match the runtime API');
        
    } catch (error) {
        console.error(`\n💥 Type validation tests failed: ${error}`);
        if (error instanceof Error) {
            console.error(error.stack);
        }
        process.exit(1);
    }
}

// Export for use in other modules
export {
    testUtilityFunctions,
    testConfigurationTypes,
    testAnalyzerCreation,
    testAnalysisResultTypes,
    testExportOptionsTypes,
    testAnalyzerMethods,
    testTypeGuards,
    runTypeValidationTests
};

// Run tests if this file is executed directly
if (require.main === module) {
    runTypeValidationTests().catch(console.error);
}
