/**
 * Stdio Transport for Fast-Context MCP Server
 */

import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { createFastContextMcpServer, DEFAULT_SERVER_CONFIG } from '../server/index.js';
import type { McpServerConfig } from '../types/index.js';

/**
 * Start a Fast-Context MCP server with stdio transport
 */
export async function startStdioServer(config: McpServerConfig = DEFAULT_SERVER_CONFIG): Promise<void> {
  const server = createFastContextMcpServer(config);
  const transport = new StdioServerTransport();

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

  } catch (error) {
    console.error(`[${config.name}] Failed to start stdio server:`, error);
    process.exit(1);
  }
}

/**
 * Create a stdio transport for the MCP server
 */
export function createStdioTransport(): StdioServerTransport {
  return new StdioServerTransport();
}
