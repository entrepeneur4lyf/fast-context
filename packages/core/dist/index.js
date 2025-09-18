"use strict";
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
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.SUPPORTED_NODE_VERSIONS = exports.VERSION = exports.createAnalyzerFromEnvironment = exports.createAnalyzerFromPreset = exports.createAnalyzer = exports.StreamingAnalyzer = exports.ConfigurationManager = exports.QueryEngine = exports.EnhancedFastContextAnalyzer = void 0;
exports.checkNodeVersion = checkNodeVersion;
exports.getPackageInfo = getPackageInfo;
// Core analyzer and components
var analyzer_1 = require("./analyzer");
Object.defineProperty(exports, "EnhancedFastContextAnalyzer", { enumerable: true, get: function () { return analyzer_1.EnhancedFastContextAnalyzer; } });
var query_1 = require("./query");
Object.defineProperty(exports, "QueryEngine", { enumerable: true, get: function () { return query_1.QueryEngine; } });
var config_1 = require("./config");
Object.defineProperty(exports, "ConfigurationManager", { enumerable: true, get: function () { return config_1.ConfigurationManager; } });
var streaming_1 = require("./streaming");
Object.defineProperty(exports, "StreamingAnalyzer", { enumerable: true, get: function () { return streaming_1.StreamingAnalyzer; } });
// Type definitions and schemas
__exportStar(require("./types"), exports);
// Utility functions
var utils_1 = require("./utils");
Object.defineProperty(exports, "createAnalyzer", { enumerable: true, get: function () { return utils_1.createAnalyzer; } });
Object.defineProperty(exports, "createAnalyzerFromPreset", { enumerable: true, get: function () { return utils_1.createAnalyzerFromPreset; } });
Object.defineProperty(exports, "createAnalyzerFromEnvironment", { enumerable: true, get: function () { return utils_1.createAnalyzerFromEnvironment; } });
// Version information
exports.VERSION = '0.1.0';
exports.SUPPORTED_NODE_VERSIONS = ['18', '20', '21'];
/**
 * Check if the current Node.js version is supported
 */
function checkNodeVersion() {
    const current = process.version;
    const majorVersion = current.split('.')[0]?.substring(1); // Remove 'v' prefix
    if (!majorVersion || !exports.SUPPORTED_NODE_VERSIONS.includes(majorVersion)) {
        return {
            supported: false,
            current,
            message: `Node.js ${current} is not supported. Please use Node.js ${exports.SUPPORTED_NODE_VERSIONS.join(', ')}.`
        };
    }
    return { supported: true, current };
}
/**
 * Get package information
 */
function getPackageInfo() {
    return {
        name: '@fast-context/core',
        version: exports.VERSION,
        description: 'Enhanced TypeScript SDK for Fast-Context with streaming analysis and advanced query capabilities',
        supportedNodeVersions: exports.SUPPORTED_NODE_VERSIONS,
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
//# sourceMappingURL=index.js.map