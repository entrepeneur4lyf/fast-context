"use strict";
/**
 * Utility functions for Fast-Context MCP Server
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateServerConfig = validateServerConfig;
exports.createLogger = createLogger;
exports.formatMcpError = formatMcpError;
exports.validateProjectPath = validateProjectPath;
exports.sanitizeFilePath = sanitizeFilePath;
exports.parseCliArgs = parseCliArgs;
exports.getEnvironmentConfig = getEnvironmentConfig;
exports.mergeConfigs = mergeConfigs;
const zod_1 = require("zod");
/**
 * Validate and parse server configuration
 */
function validateServerConfig(config) {
    const schema = zod_1.z.object({
        name: zod_1.z.string().min(1),
        version: zod_1.z.string().min(1),
        maxConcurrentAnalyses: zod_1.z.number().positive().optional(),
        defaultTimeout: zod_1.z.number().positive().optional(),
        enableLogging: zod_1.z.boolean().optional(),
        logLevel: zod_1.z.enum(['debug', 'info', 'warn', 'error']).optional()
    });
    return schema.parse(config);
}
/**
 * Create a logger function based on configuration
 */
function createLogger(config) {
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
        debug: (message, ...args) => {
            if (shouldLog && levels.debug >= currentLevel) {
                console.debug(`[${config.name}] DEBUG:`, message, ...args);
            }
        },
        info: (message, ...args) => {
            if (shouldLog && levels.info >= currentLevel) {
                console.info(`[${config.name}] INFO:`, message, ...args);
            }
        },
        warn: (message, ...args) => {
            if (shouldLog && levels.warn >= currentLevel) {
                console.warn(`[${config.name}] WARN:`, message, ...args);
            }
        },
        error: (message, ...args) => {
            if (shouldLog && levels.error >= currentLevel) {
                console.error(`[${config.name}] ERROR:`, message, ...args);
            }
        }
    };
}
/**
 * Format error messages for MCP responses
 */
function formatMcpError(error) {
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
function validateProjectPath(path) {
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
function sanitizeFilePath(filePath) {
    // Remove any path traversal attempts
    return filePath.replace(/\.\./g, '').replace(/\/+/g, '/');
}
/**
 * Parse command line arguments for server configuration
 */
function parseCliArgs(args) {
    const config = {};
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
                const level = args[++i];
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
function getEnvironmentConfig() {
    return {
        name: process.env.MCP_SERVER_NAME,
        version: process.env.MCP_SERVER_VERSION,
        enableLogging: process.env.MCP_ENABLE_LOGGING !== 'false',
        logLevel: process.env.MCP_LOG_LEVEL || 'info',
        defaultTimeout: process.env.MCP_DEFAULT_TIMEOUT ? parseInt(process.env.MCP_DEFAULT_TIMEOUT, 10) : undefined,
        maxConcurrentAnalyses: process.env.MCP_MAX_CONCURRENT ? parseInt(process.env.MCP_MAX_CONCURRENT, 10) : undefined
    };
}
/**
 * Merge configuration from multiple sources
 */
function mergeConfigs(...configs) {
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
//# sourceMappingURL=index.js.map