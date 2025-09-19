/**
 * Enhanced TypeScript types for Fast-Context SDK
 * Provides strict type safety with runtime validation
 */
import { z } from 'zod';
export type Result<T, E = Error> = {
    success: true;
    data: T;
} | {
    success: false;
    error: E;
};
export declare const Ok: <T>(data: T) => Result<T, never>;
export declare const Err: <E>(error: E) => Result<never, E>;
export declare const AnalysisConfigSchema: z.ZodObject<{
    projectRoot: z.ZodString;
    languages: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
    ignorePatterns: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
    enableCaching: z.ZodDefault<z.ZodBoolean>;
    cachePolicy: z.ZodDefault<z.ZodEnum<["auto", "minimal", "balanced", "adaptive", "persistent"]>>;
    enableWatching: z.ZodDefault<z.ZodBoolean>;
    maxFiles: z.ZodOptional<z.ZodNumber>;
    parallelProcessing: z.ZodDefault<z.ZodBoolean>;
    performance: z.ZodOptional<z.ZodObject<{
        maxMemoryMb: z.ZodDefault<z.ZodNumber>;
        timeoutMs: z.ZodDefault<z.ZodNumber>;
        workerThreads: z.ZodDefault<z.ZodNumber>;
        chunkSize: z.ZodDefault<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        maxMemoryMb: number;
        timeoutMs: number;
        workerThreads: number;
        chunkSize: number;
    }, {
        maxMemoryMb?: number | undefined;
        timeoutMs?: number | undefined;
        workerThreads?: number | undefined;
        chunkSize?: number | undefined;
    }>>;
}, "strip", z.ZodTypeAny, {
    projectRoot: string;
    enableCaching: boolean;
    cachePolicy: "auto" | "minimal" | "balanced" | "adaptive" | "persistent";
    enableWatching: boolean;
    parallelProcessing: boolean;
    languages?: string[] | undefined;
    ignorePatterns?: string[] | undefined;
    maxFiles?: number | undefined;
    performance?: {
        maxMemoryMb: number;
        timeoutMs: number;
        workerThreads: number;
        chunkSize: number;
    } | undefined;
}, {
    projectRoot: string;
    languages?: string[] | undefined;
    ignorePatterns?: string[] | undefined;
    enableCaching?: boolean | undefined;
    cachePolicy?: "auto" | "minimal" | "balanced" | "adaptive" | "persistent" | undefined;
    enableWatching?: boolean | undefined;
    maxFiles?: number | undefined;
    parallelProcessing?: boolean | undefined;
    performance?: {
        maxMemoryMb?: number | undefined;
        timeoutMs?: number | undefined;
        workerThreads?: number | undefined;
        chunkSize?: number | undefined;
    } | undefined;
}>;
export type AnalysisConfig = z.infer<typeof AnalysisConfigSchema>;
export declare const StreamingOptionsSchema: z.ZodObject<{
    signal: z.ZodOptional<z.ZodType<AbortSignal, z.ZodTypeDef, AbortSignal>>;
    progressInterval: z.ZodDefault<z.ZodNumber>;
    enableDetailedProgress: z.ZodDefault<z.ZodBoolean>;
    batchSize: z.ZodDefault<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    progressInterval: number;
    enableDetailedProgress: boolean;
    batchSize: number;
    signal?: AbortSignal | undefined;
}, {
    signal?: AbortSignal | undefined;
    progressInterval?: number | undefined;
    enableDetailedProgress?: boolean | undefined;
    batchSize?: number | undefined;
}>;
export type StreamingOptions = z.infer<typeof StreamingOptionsSchema>;
export declare const AnalysisPhaseSchema: z.ZodEnum<["initializing", "parsing", "extracting", "analyzing", "indexing", "complete", "error"]>;
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
export declare const SymbolKindSchema: z.ZodEnum<["function", "class", "interface", "type", "variable", "constant", "enum", "module", "namespace", "property", "method", "constructor", "field", "parameter", "import", "export"]>;
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
export declare class AnalysisError extends Error {
    readonly code: string;
    readonly context?: Record<string, unknown> | undefined;
    constructor(message: string, code: string, context?: Record<string, unknown> | undefined);
}
export declare class ValidationError extends AnalysisError {
    readonly validationErrors: z.ZodError;
    constructor(message: string, validationErrors: z.ZodError);
}
export declare class AnalysisCancelledError extends AnalysisError {
    constructor(message?: string);
}
export declare class ConfigurationError extends AnalysisError {
    constructor(message: string, context?: Record<string, unknown>);
}
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
    readonly complexSymbols: readonly (SymbolInfo & {
        complexity: number;
    })[];
    readonly recommendations: readonly string[];
}
export declare function isAnalysisProgress(obj: unknown): obj is AnalysisProgress;
export declare function validateConfig(config: unknown): Result<AnalysisConfig, ValidationError>;
export declare function validateStreamingOptions(options: unknown): Result<StreamingOptions, ValidationError>;
//# sourceMappingURL=index.d.ts.map