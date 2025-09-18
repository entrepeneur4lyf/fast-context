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
export { EnhancedFastContextAnalyzer } from './analyzer';
export { QueryEngine } from './query';
export { ConfigurationManager } from './config';
export { StreamingAnalyzer } from './streaming';
export * from './types';
export { createAnalyzer, createAnalyzerFromPreset, createAnalyzerFromEnvironment } from './utils';
export declare const VERSION = "0.1.0";
export declare const SUPPORTED_NODE_VERSIONS: string[];
/**
 * Check if the current Node.js version is supported
 */
export declare function checkNodeVersion(): {
    supported: boolean;
    current: string;
    message?: string;
};
/**
 * Get package information
 */
export declare function getPackageInfo(): {
    name: string;
    version: string;
    description: string;
    supportedNodeVersions: string[];
    features: string[];
};
//# sourceMappingURL=index.d.ts.map