"use strict";
/**
 * MCP Resources Implementation for Fast-Context
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerResources = registerResources;
const mcp_js_1 = require("@modelcontextprotocol/sdk/server/mcp.js");
const core_1 = require("@fast-context/core");
// Resource patterns
const RESOURCE_PATTERNS = {
    CODEBASE_ANALYSIS: 'codebase://analysis/{projectPath}',
    SYMBOL_INFO: 'symbols://project/{projectPath}/{symbolName}',
    DEPENDENCIES: 'dependencies://project/{projectPath}/{symbolName}',
    FILE_ANALYSIS: 'files://project/{projectPath}/{filePath}'
};
/**
 * Register all Fast-Context resources with the MCP server
 */
function registerResources(server) {
    // Resource: Codebase Analysis
    server.registerResource('codebase-analysis', new mcp_js_1.ResourceTemplate(RESOURCE_PATTERNS.CODEBASE_ANALYSIS, { list: undefined }), {
        title: 'Codebase Analysis',
        description: 'Complete analysis results for a codebase including symbols, dependencies, and metrics',
        mimeType: 'application/json'
    }, async (uri, { projectPath }) => {
        try {
            const config = { projectRoot: projectPath };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            const result = await analyzer.analyze();
            return {
                contents: [{
                        uri: uri.href,
                        mimeType: 'application/json',
                        text: JSON.stringify({
                            projectPath,
                            timestamp: new Date().toISOString(),
                            summary: {
                                fileCount: result.fileCount,
                                symbolCount: result.symbolCount,
                                relationshipCount: result.relationshipCount,
                                languages: result.languages,
                                durationMs: result.durationMs
                            },
                            performance: result.performance,
                            insights: result.insights,
                            recommendations: result.recommendations
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            throw new Error(`Failed to analyze codebase: ${error instanceof Error ? error.message : String(error)}`);
        }
    });
    // Resource: Symbol Information
    server.registerResource('symbol-info', new mcp_js_1.ResourceTemplate(RESOURCE_PATTERNS.SYMBOL_INFO, { list: undefined }), {
        title: 'Symbol Information',
        description: 'Detailed information about a specific symbol in the codebase',
        mimeType: 'application/json'
    }, async (uri, { projectPath, symbolName }) => {
        try {
            const config = { projectRoot: projectPath };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            return {
                contents: [{
                        uri: uri.href,
                        mimeType: 'application/json',
                        text: JSON.stringify({
                            projectPath,
                            symbolName,
                            timestamp: new Date().toISOString(),
                            symbol: {},
                            dependencies: [],
                            context: {},
                            message: 'Symbol information functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            throw new Error(`Failed to get symbol information: ${error instanceof Error ? error.message : String(error)}`);
        }
    });
    // Resource: Dependencies
    server.registerResource('dependencies', new mcp_js_1.ResourceTemplate(RESOURCE_PATTERNS.DEPENDENCIES, { list: undefined }), {
        title: 'Symbol Dependencies',
        description: 'Dependency graph and relationships for a specific symbol',
        mimeType: 'application/json'
    }, async (uri, { projectPath, symbolName }) => {
        try {
            const config = { projectRoot: projectPath };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            return {
                contents: [{
                        uri: uri.href,
                        mimeType: 'application/json',
                        text: JSON.stringify({
                            projectPath,
                            symbolName,
                            timestamp: new Date().toISOString(),
                            dependencies: [],
                            graph: {
                                nodes: [],
                                edges: []
                            },
                            message: 'Dependencies functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            throw new Error(`Failed to get dependencies: ${error instanceof Error ? error.message : String(error)}`);
        }
    });
    // Resource: File Analysis
    server.registerResource('file-analysis', new mcp_js_1.ResourceTemplate(RESOURCE_PATTERNS.FILE_ANALYSIS, { list: undefined }), {
        title: 'File Analysis',
        description: 'Detailed analysis of a specific file including symbols, complexity, and metrics',
        mimeType: 'application/json'
    }, async (uri, { projectPath, filePath }) => {
        try {
            const config = { projectRoot: projectPath };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            return {
                contents: [{
                        uri: uri.href,
                        mimeType: 'application/json',
                        text: JSON.stringify({
                            projectPath,
                            filePath,
                            timestamp: new Date().toISOString(),
                            symbols: [],
                            complexity: null,
                            metrics: {
                                symbolCount: 0,
                                functionCount: 0,
                                classCount: 0,
                                variableCount: 0
                            },
                            message: 'File analysis functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            throw new Error(`Failed to analyze file: ${error instanceof Error ? error.message : String(error)}`);
        }
    });
}
//# sourceMappingURL=resources.js.map