/**
 * Enhanced TypeScript types for Fast-Context SDK
 * Provides strict type safety with runtime validation
 */

import { z } from 'zod';

// ============================================================================
// Result Type Pattern for Error Handling
// ============================================================================

export type Result<T, E = Error> = 
  | { success: true; data: T }
  | { success: false; error: E };

export const Ok = <T>(data: T): Result<T, never> => ({ success: true, data });
export const Err = <E>(error: E): Result<never, E> => ({ success: false, error });

// ============================================================================
// Configuration Types with Validation
// ============================================================================

export const AnalysisConfigSchema = z.object({
  projectRoot: z.string().min(1, 'Project root cannot be empty'),
  languages: z.array(z.string()).optional(),
  ignorePatterns: z.array(z.string()).optional(),
  enableCaching: z.boolean().default(true),
  cachePolicy: z.enum(['auto', 'minimal', 'balanced', 'adaptive', 'persistent']).default('adaptive'),
  enableWatching: z.boolean().default(false),
  maxFiles: z.number().positive().optional(),
  parallelProcessing: z.boolean().default(true),
  performance: z.object({
    maxMemoryMb: z.number().positive().default(1024),
    timeoutMs: z.number().positive().default(30000),
    workerThreads: z.number().positive().default(4),
    chunkSize: z.number().positive().default(100)
  }).optional()
});

export type AnalysisConfig = z.infer<typeof AnalysisConfigSchema>;

export const StreamingOptionsSchema = z.object({
  signal: z.instanceof(AbortSignal).optional(),
  progressInterval: z.number().positive().default(100),
  enableDetailedProgress: z.boolean().default(false),
  batchSize: z.number().positive().default(50)
});

export type StreamingOptions = z.infer<typeof StreamingOptionsSchema>;

// ============================================================================
// Analysis Progress and Results
// ============================================================================

export const AnalysisPhaseSchema = z.enum([
  'initializing',
  'parsing',
  'extracting',
  'analyzing',
  'indexing',
  'complete',
  'error'
]);

export type AnalysisPhase = z.infer<typeof AnalysisPhaseSchema>;

export interface AnalysisProgress {
  readonly phase: AnalysisPhase;
  readonly filesProcessed: number;
  readonly totalFiles: number;
  readonly currentFile?: string;
  readonly symbolsFound: number;
  readonly relationshipsFound: number;
  readonly errors: AnalysisError[];
  readonly performance: PerformanceMetrics;
  readonly timestamp: number;
}

export interface PerformanceMetrics {
  readonly memoryUsageMb: number;
  readonly cpuUsagePercent: number;
  readonly elapsedMs: number;
  readonly estimatedRemainingMs?: number;
  readonly throughputFilesPerSecond: number;
}

export interface AnalysisResult {
  readonly fileCount: number;
  readonly symbolCount: number;
  readonly relationshipCount: number;
  readonly languages: readonly string[];
  readonly durationMs: number;
  readonly memoryUsageMb?: number;
  readonly performance: PerformanceMetrics;
  readonly summary?: string;
  readonly insights?: readonly string[];
  readonly recommendations?: readonly string[];
}

// ============================================================================
// Symbol and Query Types
// ============================================================================

export const SymbolKindSchema = z.enum([
  'function',
  'class',
  'interface',
  'type',
  'variable',
  'constant',
  'enum',
  'module',
  'namespace',
  'property',
  'method',
  'constructor',
  'field',
  'parameter',
  'import',
  'export'
]);

export type SymbolKind = z.infer<typeof SymbolKindSchema>;

export interface SymbolInfo {
  readonly name: string;
  readonly kind: SymbolKind;
  readonly filePath: string;
  readonly line: number;
  readonly column: number;
  readonly scope: string;
  readonly language: string;
  readonly documentation?: string;
  readonly signature?: string;
  readonly complexity?: number;
}

export interface SemanticQuery {
  readonly text: string;
  readonly kind?: SymbolKind;
  readonly language?: string;
  readonly maxResults?: number;
  readonly similarity?: number;
}

export interface DependencyOptions {
  readonly depth?: number;
  readonly includeTransitive?: boolean;
  readonly direction?: 'incoming' | 'outgoing' | 'both';
}

export interface ComplexityOptions {
  readonly threshold?: number;
  readonly includeMetrics?: boolean;
  readonly sortBy?: 'complexity' | 'name' | 'file';
}

// ============================================================================
// Error Types
// ============================================================================

export class AnalysisError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly context?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'AnalysisError';
  }
}

export class ValidationError extends AnalysisError {
  constructor(message: string, public readonly validationErrors: z.ZodError) {
    super(message, 'VALIDATION_ERROR', { validationErrors });
    this.name = 'ValidationError';
  }
}

export class AnalysisCancelledError extends AnalysisError {
  constructor(message: string = 'Analysis was cancelled') {
    super(message, 'ANALYSIS_CANCELLED');
    this.name = 'AnalysisCancelledError';
  }
}

export class ConfigurationError extends AnalysisError {
  constructor(message: string, context?: Record<string, unknown>) {
    super(message, 'CONFIGURATION_ERROR', context);
    this.name = 'ConfigurationError';
  }
}

// ============================================================================
// Utility Types
// ============================================================================

export type PresetName = 'fast' | 'balanced' | 'thorough';

export interface ArchitecturalPattern {
  readonly name: string;
  readonly description: string;
  readonly confidence: number;
  readonly examples: readonly string[];
}

export interface DependencyGraph {
  readonly nodes: readonly SymbolInfo[];
  readonly edges: readonly DependencyEdge[];
}

export interface DependencyEdge {
  readonly from: string;
  readonly to: string;
  readonly type: 'calls' | 'imports' | 'extends' | 'implements' | 'uses';
  readonly weight: number;
}

export interface ComplexityReport {
  readonly averageComplexity: number;
  readonly maxComplexity: number;
  readonly complexSymbols: readonly (SymbolInfo & { complexity: number })[];
  readonly recommendations: readonly string[];
}

// ============================================================================
// Type Guards and Validators
// ============================================================================

export function isAnalysisProgress(obj: unknown): obj is AnalysisProgress {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'phase' in obj &&
    'filesProcessed' in obj &&
    'totalFiles' in obj &&
    'symbolsFound' in obj
  );
}

export function validateConfig(config: unknown): Result<AnalysisConfig, ValidationError> {
  try {
    const validated = AnalysisConfigSchema.parse(config);
    return Ok(validated);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return Err(new ValidationError('Invalid configuration', error));
    }
    return Err(new ValidationError('Unknown validation error', error as z.ZodError));
  }
}

export function validateStreamingOptions(options: unknown): Result<StreamingOptions, ValidationError> {
  try {
    const validated = StreamingOptionsSchema.parse(options);
    return Ok(validated);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return Err(new ValidationError('Invalid streaming options', error));
    }
    return Err(new ValidationError('Unknown validation error', error as z.ZodError));
  }
}
