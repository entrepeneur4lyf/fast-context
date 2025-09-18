/**
 * @fast-context/core - Enhanced TypeScript SDK
 * 
 * This package provides a modern, type-safe wrapper around the Fast-Context
 * native analyzer with streaming capabilities, advanced querying, and
 * comprehensive error handling.
 * 
 * @example
 * ```typescript
 * import { EnhancedFastContextAnalyzer, ConfigurationManager } from '@fast-context/core';
 * 
 * // Create analyzer with preset configuration
 * const analyzer = EnhancedFastContextAnalyzer.fromPreset('balanced', './my-project');
 * 
 * // Stream analysis with progress tracking
 * for await (const progress of analyzer.analyzeStream()) {
 *   console.log(`Progress: ${progress.filesProcessed}/${progress.totalFiles}`);
 * }
 * 
 * // Use advanced query engine
 * const queryEngine = analyzer.getQueryEngine();
 * const symbols = await queryEngine.findSymbols({ text: 'user authentication' });
 * ```
 */

// Core analyzer and components
export { EnhancedFastContextAnalyzer } from './analyzer';
export { QueryEngine } from './query';
export { ConfigurationManager } from './config';
export { StreamingAnalyzer } from './streaming';

// Type definitions and schemas
export * from './types';

// Utility functions
export { createAnalyzer, createAnalyzerFromPreset, createAnalyzerFromEnvironment } from './utils';

// Version information
export const VERSION = '0.1.0';
export const SUPPORTED_NODE_VERSIONS = ['18', '20', '21'];

/**
 * Check if the current Node.js version is supported
 */
export function checkNodeVersion(): { supported: boolean; current: string; message?: string } {
  const current = process.version;
  const majorVersion = current.split('.')[0]?.substring(1); // Remove 'v' prefix
  
  if (!majorVersion || !SUPPORTED_NODE_VERSIONS.includes(majorVersion)) {
    return {
      supported: false,
      current,
      message: `Node.js ${current} is not supported. Please use Node.js ${SUPPORTED_NODE_VERSIONS.join(', ')}.`
    };
  }
  
  return { supported: true, current };
}

/**
 * Get package information
 */
export function getPackageInfo() {
  return {
    name: '@fast-context/core',
    version: VERSION,
    description: 'Enhanced TypeScript SDK for Fast-Context with streaming analysis and advanced query capabilities',
    supportedNodeVersions: SUPPORTED_NODE_VERSIONS,
    features: [
      'Streaming analysis with progress tracking',
      'Advanced query engine with semantic search',
      'Type-safe configuration management',
      'Comprehensive error handling',
      'Performance monitoring',
      'Cancellation support'
    ]
  };
}
