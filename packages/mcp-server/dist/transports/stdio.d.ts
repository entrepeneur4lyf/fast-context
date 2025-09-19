/**
 * Stdio Transport for Fast-Context MCP Server
 */
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import type { McpServerConfig } from '../types/index.js';
/**
 * Start a Fast-Context MCP server with stdio transport
 */
export declare function startStdioServer(config?: McpServerConfig): Promise<void>;
/**
 * Create a stdio transport for the MCP server
 */
export declare function createStdioTransport(): StdioServerTransport;
//# sourceMappingURL=stdio.d.ts.map