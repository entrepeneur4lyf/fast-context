/**
 * Advanced Query Engine
 * Provides semantic search, relationship analysis, and architectural pattern detection
 */
import { AnalysisConfig, SymbolInfo, SemanticQuery, DependencyOptions, ComplexityOptions, ArchitecturalPattern, DependencyGraph, ComplexityReport, Result, AnalysisError } from '../types';
export declare class QueryEngine {
    private readonly nativeAnalyzer;
    private readonly cache;
    constructor(nativeAnalyzer: any, _config: AnalysisConfig);
    /**
     * Find symbols using semantic search
     */
    findSymbols(query: SemanticQuery): Promise<Result<SymbolInfo[], AnalysisError>>;
    /**
     * Find similar code using semantic similarity
     */
    findSimilarCode(code: string, threshold?: number): Promise<Result<SymbolInfo[], AnalysisError>>;
    /**
     * Get symbol dependencies with relationship analysis
     */
    getSymbolDependencies(symbol: string, options?: DependencyOptions): Promise<Result<DependencyGraph, AnalysisError>>;
    /**
     * Get symbol usages across the codebase
     */
    getSymbolUsages(symbol: string): Promise<Result<SymbolInfo[], AnalysisError>>;
    /**
     * Detect architectural patterns in the codebase
     */
    detectPatterns(): Promise<Result<ArchitecturalPattern[], AnalysisError>>;
    /**
     * Analyze code complexity
     */
    analyzeComplexity(options?: ComplexityOptions): Promise<Result<ComplexityReport, AnalysisError>>;
    /**
     * Find code smells and potential issues
     */
    findCodeSmells(): Promise<Result<SymbolInfo[], AnalysisError>>;
    /**
     * Clear query cache
     */
    clearCache(): Promise<void>;
    private performSymbolSearch;
    private simulateSimilarCodeSearch;
    private buildDependencyGraph;
    private findSymbolUsages;
    private analyzeArchitecturalPatterns;
    private generateComplexityReport;
    private detectCodeSmells;
    private simulateSymbolSearch;
    private createMockSymbol;
    private getCacheKey;
}
//# sourceMappingURL=index.d.ts.map