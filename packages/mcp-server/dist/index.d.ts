/**
 * Fast-Context MCP Server
 *
 * A Model Context Protocol server that provides AI assistants with access to
 * Fast-Context's powerful codebase analysis capabilities.
 */
export { createFastContextMcpServer, createDefaultFastContextMcpServer, DEFAULT_SERVER_CONFIG, registerTools, registerResources, registerPrompts } from './server/index.js';
export { startStdioServer, createStdioTransport } from './transports/stdio.js';
export { startHttpServer } from './transports/http.js';
export type { HttpServerConfig } from './transports/http.js';
export type { McpServerConfig, AnalyzeCodebaseInput, FindSymbolsInput, GetSymbolContextInput, AnalyzeDependenciesInput, DetectPatternsInput, GetComplexityMetricsInput, QuerySemanticInput, CodeReviewPromptArgs, RefactoringSuggestionsPromptArgs, ArchitectureAnalysisPromptArgs, DocumentationGenerationPromptArgs } from './types/index.js';
export { AnalyzeCodebaseInputSchema, FindSymbolsInputSchema, GetSymbolContextInputSchema, AnalyzeDependenciesInputSchema, DetectPatternsInputSchema, GetComplexityMetricsInputSchema, QuerySemanticInputSchema, CodeReviewPromptArgsSchema, RefactoringSuggestionsPromptArgsSchema, ArchitectureAnalysisPromptArgsSchema, DocumentationGenerationPromptArgsSchema, RESOURCE_PATTERNS } from './types/index.js';
export { validateServerConfig, createLogger, formatMcpError, validateProjectPath, sanitizeFilePath, parseCliArgs, getEnvironmentConfig, mergeConfigs } from './utils/index.js';
//# sourceMappingURL=index.d.ts.map