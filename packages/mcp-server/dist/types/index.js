"use strict";
/**
 * Types for Fast-Context MCP Server
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.RESOURCE_PATTERNS = exports.DocumentationGenerationPromptArgsSchema = exports.ArchitectureAnalysisPromptArgsSchema = exports.RefactoringSuggestionsPromptArgsSchema = exports.CodeReviewPromptArgsSchema = exports.QuerySemanticInputSchema = exports.GetComplexityMetricsInputSchema = exports.DetectPatternsInputSchema = exports.AnalyzeDependenciesInputSchema = exports.GetSymbolContextInputSchema = exports.FindSymbolsInputSchema = exports.AnalyzeCodebaseInputSchema = void 0;
const zod_1 = require("zod");
// Tool Input Schemas
exports.AnalyzeCodebaseInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    options: zod_1.z.object({
        enableCaching: zod_1.z.boolean().optional(),
        parallelProcessing: zod_1.z.boolean().optional(),
        languages: zod_1.z.array(zod_1.z.string()).optional(),
        ignorePatterns: zod_1.z.array(zod_1.z.string()).optional(),
        maxFiles: zod_1.z.number().positive().optional()
    }).optional().describe('Analysis configuration options')
});
exports.FindSymbolsInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    query: zod_1.z.string().describe('Search query for symbols'),
    kind: zod_1.z.array(zod_1.z.enum(['function', 'class', 'variable', 'interface', 'type', 'enum', 'module'])).optional().describe('Filter by symbol kinds'),
    limit: zod_1.z.number().positive().max(1000).default(100).describe('Maximum number of results to return')
});
exports.GetSymbolContextInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    symbolName: zod_1.z.string().describe('Name of the symbol to analyze'),
    includeReferences: zod_1.z.boolean().default(true).describe('Include symbol references and usage'),
    depth: zod_1.z.number().positive().max(10).default(3).describe('Depth of dependency analysis')
});
exports.AnalyzeDependenciesInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    symbolName: zod_1.z.string().optional().describe('Specific symbol to analyze (if not provided, analyzes entire project)'),
    depth: zod_1.z.number().positive().max(10).default(5).describe('Maximum depth of dependency traversal'),
    includeExternal: zod_1.z.boolean().default(false).describe('Include external dependencies')
});
exports.DetectPatternsInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    patterns: zod_1.z.array(zod_1.z.enum(['mvc', 'repository', 'factory', 'singleton', 'observer', 'strategy', 'adapter'])).optional().describe('Specific patterns to detect'),
    includeRecommendations: zod_1.z.boolean().default(true).describe('Include improvement recommendations')
});
exports.GetComplexityMetricsInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    filePath: zod_1.z.string().optional().describe('Specific file to analyze (if not provided, analyzes entire project)'),
    includeDetails: zod_1.z.boolean().default(false).describe('Include detailed complexity breakdown')
});
exports.QuerySemanticInputSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root directory'),
    query: zod_1.z.string().describe('Natural language query for semantic search'),
    limit: zod_1.z.number().positive().max(100).default(20).describe('Maximum number of results'),
    similarity: zod_1.z.number().min(0).max(1).default(0.7).describe('Minimum similarity threshold')
});
// Prompt Argument Schemas
exports.CodeReviewPromptArgsSchema = zod_1.z.object({
    filePath: zod_1.z.string().describe('Path to the file to review'),
    focusAreas: zod_1.z.array(zod_1.z.enum(['performance', 'security', 'maintainability', 'style', 'bugs'])).optional().describe('Specific areas to focus on'),
    severity: zod_1.z.enum(['low', 'medium', 'high']).default('medium').describe('Review severity level')
});
exports.RefactoringSuggestionsPromptArgsSchema = zod_1.z.object({
    symbolName: zod_1.z.string().describe('Symbol to analyze for refactoring'),
    refactoringType: zod_1.z.enum(['extract', 'inline', 'rename', 'move', 'optimize']).optional().describe('Type of refactoring to suggest'),
    includeExamples: zod_1.z.boolean().default(true).describe('Include code examples in suggestions')
});
exports.ArchitectureAnalysisPromptArgsSchema = zod_1.z.object({
    projectPath: zod_1.z.string().describe('Path to the project root'),
    analysisType: zod_1.z.enum(['overview', 'patterns', 'dependencies', 'quality']).default('overview').describe('Type of architecture analysis'),
    includeRecommendations: zod_1.z.boolean().default(true).describe('Include improvement recommendations')
});
exports.DocumentationGenerationPromptArgsSchema = zod_1.z.object({
    symbolName: zod_1.z.string().describe('Symbol to generate documentation for'),
    format: zod_1.z.enum(['jsdoc', 'markdown', 'typescript']).default('jsdoc').describe('Documentation format'),
    includeExamples: zod_1.z.boolean().default(true).describe('Include usage examples')
});
// Resource URI patterns
exports.RESOURCE_PATTERNS = {
    CODEBASE_ANALYSIS: 'codebase://analysis/{projectPath}',
    SYMBOL_INFO: 'symbols://project/{projectPath}/{symbolName}',
    DEPENDENCIES: 'dependencies://project/{projectPath}/{symbolName}',
    FILE_ANALYSIS: 'files://project/{projectPath}/{filePath}'
};
//# sourceMappingURL=index.js.map