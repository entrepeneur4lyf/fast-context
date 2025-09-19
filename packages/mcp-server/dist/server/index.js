"use strict";
/**
 * Fast-Context MCP Server Implementation
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerPrompts = exports.registerResources = exports.registerTools = exports.DEFAULT_SERVER_CONFIG = void 0;
exports.createFastContextMcpServer = createFastContextMcpServer;
exports.createDefaultFastContextMcpServer = createDefaultFastContextMcpServer;
const mcp_js_1 = require("@modelcontextprotocol/sdk/server/mcp.js");
const tools_js_1 = require("./tools.js");
Object.defineProperty(exports, "registerTools", { enumerable: true, get: function () { return tools_js_1.registerTools; } });
const resources_js_1 = require("./resources.js");
Object.defineProperty(exports, "registerResources", { enumerable: true, get: function () { return resources_js_1.registerResources; } });
const prompts_js_1 = require("./prompts.js");
Object.defineProperty(exports, "registerPrompts", { enumerable: true, get: function () { return prompts_js_1.registerPrompts; } });
/**
 * Create and configure a Fast-Context MCP server
 */
function createFastContextMcpServer(config) {
    const server = new mcp_js_1.McpServer({
        name: config.name,
        version: config.version
    });
    // Register all capabilities
    (0, tools_js_1.registerTools)(server);
    (0, resources_js_1.registerResources)(server);
    (0, prompts_js_1.registerPrompts)(server);
    // Note: Error handling would be added here if the MCP server supports it
    return server;
}
/**
 * Default server configuration
 */
exports.DEFAULT_SERVER_CONFIG = {
    name: 'fast-context-mcp-server',
    version: '0.1.0',
    maxConcurrentAnalyses: 5,
    defaultTimeout: 30000, // 30 seconds
    enableLogging: true,
    logLevel: 'info'
};
/**
 * Create a Fast-Context MCP server with default configuration
 */
function createDefaultFastContextMcpServer() {
    return createFastContextMcpServer(exports.DEFAULT_SERVER_CONFIG);
}
//# sourceMappingURL=index.js.map