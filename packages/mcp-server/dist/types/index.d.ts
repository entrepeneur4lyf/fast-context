/**
 * Types for Fast-Context MCP Server
 */
import { z } from 'zod';
export declare const AnalyzeCodebaseInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    options: z.ZodOptional<z.ZodObject<{
        enableCaching: z.ZodOptional<z.ZodBoolean>;
        parallelProcessing: z.ZodOptional<z.ZodBoolean>;
        languages: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
        ignorePatterns: z.ZodOptional<z.ZodArray<z.ZodString, "many">>;
        maxFiles: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        languages?: string[] | undefined;
        ignorePatterns?: string[] | undefined;
        enableCaching?: boolean | undefined;
        parallelProcessing?: boolean | undefined;
        maxFiles?: number | undefined;
    }, {
        languages?: string[] | undefined;
        ignorePatterns?: string[] | undefined;
        enableCaching?: boolean | undefined;
        parallelProcessing?: boolean | undefined;
        maxFiles?: number | undefined;
    }>>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    options?: {
        languages?: string[] | undefined;
        ignorePatterns?: string[] | undefined;
        enableCaching?: boolean | undefined;
        parallelProcessing?: boolean | undefined;
        maxFiles?: number | undefined;
    } | undefined;
}, {
    projectPath: string;
    options?: {
        languages?: string[] | undefined;
        ignorePatterns?: string[] | undefined;
        enableCaching?: boolean | undefined;
        parallelProcessing?: boolean | undefined;
        maxFiles?: number | undefined;
    } | undefined;
}>;
export declare const FindSymbolsInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    query: z.ZodString;
    kind: z.ZodOptional<z.ZodArray<z.ZodEnum<["function", "class", "variable", "interface", "type", "enum", "module"]>, "many">>;
    limit: z.ZodDefault<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    query: string;
    limit: number;
    kind?: ("function" | "type" | "class" | "variable" | "interface" | "enum" | "module")[] | undefined;
}, {
    projectPath: string;
    query: string;
    kind?: ("function" | "type" | "class" | "variable" | "interface" | "enum" | "module")[] | undefined;
    limit?: number | undefined;
}>;
export declare const GetSymbolContextInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    symbolName: z.ZodString;
    includeReferences: z.ZodDefault<z.ZodBoolean>;
    depth: z.ZodDefault<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    symbolName: string;
    includeReferences: boolean;
    depth: number;
}, {
    projectPath: string;
    symbolName: string;
    includeReferences?: boolean | undefined;
    depth?: number | undefined;
}>;
export declare const AnalyzeDependenciesInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    symbolName: z.ZodOptional<z.ZodString>;
    depth: z.ZodDefault<z.ZodNumber>;
    includeExternal: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    depth: number;
    includeExternal: boolean;
    symbolName?: string | undefined;
}, {
    projectPath: string;
    symbolName?: string | undefined;
    depth?: number | undefined;
    includeExternal?: boolean | undefined;
}>;
export declare const DetectPatternsInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    patterns: z.ZodOptional<z.ZodArray<z.ZodEnum<["mvc", "repository", "factory", "singleton", "observer", "strategy", "adapter"]>, "many">>;
    includeRecommendations: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    includeRecommendations: boolean;
    patterns?: ("mvc" | "repository" | "factory" | "singleton" | "observer" | "strategy" | "adapter")[] | undefined;
}, {
    projectPath: string;
    patterns?: ("mvc" | "repository" | "factory" | "singleton" | "observer" | "strategy" | "adapter")[] | undefined;
    includeRecommendations?: boolean | undefined;
}>;
export declare const GetComplexityMetricsInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    filePath: z.ZodOptional<z.ZodString>;
    includeDetails: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    includeDetails: boolean;
    filePath?: string | undefined;
}, {
    projectPath: string;
    filePath?: string | undefined;
    includeDetails?: boolean | undefined;
}>;
export declare const QuerySemanticInputSchema: z.ZodObject<{
    projectPath: z.ZodString;
    query: z.ZodString;
    limit: z.ZodDefault<z.ZodNumber>;
    similarity: z.ZodDefault<z.ZodNumber>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    query: string;
    limit: number;
    similarity: number;
}, {
    projectPath: string;
    query: string;
    limit?: number | undefined;
    similarity?: number | undefined;
}>;
export declare const CodeReviewPromptArgsSchema: z.ZodObject<{
    filePath: z.ZodString;
    focusAreas: z.ZodOptional<z.ZodArray<z.ZodEnum<["performance", "security", "maintainability", "style", "bugs"]>, "many">>;
    severity: z.ZodDefault<z.ZodEnum<["low", "medium", "high"]>>;
}, "strip", z.ZodTypeAny, {
    filePath: string;
    severity: "medium" | "low" | "high";
    focusAreas?: ("performance" | "security" | "maintainability" | "style" | "bugs")[] | undefined;
}, {
    filePath: string;
    severity?: "medium" | "low" | "high" | undefined;
    focusAreas?: ("performance" | "security" | "maintainability" | "style" | "bugs")[] | undefined;
}>;
export declare const RefactoringSuggestionsPromptArgsSchema: z.ZodObject<{
    symbolName: z.ZodString;
    refactoringType: z.ZodOptional<z.ZodEnum<["extract", "inline", "rename", "move", "optimize"]>>;
    includeExamples: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    symbolName: string;
    includeExamples: boolean;
    refactoringType?: "extract" | "inline" | "rename" | "move" | "optimize" | undefined;
}, {
    symbolName: string;
    refactoringType?: "extract" | "inline" | "rename" | "move" | "optimize" | undefined;
    includeExamples?: boolean | undefined;
}>;
export declare const ArchitectureAnalysisPromptArgsSchema: z.ZodObject<{
    projectPath: z.ZodString;
    analysisType: z.ZodDefault<z.ZodEnum<["overview", "patterns", "dependencies", "quality"]>>;
    includeRecommendations: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    projectPath: string;
    includeRecommendations: boolean;
    analysisType: "patterns" | "dependencies" | "overview" | "quality";
}, {
    projectPath: string;
    includeRecommendations?: boolean | undefined;
    analysisType?: "patterns" | "dependencies" | "overview" | "quality" | undefined;
}>;
export declare const DocumentationGenerationPromptArgsSchema: z.ZodObject<{
    symbolName: z.ZodString;
    format: z.ZodDefault<z.ZodEnum<["jsdoc", "markdown", "typescript"]>>;
    includeExamples: z.ZodDefault<z.ZodBoolean>;
}, "strip", z.ZodTypeAny, {
    symbolName: string;
    includeExamples: boolean;
    format: "markdown" | "jsdoc" | "typescript";
}, {
    symbolName: string;
    includeExamples?: boolean | undefined;
    format?: "markdown" | "jsdoc" | "typescript" | undefined;
}>;
export type AnalyzeCodebaseInput = z.infer<typeof AnalyzeCodebaseInputSchema>;
export type FindSymbolsInput = z.infer<typeof FindSymbolsInputSchema>;
export type GetSymbolContextInput = z.infer<typeof GetSymbolContextInputSchema>;
export type AnalyzeDependenciesInput = z.infer<typeof AnalyzeDependenciesInputSchema>;
export type DetectPatternsInput = z.infer<typeof DetectPatternsInputSchema>;
export type GetComplexityMetricsInput = z.infer<typeof GetComplexityMetricsInputSchema>;
export type QuerySemanticInput = z.infer<typeof QuerySemanticInputSchema>;
export type CodeReviewPromptArgs = z.infer<typeof CodeReviewPromptArgsSchema>;
export type RefactoringSuggestionsPromptArgs = z.infer<typeof RefactoringSuggestionsPromptArgsSchema>;
export type ArchitectureAnalysisPromptArgs = z.infer<typeof ArchitectureAnalysisPromptArgsSchema>;
export type DocumentationGenerationPromptArgs = z.infer<typeof DocumentationGenerationPromptArgsSchema>;
export interface McpServerConfig {
    name: string;
    version: string;
    maxConcurrentAnalyses?: number;
    defaultTimeout?: number;
    enableLogging?: boolean;
    logLevel?: 'debug' | 'info' | 'warn' | 'error';
}
export declare const RESOURCE_PATTERNS: {
    readonly CODEBASE_ANALYSIS: "codebase://analysis/{projectPath}";
    readonly SYMBOL_INFO: "symbols://project/{projectPath}/{symbolName}";
    readonly DEPENDENCIES: "dependencies://project/{projectPath}/{symbolName}";
    readonly FILE_ANALYSIS: "files://project/{projectPath}/{filePath}";
};
//# sourceMappingURL=index.d.ts.map