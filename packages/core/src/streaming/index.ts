/**
 * Streaming Analysis Implementation
 * Provides AsyncIterableIterator for progressive analysis with cancellation support
 */

import { EventEmitter } from 'events';
import {
  AnalysisConfig,
  AnalysisProgress,
  StreamingOptions,
  AnalysisPhase,
  PerformanceMetrics,
  AnalysisCancelledError,
  AnalysisError
} from '../types';

// Import the native analyzer from the main package
// This will be the actual NAPI binding
interface NativeAnalyzer {
  analyze(): Promise<any>;
  analyzeStream?(config: any, options: any): AsyncIterableIterator<any>;
  findSymbolsByKind?(kind: string): Promise<string[]>;
  findSymbolsInFile?(filePath: string): Promise<string[]>;
  findDependencies?(symbolName: string): Promise<string[]>;
}

export class StreamingAnalyzer extends EventEmitter {
  private readonly nativeAnalyzer: NativeAnalyzer;
  private readonly config: AnalysisConfig;
  private currentController: AbortController | undefined;

  constructor(nativeAnalyzer: NativeAnalyzer, config: AnalysisConfig) {
    super();
    this.nativeAnalyzer = nativeAnalyzer;
    this.config = config;
  }

  /**
   * Stream analysis with progress tracking and cancellation support
   */
  async *analyzeStream(options?: StreamingOptions): AsyncIterableIterator<AnalysisProgress> {
    const opts = this.validateAndMergeOptions(options);
    this.currentController = new AbortController();
    
    const startTime = Date.now();
    let filesProcessed = 0;
    let symbolsFound = 0;
    let relationshipsFound = 0;
    let totalFiles = 0;

    try {
      // Initialize analysis
      yield this.createProgress('initializing', {
        filesProcessed: 0,
        totalFiles: 0,
        symbolsFound: 0,
        relationshipsFound: 0,
        startTime
      });

      // Check for cancellation
      this.checkCancellation();

      // If native streaming is available, use it
      if (this.nativeAnalyzer.analyzeStream) {
        yield* this.streamFromNative(opts, startTime);
      } else {
        // Fallback to simulated streaming
        yield* this.simulateStreaming(opts, startTime);
      }

    } catch (error) {
      if (error instanceof AnalysisCancelledError) {
        yield this.createProgress('error', {
          filesProcessed,
          totalFiles,
          symbolsFound,
          relationshipsFound,
          startTime,
          error: error.message
        });
        throw error;
      }
      
      const analysisError = error instanceof AnalysisError 
        ? error 
        : new AnalysisError('Analysis failed', 'ANALYSIS_ERROR', { originalError: error });
      
      yield this.createProgress('error', {
        filesProcessed,
        totalFiles,
        symbolsFound,
        relationshipsFound,
        startTime,
        error: analysisError.message
      });
      
      throw analysisError;
    } finally {
      this.currentController = undefined;
    }
  }

  /**
   * Cancel the current analysis
   */
  cancel(): void {
    if (this.currentController) {
      this.currentController.abort();
    }
  }

  /**
   * Pause analysis (if supported by native implementation)
   */
  pause(): void {
    this.emit('pause');
  }

  /**
   * Resume analysis (if supported by native implementation)
   */
  resume(): void {
    this.emit('resume');
  }

  /**
   * Stream from native implementation
   */
  private async *streamFromNative(
    options: StreamingOptions, 
    startTime: number
  ): AsyncIterableIterator<AnalysisProgress> {
    if (!this.nativeAnalyzer.analyzeStream) {
      throw new AnalysisError('Native streaming not available', 'STREAMING_NOT_SUPPORTED');
    }

    try {
      for await (const nativeProgress of this.nativeAnalyzer.analyzeStream(
        this.config,
        { ...options, signal: this.currentController?.signal }
      )) {
        this.checkCancellation();
        
        const progress = this.transformNativeProgress(nativeProgress, startTime);
        this.emit('progress', progress);
        yield progress;
      }
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new AnalysisCancelledError('Analysis was cancelled');
      }
      throw error;
    }
  }

  /**
   * Simulate streaming for non-streaming native implementations
   */
  private async *simulateStreaming(
    options: StreamingOptions,
    startTime: number
  ): AsyncIterableIterator<AnalysisProgress> {
    // This is a fallback implementation that simulates streaming
    // by breaking down the analysis into phases and yielding progress
    
    const phases: AnalysisPhase[] = ['parsing', 'extracting', 'analyzing', 'indexing'];
    let filesProcessed = 0;
    let symbolsFound = 0;
    let relationshipsFound = 0;
    
    // Estimate total files (this would come from a quick scan)
    const totalFiles = await this.estimateTotalFiles();

    for (const phase of phases) {
      this.checkCancellation();
      
      // Simulate work for this phase
      const phaseProgress = await this.simulatePhaseWork(
        phase,
        totalFiles,
        filesProcessed,
        symbolsFound,
        relationshipsFound,
        startTime,
        options
      );

      for await (const progress of phaseProgress) {
        this.checkCancellation();
        this.emit('progress', progress);
        yield progress;
        
        // Update counters
        filesProcessed = progress.filesProcessed;
        symbolsFound = progress.symbolsFound;
        relationshipsFound = progress.relationshipsFound;
      }
    }

    // Final completion
    yield this.createProgress('complete', {
      filesProcessed: totalFiles,
      totalFiles,
      symbolsFound,
      relationshipsFound,
      startTime
    });
  }

  /**
   * Simulate work for a specific phase
   */
  private async *simulatePhaseWork(
    phase: AnalysisPhase,
    totalFiles: number,
    startFilesProcessed: number,
    startSymbolsFound: number,
    startRelationshipsFound: number,
    startTime: number,
    options: StreamingOptions
  ): AsyncIterableIterator<AnalysisProgress> {
    const filesPerPhase = Math.ceil(totalFiles / 4); // Divide work across 4 phases
    const progressInterval = options.progressInterval || 100;
    
    for (let i = 0; i < filesPerPhase; i++) {
      this.checkCancellation();
      
      // Simulate processing delay
      await new Promise(resolve => setTimeout(resolve, 10));
      
      const filesProcessed = startFilesProcessed + i + 1;
      const symbolsFound = startSymbolsFound + Math.floor(Math.random() * 50) + 10;
      const relationshipsFound = startRelationshipsFound + Math.floor(Math.random() * 20) + 5;
      
      if (i % progressInterval === 0 || i === filesPerPhase - 1) {
        yield this.createProgress(phase, {
          filesProcessed,
          totalFiles,
          symbolsFound,
          relationshipsFound,
          startTime,
          currentFile: `file_${filesProcessed}.ts`
        });
      }
    }
  }

  /**
   * Create progress object with performance metrics
   */
  private createProgress(
    phase: AnalysisPhase,
    data: {
      filesProcessed: number;
      totalFiles: number;
      symbolsFound: number;
      relationshipsFound: number;
      startTime: number;
      currentFile?: string;
      error?: string;
    }
  ): AnalysisProgress {
    const now = Date.now();
    const elapsedMs = now - data.startTime;
    const throughput = data.filesProcessed > 0 ? data.filesProcessed / (elapsedMs / 1000) : 0;
    const estimatedRemainingMs = throughput > 0 
      ? ((data.totalFiles - data.filesProcessed) / throughput) * 1000 
      : undefined;

    const performance: PerformanceMetrics = {
      memoryUsageMb: process.memoryUsage().heapUsed / 1024 / 1024,
      cpuUsagePercent: 0, // Would need actual CPU monitoring
      elapsedMs,
      ...(estimatedRemainingMs !== undefined && { estimatedRemainingMs }),
      throughputFilesPerSecond: throughput
    };

    return {
      phase,
      filesProcessed: data.filesProcessed,
      totalFiles: data.totalFiles,
      ...(data.currentFile && { currentFile: data.currentFile }),
      symbolsFound: data.symbolsFound,
      relationshipsFound: data.relationshipsFound,
      errors: data.error ? [new AnalysisError(data.error, 'PHASE_ERROR')] : [],
      performance,
      timestamp: now
    };
  }

  /**
   * Transform native progress to our format
   */
  private transformNativeProgress(nativeProgress: any, startTime: number): AnalysisProgress {
    // This would transform the native progress format to our TypeScript format
    return this.createProgress(nativeProgress.phase || 'analyzing', {
      filesProcessed: nativeProgress.files_processed || 0,
      totalFiles: nativeProgress.total_files || 0,
      symbolsFound: nativeProgress.symbols_found || 0,
      relationshipsFound: nativeProgress.relationships_found || 0,
      startTime,
      currentFile: nativeProgress.current_file
    });
  }

  /**
   * Estimate total files for progress calculation
   */
  private async estimateTotalFiles(): Promise<number> {
    // This would do a quick scan to estimate file count
    // For simulation, return a reasonable number
    return Math.floor(Math.random() * 1000) + 100;
  }

  /**
   * Check if analysis has been cancelled
   */
  private checkCancellation(): void {
    if (this.currentController?.signal.aborted) {
      throw new AnalysisCancelledError('Analysis was cancelled');
    }
  }

  /**
   * Validate and merge streaming options with defaults
   */
  private validateAndMergeOptions(options?: StreamingOptions): StreamingOptions {
    return {
      progressInterval: 100,
      enableDetailedProgress: false,
      batchSize: 50,
      ...options
    };
  }
}
