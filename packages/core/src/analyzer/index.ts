/**
 * Enhanced FastContextAnalyzer
 * Wraps the native implementation with streaming, type safety, and advanced features
 */

import { EventEmitter } from 'events';
import {
  AnalysisConfig,
  AnalysisProgress,
  AnalysisResult,
  StreamingOptions,
  Result,
  Ok,
  Err,
  AnalysisError,
  validateConfig,
  validateStreamingOptions
} from '../types';
import { ConfigurationManager } from '../config';
import { StreamingAnalyzer } from '../streaming';
import { QueryEngine } from '../query';

// This would import the actual native binding
// For now, we'll use a type that matches the expected interface
declare const FastContextAnalyzer: any;

export class EnhancedFastContextAnalyzer extends EventEmitter {
  private readonly nativeAnalyzer: any;
  private readonly config: AnalysisConfig;
  private readonly streamingAnalyzer: StreamingAnalyzer;
  private readonly queryEngine: QueryEngine;
  private currentAnalysis: Promise<AnalysisResult> | undefined;

  constructor(config: AnalysisConfig | unknown) {
    super();
    
    // Validate configuration
    const configValidation = validateConfig(config);
    if (!configValidation.success) {
      throw configValidation.error;
    }
    
    this.config = configValidation.data;
    
    // Initialize native analyzer with validated config
    try {
      // Convert our config format to the native format
      const nativeConfig = this.convertToNativeConfig(this.config);
      this.nativeAnalyzer = new FastContextAnalyzer(nativeConfig);
    } catch (error) {
      throw new AnalysisError(
        'Failed to initialize native analyzer',
        'NATIVE_INIT_ERROR',
        { originalError: error }
      );
    }
    
    // Initialize enhanced components
    this.streamingAnalyzer = new StreamingAnalyzer(this.nativeAnalyzer, this.config);
    this.queryEngine = new QueryEngine(this.nativeAnalyzer, this.config);
    
    // Forward events from streaming analyzer
    this.streamingAnalyzer.on('progress', (progress) => {
      this.emit('progress', progress);
    });
  }

  /**
   * Create analyzer from configuration object with validation
   */
  static create(config: unknown): Result<EnhancedFastContextAnalyzer, AnalysisError> {
    try {
      const analyzer = new EnhancedFastContextAnalyzer(config);
      return Ok(analyzer);
    } catch (error) {
      if (error instanceof AnalysisError) {
        return Err(error);
      }
      return Err(new AnalysisError(
        'Failed to create analyzer',
        'CREATION_ERROR',
        { originalError: error }
      ));
    }
  }

  /**
   * Create analyzer from preset configuration
   */
  static fromPreset(preset: 'fast' | 'balanced' | 'thorough', projectRoot: string): EnhancedFastContextAnalyzer {
    const config = ConfigurationManager.getPreset(preset);
    config.projectRoot = projectRoot;
    return new EnhancedFastContextAnalyzer(config);
  }

  /**
   * Create analyzer from environment configuration
   */
  static fromEnvironment(): EnhancedFastContextAnalyzer {
    const config = ConfigurationManager.loadFromEnvironment();
    return new EnhancedFastContextAnalyzer(config);
  }

  /**
   * Stream analysis with progress tracking and cancellation support
   */
  async *analyzeStream(options?: StreamingOptions): AsyncIterableIterator<AnalysisProgress> {
    // Validate streaming options
    if (options) {
      const optionsValidation = validateStreamingOptions(options);
      if (!optionsValidation.success) {
        throw optionsValidation.error;
      }
    }

    // Prevent multiple concurrent analyses
    if (this.currentAnalysis) {
      throw new AnalysisError(
        'Analysis already in progress',
        'ANALYSIS_IN_PROGRESS'
      );
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
    } catch (error) {
      this.emit('analysisError', error);
      throw error;
    } finally {
      this.currentAnalysis = undefined;
    }
  }

  /**
   * Traditional promise-based analysis
   */
  async analyze(): Promise<AnalysisResult> {
    if (this.currentAnalysis) {
      return this.currentAnalysis;
    }

    this.currentAnalysis = this.performAnalysis();
    return this.currentAnalysis;
  }

  /**
   * Cancel the current analysis
   */
  cancel(): void {
    this.streamingAnalyzer.cancel();
    this.emit('analysisCancelled');
  }

  /**
   * Get the query engine for advanced queries
   */
  getQueryEngine(): QueryEngine {
    return this.queryEngine;
  }

  /**
   * Update configuration (creates new analyzer instance)
   */
  updateConfig(updates: Partial<AnalysisConfig>): Result<EnhancedFastContextAnalyzer, AnalysisError> {
    const newConfig = { ...this.config, ...updates };
    return EnhancedFastContextAnalyzer.create(newConfig);
  }

  /**
   * Get current configuration
   */
  getConfig(): AnalysisConfig {
    return { ...this.config };
  }

  /**
   * Get configuration summary for debugging
   */
  getConfigSummary(): string {
    return ConfigurationManager.getConfigSummary(this.config);
  }

  /**
   * Check if analysis is currently running
   */
  isAnalyzing(): boolean {
    return this.currentAnalysis !== undefined;
  }

  /**
   * Get supported languages
   */
  static getSupportedLanguages(): string[] {
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
  static detectLanguage(filePath: string): string | null {
    const extension = filePath.split('.').pop()?.toLowerCase();
    
    const languageMap: Record<string, string> = {
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
  private async performAnalysis(): Promise<AnalysisResult> {
    try {
      const results: AnalysisProgress[] = [];
      
      for await (const progress of this.analyzeStream()) {
        results.push(progress);
      }
      
      const finalProgress = results[results.length - 1];
      if (!finalProgress) {
        throw new AnalysisError('No analysis results received', 'NO_RESULTS');
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
    } catch (error) {
      if (error instanceof AnalysisError) {
        throw error;
      }
      throw new AnalysisError(
        'Analysis failed',
        'ANALYSIS_ERROR',
        { originalError: error }
      );
    }
  }

  /**
   * Convert our config format to native config format
   */
  private convertToNativeConfig(config: AnalysisConfig): any {
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

// Re-export for convenience
export { QueryEngine } from '../query';
export { ConfigurationManager } from '../config';
export * from '../types';
