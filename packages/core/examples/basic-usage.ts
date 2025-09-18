/**
 * Basic Usage Example for Enhanced Fast-Context TypeScript SDK
 * 
 * This example demonstrates the key features of the enhanced SDK:
 * - Type-safe configuration
 * - Streaming analysis with progress tracking
 * - Advanced query engine
 * - Error handling with Result types
 */

import {
  EnhancedFastContextAnalyzer,
  ConfigurationManager,
  createAnalyzerFromPreset,
  createSmartConfig,
  formatDuration,
  formatFileSize,
  calculateProgress,
  AnalysisProgress,
  AnalysisError
} from '../src';

async function basicUsageExample() {
  console.log('🚀 Enhanced Fast-Context TypeScript SDK Example\n');

  try {
    // Example 1: Create analyzer with preset configuration
    console.log('📋 Example 1: Using Preset Configuration');
    const analyzer = createAnalyzerFromPreset('balanced', process.cwd());
    console.log('✅ Analyzer created with balanced preset\n');

    // Example 2: Stream analysis with progress tracking
    console.log('📊 Example 2: Streaming Analysis with Progress');
    
    try {
      for await (const progress of analyzer.analyzeStream()) {

        
        const percentage = calculateProgress(progress.filesProcessed, progress.totalFiles);
        const duration = formatDuration(progress.performance.elapsedMs);
        const memory = formatFileSize(progress.performance.memoryUsageMb * 1024 * 1024);
        
        console.log(`  Phase: ${progress.phase.padEnd(12)} | ` +
                   `Progress: ${percentage}% (${progress.filesProcessed}/${progress.totalFiles}) | ` +
                   `Duration: ${duration} | Memory: ${memory}`);
        
        if (progress.currentFile) {
          console.log(`  Current: ${progress.currentFile}`);
        }
        
        if (progress.phase === 'complete') {
          console.log(`✅ Analysis completed! Found ${progress.symbolsFound} symbols and ${progress.relationshipsFound} relationships\n`);
          break;
        }
      }
    } catch (error) {
      if (error instanceof AnalysisError && error.code === 'ANALYSIS_CANCELLED') {
        console.log('⚠️  Analysis was cancelled\n');
      } else {
        console.error('❌ Analysis failed:', error.message, '\n');
      }
    }

    // Example 3: Advanced Query Engine
    console.log('🔍 Example 3: Advanced Query Engine');
    const queryEngine = analyzer.getQueryEngine();
    
    // Find symbols by semantic search
    const symbolsResult = await queryEngine.findSymbols({
      text: 'user authentication',
      kind: 'function',
      maxResults: 5
    });
    
    if (symbolsResult.success) {
      console.log(`Found ${symbolsResult.data.length} authentication-related functions:`);
      symbolsResult.data.forEach(symbol => {
        console.log(`  - ${symbol.name} (${symbol.filePath}:${symbol.line})`);
      });
    } else {
      console.error('❌ Symbol search failed:', symbolsResult.error.message);
    }
    console.log();

    // Analyze code complexity
    const complexityResult = await queryEngine.analyzeComplexity({ threshold: 10 });
    if (complexityResult.success) {
      const report = complexityResult.data;
      console.log(`📈 Complexity Analysis:`);
      console.log(`  Average complexity: ${report.averageComplexity}`);
      console.log(`  Max complexity: ${report.maxComplexity}`);
      console.log(`  Complex symbols: ${report.complexSymbols.length}`);
      
      if (report.recommendations.length > 0) {
        console.log(`  Recommendations:`);
        report.recommendations.forEach(rec => console.log(`    - ${rec}`));
      }
    }
    console.log();

    // Detect architectural patterns
    const patternsResult = await queryEngine.detectPatterns();
    if (patternsResult.success) {
      console.log(`🏗️  Architectural Patterns Detected:`);
      patternsResult.data.forEach(pattern => {
        console.log(`  - ${pattern.name} (confidence: ${(pattern.confidence * 100).toFixed(1)}%)`);
        console.log(`    ${pattern.description}`);
      });
    }
    console.log();

    // Example 4: Configuration Management
    console.log('⚙️  Example 4: Configuration Management');
    
    // Load configuration from environment
    ConfigurationManager.loadFromEnvironment();
    console.log('Environment configuration loaded');
    
    // Get configuration summary
    const summary = analyzer.getConfigSummary();
    console.log('Current configuration:');
    console.log(summary.split('\n').map(line => `  ${line}`).join('\n'));
    console.log();

    // Example 5: Smart configuration based on project detection
    console.log('🧠 Example 5: Smart Configuration');
    const smartConfig = await createSmartConfig(process.cwd());
    console.log(`Smart configuration created for project type detection`);
    console.log(`Languages: ${smartConfig.languages?.join(', ') || 'auto-detect'}`);
    console.log(`Cache policy: ${smartConfig.cachePolicy}`);
    console.log();

    // Example 6: Error handling with Result types
    console.log('🛡️  Example 6: Type-Safe Error Handling');
    
    // Demonstrate validation error
    const invalidConfigResult = EnhancedFastContextAnalyzer.create({
      projectRoot: '', // Invalid: empty string
      maxFiles: -1     // Invalid: negative number
    });
    
    if (!invalidConfigResult.success) {
      console.log('✅ Configuration validation caught errors:');
      console.log(`  Error: ${invalidConfigResult.error.message}`);
      console.log(`  Code: ${invalidConfigResult.error.code}`);
    }
    console.log();

    console.log('🎉 All examples completed successfully!');
    
  } catch (error) {
    console.error('❌ Example failed:', error);
    process.exit(1);
  }
}

async function streamingCancellationExample() {
  console.log('\n🛑 Streaming Cancellation Example');
  
  const analyzer = createAnalyzerFromPreset('fast', process.cwd());
  
  // Start analysis
  const analysisPromise = (async () => {
    try {
      for await (const progress of analyzer.analyzeStream()) {
        console.log(`  Progress: ${progress.phase} - ${progress.filesProcessed} files`);
        
        // Simulate some processing time
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    } catch (error) {
      if (error instanceof AnalysisError && error.code === 'ANALYSIS_CANCELLED') {
        console.log('✅ Analysis was successfully cancelled');
      } else {
        throw error;
      }
    }
  })();
  
  // Cancel after 500ms
  setTimeout(() => {
    console.log('  Cancelling analysis...');
    analyzer.cancel();
  }, 500);
  
  await analysisPromise;
}

async function performanceMonitoringExample() {
  console.log('\n📊 Performance Monitoring Example');
  
  const analyzer = createAnalyzerFromPreset('thorough', process.cwd());
  
  // Monitor performance during analysis
  analyzer.on('progress', (progress: AnalysisProgress) => {
    const metrics = progress.performance;
    
    if (progress.filesProcessed % 10 === 0) { // Log every 10 files
      console.log(`  Performance Metrics:`);
      console.log(`    Memory: ${formatFileSize(metrics.memoryUsageMb * 1024 * 1024)}`);
      console.log(`    Throughput: ${metrics.throughputFilesPerSecond.toFixed(1)} files/sec`);
      
      if (metrics.estimatedRemainingMs) {
        console.log(`    ETA: ${formatDuration(metrics.estimatedRemainingMs)}`);
      }
    }
  });
  
  try {
    const result = await analyzer.analyze();
    console.log(`✅ Analysis completed in ${formatDuration(result.durationMs)}`);
    console.log(`   Final memory usage: ${formatFileSize((result.memoryUsageMb || 0) * 1024 * 1024)}`);
  } catch (error) {
    console.error('❌ Performance monitoring example failed:', error);
  }
}

// Run examples
async function runAllExamples() {
  await basicUsageExample();
  await streamingCancellationExample();
  await performanceMonitoringExample();
}

// Only run if this file is executed directly
if (require.main === module) {
  runAllExamples().catch(console.error);
}

export {
  basicUsageExample,
  streamingCancellationExample,
  performanceMonitoringExample,
  runAllExamples
};
