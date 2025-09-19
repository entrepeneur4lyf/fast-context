/**
 * Fast-Context MCP Server
 * 
 * A Model Context Protocol server that provides AI assistants with access to
 * Fast-Context's powerful codebase analysis capabilities.
 */

// Main server exports
export {
  createFastContextMcpServer,
  createDefaultFastContextMcpServer,
  DEFAULT_SERVER_CONFIG,
  registerTools,
  registerResources,
  registerPrompts
} from './server/index.js';

// Transport exports
export { startStdioServer, createStdioTransport } from './transports/stdio.js';
export { startHttpServer } from './transports/http.js';
export type { HttpServerConfig } from './transports/http.js';

// Type exports
export type {
  McpServerConfig,
  AnalyzeCodebaseInput,
  FindSymbolsInput,
  GetSymbolContextInput,
  AnalyzeDependenciesInput,
  DetectPatternsInput,
  GetComplexityMetricsInput,
  QuerySemanticInput,
  CodeReviewPromptArgs,
  RefactoringSuggestionsPromptArgs,
  ArchitectureAnalysisPromptArgs,
  DocumentationGenerationPromptArgs
} from './types/index.js';

// Schema exports for external validation
export {
  AnalyzeCodebaseInputSchema,
  FindSymbolsInputSchema,
  GetSymbolContextInputSchema,
  AnalyzeDependenciesInputSchema,
  DetectPatternsInputSchema,
  GetComplexityMetricsInputSchema,
  QuerySemanticInputSchema,
  CodeReviewPromptArgsSchema,
  RefactoringSuggestionsPromptArgsSchema,
  ArchitectureAnalysisPromptArgsSchema,
  DocumentationGenerationPromptArgsSchema,
  RESOURCE_PATTERNS
} from './types/index.js';

// Utility exports
export {
  validateServerConfig,
  createLogger,
  formatMcpError,
  validateProjectPath,
  sanitizeFilePath,
  parseCliArgs,
  getEnvironmentConfig,
  mergeConfigs
} from './utils/index.js';
