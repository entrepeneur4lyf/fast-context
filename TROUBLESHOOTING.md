# Fast-Context Troubleshooting Guide

Comprehensive troubleshooting guide for common issues, performance problems, and debugging techniques.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Runtime Errors](#runtime-errors)
- [Performance Problems](#performance-problems)
- [File Watching Issues](#file-watching-issues)
- [Memory Problems](#memory-problems)
- [Platform-Specific Issues](#platform-specific-issues)
- [Debug Mode](#debug-mode)
- [Getting Help](#getting-help)

## Installation Issues

### Issue: Module Not Found

**Error**: `Error: Cannot find module 'fast-context'`

**Cause**: Package not installed or incorrect import path.

**Solutions**:
```bash
# Verify installation
npm ls fast-context

# Reinstall if missing
npm install fast-context

# Check for correct import
const { FastContextAnalyzer } = require('fast-context');
```

### Issue: Native Module Load Error

**Error**: `Error: The specified module could not be found.` (Windows)
**Error**: `Error: dlopen(...): Library not loaded` (macOS)
**Error**: `Error: libc.so.6: version 'GLIBC_2.x' not found` (Linux)

**Cause**: Missing native dependencies or incompatible system libraries.

**Solutions**:

**Windows**:
```bash
# Install Visual C++ Redistributables
# Download from Microsoft official site

# Check Node.js architecture matches
node -p "process.arch"
npm list fast-context
```

**macOS**:
```bash
# Update Xcode command line tools
xcode-select --install

# Check system compatibility
sw_vers -productVersion
node -p "process.arch"
```

**Linux**:
```bash
# Update system libraries
sudo apt update && sudo apt upgrade  # Ubuntu/Debian
sudo yum update                       # CentOS/RHEL

# Check GLIBC version
ldd --version

# For older systems, try different Node.js version
nvm use 14  # Use older Node.js version
```

### Issue: Permission Denied During Install

**Error**: `Error: EACCES: permission denied`

**Cause**: Insufficient permissions for global installation or cache directory.

**Solutions**:
```bash
# Use npx instead of global install
npx fast-context-cli analyze ./src

# Fix npm permissions (recommended)
npm config set prefix ~/.npm-global
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc

# Alternative: use Node version manager
nvm use node
npm install fast-context
```

## Runtime Errors

### Issue: Analysis Fails with "Project Root Not Found"

**Error**: `AnalysisError: Project root directory does not exist: /path/to/project`

**Cause**: Invalid project path or insufficient permissions.

**Solutions**:
```javascript
// Verify path exists and is accessible
const fs = require('fs');
const path = require('path');

const projectRoot = '/path/to/project';
if (!fs.existsSync(projectRoot)) {
    console.error(`Path does not exist: ${projectRoot}`);
}

// Use absolute path
const analyzer = new FastContextAnalyzer({
    projectRoot: path.resolve('./my-project'),
    ignorePatterns: ['node_modules/**']
});

// Check permissions
try {
    fs.accessSync(projectRoot, fs.constants.R_OK);
    console.log('Directory is readable');
} catch (err) {
    console.error('Permission denied:', err.message);
}
```

### Issue: "Parse Timeout" Errors

**Error**: `ParseError: Parse timeout exceeded for file: large-file.js`

**Cause**: Large files exceeding parse timeout limit.

**Solutions**:
```javascript
// Increase timeout
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    parseTimeout: 30000, // 30 seconds instead of default 5
    maxFileSize: 5 * 1024 * 1024 // 5MB limit
});

// Add problematic files to ignore patterns
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: [
        'node_modules/**',
        'dist/**',
        '**/*.min.js',    // Ignore minified files
        '**/bundle.js',   // Ignore large bundles
        'large-generated-file.js'
    ]
});
```

### Issue: "Memory Limit Exceeded"

**Error**: `AnalysisError: Memory usage exceeded limit during analysis`

**Cause**: Large codebase causing memory exhaustion.

**Solutions**:
```javascript
// Use streaming analysis for large codebases
analyzer.findSymbolsStreaming(
    '.*',
    {
        chunkSize: 1000,        // Smaller chunks
        includeProgress: true
    },
    (chunk) => {
        processChunk(chunk);
        // Process immediately and don't store
    }
);

// Limit analysis scope
const result = analyzer.analyze({
    maxSymbols: 50000,      // Limit symbol count
    includeTests: false,    // Exclude test files
    forceRefresh: false     // Use cache when possible
});

// Increase Node.js memory limit
// node --max-old-space-size=4096 your-script.js
```

## Performance Problems

### Issue: Analysis Takes Too Long

**Symptoms**: Analysis taking minutes instead of seconds.

**Diagnosis**:
```javascript
// Enable timing breakdown
const start = Date.now();
const result = analyzer.analyze();
const duration = Date.now() - start;

console.log(`Analysis took ${duration}ms`);
console.log(`Files: ${result.fileCount}`);
console.log(`Symbols: ${result.symbolCount}`);
console.log(`Memory: ${result.memoryUsageMb}MB`);

// Check for excessive files
if (result.fileCount > 10000) {
    console.warn('Large number of files - consider ignore patterns');
}

// Check for complex files
if (result.symbolCount / result.fileCount > 1000) {
    console.warn('High symbol-to-file ratio - may have large generated files');
}
```

**Solutions**:
```javascript
// Optimize ignore patterns
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
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
        
        // Generated files
        '**/*.generated.*',
        '**/generated/**',
        
        // Large data files
        '**/*.json',     // If contains large config files
        '**/*.log',
        
        // IDE and system files
        '.git/**',
        '.svn/**',
        '.vscode/**',
        '.idea/**',
        '**/.DS_Store',
        
        // Test coverage
        'coverage/**',
        '.nyc_output/**'
    ]
});

// Use language filters
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    languageFilters: ['javascript', 'typescript'], // Focus on specific languages
    ignorePatterns: ['node_modules/**']
});
```

### Issue: High Memory Usage

**Symptoms**: Node.js process using excessive RAM (>1GB for medium projects).

**Diagnosis**:
```javascript
// Monitor memory usage
function checkMemory() {
    const used = process.memoryUsage();
    console.log({
        rss: Math.round(used.rss / 1024 / 1024) + 'MB',      // Total memory
        heapTotal: Math.round(used.heapTotal / 1024 / 1024) + 'MB',
        heapUsed: Math.round(used.heapUsed / 1024 / 1024) + 'MB',
        external: Math.round(used.external / 1024 / 1024) + 'MB'
    });
}

checkMemory();
const result = analyzer.analyze();
checkMemory();
```

**Solutions**:
```javascript
// Enable garbage collection hints
if (global.gc) {
    global.gc();
}

// Use streaming for large queries
analyzer.findSymbolsStreaming('.*', { chunkSize: 500 }, (chunk) => {
    // Process chunk immediately
    processSymbols(chunk.symbols);
    
    // Help GC by nullifying processed data
    chunk.symbols = null;
});

// Disable caching for one-time analysis
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    enableCaching: false
});
```

### Issue: Slow File Watching Response

**Symptoms**: File change callbacks delayed by several seconds.

**Diagnosis**:
```javascript
analyzer.startWatching((changeBatch) => {
    const delay = Date.now() - changeBatch.batchTimestamp;
    console.log(`File change delay: ${delay}ms`);
    
    if (delay > 2000) {
        console.warn('High file watching latency detected');
    }
}, {
    debounceMs: 500,     // Try reducing debounce
    batchSize: 10        // Try smaller batches
});
```

**Solutions**:
```javascript
// Optimize file watching configuration
analyzer.startWatching(callback, {
    debounceMs: 200,     // Faster response
    batchSize: 25,       // Smaller batches
    watchTests: false    // Exclude test files
});

// Check system file watcher limits (Linux)
// cat /proc/sys/fs/inotify/max_user_watches
// echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
```

## File Watching Issues

### Issue: File Changes Not Detected

**Symptoms**: Callbacks not triggered when files change.

**Diagnosis**:
```javascript
// Test file watching manually
const fs = require('fs');
const path = require('path');

const testFile = path.join(process.cwd(), 'test-watch.js');

analyzer.startWatching((changeBatch) => {
    console.log('Change detected:', changeBatch);
});

// Create test file
setTimeout(() => {
    fs.writeFileSync(testFile, 'console.log("test");');
    console.log('Created test file');
}, 1000);

// Modify test file
setTimeout(() => {
    fs.appendFileSync(testFile, '\nconsole.log("modified");');
    console.log('Modified test file');
}, 2000);

// Delete test file
setTimeout(() => {
    fs.unlinkSync(testFile);
    console.log('Deleted test file');
}, 3000);
```

**Solutions**:
```javascript
// Check ignore patterns
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: [
        'node_modules/**',
        // Make sure your files aren't being ignored
        // '**/*.js',  // This would ignore ALL JS files!
    ]
});

// Verify file extensions are watched
// Check WatcherConfig.watched_extensions in the source

// Platform-specific solutions
if (process.platform === 'linux') {
    // Check inotify limits
    console.log('Check: cat /proc/sys/fs/inotify/max_user_watches');
}

if (process.platform === 'darwin') {
    // macOS may have FSEvents issues with network drives
    console.log('Ensure watching local filesystem, not network drive');
}
```

### Issue: Too Many File Change Events

**Symptoms**: Overwhelming number of change callbacks.

**Solutions**:
```javascript
// Increase debounce time
analyzer.startWatching(callback, {
    debounceMs: 2000,    // Wait 2 seconds before batching
    batchSize: 100,      // Allow larger batches
    watchTests: false    // Exclude noisy test files
});

// Filter changes in callback
analyzer.startWatching((changeBatch) => {
    // Only process significant changes
    const significantChanges = changeBatch.changes.filter(change => {
        return change.affectsAnalysis && 
               !change.filePath.includes('test') &&
               !change.filePath.includes('.tmp');
    });
    
    if (significantChanges.length > 0) {
        console.log(`Processing ${significantChanges.length} significant changes`);
        // Process changes...
    }
});

// Add more specific ignore patterns
const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: [
        'node_modules/**',
        '.git/**',
        '**/*.tmp',
        '**/*.log',
        '**/*.swp',
        '**/*.swo',
        '**/.*',           // Hidden files
        'logs/**',
        '.cache/**'
    ]
});
```

## Memory Problems

### Issue: Memory Leaks During File Watching

**Symptoms**: Memory usage continuously grows during file watching.

**Diagnosis**:
```javascript
// Monitor memory over time
setInterval(() => {
    const used = process.memoryUsage();
    console.log({
        timestamp: new Date().toISOString(),
        heapUsed: Math.round(used.heapUsed / 1024 / 1024) + 'MB',
        external: Math.round(used.external / 1024 / 1024) + 'MB'
    });
}, 10000); // Log every 10 seconds
```

**Solutions**:
```javascript
// Properly stop watching when done
process.on('SIGINT', () => {
    console.log('Stopping file watcher...');
    analyzer.stopWatching();
    process.exit(0);
});

// Use timeout for long-running watches
setTimeout(() => {
    console.log('Stopping file watcher after timeout');
    analyzer.stopWatching();
}, 5 * 60 * 1000); // Stop after 5 minutes

// Force garbage collection periodically (development only)
if (process.env.NODE_ENV === 'development' && global.gc) {
    setInterval(() => {
        global.gc();
        console.log('Forced garbage collection');
    }, 30000);
}
```

### Issue: Out of Memory During Large Analysis

**Error**: `FATAL ERROR: Reached heap limit Allocation failed - JavaScript heap out of memory`

**Solutions**:
```bash
# Increase Node.js memory limit
node --max-old-space-size=8192 your-script.js  # 8GB

# Alternative: Use stream processing
```

```javascript
// Process in smaller chunks
async function analyzeLargeCodebase() {
    const analyzer = new FastContextAnalyzer({
        projectRoot: process.cwd(),
        ignorePatterns: ['node_modules/**']
    });
    
    // Use streaming to avoid loading everything into memory
    return new Promise((resolve, reject) => {
        const results = [];
        
        analyzer.findSymbolsStreaming('.*', 
            { chunkSize: 1000 },
            (chunk) => {
                // Process chunk immediately
                const processed = processChunk(chunk.symbols);
                results.push(processed);
                
                // Clear chunk data to help GC
                chunk.symbols = null;
                
                if (chunk.isLast) {
                    resolve(results);
                }
            }
        );
    });
}
```

## Platform-Specific Issues

### Windows Issues

**Issue**: Path Separator Problems
```javascript
// Use path.join instead of manual concatenation
const path = require('path');

const analyzer = new FastContextAnalyzer({
    projectRoot: path.join(process.cwd(), 'src'), // ✅ Cross-platform
    // projectRoot: process.cwd() + '/src',       // ❌ Unix-only
});
```

**Issue**: Long Path Names
```javascript
// Windows has 260 character path limit
// Enable long path support or use shorter paths
const analyzer = new FastContextAnalyzer({
    projectRoot: 'C:\\proj',  // Shorter root path
    ignorePatterns: [
        'node_modules/**',
        '**/very-long-directory-names/**'
    ]
});
```

### macOS Issues

**Issue**: FSEvents Permission Denied
```bash
# Grant full disk access to Terminal/IDE
# System Preferences > Security & Privacy > Privacy > Full Disk Access
```

**Issue**: Rosetta 2 Compatibility (Apple Silicon)
```bash
# Check if running under Rosetta
uname -a

# Install native ARM64 Node.js
arch -arm64 node --version

# Or force x86_64 if needed
arch -x86_64 node your-script.js
```

### Linux Issues

**Issue**: inotify Limits
```bash
# Check current limits
cat /proc/sys/fs/inotify/max_user_watches
cat /proc/sys/fs/inotify/max_user_instances

# Increase limits temporarily
sudo sysctl fs.inotify.max_user_watches=524288
sudo sysctl fs.inotify.max_user_instances=512

# Make permanent
echo 'fs.inotify.max_user_watches=524288' | sudo tee -a /etc/sysctl.conf
echo 'fs.inotify.max_user_instances=512' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

**Issue**: GLIBC Version Compatibility
```bash
# Check GLIBC version
ldd --version

# For older systems (CentOS 7, Ubuntu 16.04)
# Try different Node.js versions
nvm install 14
nvm use 14
npm install fast-context
```

## Debug Mode

### Enabling Debug Output

```javascript
// Set debug environment variable
process.env.FAST_CONTEXT_DEBUG = '1';

const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: ['node_modules/**']
});

// This will now output detailed debug information
const result = analyzer.analyze();
```

### Verbose Logging

```javascript
// Create custom logger
function createDebugLogger() {
    return {
        debug: (msg, data) => console.log(`[DEBUG] ${msg}`, data || ''),
        info: (msg, data) => console.log(`[INFO] ${msg}`, data || ''),
        warn: (msg, data) => console.warn(`[WARN] ${msg}`, data || ''),
        error: (msg, data) => console.error(`[ERROR] ${msg}`, data || '')
    };
}

const logger = createDebugLogger();

// Log analysis progress
analyzer.startWatching((changeBatch) => {
    logger.info(`File changes received: ${changeBatch.changeCount}`);
    
    changeBatch.changes.forEach(change => {
        logger.debug(`${change.changeType}: ${change.filePath}`);
    });
    
    if (changeBatch.requiresReanalysis) {
        logger.info('Triggering reanalysis...');
        const start = Date.now();
        const result = analyzer.analyze();
        const duration = Date.now() - start;
        logger.info(`Reanalysis completed in ${duration}ms`);
    }
});
```

### Performance Profiling

```javascript
// Create performance profiler
class PerformanceProfiler {
    constructor() {
        this.marks = new Map();
        this.measures = new Map();
    }
    
    mark(name) {
        this.marks.set(name, Date.now());
    }
    
    measure(name, startMark) {
        const start = this.marks.get(startMark);
        const end = Date.now();
        const duration = end - start;
        this.measures.set(name, duration);
        console.log(`${name}: ${duration}ms`);
        return duration;
    }
    
    report() {
        console.log('\n=== Performance Report ===');
        for (const [name, duration] of this.measures) {
            console.log(`${name}: ${duration}ms`);
        }
    }
}

// Usage
const profiler = new PerformanceProfiler();

profiler.mark('analysis-start');
const result = analyzer.analyze();
profiler.measure('total-analysis', 'analysis-start');

profiler.mark('file-watching-start');
analyzer.startWatching((changeBatch) => {
    profiler.measure('file-change-callback', 'file-watching-start');
    profiler.mark('file-watching-start'); // Reset for next callback
});

// Report after analysis
setTimeout(() => {
    profiler.report();
}, 5000);
```

## Getting Help

### Collecting Debug Information

When reporting issues, please collect this information:

```javascript
// System information
console.log('System Info:', {
    platform: process.platform,
    arch: process.arch,
    nodeVersion: process.version,
    npmVersion: require('child_process').execSync('npm --version').toString().trim()
});

// Fast-Context configuration
const config = {
    projectRoot: analyzer.projectRoot,
    ignorePatterns: analyzer.ignorePatterns,
    // ... other config
};
console.log('Analyzer Config:', config);

// Analysis result (without sensitive data)
const result = analyzer.analyze();
console.log('Analysis Result:', {
    fileCount: result.fileCount,
    symbolCount: result.symbolCount,
    languages: result.languages,
    durationMs: result.durationMs,
    memoryUsageMb: result.memoryUsageMb,
    errors: result.errors ? result.errors.length : 0
});

// Error details (if any)
if (result.errors) {
    result.errors.forEach((error, index) => {
        console.log(`Error ${index + 1}:`, {
            type: error.type,
            message: error.message,
            filePath: error.filePath,
            severity: error.severity
        });
    });
}
```

### Creating Minimal Reproduction

Create a minimal example that reproduces the issue:

```javascript
// minimal-repro.js
const { FastContextAnalyzer } = require('fast-context');

// Minimal configuration
const analyzer = new FastContextAnalyzer({
    projectRoot: __dirname,
    ignorePatterns: ['node_modules/**']
});

try {
    const result = analyzer.analyze();
    console.log('SUCCESS:', result);
} catch (error) {
    console.error('ERROR:', error.message);
    console.error('Stack:', error.stack);
}
```

### Where to Report Issues

1. **GitHub Issues**: [https://github.com/fast-context/fast-context/issues](https://github.com/fast-context/fast-context/issues)
2. **Discussions**: [https://github.com/fast-context/fast-context/discussions](https://github.com/fast-context/fast-context/discussions)
3. **Stack Overflow**: Tag with `fast-context` and `node.js`

### Issue Template

```markdown
## Issue Description
Brief description of the problem

## Environment
- OS: [Windows 10/macOS 12/Ubuntu 20.04]
- Node.js: [version]
- Fast-Context: [version]
- Project size: [number of files/symbols]

## Configuration
```javascript
// Your analyzer configuration
```

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What you expected to happen

## Actual Behavior
What actually happened

## Error Messages
```
Paste any error messages or logs
```

## Additional Context
Any other relevant information
```

### Community Resources

- **Documentation**: [https://docs.fast-context.dev](https://docs.fast-context.dev)
- **Examples Repository**: [https://github.com/fast-context/examples](https://github.com/fast-context/examples)
- **Discord Community**: [https://discord.gg/fast-context](https://discord.gg/fast-context)
- **Twitter**: [@FastContextDev](https://twitter.com/FastContextDev)

Remember: When asking for help, provide as much relevant information as possible, including your configuration, error messages, and steps to reproduce the issue.