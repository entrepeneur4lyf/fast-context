"use strict";
/**
 * HTTP Transport for Fast-Context MCP Server
 */
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.startHttpServer = startHttpServer;
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const node_crypto_1 = require("node:crypto");
const streamableHttp_js_1 = require("@modelcontextprotocol/sdk/server/streamableHttp.js");
const types_js_1 = require("@modelcontextprotocol/sdk/types.js");
const index_js_1 = require("../server/index.js");
/**
 * Start a Fast-Context MCP server with HTTP transport
 */
async function startHttpServer(config = { ...index_js_1.DEFAULT_SERVER_CONFIG, port: 3000 }) {
    const app = (0, express_1.default)();
    app.use(express_1.default.json());
    // Configure CORS if enabled
    if (config.enableCors !== false) {
        app.use((0, cors_1.default)({
            origin: config.corsOrigins || '*',
            exposedHeaders: ['Mcp-Session-Id'],
            allowedHeaders: ['Content-Type', 'mcp-session-id'],
        }));
    }
    // Map to store transports by session ID (for stateful mode)
    const transports = {};
    if (config.stateless) {
        // Stateless mode: create new server instance for each request
        app.post('/mcp', async (req, res) => {
            try {
                const server = (0, index_js_1.createFastContextMcpServer)(config);
                const transport = new streamableHttp_js_1.StreamableHTTPServerTransport({
                    sessionIdGenerator: undefined, // Disable session management
                });
                res.on('close', () => {
                    if (config.enableLogging && config.logLevel === 'debug') {
                        console.log(`[${config.name}] Request closed`);
                    }
                    transport.close();
                    server.close();
                });
                await server.connect(transport);
                await transport.handleRequest(req, res, req.body);
            }
            catch (error) {
                console.error(`[${config.name}] Error handling MCP request:`, error);
                if (!res.headersSent) {
                    res.status(500).json({
                        jsonrpc: '2.0',
                        error: {
                            code: -32603,
                            message: 'Internal server error',
                        },
                        id: null,
                    });
                }
            }
        });
        // GET and DELETE not supported in stateless mode
        app.get('/mcp', (req, res) => {
            res.status(405).json({
                jsonrpc: '2.0',
                error: {
                    code: -32000,
                    message: 'Method not allowed in stateless mode',
                },
                id: null,
            });
        });
        app.delete('/mcp', (req, res) => {
            res.status(405).json({
                jsonrpc: '2.0',
                error: {
                    code: -32000,
                    message: 'Method not allowed in stateless mode',
                },
                id: null,
            });
        });
    }
    else {
        // Stateful mode: manage sessions
        app.post('/mcp', async (req, res) => {
            const sessionId = req.headers['mcp-session-id'];
            let transport;
            if (sessionId && transports[sessionId]) {
                // Reuse existing transport
                transport = transports[sessionId];
            }
            else if (!sessionId && (0, types_js_1.isInitializeRequest)(req.body)) {
                // New initialization request
                transport = new streamableHttp_js_1.StreamableHTTPServerTransport({
                    sessionIdGenerator: () => (0, node_crypto_1.randomUUID)(),
                    onsessioninitialized: (sessionId) => {
                        transports[sessionId] = transport;
                        if (config.enableLogging && config.logLevel === 'debug') {
                            console.log(`[${config.name}] Session initialized: ${sessionId}`);
                        }
                    },
                    enableDnsRebindingProtection: config.enableDnsRebindingProtection || false,
                    allowedHosts: config.allowedHosts || ['127.0.0.1', 'localhost'],
                });
                // Clean up transport when closed
                transport.onclose = () => {
                    if (transport.sessionId) {
                        delete transports[transport.sessionId];
                        if (config.enableLogging && config.logLevel === 'debug') {
                            console.log(`[${config.name}] Session closed: ${transport.sessionId}`);
                        }
                    }
                };
                const server = (0, index_js_1.createFastContextMcpServer)(config);
                await server.connect(transport);
            }
            else {
                // Invalid request
                res.status(400).json({
                    jsonrpc: '2.0',
                    error: {
                        code: -32000,
                        message: 'Bad Request: No valid session ID provided',
                    },
                    id: null,
                });
                return;
            }
            // Handle the request
            await transport.handleRequest(req, res, req.body);
        });
        // Reusable handler for GET and DELETE requests
        const handleSessionRequest = async (req, res) => {
            const sessionId = req.headers['mcp-session-id'];
            if (!sessionId || !transports[sessionId]) {
                res.status(400).send('Invalid or missing session ID');
                return;
            }
            const transport = transports[sessionId];
            await transport.handleRequest(req, res);
        };
        // Handle GET requests for server-to-client notifications via SSE
        app.get('/mcp', handleSessionRequest);
        // Handle DELETE requests for session termination
        app.delete('/mcp', handleSessionRequest);
    }
    // Health check endpoint
    app.get('/health', (req, res) => {
        res.json({
            status: 'healthy',
            server: config.name,
            version: config.version,
            mode: config.stateless ? 'stateless' : 'stateful',
            activeSessions: Object.keys(transports).length
        });
    });
    // Start the server
    const port = config.port || 3000;
    const host = config.host || '0.0.0.0';
    app.listen(port, host, () => {
        if (config.enableLogging) {
            console.log(`[${config.name}] HTTP server listening on ${host}:${port}`);
            console.log(`[${config.name}] Mode: ${config.stateless ? 'stateless' : 'stateful'}`);
            console.log(`[${config.name}] Health check: http://${host}:${port}/health`);
        }
    });
    // Handle graceful shutdown
    process.on('SIGINT', () => {
        if (config.enableLogging) {
            console.log(`[${config.name}] Received SIGINT, shutting down...`);
        }
        // Close all active transports
        Object.values(transports).forEach(transport => transport.close());
        process.exit(0);
    });
    process.on('SIGTERM', () => {
        if (config.enableLogging) {
            console.log(`[${config.name}] Received SIGTERM, shutting down...`);
        }
        // Close all active transports
        Object.values(transports).forEach(transport => transport.close());
        process.exit(0);
    });
}
//# sourceMappingURL=http.js.map