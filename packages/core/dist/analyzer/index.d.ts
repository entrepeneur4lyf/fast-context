/**
 * Enhanced FastContextAnalyzer
 * Wraps the native implementation with streaming, type safety, and advanced features
 */
import { EventEmitter } from 'events';
import { AnalysisConfig, AnalysisProgress, AnalysisResult, StreamingOptions, Result, AnalysisError } from '../types';
import { QueryEngine } from '../query';
export declare class EnhancedFastContextAnalyzer extends EventEmitter {
    private readonly nativeAnalyzer;
    private readonly config;
    private readonly streamingAnalyzer;
    private readonly queryEngine;
    private currentAnalysis;
    constructor(config: AnalysisConfig | unknown);
    /**
     * Create analyzer from configuration object with validation
     */
    static create(config: unknown): Result<EnhancedFastContextAnalyzer, AnalysisError>;
    /**
     * Create analyzer from preset configuration
     */
    static fromPreset(preset: 'fast' | 'balanced' | 'thorough', projectRoot: string): EnhancedFastContextAnalyzer;
    /**
     * Create analyzer from environment configuration
     */
    static fromEnvironment(): EnhancedFastContextAnalyzer;
    /**
     * Stream analysis with progress tracking and cancellation support
     */
    analyzeStream(options?: StreamingOptions): AsyncIterableIterator<AnalysisProgress>;
    /**
     * Traditional promise-based analysis
     */
    analyze(): Promise<AnalysisResult>;
    /**
     * Cancel the current analysis
     */
    cancel(): void;
    /**
     * Get the query engine for advanced queries
     */
    getQueryEngine(): QueryEngine;
    /**
     * Update configuration (creates new analyzer instance)
     */
    updateConfig(updates: Partial<AnalysisConfig>): Result<EnhancedFastContextAnalyzer, AnalysisError>;
    /**
     * Get current configuration
     */
    getConfig(): AnalysisConfig;
    /**
     * Get configuration summary for debugging
     */
    getConfigSummary(): string;
    /**
     * Check if analysis is currently running
     */
    isAnalyzing(): boolean;
    /**
     * Get supported languages
     */
    static getSupportedLanguages(): string[];
    /**
     * Detect language from file path
     */
    static detectLanguage(filePath: string): string | null;
    /**
     * Perform the actual analysis
     */
    private performAnalysis;
    /**
     * Convert our config format to native config format
     */
    private convertToNativeConfig;
}
export { QueryEngine } from '../query';
export { ConfigurationManager } from '../config';
export * from '../types';
//# sourceMappingURL=index.d.ts.map