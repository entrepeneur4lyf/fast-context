/**
 * Fast-Context MCP Server Implementation
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { registerTools } from './tools.js';
import { registerResources } from './resources.js';
import { registerPrompts } from './prompts.js';
import type { McpServerConfig } from '../types/index.js';

/**
 * Create and configure a Fast-Context MCP server
 */
export function createFastContextMcpServer(config: McpServerConfig): McpServer {
  const server = new McpServer({
    name: config.name,
    version: config.version
  });

  // Register all capabilities
  registerTools(server);
  registerResources(server);
  registerPrompts(server);

  // Note: Error handling would be added here if the MCP server supports it

  return server;
}

/**
 * Default server configuration
 */
export const DEFAULT_SERVER_CONFIG: McpServerConfig = {
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
export function createDefaultFastContextMcpServer(): McpServer {
  return createFastContextMcpServer(DEFAULT_SERVER_CONFIG);
}

// Re-export types and utilities
export type { McpServerConfig } from '../types/index.js';
export { registerTools, registerResources, registerPrompts };
