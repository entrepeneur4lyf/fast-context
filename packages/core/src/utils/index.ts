/**
 * Utility functions for the Enhanced Fast-Context SDK
 */

import { EnhancedFastContextAnalyzer } from '../analyzer';
import { ConfigurationManager } from '../config';
import {
  AnalysisConfig,
  PresetName,
  Result,
  Ok,
  Err,
  AnalysisError,
  validateConfig
} from '../types';

/**
 * Create analyzer with automatic configuration validation
 */
export function createAnalyzer(config: unknown): Result<EnhancedFastContextAnalyzer, AnalysisError> {
  return EnhancedFastContextAnalyzer.create(config);
}

/**
 * Create analyzer from preset with project root
 */
export function createAnalyzerFromPreset(
  preset: PresetName,
  projectRoot: string
): EnhancedFastContextAnalyzer {
  return EnhancedFastContextAnalyzer.fromPreset(preset, projectRoot);
}

/**
 * Create analyzer from environment variables
 */
export function createAnalyzerFromEnvironment(): EnhancedFastContextAnalyzer {
  return EnhancedFastContextAnalyzer.fromEnvironment();
}

/**
 * Validate configuration without creating analyzer
 */
export function validateAnalysisConfig(config: unknown): Result<AnalysisConfig, AnalysisError> {
  const validation = validateConfig(config);
  if (!validation.success) {
    return Err(new AnalysisError(
      'Configuration validation failed',
      'VALIDATION_ERROR',
      { validationError: validation.error }
    ));
  }
  return Ok(validation.data);
}

/**
 * Get default configuration for a project type
 */
export function getDefaultConfigForProject(projectType: 'web' | 'backend' | 'mobile' | 'desktop'): Partial<AnalysisConfig> {
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
export function mergeConfigs(...configs: Partial<AnalysisConfig>[]): Result<AnalysisConfig, AnalysisError> {
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
    }), {} as Partial<AnalysisConfig>);

    return validateAnalysisConfig(merged);
  } catch (error) {
    return Err(new AnalysisError(
      'Failed to merge configurations',
      'CONFIG_MERGE_ERROR',
      { originalError: error }
    ));
  }
}

/**
 * Create configuration for CI/CD environments
 */
export function createCIConfig(_projectRoot: string): AnalysisConfig {
  return ConfigurationManager.getPreset('fast');
}

/**
 * Create configuration for development environments
 */
export function createDevConfig(projectRoot: string): AnalysisConfig {
  const config = ConfigurationManager.getPreset('balanced');
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
export function createProdConfig(projectRoot: string): AnalysisConfig {
  const config = ConfigurationManager.getPreset('thorough');
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
export async function detectProjectType(projectRoot: string): Promise<'web' | 'backend' | 'mobile' | 'desktop' | 'unknown'> {
  try {
    const fs = await import('fs');
    const path = await import('path');
    
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
  } catch {
    return 'unknown';
  }
}

/**
 * Create smart configuration based on project detection
 */
export async function createSmartConfig(projectRoot: string): Promise<AnalysisConfig> {
  const projectType = await detectProjectType(projectRoot);

  if (projectType === 'unknown') {
    // Return a basic config for unknown project types
    return {
      projectRoot,
      enableCaching: true,
      parallelProcessing: true
    } as AnalysisConfig;
  }

  const defaultConfig = getDefaultConfigForProject(projectType);

  return {
    projectRoot,
    ...defaultConfig
  } as AnalysisConfig;
}

/**
 * Format file size in human-readable format
 */
export function formatFileSize(bytes: number): string {
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
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3600000) return `${(ms / 60000).toFixed(1)}m`;
  return `${(ms / 3600000).toFixed(1)}h`;
}

/**
 * Calculate analysis progress percentage
 */
export function calculateProgress(filesProcessed: number, totalFiles: number): number {
  if (totalFiles === 0) return 0;
  return Math.min(100, Math.round((filesProcessed / totalFiles) * 100));
}

/**
 * Estimate remaining time based on current progress
 */
export function estimateRemainingTime(
  filesProcessed: number,
  totalFiles: number,
  elapsedMs: number
): number | null {
  if (filesProcessed === 0 || totalFiles === 0) return null;
  
  const avgTimePerFile = elapsedMs / filesProcessed;
  const remainingFiles = totalFiles - filesProcessed;
  
  return remainingFiles * avgTimePerFile;
}
