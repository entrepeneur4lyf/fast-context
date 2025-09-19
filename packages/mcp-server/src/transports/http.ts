/**
 * HTTP Transport for Fast-Context MCP Server
 */

import express from 'express';
import cors from 'cors';
import { randomUUID } from 'node:crypto';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import { isInitializeRequest } from '@modelcontextprotocol/sdk/types.js';
import { createFastContextMcpServer, DEFAULT_SERVER_CONFIG } from '../server/index.js';
import type { McpServerConfig } from '../types/index.js';

export interface HttpServerConfig extends McpServerConfig {
  port?: number;
  host?: string;
  enableCors?: boolean;
  corsOrigins?: string | string[];
  enableDnsRebindingProtection?: boolean;
  allowedHosts?: string[];
  stateless?: boolean;
}

/**
 * Start a Fast-Context MCP server with HTTP transport
 */
export async function startHttpServer(config: HttpServerConfig = { ...DEFAULT_SERVER_CONFIG, port: 3000 }): Promise<void> {
  const app = express();
  app.use(express.json());

  // Configure CORS if enabled
  if (config.enableCors !== false) {
    app.use(cors({
      origin: config.corsOrigins || '*',
      exposedHeaders: ['Mcp-Session-Id'],
      allowedHeaders: ['Content-Type', 'mcp-session-id'],
    }));
  }

  // Map to store transports by session ID (for stateful mode)
  const transports: { [sessionId: string]: StreamableHTTPServerTransport } = {};

  if (config.stateless) {
    // Stateless mode: create new server instance for each request
    app.post('/mcp', async (req, res) => {
      try {
        const server = createFastContextMcpServer(config);
        const transport = new StreamableHTTPServerTransport({
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
      } catch (error) {
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

  } else {
    // Stateful mode: manage sessions
    app.post('/mcp', async (req, res) => {
      const sessionId = req.headers['mcp-session-id'] as string | undefined;
      let transport: StreamableHTTPServerTransport;

      if (sessionId && transports[sessionId]) {
        // Reuse existing transport
        transport = transports[sessionId];
      } else if (!sessionId && isInitializeRequest(req.body)) {
        // New initialization request
        transport = new StreamableHTTPServerTransport({
          sessionIdGenerator: () => randomUUID(),
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

        const server = createFastContextMcpServer(config);
        await server.connect(transport);
      } else {
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
    const handleSessionRequest = async (req: express.Request, res: express.Response) => {
      const sessionId = req.headers['mcp-session-id'] as string | undefined;
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
