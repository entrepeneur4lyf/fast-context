"use strict";
/**
 * Enhanced FastContextAnalyzer
 * Wraps the native implementation with streaming, type safety, and advanced features
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ConfigurationManager = exports.QueryEngine = exports.EnhancedFastContextAnalyzer = void 0;
const events_1 = require("events");
const types_1 = require("../types");
const config_1 = require("../config");
const streaming_1 = require("../streaming");
const query_1 = require("../query");
class EnhancedFastContextAnalyzer extends events_1.EventEmitter {
    constructor(config) {
        super();
        // Validate configuration
        const configValidation = (0, types_1.validateConfig)(config);
        if (!configValidation.success) {
            throw configValidation.error;
        }
        this.config = configValidation.data;
        // Initialize native analyzer with validated config
        try {
            // Convert our config format to the native format
            const nativeConfig = this.convertToNativeConfig(this.config);
            this.nativeAnalyzer = new FastContextAnalyzer(nativeConfig);
        }
        catch (error) {
            throw new types_1.AnalysisError('Failed to initialize native analyzer', 'NATIVE_INIT_ERROR', { originalError: error });
        }
        // Initialize enhanced components
        this.streamingAnalyzer = new streaming_1.StreamingAnalyzer(this.nativeAnalyzer, this.config);
        this.queryEngine = new query_1.QueryEngine(this.nativeAnalyzer, this.config);
        // Forward events from streaming analyzer
        this.streamingAnalyzer.on('progress', (progress) => {
            this.emit('progress', progress);
        });
    }
    /**
     * Create analyzer from configuration object with validation
     */
    static create(config) {
        try {
            const analyzer = new EnhancedFastContextAnalyzer(config);
            return (0, types_1.Ok)(analyzer);
        }
        catch (error) {
            if (error instanceof types_1.AnalysisError) {
                return (0, types_1.Err)(error);
            }
            return (0, types_1.Err)(new types_1.AnalysisError('Failed to create analyzer', 'CREATION_ERROR', { originalError: error }));
        }
    }
    /**
     * Create analyzer from preset configuration
     */
    static fromPreset(preset, projectRoot) {
        const config = config_1.ConfigurationManager.getPreset(preset);
        config.projectRoot = projectRoot;
        return new EnhancedFastContextAnalyzer(config);
    }
    /**
     * Create analyzer from environment configuration
     */
    static fromEnvironment() {
        const config = config_1.ConfigurationManager.loadFromEnvironment();
        return new EnhancedFastContextAnalyzer(config);
    }
    /**
     * Stream analysis with progress tracking and cancellation support
     */
    async *analyzeStream(options) {
        // Validate streaming options
        if (options) {
            const optionsValidation = (0, types_1.validateStreamingOptions)(options);
            if (!optionsValidation.success) {
                throw optionsValidation.error;
            }
        }
        // Prevent multiple concurrent analyses
        if (this.currentAnalysis) {
            throw new types_1.AnalysisError('Analysis already in progress', 'ANALYSIS_IN_PROGRESS');
        }
        try {
            this.emit('analysisStarted');
            for await (const progress of this.streamingAnalyzer.analyzeStream(options)) {
                yield progress;
                // Emit completion event when done
                if (progress.phase === 'complete') {
                    this.emit('analysisCompleted', progress);
                }
            }
        }
        catch (error) {
            this.emit('analysisError', error);
            throw error;
        }
        finally {
            this.currentAnalysis = undefined;
        }
    }
    /**
     * Traditional promise-based analysis
     */
    async analyze() {
        if (this.currentAnalysis) {
            return this.currentAnalysis;
        }
        this.currentAnalysis = this.performAnalysis();
        return this.currentAnalysis;
    }
    /**
     * Cancel the current analysis
     */
    cancel() {
        this.streamingAnalyzer.cancel();
        this.emit('analysisCancelled');
    }
    /**
     * Get the query engine for advanced queries
     */
    getQueryEngine() {
        return this.queryEngine;
    }
    /**
     * Update configuration (creates new analyzer instance)
     */
    updateConfig(updates) {
        const newConfig = { ...this.config, ...updates };
        return EnhancedFastContextAnalyzer.create(newConfig);
    }
    /**
     * Get current configuration
     */
    getConfig() {
        return { ...this.config };
    }
    /**
     * Get configuration summary for debugging
     */
    getConfigSummary() {
        return config_1.ConfigurationManager.getConfigSummary(this.config);
    }
    /**
     * Check if analysis is currently running
     */
    isAnalyzing() {
        return this.currentAnalysis !== undefined;
    }
    /**
     * Get supported languages
     */
    static getSupportedLanguages() {
        // This would call the native implementation
        // For now, return a static list
        return [
            'javascript', 'typescript', 'python', 'rust', 'java', 'c', 'cpp',
            'go', 'ruby', 'php', 'swift', 'kotlin', 'scala', 'csharp',
            'html', 'css', 'json', 'yaml', 'xml', 'markdown'
        ];
    }
    /**
     * Detect language from file path
     */
    static detectLanguage(filePath) {
        const extension = filePath.split('.').pop()?.toLowerCase();
        const languageMap = {
            'js': 'javascript',
            'jsx': 'javascript',
            'ts': 'typescript',
            'tsx': 'typescript',
            'py': 'python',
            'rs': 'rust',
            'java': 'java',
            'c': 'c',
            'cpp': 'cpp',
            'cc': 'cpp',
            'cxx': 'cpp',
            'go': 'go',
            'rb': 'ruby',
            'php': 'php',
            'swift': 'swift',
            'kt': 'kotlin',
            'scala': 'scala',
            'cs': 'csharp',
            'html': 'html',
            'css': 'css',
            'json': 'json',
            'yaml': 'yaml',
            'yml': 'yaml',
            'xml': 'xml',
            'md': 'markdown'
        };
        return extension ? languageMap[extension] || null : null;
    }
    /**
     * Perform the actual analysis
     */
    async performAnalysis() {
        try {
            const results = [];
            for await (const progress of this.analyzeStream()) {
                results.push(progress);
            }
            const finalProgress = results[results.length - 1];
            if (!finalProgress) {
                throw new types_1.AnalysisError('No analysis results received', 'NO_RESULTS');
            }
            // Convert progress to result
            return {
                fileCount: finalProgress.filesProcessed,
                symbolCount: finalProgress.symbolsFound,
                relationshipCount: finalProgress.relationshipsFound,
                languages: [], // Would be populated from actual analysis
                durationMs: finalProgress.performance.elapsedMs,
                memoryUsageMb: finalProgress.performance.memoryUsageMb,
                performance: finalProgress.performance
            };
        }
        catch (error) {
            if (error instanceof types_1.AnalysisError) {
                throw error;
            }
            throw new types_1.AnalysisError('Analysis failed', 'ANALYSIS_ERROR', { originalError: error });
        }
    }
    /**
     * Convert our config format to native config format
     */
    convertToNativeConfig(config) {
        return {
            project_root: config.projectRoot,
            languages: config.languages || null,
            ignore_patterns: config.ignorePatterns || null,
            enable_caching: config.enableCaching,
            cache_policy: config.cachePolicy,
            enable_watching: config.enableWatching,
            max_files: config.maxFiles || null,
            parallel_processing: config.parallelProcessing,
            enable_experimental_architecture: false // Keep legacy for now
        };
    }
}
exports.EnhancedFastContextAnalyzer = EnhancedFastContextAnalyzer;
// Re-export for convenience
var query_2 = require("../query");
Object.defineProperty(exports, "QueryEngine", { enumerable: true, get: function () { return query_2.QueryEngine; } });
var config_2 = require("../config");
Object.defineProperty(exports, "ConfigurationManager", { enumerable: true, get: function () { return config_2.ConfigurationManager; } });
__exportStar(require("../types"), exports);
//# sourceMappingURL=index.js.map