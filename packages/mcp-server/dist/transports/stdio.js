"use strict";
/**
 * Stdio Transport for Fast-Context MCP Server
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.startStdioServer = startStdioServer;
exports.createStdioTransport = createStdioTransport;
const stdio_js_1 = require("@modelcontextprotocol/sdk/server/stdio.js");
const index_js_1 = require("../server/index.js");
/**
 * Start a Fast-Context MCP server with stdio transport
 */
async function startStdioServer(config = index_js_1.DEFAULT_SERVER_CONFIG) {
    const server = (0, index_js_1.createFastContextMcpServer)(config);
    const transport = new stdio_js_1.StdioServerTransport();
    if (config.enableLogging) {
        console.error(`[${config.name}] Starting stdio server...`);
    }
    try {
        await server.connect(transport);
        if (config.enableLogging) {
            console.error(`[${config.name}] Stdio server started successfully`);
            console.error(`[${config.name}] Listening for MCP messages on stdin/stdout`);
        }
        // Handle graceful shutdown
        process.on('SIGINT', async () => {
            if (config.enableLogging) {
                console.error(`[${config.name}] Received SIGINT, shutting down...`);
            }
            await server.close();
            process.exit(0);
        });
        process.on('SIGTERM', async () => {
            if (config.enableLogging) {
                console.error(`[${config.name}] Received SIGTERM, shutting down...`);
            }
            await server.close();
            process.exit(0);
        });
    }
    catch (error) {
        console.error(`[${config.name}] Failed to start stdio server:`, error);
        process.exit(1);
    }
}
/**
 * Create a stdio transport for the MCP server
 */
function createStdioTransport() {
    return new stdio_js_1.StdioServerTransport();
}
//# sourceMappingURL=stdio.js.map