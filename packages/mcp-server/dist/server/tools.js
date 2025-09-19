"use strict";
/**
 * MCP Tools Implementation for Fast-Context
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerTools = registerTools;
const core_1 = require("@fast-context/core");
/**
 * Register all Fast-Context analysis tools with the MCP server
 */
function registerTools(server) {
    // Tool: Analyze Codebase
    server.registerTool('analyze_codebase', {
        title: 'Analyze Codebase',
        description: 'Perform comprehensive analysis of a codebase including symbols, dependencies, and metrics'
    }, async (args) => {
        const { projectPath, options } = args;
        try {
            // Create analyzer with configuration
            const config = {
                projectRoot: projectPath,
                enableCaching: options?.enableCaching ?? true,
                parallelProcessing: options?.parallelProcessing ?? true,
                languages: options?.languages,
                ignorePatterns: options?.ignorePatterns,
                maxFiles: options?.maxFiles
            };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            // Perform analysis
            const result = await analyzer.analyze();
            return {
                content: [{
                        type: 'text',
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
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Find Symbols
    server.registerTool('find_symbols', {
        title: 'Find Symbols',
        description: 'Search for symbols in the codebase with optional filtering'
    }, async (args) => {
        const { projectPath, query, kind, limit = 50 } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            query,
                            kind,
                            limit,
                            timestamp: new Date().toISOString(),
                            symbols: [],
                            message: 'Symbol search functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Get Symbol Context
    server.registerTool('get_symbol_context', {
        title: 'Get Symbol Context',
        description: 'Get detailed context and information about a specific symbol'
    }, async (args) => {
        const { projectPath, symbolName, includeReferences = true, depth = 3 } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            symbolName,
                            includeReferences,
                            depth,
                            timestamp: new Date().toISOString(),
                            context: {},
                            message: 'Symbol context functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Analyze Dependencies
    server.registerTool('analyze_dependencies', {
        title: 'Analyze Dependencies',
        description: 'Analyze dependency relationships in the codebase'
    }, async (args) => {
        const { projectPath, symbolName, depth = 5, includeExternal = false } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            symbolName,
                            depth,
                            includeExternal,
                            timestamp: new Date().toISOString(),
                            dependencies: [],
                            message: 'Dependency analysis functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Detect Patterns
    server.registerTool('detect_patterns', {
        title: 'Detect Patterns',
        description: 'Detect architectural and design patterns in the codebase'
    }, async (args) => {
        const { projectPath, patterns, includeRecommendations = true } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            patterns,
                            includeRecommendations,
                            timestamp: new Date().toISOString(),
                            detectedPatterns: [],
                            recommendations: [],
                            message: 'Pattern detection functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Get Complexity Metrics
    server.registerTool('get_complexity_metrics', {
        title: 'Get Complexity Metrics',
        description: 'Analyze code complexity metrics for files or the entire codebase'
    }, async (args) => {
        const { projectPath, filePath, includeDetails = true } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            filePath,
                            includeDetails,
                            timestamp: new Date().toISOString(),
                            metrics: {},
                            message: 'Complexity metrics functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
    // Tool: Query Semantic
    server.registerTool('query_semantic', {
        title: 'Semantic Query',
        description: 'Perform semantic search across the codebase'
    }, async (args) => {
        const { projectPath, query, limit = 20, similarity = 0.7 } = args;
        try {
            return {
                content: [{
                        type: 'text',
                        text: JSON.stringify({
                            projectPath,
                            query,
                            limit,
                            similarity,
                            timestamp: new Date().toISOString(),
                            results: [],
                            message: 'Semantic query functionality will be implemented when the query engine is available'
                        }, null, 2)
                    }]
            };
        }
        catch (error) {
            return {
                content: [{
                        type: 'text',
                        text: `Error: ${error instanceof Error ? error.message : String(error)}`
                    }],
                isError: true
            };
        }
    });
}
//# sourceMappingURL=tools.js.map