/**
 * Utility functions for Fast-Context MCP Server
 */
import type { McpServerConfig } from '../types/index.js';
/**
 * Validate and parse server configuration
 */
export declare function validateServerConfig(config: unknown): McpServerConfig;
/**
 * Create a logger function based on configuration
 */
export declare function createLogger(config: McpServerConfig): {
    debug: (message: string, ...args: any[]) => void;
    info: (message: string, ...args: any[]) => void;
    warn: (message: string, ...args: any[]) => void;
    error: (message: string, ...args: any[]) => void;
};
/**
 * Format error messages for MCP responses
 */
export declare function formatMcpError(error: unknown): string;
/**
 * Validate project path
 */
export declare function validateProjectPath(path: string): boolean;
/**
 * Sanitize file paths to prevent directory traversal
 */
export declare function sanitizeFilePath(filePath: string): string;
/**
 * Parse command line arguments for server configuration
 */
export declare function parseCliArgs(args: string[]): Partial<McpServerConfig>;
/**
 * Get environment-based configuration
 */
export declare function getEnvironmentConfig(): Partial<McpServerConfig>;
/**
 * Merge configuration from multiple sources
 */
export declare function mergeConfigs(...configs: Partial<McpServerConfig>[]): McpServerConfig;
//# sourceMappingURL=index.d.ts.map