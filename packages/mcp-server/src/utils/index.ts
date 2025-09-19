/**
 * Utility functions for Fast-Context MCP Server
 */

import { z } from 'zod';
import type { McpServerConfig } from '../types/index.js';

/**
 * Validate and parse server configuration
 */
export function validateServerConfig(config: unknown): McpServerConfig {
  const schema = z.object({
    name: z.string().min(1),
    version: z.string().min(1),
    maxConcurrentAnalyses: z.number().positive().optional(),
    defaultTimeout: z.number().positive().optional(),
    enableLogging: z.boolean().optional(),
    logLevel: z.enum(['debug', 'info', 'warn', 'error']).optional()
  });

  return schema.parse(config);
}

/**
 * Create a logger function based on configuration
 */
export function createLogger(config: McpServerConfig) {
  const shouldLog = config.enableLogging !== false;
  const logLevel = config.logLevel || 'info';
  
  const levels = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3
  };

  const currentLevel = levels[logLevel];

  return {
    debug: (message: string, ...args: any[]) => {
      if (shouldLog && levels.debug >= currentLevel) {
        console.debug(`[${config.name}] DEBUG:`, message, ...args);
      }
    },
    info: (message: string, ...args: any[]) => {
      if (shouldLog && levels.info >= currentLevel) {
        console.info(`[${config.name}] INFO:`, message, ...args);
      }
    },
    warn: (message: string, ...args: any[]) => {
      if (shouldLog && levels.warn >= currentLevel) {
        console.warn(`[${config.name}] WARN:`, message, ...args);
      }
    },
    error: (message: string, ...args: any[]) => {
      if (shouldLog && levels.error >= currentLevel) {
        console.error(`[${config.name}] ERROR:`, message, ...args);
      }
    }
  };
}

/**
 * Format error messages for MCP responses
 */
export function formatMcpError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  return 'An unknown error occurred';
}

/**
 * Validate project path
 */
export function validateProjectPath(path: string): boolean {
  if (!path || typeof path !== 'string') {
    return false;
  }
  
  // Basic validation - in a real implementation, you might want to check
  // if the path exists and is accessible
  return path.length > 0 && !path.includes('..') && !path.startsWith('/');
}

/**
 * Sanitize file paths to prevent directory traversal
 */
export function sanitizeFilePath(filePath: string): string {
  // Remove any path traversal attempts
  return filePath.replace(/\.\./g, '').replace(/\/+/g, '/');
}

/**
 * Parse command line arguments for server configuration
 */
export function parseCliArgs(args: string[]): Partial<McpServerConfig> {
  const config: Partial<McpServerConfig> = {};
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    switch (arg) {
      case '--name':
        config.name = args[++i];
        break;
      case '--version':
        config.version = args[++i];
        break;
      case '--log-level':
        const level = args[++i] as 'debug' | 'info' | 'warn' | 'error';
        if (['debug', 'info', 'warn', 'error'].includes(level)) {
          config.logLevel = level;
        }
        break;
      case '--no-logging':
        config.enableLogging = false;
        break;
      case '--timeout':
        const timeout = parseInt(args[++i], 10);
        if (!isNaN(timeout) && timeout > 0) {
          config.defaultTimeout = timeout;
        }
        break;
      case '--max-concurrent':
        const maxConcurrent = parseInt(args[++i], 10);
        if (!isNaN(maxConcurrent) && maxConcurrent > 0) {
          config.maxConcurrentAnalyses = maxConcurrent;
        }
        break;
    }
  }
  
  return config;
}

/**
 * Get environment-based configuration
 */
export function getEnvironmentConfig(): Partial<McpServerConfig> {
  return {
    name: process.env.MCP_SERVER_NAME,
    version: process.env.MCP_SERVER_VERSION,
    enableLogging: process.env.MCP_ENABLE_LOGGING !== 'false',
    logLevel: (process.env.MCP_LOG_LEVEL as 'debug' | 'info' | 'warn' | 'error') || 'info',
    defaultTimeout: process.env.MCP_DEFAULT_TIMEOUT ? parseInt(process.env.MCP_DEFAULT_TIMEOUT, 10) : undefined,
    maxConcurrentAnalyses: process.env.MCP_MAX_CONCURRENT ? parseInt(process.env.MCP_MAX_CONCURRENT, 10) : undefined
  };
}

/**
 * Merge configuration from multiple sources
 */
export function mergeConfigs(...configs: Partial<McpServerConfig>[]): McpServerConfig {
  const merged = configs.reduce((acc, config) => ({
    ...acc,
    ...Object.fromEntries(Object.entries(config).filter(([_, value]) => value !== undefined))
  }), {});

  // Ensure required fields have defaults
  return {
    name: 'fast-context-mcp-server',
    version: '0.1.0',
    maxConcurrentAnalyses: 5,
    defaultTimeout: 30000,
    enableLogging: true,
    logLevel: 'info',
    ...merged
  };
}
