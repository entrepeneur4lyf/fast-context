/**
 * HTTP Transport for Fast-Context MCP Server
 */
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
export declare function startHttpServer(config?: HttpServerConfig): Promise<void>;
//# sourceMappingURL=http.d.ts.map