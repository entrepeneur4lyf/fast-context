/**
 * Types for Fast-Context MCP Server
 */

import { z } from 'zod';
import type { AnalysisConfig, SymbolInfo, DependencyInfo, AnalysisResult } from '@fast-context/core';

// Tool Input Schemas
export const AnalyzeCodebaseInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  options: z.object({
    enableCaching: z.boolean().optional(),
    parallelProcessing: z.boolean().optional(),
    languages: z.array(z.string()).optional(),
    ignorePatterns: z.array(z.string()).optional(),
    maxFiles: z.number().positive().optional()
  }).optional().describe('Analysis configuration options')
});

export const FindSymbolsInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  query: z.string().describe('Search query for symbols'),
  kind: z.array(z.enum(['function', 'class', 'variable', 'interface', 'type', 'enum', 'module'])).optional().describe('Filter by symbol kinds'),
  limit: z.number().positive().max(1000).default(100).describe('Maximum number of results to return')
});

export const GetSymbolContextInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  symbolName: z.string().describe('Name of the symbol to analyze'),
  includeReferences: z.boolean().default(true).describe('Include symbol references and usage'),
  depth: z.number().positive().max(10).default(3).describe('Depth of dependency analysis')
});

export const AnalyzeDependenciesInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  symbolName: z.string().optional().describe('Specific symbol to analyze (if not provided, analyzes entire project)'),
  depth: z.number().positive().max(10).default(5).describe('Maximum depth of dependency traversal'),
  includeExternal: z.boolean().default(false).describe('Include external dependencies')
});

export const DetectPatternsInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  patterns: z.array(z.enum(['mvc', 'repository', 'factory', 'singleton', 'observer', 'strategy', 'adapter'])).optional().describe('Specific patterns to detect'),
  includeRecommendations: z.boolean().default(true).describe('Include improvement recommendations')
});

export const GetComplexityMetricsInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  filePath: z.string().optional().describe('Specific file to analyze (if not provided, analyzes entire project)'),
  includeDetails: z.boolean().default(false).describe('Include detailed complexity breakdown')
});

export const QuerySemanticInputSchema = z.object({
  projectPath: z.string().describe('Path to the project root directory'),
  query: z.string().describe('Natural language query for semantic search'),
  limit: z.number().positive().max(100).default(20).describe('Maximum number of results'),
  similarity: z.number().min(0).max(1).default(0.7).describe('Minimum similarity threshold')
});

// Prompt Argument Schemas
export const CodeReviewPromptArgsSchema = z.object({
  filePath: z.string().describe('Path to the file to review'),
  focusAreas: z.array(z.enum(['performance', 'security', 'maintainability', 'style', 'bugs'])).optional().describe('Specific areas to focus on'),
  severity: z.enum(['low', 'medium', 'high']).default('medium').describe('Review severity level')
});

export const RefactoringSuggestionsPromptArgsSchema = z.object({
  symbolName: z.string().describe('Symbol to analyze for refactoring'),
  refactoringType: z.enum(['extract', 'inline', 'rename', 'move', 'optimize']).optional().describe('Type of refactoring to suggest'),
  includeExamples: z.boolean().default(true).describe('Include code examples in suggestions')
});

export const ArchitectureAnalysisPromptArgsSchema = z.object({
  projectPath: z.string().describe('Path to the project root'),
  analysisType: z.enum(['overview', 'patterns', 'dependencies', 'quality']).default('overview').describe('Type of architecture analysis'),
  includeRecommendations: z.boolean().default(true).describe('Include improvement recommendations')
});

export const DocumentationGenerationPromptArgsSchema = z.object({
  symbolName: z.string().describe('Symbol to generate documentation for'),
  format: z.enum(['jsdoc', 'markdown', 'typescript']).default('jsdoc').describe('Documentation format'),
  includeExamples: z.boolean().default(true).describe('Include usage examples')
});

// Type exports for use in implementation
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

// Server Configuration
export interface McpServerConfig {
  name: string;
  version: string;
  maxConcurrentAnalyses?: number;
  defaultTimeout?: number;
  enableLogging?: boolean;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

// Resource URI patterns
export const RESOURCE_PATTERNS = {
  CODEBASE_ANALYSIS: 'codebase://analysis/{projectPath}',
  SYMBOL_INFO: 'symbols://project/{projectPath}/{symbolName}',
  DEPENDENCIES: 'dependencies://project/{projectPath}/{symbolName}',
  FILE_ANALYSIS: 'files://project/{projectPath}/{filePath}'
} as const;
