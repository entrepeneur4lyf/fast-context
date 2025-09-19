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
export declare function createFastContextMcpServer(config: McpServerConfig): McpServer;
/**
 * Default server configuration
 */
export declare const DEFAULT_SERVER_CONFIG: McpServerConfig;
/**
 * Create a Fast-Context MCP server with default configuration
 */
export declare function createDefaultFastContextMcpServer(): McpServer;
export type { McpServerConfig } from '../types/index.js';
export { registerTools, registerResources, registerPrompts };
//# sourceMappingURL=index.d.ts.map