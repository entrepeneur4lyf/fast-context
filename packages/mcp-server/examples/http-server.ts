#!/usr/bin/env node

/**
 * Fast-Context MCP Server - HTTP Example
 * 
 * This example demonstrates how to run the Fast-Context MCP server
 * with HTTP transport for web-based AI assistant integration.
 */

import { startHttpServer, parseCliArgs, getEnvironmentConfig, mergeConfigs } from '../src/index.js';
import type { HttpServerConfig } from '../src/index.js';

function parseHttpCliArgs(args: string[]): Partial<HttpServerConfig> {
  const config: Partial<HttpServerConfig> = {};
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    switch (arg) {
      case '--port':
        const port = parseInt(args[++i], 10);
        if (!isNaN(port) && port > 0 && port < 65536) {
          config.port = port;
        }
        break;
      case '--host':
        config.host = args[++i];
        break;
      case '--no-cors':
        config.enableCors = false;
        break;
      case '--cors-origins':
        const origins = args[++i].split(',');
        config.corsOrigins = origins;
        break;
      case '--stateless':
        config.stateless = true;
        break;
      case '--dns-protection':
        config.enableDnsRebindingProtection = true;
        break;
      case '--allowed-hosts':
        const hosts = args[++i].split(',');
        config.allowedHosts = hosts;
        break;
    }
  }
  
  return config;
}

function getHttpEnvironmentConfig(): Partial<HttpServerConfig> {
  return {
    port: process.env.MCP_HTTP_PORT ? parseInt(process.env.MCP_HTTP_PORT, 10) : undefined,
    host: process.env.MCP_HTTP_HOST,
    enableCors: process.env.MCP_ENABLE_CORS !== 'false',
    corsOrigins: process.env.MCP_CORS_ORIGINS?.split(','),
    stateless: process.env.MCP_STATELESS === 'true',
    enableDnsRebindingProtection: process.env.MCP_DNS_PROTECTION === 'true',
    allowedHosts: process.env.MCP_ALLOWED_HOSTS?.split(',')
  };
}

async function main() {
  try {
    // Parse command line arguments
    const cliConfig = parseCliArgs(process.argv.slice(2));
    const httpCliConfig = parseHttpCliArgs(process.argv.slice(2));
    
    // Get environment configuration
    const envConfig = getEnvironmentConfig();
    const httpEnvConfig = getHttpEnvironmentConfig();
    
    // Merge configurations with CLI taking precedence
    const config = mergeConfigs(
      {
        name: 'fast-context-mcp-server',
        version: '0.1.0',
        enableLogging: true,
        logLevel: 'info',
        port: 3000,
        host: '0.0.0.0',
        enableCors: true,
        stateless: false,
        enableDnsRebindingProtection: false,
        allowedHosts: ['127.0.0.1', 'localhost']
      },
      envConfig,
      httpEnvConfig,
      cliConfig,
      httpCliConfig
    ) as HttpServerConfig;

    console.log('Starting Fast-Context MCP HTTP Server...');
    console.log('Configuration:');
    console.log(`  Name: ${config.name}`);
    console.log(`  Version: ${config.version}`);
    console.log(`  Port: ${config.port}`);
    console.log(`  Host: ${config.host}`);
    console.log(`  Mode: ${config.stateless ? 'stateless' : 'stateful'}`);
    console.log(`  CORS: ${config.enableCors ? 'enabled' : 'disabled'}`);
    console.log(`  DNS Protection: ${config.enableDnsRebindingProtection ? 'enabled' : 'disabled'}`);
    console.log('');

    // Start the HTTP server
    await startHttpServer(config);
    
  } catch (error) {
    console.error('Failed to start Fast-Context MCP HTTP server:', error);
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

// Print usage information
if (process.argv.includes('--help') || process.argv.includes('-h')) {
  console.log(`
Fast-Context MCP HTTP Server

Usage: node http-server.js [options]

Options:
  --name <name>              Server name (default: fast-context-mcp-server)
  --version <version>        Server version (default: 0.1.0)
  --port <port>              HTTP port (default: 3000)
  --host <host>              HTTP host (default: 0.0.0.0)
  --log-level <level>        Log level: debug, info, warn, error (default: info)
  --no-logging               Disable logging
  --timeout <ms>             Default timeout in milliseconds (default: 30000)
  --max-concurrent <num>     Max concurrent analyses (default: 5)
  --no-cors                  Disable CORS
  --cors-origins <origins>   Comma-separated CORS origins (default: *)
  --stateless                Run in stateless mode
  --dns-protection           Enable DNS rebinding protection
  --allowed-hosts <hosts>    Comma-separated allowed hosts for DNS protection
  --help, -h                 Show this help message

Environment Variables:
  MCP_SERVER_NAME           Server name
  MCP_SERVER_VERSION        Server version
  MCP_HTTP_PORT             HTTP port
  MCP_HTTP_HOST             HTTP host
  MCP_LOG_LEVEL             Log level
  MCP_ENABLE_LOGGING        Enable logging (true/false)
  MCP_DEFAULT_TIMEOUT       Default timeout in milliseconds
  MCP_MAX_CONCURRENT        Max concurrent analyses
  MCP_ENABLE_CORS           Enable CORS (true/false)
  MCP_CORS_ORIGINS          Comma-separated CORS origins
  MCP_STATELESS             Run in stateless mode (true/false)
  MCP_DNS_PROTECTION        Enable DNS rebinding protection (true/false)
  MCP_ALLOWED_HOSTS         Comma-separated allowed hosts

Examples:
  node http-server.js --port 8080 --log-level debug
  node http-server.js --stateless --no-cors
  node http-server.js --port 3000 --cors-origins "https://example.com,https://app.example.com"
`);
  process.exit(0);
}

// Start the server
main();
