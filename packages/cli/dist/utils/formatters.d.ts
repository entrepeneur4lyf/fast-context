/**
 * Output formatters for CLI results
 */
export interface FormatOptions {
    format: 'table' | 'json' | 'yaml' | 'markdown';
    verbose?: boolean;
    quiet?: boolean;
    json?: boolean;
}
export interface AnalysisResults {
    fileCount: number;
    symbolCount: number;
    relationshipCount: number;
    languages: readonly string[];
    durationMs: number;
    performance?: {
        memoryUsageMb: number;
        cpuUsagePercent: number;
        elapsedMs: number;
        estimatedRemainingMs?: number;
        throughputFilesPerSecond: number;
    };
    insights?: readonly string[];
    recommendations?: readonly string[];
    symbols?: any[];
    dependencies?: any[];
    patterns?: any[];
    metrics?: any;
}
export declare function formatAnalysisResults(results: AnalysisResults, options: FormatOptions): Promise<string>;
export declare function formatSymbolList(symbols: any[], options: FormatOptions): string;
export declare function formatDependencyGraph(dependencies: any[], options: FormatOptions): string;
//# sourceMappingURL=formatters.d.ts.map