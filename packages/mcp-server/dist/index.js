"use strict";
/**
 * Fast-Context MCP Server
 *
 * A Model Context Protocol server that provides AI assistants with access to
 * Fast-Context's powerful codebase analysis capabilities.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.mergeConfigs = exports.getEnvironmentConfig = exports.parseCliArgs = exports.sanitizeFilePath = exports.validateProjectPath = exports.formatMcpError = exports.createLogger = exports.validateServerConfig = exports.RESOURCE_PATTERNS = exports.DocumentationGenerationPromptArgsSchema = exports.ArchitectureAnalysisPromptArgsSchema = exports.RefactoringSuggestionsPromptArgsSchema = exports.CodeReviewPromptArgsSchema = exports.QuerySemanticInputSchema = exports.GetComplexityMetricsInputSchema = exports.DetectPatternsInputSchema = exports.AnalyzeDependenciesInputSchema = exports.GetSymbolContextInputSchema = exports.FindSymbolsInputSchema = exports.AnalyzeCodebaseInputSchema = exports.startHttpServer = exports.createStdioTransport = exports.startStdioServer = exports.registerPrompts = exports.registerResources = exports.registerTools = exports.DEFAULT_SERVER_CONFIG = exports.createDefaultFastContextMcpServer = exports.createFastContextMcpServer = void 0;
// Main server exports
var index_js_1 = require("./server/index.js");
Object.defineProperty(exports, "createFastContextMcpServer", { enumerable: true, get: function () { return index_js_1.createFastContextMcpServer; } });
Object.defineProperty(exports, "createDefaultFastContextMcpServer", { enumerable: true, get: function () { return index_js_1.createDefaultFastContextMcpServer; } });
Object.defineProperty(exports, "DEFAULT_SERVER_CONFIG", { enumerable: true, get: function () { return index_js_1.DEFAULT_SERVER_CONFIG; } });
Object.defineProperty(exports, "registerTools", { enumerable: true, get: function () { return index_js_1.registerTools; } });
Object.defineProperty(exports, "registerResources", { enumerable: true, get: function () { return index_js_1.registerResources; } });
Object.defineProperty(exports, "registerPrompts", { enumerable: true, get: function () { return index_js_1.registerPrompts; } });
// Transport exports
var stdio_js_1 = require("./transports/stdio.js");
Object.defineProperty(exports, "startStdioServer", { enumerable: true, get: function () { return stdio_js_1.startStdioServer; } });
Object.defineProperty(exports, "createStdioTransport", { enumerable: true, get: function () { return stdio_js_1.createStdioTransport; } });
var http_js_1 = require("./transports/http.js");
Object.defineProperty(exports, "startHttpServer", { enumerable: true, get: function () { return http_js_1.startHttpServer; } });
// Schema exports for external validation
var index_js_2 = require("./types/index.js");
Object.defineProperty(exports, "AnalyzeCodebaseInputSchema", { enumerable: true, get: function () { return index_js_2.AnalyzeCodebaseInputSchema; } });
Object.defineProperty(exports, "FindSymbolsInputSchema", { enumerable: true, get: function () { return index_js_2.FindSymbolsInputSchema; } });
Object.defineProperty(exports, "GetSymbolContextInputSchema", { enumerable: true, get: function () { return index_js_2.GetSymbolContextInputSchema; } });
Object.defineProperty(exports, "AnalyzeDependenciesInputSchema", { enumerable: true, get: function () { return index_js_2.AnalyzeDependenciesInputSchema; } });
Object.defineProperty(exports, "DetectPatternsInputSchema", { enumerable: true, get: function () { return index_js_2.DetectPatternsInputSchema; } });
Object.defineProperty(exports, "GetComplexityMetricsInputSchema", { enumerable: true, get: function () { return index_js_2.GetComplexityMetricsInputSchema; } });
Object.defineProperty(exports, "QuerySemanticInputSchema", { enumerable: true, get: function () { return index_js_2.QuerySemanticInputSchema; } });
Object.defineProperty(exports, "CodeReviewPromptArgsSchema", { enumerable: true, get: function () { return index_js_2.CodeReviewPromptArgsSchema; } });
Object.defineProperty(exports, "RefactoringSuggestionsPromptArgsSchema", { enumerable: true, get: function () { return index_js_2.RefactoringSuggestionsPromptArgsSchema; } });
Object.defineProperty(exports, "ArchitectureAnalysisPromptArgsSchema", { enumerable: true, get: function () { return index_js_2.ArchitectureAnalysisPromptArgsSchema; } });
Object.defineProperty(exports, "DocumentationGenerationPromptArgsSchema", { enumerable: true, get: function () { return index_js_2.DocumentationGenerationPromptArgsSchema; } });
Object.defineProperty(exports, "RESOURCE_PATTERNS", { enumerable: true, get: function () { return index_js_2.RESOURCE_PATTERNS; } });
// Utility exports
var index_js_3 = require("./utils/index.js");
Object.defineProperty(exports, "validateServerConfig", { enumerable: true, get: function () { return index_js_3.validateServerConfig; } });
Object.defineProperty(exports, "createLogger", { enumerable: true, get: function () { return index_js_3.createLogger; } });
Object.defineProperty(exports, "formatMcpError", { enumerable: true, get: function () { return index_js_3.formatMcpError; } });
Object.defineProperty(exports, "validateProjectPath", { enumerable: true, get: function () { return index_js_3.validateProjectPath; } });
Object.defineProperty(exports, "sanitizeFilePath", { enumerable: true, get: function () { return index_js_3.sanitizeFilePath; } });
Object.defineProperty(exports, "parseCliArgs", { enumerable: true, get: function () { return index_js_3.parseCliArgs; } });
Object.defineProperty(exports, "getEnvironmentConfig", { enumerable: true, get: function () { return index_js_3.getEnvironmentConfig; } });
Object.defineProperty(exports, "mergeConfigs", { enumerable: true, get: function () { return index_js_3.mergeConfigs; } });
//# sourceMappingURL=index.js.map