#!/usr/bin/env node

/**
 * Fast-Context MCP Server - Stdio Example
 * 
 * This example demonstrates how to run the Fast-Context MCP server
 * with stdio transport for direct integration with AI assistants.
 */

import { startStdioServer, parseCliArgs, getEnvironmentConfig, mergeConfigs } from '../src/index.js';

async function main() {
  try {
    // Parse command line arguments
    const cliConfig = parseCliArgs(process.argv.slice(2));
    
    // Get environment configuration
    const envConfig = getEnvironmentConfig();
    
    // Merge configurations with CLI taking precedence
    const config = mergeConfigs(
      {
        name: 'fast-context-mcp-server',
        version: '0.1.0',
        enableLogging: true,
        logLevel: 'info'
      },
      envConfig,
      cliConfig
    );

    // Start the stdio server
    await startStdioServer(config);
    
  } catch (error) {
    console.error('Failed to start Fast-Context MCP server:', error);
    process.exit(1);
  }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
  process.exit(1);
});

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error('Uncaught Exception:', error);
  process.exit(1);
});

// Start the server
main();
