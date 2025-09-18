"use strict";
/**
 * Enhanced TypeScript types for Fast-Context SDK
 * Provides strict type safety with runtime validation
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.ConfigurationError = exports.AnalysisCancelledError = exports.ValidationError = exports.AnalysisError = exports.SymbolKindSchema = exports.AnalysisPhaseSchema = exports.StreamingOptionsSchema = exports.AnalysisConfigSchema = exports.Err = exports.Ok = void 0;
exports.isAnalysisProgress = isAnalysisProgress;
exports.validateConfig = validateConfig;
exports.validateStreamingOptions = validateStreamingOptions;
const zod_1 = require("zod");
const Ok = (data) => ({ success: true, data });
exports.Ok = Ok;
const Err = (error) => ({ success: false, error });
exports.Err = Err;
// ============================================================================
// Configuration Types with Validation
// ============================================================================
exports.AnalysisConfigSchema = zod_1.z.object({
    projectRoot: zod_1.z.string().min(1, 'Project root cannot be empty'),
    languages: zod_1.z.array(zod_1.z.string()).optional(),
    ignorePatterns: zod_1.z.array(zod_1.z.string()).optional(),
    enableCaching: zod_1.z.boolean().default(true),
    cachePolicy: zod_1.z.enum(['auto', 'minimal', 'balanced', 'adaptive', 'persistent']).default('adaptive'),
    enableWatching: zod_1.z.boolean().default(false),
    maxFiles: zod_1.z.number().positive().optional(),
    parallelProcessing: zod_1.z.boolean().default(true),
    performance: zod_1.z.object({
        maxMemoryMb: zod_1.z.number().positive().default(1024),
        timeoutMs: zod_1.z.number().positive().default(30000),
        workerThreads: zod_1.z.number().positive().default(4),
        chunkSize: zod_1.z.number().positive().default(100)
    }).optional()
});
exports.StreamingOptionsSchema = zod_1.z.object({
    signal: zod_1.z.instanceof(AbortSignal).optional(),
    progressInterval: zod_1.z.number().positive().default(100),
    enableDetailedProgress: zod_1.z.boolean().default(false),
    batchSize: zod_1.z.number().positive().default(50)
});
// ============================================================================
// Analysis Progress and Results
// ============================================================================
exports.AnalysisPhaseSchema = zod_1.z.enum([
    'initializing',
    'parsing',
    'extracting',
    'analyzing',
    'indexing',
    'complete',
    'error'
]);
// ============================================================================
// Symbol and Query Types
// ============================================================================
exports.SymbolKindSchema = zod_1.z.enum([
    'function',
    'class',
    'interface',
    'type',
    'variable',
    'constant',
    'enum',
    'module',
    'namespace',
    'property',
    'method',
    'constructor',
    'field',
    'parameter',
    'import',
    'export'
]);
// ============================================================================
// Error Types
// ============================================================================
class AnalysisError extends Error {
    constructor(message, code, context) {
        super(message);
        this.code = code;
        this.context = context;
        this.name = 'AnalysisError';
    }
}
exports.AnalysisError = AnalysisError;
class ValidationError extends AnalysisError {
    constructor(message, validationErrors) {
        super(message, 'VALIDATION_ERROR', { validationErrors });
        this.validationErrors = validationErrors;
        this.name = 'ValidationError';
    }
}
exports.ValidationError = ValidationError;
class AnalysisCancelledError extends AnalysisError {
    constructor(message = 'Analysis was cancelled') {
        super(message, 'ANALYSIS_CANCELLED');
        this.name = 'AnalysisCancelledError';
    }
}
exports.AnalysisCancelledError = AnalysisCancelledError;
class ConfigurationError extends AnalysisError {
    constructor(message, context) {
        super(message, 'CONFIGURATION_ERROR', context);
        this.name = 'ConfigurationError';
    }
}
exports.ConfigurationError = ConfigurationError;
// ============================================================================
// Type Guards and Validators
// ============================================================================
function isAnalysisProgress(obj) {
    return (typeof obj === 'object' &&
        obj !== null &&
        'phase' in obj &&
        'filesProcessed' in obj &&
        'totalFiles' in obj &&
        'symbolsFound' in obj);
}
function validateConfig(config) {
    try {
        const validated = exports.AnalysisConfigSchema.parse(config);
        return (0, exports.Ok)(validated);
    }
    catch (error) {
        if (error instanceof zod_1.z.ZodError) {
            return (0, exports.Err)(new ValidationError('Invalid configuration', error));
        }
        return (0, exports.Err)(new ValidationError('Unknown validation error', error));
    }
}
function validateStreamingOptions(options) {
    try {
        const validated = exports.StreamingOptionsSchema.parse(options);
        return (0, exports.Ok)(validated);
    }
    catch (error) {
        if (error instanceof zod_1.z.ZodError) {
            return (0, exports.Err)(new ValidationError('Invalid streaming options', error));
        }
        return (0, exports.Err)(new ValidationError('Unknown validation error', error));
    }
}
//# sourceMappingURL=index.js.map