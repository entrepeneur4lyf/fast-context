"use strict";
/**
 * Utility functions for the Enhanced Fast-Context SDK
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
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAnalyzer = createAnalyzer;
exports.createAnalyzerFromPreset = createAnalyzerFromPreset;
exports.createAnalyzerFromEnvironment = createAnalyzerFromEnvironment;
exports.validateAnalysisConfig = validateAnalysisConfig;
exports.getDefaultConfigForProject = getDefaultConfigForProject;
exports.mergeConfigs = mergeConfigs;
exports.createCIConfig = createCIConfig;
exports.createDevConfig = createDevConfig;
exports.createProdConfig = createProdConfig;
exports.detectProjectType = detectProjectType;
exports.createSmartConfig = createSmartConfig;
exports.formatFileSize = formatFileSize;
exports.formatDuration = formatDuration;
exports.calculateProgress = calculateProgress;
exports.estimateRemainingTime = estimateRemainingTime;
const analyzer_1 = require("../analyzer");
const config_1 = require("../config");
const types_1 = require("../types");
/**
 * Create analyzer with automatic configuration validation
 */
function createAnalyzer(config) {
    return analyzer_1.EnhancedFastContextAnalyzer.create(config);
}
/**
 * Create analyzer from preset with project root
 */
function createAnalyzerFromPreset(preset, projectRoot) {
    return analyzer_1.EnhancedFastContextAnalyzer.fromPreset(preset, projectRoot);
}
/**
 * Create analyzer from environment variables
 */
function createAnalyzerFromEnvironment() {
    return analyzer_1.EnhancedFastContextAnalyzer.fromEnvironment();
}
/**
 * Validate configuration without creating analyzer
 */
function validateAnalysisConfig(config) {
    const validation = (0, types_1.validateConfig)(config);
    if (!validation.success) {
        return (0, types_1.Err)(new types_1.AnalysisError('Configuration validation failed', 'VALIDATION_ERROR', { validationError: validation.error }));
    }
    return (0, types_1.Ok)(validation.data);
}
/**
 * Get default configuration for a project type
 */
function getDefaultConfigForProject(projectType) {
    const baseConfig = {
        enableCaching: true,
        parallelProcessing: true
    };
    switch (projectType) {
        case 'web':
            return {
                ...baseConfig,
                languages: ['javascript', 'typescript', 'html', 'css'],
                ignorePatterns: ['node_modules/**', 'dist/**', 'build/**', '.next/**'],
                cachePolicy: 'adaptive'
            };
        case 'backend':
            return {
                ...baseConfig,
                languages: ['javascript', 'typescript', 'python', 'java', 'go', 'rust'],
                ignorePatterns: ['node_modules/**', 'target/**', '__pycache__/**', 'venv/**'],
                cachePolicy: 'persistent'
            };
        case 'mobile':
            return {
                ...baseConfig,
                languages: ['javascript', 'typescript', 'swift', 'kotlin', 'java'],
                ignorePatterns: ['node_modules/**', 'ios/build/**', 'android/build/**'],
                cachePolicy: 'balanced'
            };
        case 'desktop':
            return {
                ...baseConfig,
                languages: ['javascript', 'typescript', 'rust', 'cpp', 'csharp'],
                ignorePatterns: ['node_modules/**', 'target/**', 'bin/**', 'obj/**'],
                cachePolicy: 'adaptive'
            };
        default:
            return baseConfig;
    }
}
/**
 * Merge multiple configuration objects with validation
 */
function mergeConfigs(...configs) {
    try {
        const merged = configs.reduce((acc, config) => ({
            ...acc,
            ...config,
            // Special handling for nested objects
            ...(config.performance && { performance: config.performance }),
            // Merge arrays instead of replacing
            languages: config.languages || acc.languages,
            ignorePatterns: [
                ...(acc.ignorePatterns || []),
                ...(config.ignorePatterns || [])
            ].filter((pattern, index, arr) => arr.indexOf(pattern) === index) // Remove duplicates
        }), {});
        return validateAnalysisConfig(merged);
    }
    catch (error) {
        return (0, types_1.Err)(new types_1.AnalysisError('Failed to merge configurations', 'CONFIG_MERGE_ERROR', { originalError: error }));
    }
}
/**
 * Create configuration for CI/CD environments
 */
function createCIConfig(_projectRoot) {
    return config_1.ConfigurationManager.getPreset('fast');
}
/**
 * Create configuration for development environments
 */
function createDevConfig(projectRoot) {
    const config = config_1.ConfigurationManager.getPreset('balanced');
    return {
        ...config,
        projectRoot,
        enableWatching: true,
        cachePolicy: 'adaptive'
    };
}
/**
 * Create configuration for production analysis
 */
function createProdConfig(projectRoot) {
    const config = config_1.ConfigurationManager.getPreset('thorough');
    return {
        ...config,
        projectRoot,
        enableWatching: false,
        cachePolicy: 'persistent'
    };
}
/**
 * Detect project type from directory structure
 */
async function detectProjectType(projectRoot) {
    try {
        const fs = await Promise.resolve().then(() => __importStar(require('fs')));
        const path = await Promise.resolve().then(() => __importStar(require('path')));
        // Check for common files/directories
        const packageJsonPath = path.join(projectRoot, 'package.json');
        const cargoTomlPath = path.join(projectRoot, 'Cargo.toml');
        const requirementsPath = path.join(projectRoot, 'requirements.txt');
        const androidPath = path.join(projectRoot, 'android');
        const iosPath = path.join(projectRoot, 'ios');
        if (fs.existsSync(packageJsonPath)) {
            const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
            // Check dependencies for project type indicators
            const deps = { ...packageJson.dependencies, ...packageJson.devDependencies };
            if (deps.react || deps.vue || deps.angular || deps['@angular/core']) {
                return 'web';
            }
            if (deps.express || deps.fastify || deps.koa || deps.nestjs) {
                return 'backend';
            }
            if (deps['react-native'] || fs.existsSync(androidPath) || fs.existsSync(iosPath)) {
                return 'mobile';
            }
            if (deps.electron || deps.tauri) {
                return 'desktop';
            }
        }
        if (fs.existsSync(cargoTomlPath)) {
            return 'backend'; // Assume Rust projects are backend
        }
        if (fs.existsSync(requirementsPath)) {
            return 'backend'; // Assume Python projects are backend
        }
        return 'unknown';
    }
    catch {
        return 'unknown';
    }
}
/**
 * Create smart configuration based on project detection
 */
async function createSmartConfig(projectRoot) {
    const projectType = await detectProjectType(projectRoot);
    if (projectType === 'unknown') {
        // Return a basic config for unknown project types
        return {
            projectRoot,
            enableCaching: true,
            parallelProcessing: true
        };
    }
    const defaultConfig = getDefaultConfigForProject(projectType);
    return {
        projectRoot,
        ...defaultConfig
    };
}
/**
 * Format file size in human-readable format
 */
function formatFileSize(bytes) {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex++;
    }
    return `${size.toFixed(1)} ${units[unitIndex]}`;
}
/**
 * Format duration in human-readable format
 */
function formatDuration(ms) {
    if (ms < 1000)
        return `${ms}ms`;
    if (ms < 60000)
        return `${(ms / 1000).toFixed(1)}s`;
    if (ms < 3600000)
        return `${(ms / 60000).toFixed(1)}m`;
    return `${(ms / 3600000).toFixed(1)}h`;
}
/**
 * Calculate analysis progress percentage
 */
function calculateProgress(filesProcessed, totalFiles) {
    if (totalFiles === 0)
        return 0;
    return Math.min(100, Math.round((filesProcessed / totalFiles) * 100));
}
/**
 * Estimate remaining time based on current progress
 */
function estimateRemainingTime(filesProcessed, totalFiles, elapsedMs) {
    if (filesProcessed === 0 || totalFiles === 0)
        return null;
    const avgTimePerFile = elapsedMs / filesProcessed;
    const remainingFiles = totalFiles - filesProcessed;
    return remainingFiles * avgTimePerFile;
}
//# sourceMappingURL=index.js.map