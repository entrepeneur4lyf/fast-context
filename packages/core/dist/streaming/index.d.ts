/**
 * Streaming Analysis Implementation
 * Provides AsyncIterableIterator for progressive analysis with cancellation support
 */
import { EventEmitter } from 'events';
import { AnalysisConfig, AnalysisProgress, StreamingOptions } from '../types';
interface NativeAnalyzer {
    analyze(): Promise<any>;
    analyzeStream?(config: any, options: any): AsyncIterableIterator<any>;
    findSymbolsByKind?(kind: string): Promise<string[]>;
    findSymbolsInFile?(filePath: string): Promise<string[]>;
    findDependencies?(symbolName: string): Promise<string[]>;
}
export declare class StreamingAnalyzer extends EventEmitter {
    private readonly nativeAnalyzer;
    private readonly config;
    private currentController;
    constructor(nativeAnalyzer: NativeAnalyzer, config: AnalysisConfig);
    /**
     * Stream analysis with progress tracking and cancellation support
     */
    analyzeStream(options?: StreamingOptions): AsyncIterableIterator<AnalysisProgress>;
    /**
     * Cancel the current analysis
     */
    cancel(): void;
    /**
     * Pause analysis (if supported by native implementation)
     */
    pause(): void;
    /**
     * Resume analysis (if supported by native implementation)
     */
    resume(): void;
    /**
     * Stream from native implementation
     */
    private streamFromNative;
    /**
     * Simulate streaming for non-streaming native implementations
     */
    private simulateStreaming;
    /**
     * Simulate work for a specific phase
     */
    private simulatePhaseWork;
    /**
     * Create progress object with performance metrics
     */
    private createProgress;
    /**
     * Transform native progress to our format
     */
    private transformNativeProgress;
    /**
     * Estimate total files for progress calculation
     */
    private estimateTotalFiles;
    /**
     * Check if analysis has been cancelled
     */
    private checkCancellation;
    /**
     * Validate and merge streaming options with defaults
     */
    private validateAndMergeOptions;
}
export {};
//# sourceMappingURL=index.d.ts.map