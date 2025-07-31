# Fast-Context Examples

This document provides comprehensive examples of using Fast-Context for different scenarios and use cases.

## Basic Examples

### Simple Analysis

```javascript
const { FastContextAnalyzer } = require('fast-context');

async function analyzeProject() {
    const analyzer = new FastContextAnalyzer({
        projectRoot: process.cwd(),
        ignorePatterns: ['node_modules/**', '.git/**']
    });

    const result = analyzer.analyze();
    
    console.log('Analysis Results:');
    console.log(`📁 Files: ${result.fileCount}`);
    console.log(`🔍 Symbols: ${result.symbolCount}`);
    console.log(`🔗 Relationships: ${result.relationshipCount}`);
    console.log(`⏱️  Duration: ${result.durationMs}ms`);
    console.log(`💾 Memory: ${result.memoryUsageMb}MB`);
    console.log(`🌐 Languages: ${result.languages.join(', ')}`);
}

analyzeProject().catch(console.error);
```

### File Watching with Detailed Logging

```javascript
const { FastContextAnalyzer } = require('fast-context');

function setupFileWatcher() {
    const analyzer = new FastContextAnalyzer({
        projectRoot: './src',
        ignorePatterns: ['**/*.test.js', '**/*.spec.js', 'node_modules/**']
    });

    analyzer.startWatching((changeBatch) => {
        console.log(`\n📁 File Change Event - ${new Date().toISOString()}`);
        console.log(`   Changes: ${changeBatch.changeCount}`);
        console.log(`   Impact: ${changeBatch.impactLevel}`);
        console.log(`   Needs Reanalysis: ${changeBatch.requiresReanalysis}`);
        
        changeBatch.changes.forEach((change, index) => {
            console.log(`   ${index + 1}. ${change.changeType.toUpperCase()}: ${change.filePath}`);
            if (change.language) {
                console.log(`      Language: ${change.language}`);
                console.log(`      Affects Analysis: ${change.affectsAnalysis}`);
            }
        });

        // Trigger reanalysis for significant changes
        if (changeBatch.requiresReanalysis && changeBatch.impactLevel !== 'low') {
            console.log('🔄 Triggering reanalysis...');
            const newResult = analyzer.analyze();
            console.log(`✅ Reanalysis complete: ${newResult.symbolCount} symbols`);
        }
    });

    console.log('🔍 File watcher started. Make changes to files in ./src to see events.');
    
    // Stop watching after 5 minutes
    setTimeout(() => {
        analyzer.stopWatching();
        console.log('🛑 File watcher stopped.');
    }, 5 * 60 * 1000);
}

setupFileWatcher();
```

## Advanced Examples

### Multi-Project Analysis

```javascript
const { FastContextAnalyzer } = require('fast-context');
const path = require('path');

async function compareProjects(projectPaths) {
    const results = [];
    
    for (const projectPath of projectPaths) {
        console.log(`\n📊 Analyzing ${path.basename(projectPath)}...`);
        
        const analyzer = new FastContextAnalyzer({
            projectRoot: projectPath,
            ignorePatterns: [
                'node_modules/**',
                'target/**',
                '.git/**',
                'dist/**',
                'build/**'
            ]
        });

        const result = analyzer.analyze();
        results.push({
            project: path.basename(projectPath),
            ...result
        });
    }

    // Generate comparison report
    console.log('\n📈 Project Comparison Report');
    console.log('='.repeat(50));
    
    results.forEach(result => {
        console.log(`\n${result.project}:`);
        console.log(`  Files: ${result.fileCount}`);
        console.log(`  Symbols: ${result.symbolCount}`);
        console.log(`  Relationships: ${result.relationshipCount}`);
        console.log(`  Duration: ${result.durationMs}ms`);
        console.log(`  Languages: ${result.languages.join(', ')}`);
        console.log(`  Symbols/File: ${(result.symbolCount / result.fileCount).toFixed(1)}`);
    });

    // Find the most complex project
    const mostComplex = results.reduce((max, curr) => 
        curr.symbolCount > max.symbolCount ? curr : max
    );
    
    console.log(`\n🏆 Most Complex Project: ${mostComplex.project}`);
    console.log(`   ${mostComplex.symbolCount} symbols across ${mostComplex.fileCount} files`);
}

// Usage
compareProjects([
    './frontend',
    './backend',
    './shared-utils'
]).catch(console.error);
```

### Language-Specific Analysis

```javascript
const { FastContextAnalyzer } = require('fast-context');

async function analyzeByLanguage() {
    const languages = ['javascript', 'typescript', 'rust', 'python'];
    const results = {};

    for (const language of languages) {
        console.log(`\n🔍 Analyzing ${language} code...`);
        
        const analyzer = new FastContextAnalyzer({
            projectRoot: process.cwd(),
            languageFilters: [language],
            ignorePatterns: ['node_modules/**', 'target/**', '.git/**']
        });

        try {
            const result = analyzer.analyze();
            results[language] = result;
            
            console.log(`  ✅ Found ${result.symbolCount} ${language} symbols in ${result.fileCount} files`);
        } catch (error) {
            console.log(`  ❌ No ${language} files found`);
            results[language] = null;
        }
    }

    // Generate language breakdown
    console.log('\n📊 Language Breakdown');
    console.log('='.repeat(40));
    
    const totalSymbols = Object.values(results)
        .filter(Boolean)
        .reduce((sum, result) => sum + result.symbolCount, 0);

    Object.entries(results).forEach(([lang, result]) => {
        if (result) {
            const percentage = ((result.symbolCount / totalSymbols) * 100).toFixed(1);
            console.log(`${lang.padEnd(12)}: ${result.symbolCount.toString().padStart(6)} symbols (${percentage}%)`);
        }
    });

    console.log(`${'Total'.padEnd(12)}: ${totalSymbols.toString().padStart(6)} symbols (100.0%)`);
}

analyzeByLanguage().catch(console.error);
```

### Performance Monitoring

```javascript
const { FastContextAnalyzer } = require('fast-context');

function createPerformanceMonitor() {
    return {
        analysisCount: 0,
        totalDuration: 0,
        peakMemory: 0,
        
        recordAnalysis(result) {
            this.analysisCount++;
            this.totalDuration += result.durationMs;
            this.peakMemory = Math.max(this.peakMemory, result.memoryUsageMb);
            
            console.log(`📊 Analysis #${this.analysisCount}:`);
            console.log(`   Duration: ${result.durationMs}ms`);
            console.log(`   Memory: ${result.memoryUsageMb}MB`);
            console.log(`   Avg Duration: ${(this.totalDuration / this.analysisCount).toFixed(1)}ms`);
            console.log(`   Peak Memory: ${this.peakMemory}MB`);
        }
    };
}

async function monitorPerformance() {
    const analyzer = new FastContextAnalyzer({
        projectRoot: process.cwd(),
        ignorePatterns: ['node_modules/**', '.git/**']
    });

    const monitor = createPerformanceMonitor();

    // Initial analysis
    console.log('🚀 Starting performance monitoring...');
    let result = analyzer.analyze();
    monitor.recordAnalysis(result);

    // Set up file watching with performance tracking
    analyzer.startWatching((changeBatch) => {
        console.log(`\n🔄 File changes detected (${changeBatch.changeCount} changes)`);
        
        if (changeBatch.requiresReanalysis) {
            const startTime = Date.now();
            const newResult = analyzer.analyze();
            const actualDuration = Date.now() - startTime;
            
            // Compare reported vs actual duration
            console.log(`⏱️  Reported: ${newResult.durationMs}ms, Actual: ${actualDuration}ms`);
            
            monitor.recordAnalysis(newResult);
        }
    });

    // Periodic analysis for benchmarking
    const interval = setInterval(() => {
        console.log('\n⏰ Periodic analysis...');
        const benchmarkResult = analyzer.analyze();
        monitor.recordAnalysis(benchmarkResult);
    }, 30000); // Every 30 seconds

    // Stop after 5 minutes
    setTimeout(() => {
        clearInterval(interval);
        analyzer.stopWatching();
        
        console.log('\n📈 Final Performance Summary:');
        console.log(`   Total Analyses: ${monitor.analysisCount}`);
        console.log(`   Average Duration: ${(monitor.totalDuration / monitor.analysisCount).toFixed(1)}ms`);
        console.log(`   Peak Memory Usage: ${monitor.peakMemory}MB`);
    }, 5 * 60 * 1000);
}

monitorPerformance().catch(console.error);
```

## Integration Examples

### Express.js API Integration

```javascript
const express = require('express');
const { FastContextAnalyzer } = require('fast-context');

const app = express();
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: ['node_modules/**', '.git/**']
});

// Analyze endpoint
app.get('/api/analyze', (req, res) => {
    try {
        const result = analyzer.analyze();
        res.json({
            success: true,
            data: result,
            timestamp: new Date().toISOString()
        });
    } catch (error) {
        res.status(500).json({
            success: false,
            error: error.message
        });
    }
});

// File watching with WebSocket notifications
const WebSocket = require('ws');
const wss = new WebSocket.Server({ port: 8080 });

analyzer.startWatching((changeBatch) => {
    const notification = {
        type: 'file_changes',
        data: changeBatch,
        timestamp: new Date().toISOString()
    };

    // Broadcast to all connected clients
    wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(JSON.stringify(notification));
        }
    });
});

app.listen(3000, () => {
    console.log('🚀 Analysis API running on http://localhost:3000');
    console.log('📡 WebSocket server running on ws://localhost:8080');
});
```

### CI/CD Integration

```javascript
const { FastContextAnalyzer } = require('fast-context');

async function ciAnalysis() {
    const analyzer = new FastContextAnalyzer({
        projectRoot: process.cwd(),
        ignorePatterns: [
            'node_modules/**',
            '.git/**',
            'dist/**',
            'build/**',
            'coverage/**'
        ]
    });

    console.log('🔍 Running codebase analysis for CI/CD...');
    
    const result = analyzer.analyze();
    
    // Set quality gates
    const qualityGates = {
        maxAnalysisTime: 5000, // 5 seconds
        maxMemoryUsage: 100,   // 100MB
        minSymbolCount: 100,   // At least 100 symbols
        requiredLanguages: ['javascript', 'typescript']
    };

    const issues = [];

    // Check performance
    if (result.durationMs > qualityGates.maxAnalysisTime) {
        issues.push(`Analysis took ${result.durationMs}ms (limit: ${qualityGates.maxAnalysisTime}ms)`);
    }

    if (result.memoryUsageMb > qualityGates.maxMemoryUsage) {
        issues.push(`Memory usage ${result.memoryUsageMb}MB (limit: ${qualityGates.maxMemoryUsage}MB)`);
    }

    // Check code coverage
    if (result.symbolCount < qualityGates.minSymbolCount) {
        issues.push(`Only ${result.symbolCount} symbols found (minimum: ${qualityGates.minSymbolCount})`);
    }

    // Check required languages
    const missingLanguages = qualityGates.requiredLanguages.filter(
        lang => !result.languages.map(l => l.toLowerCase()).includes(lang)
    );
    
    if (missingLanguages.length > 0) {
        issues.push(`Missing required languages: ${missingLanguages.join(', ')}`);
    }

    // Report results
    console.log('\n📊 Analysis Results:');
    console.log(`   Files: ${result.fileCount}`);
    console.log(`   Symbols: ${result.symbolCount}`);
    console.log(`   Languages: ${result.languages.join(', ')}`);
    console.log(`   Duration: ${result.durationMs}ms`);
    console.log(`   Memory: ${result.memoryUsageMb}MB`);

    if (issues.length === 0) {
        console.log('\n✅ All quality gates passed!');
        process.exit(0);
    } else {
        console.log('\n❌ Quality gate failures:');
        issues.forEach(issue => console.log(`   - ${issue}`));
        process.exit(1);
    }
}

// Run CI analysis
ciAnalysis().catch(error => {
    console.error('❌ CI analysis failed:', error);
    process.exit(1);
});
```

## Configuration Examples

### Large Codebase Configuration

```javascript
const { FastContextAnalyzer } = require('fast-context');

// Configuration for analyzing large codebases (>100k LOC)
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    
    // Comprehensive ignore patterns
    ignorePatterns: [
        // Dependencies
        'node_modules/**',
        'vendor/**',
        'target/**',
        
        // Build outputs
        'dist/**',
        'build/**',
        'out/**',
        '**/*.min.js',
        '**/*.bundle.js',
        
        // Version control
        '.git/**',
        '.svn/**',
        '.hg/**',
        
        // IDE files
        '.vscode/**',
        '.idea/**',
        '**/*.swp',
        '**/*.swo',
        
        // Temporary files
        '**/*.tmp',
        '**/*.temp',
        '**/.DS_Store',
        
        // Logs
        'logs/**',
        '**/*.log',
        
        // Test coverage
        'coverage/**',
        '.nyc_output/**',
        
        // Documentation builds
        'docs/build/**',
        '_site/**'
    ],
    
    // Focus on main languages
    languageFilters: [
        'javascript',
        'typescript',
        'rust',
        'python',
        'java',
        'go'
    ]
});
```

These examples demonstrate the flexibility and power of Fast-Context for various use cases, from simple analysis to complex CI/CD integration and performance monitoring.