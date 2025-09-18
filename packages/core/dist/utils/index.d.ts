/**
 * Utility functions for the Enhanced Fast-Context SDK
 */
import { EnhancedFastContextAnalyzer } from '../analyzer';
import { AnalysisConfig, PresetName, Result, AnalysisError } from '../types';
/**
 * Create analyzer with automatic configuration validation
 */
export declare function createAnalyzer(config: unknown): Result<EnhancedFastContextAnalyzer, AnalysisError>;
/**
 * Create analyzer from preset with project root
 */
export declare function createAnalyzerFromPreset(preset: PresetName, projectRoot: string): EnhancedFastContextAnalyzer;
/**
 * Create analyzer from environment variables
 */
export declare function createAnalyzerFromEnvironment(): EnhancedFastContextAnalyzer;
/**
 * Validate configuration without creating analyzer
 */
export declare function validateAnalysisConfig(config: unknown): Result<AnalysisConfig, AnalysisError>;
/**
 * Get default configuration for a project type
 */
export declare function getDefaultConfigForProject(projectType: 'web' | 'backend' | 'mobile' | 'desktop'): Partial<AnalysisConfig>;
/**
 * Merge multiple configuration objects with validation
 */
export declare function mergeConfigs(...configs: Partial<AnalysisConfig>[]): Result<AnalysisConfig, AnalysisError>;
/**
 * Create configuration for CI/CD environments
 */
export declare function createCIConfig(_projectRoot: string): AnalysisConfig;
/**
 * Create configuration for development environments
 */
export declare function createDevConfig(projectRoot: string): AnalysisConfig;
/**
 * Create configuration for production analysis
 */
export declare function createProdConfig(projectRoot: string): AnalysisConfig;
/**
 * Detect project type from directory structure
 */
export declare function detectProjectType(projectRoot: string): Promise<'web' | 'backend' | 'mobile' | 'desktop' | 'unknown'>;
/**
 * Create smart configuration based on project detection
 */
export declare function createSmartConfig(projectRoot: string): Promise<AnalysisConfig>;
/**
 * Format file size in human-readable format
 */
export declare function formatFileSize(bytes: number): string;
/**
 * Format duration in human-readable format
 */
export declare function formatDuration(ms: number): string;
/**
 * Calculate analysis progress percentage
 */
export declare function calculateProgress(filesProcessed: number, totalFiles: number): number;
/**
 * Estimate remaining time based on current progress
 */
export declare function estimateRemainingTime(filesProcessed: number, totalFiles: number, elapsedMs: number): number | null;
//# sourceMappingURL=index.d.ts.map