/**
 * Configuration Management System
 * Provides schema validation, environment-based configuration, and performance presets
 */

import { z } from 'zod';
import { readFileSync } from 'fs';
import { join } from 'path';
import {
  AnalysisConfig,
  AnalysisConfigSchema,
  PresetName,
  Result,
  Ok,
  Err,
  ValidationError,
  ConfigurationError
} from '../types';

export class ConfigurationManager {
  private static readonly DEFAULT_CONFIG: Partial<AnalysisConfig> = {
    enableCaching: true,
    cachePolicy: 'adaptive',
    enableWatching: false,
    parallelProcessing: true,
    performance: {
      maxMemoryMb: 1024,
      timeoutMs: 30000,
      workerThreads: 4,
      chunkSize: 100
    }
  };

  private static readonly PRESETS: Record<PresetName, Partial<AnalysisConfig>> = {
    fast: {
      enableCaching: true,
      cachePolicy: 'minimal',
      parallelProcessing: true,
      performance: {
        maxMemoryMb: 256,
        timeoutMs: 10000,
        workerThreads: 2,
        chunkSize: 50
      }
    },
    balanced: {
      enableCaching: true,
      cachePolicy: 'adaptive',
      parallelProcessing: true,
      performance: {
        maxMemoryMb: 512,
        timeoutMs: 30000,
        workerThreads: 4,
        chunkSize: 100
      }
    },
    thorough: {
      enableCaching: true,
      cachePolicy: 'persistent',
      parallelProcessing: true,
      performance: {
        maxMemoryMb: 2048,
        timeoutMs: 120000,
        workerThreads: 8,
        chunkSize: 200
      }
    }
  };

  /**
   * Validate configuration with comprehensive error reporting
   */
  static validate(config: unknown): Result<AnalysisConfig, ValidationError> {
    try {
      const validated = AnalysisConfigSchema.parse(config);
      
      // Additional business logic validation
      const businessValidation = this.validateBusinessRules(validated);
      if (!businessValidation.success) {
        return businessValidation;
      }
      
      return Ok(validated);
    } catch (error) {
      if (error instanceof z.ZodError) {
        return Err(new ValidationError('Configuration validation failed', error));
      }
      return Err(new ValidationError('Unknown validation error', error as z.ZodError));
    }
  }

  /**
   * Load configuration from environment variables
   */
  static loadFromEnvironment(): AnalysisConfig {
    const envConfig: Partial<AnalysisConfig> = {
      projectRoot: process.env.FAST_CONTEXT_PROJECT_ROOT ?? process.cwd(),
      languages: process.env.FAST_CONTEXT_LANGUAGES?.split(',').map(l => l.trim()),
      ignorePatterns: process.env.FAST_CONTEXT_IGNORE_PATTERNS?.split(',').map(p => p.trim()),
      enableCaching: process.env.FAST_CONTEXT_CACHE !== 'false',
      cachePolicy: (process.env.FAST_CONTEXT_CACHE_POLICY as any) ?? 'adaptive',
      enableWatching: process.env.FAST_CONTEXT_WATCH === 'true',
      maxFiles: process.env.FAST_CONTEXT_MAX_FILES ? parseInt(process.env.FAST_CONTEXT_MAX_FILES, 10) : undefined,
      parallelProcessing: process.env.FAST_CONTEXT_PARALLEL !== 'false'
    };

    // Performance settings from environment
    if (process.env.FAST_CONTEXT_MAX_MEMORY_MB || 
        process.env.FAST_CONTEXT_TIMEOUT_MS || 
        process.env.FAST_CONTEXT_WORKER_THREADS) {
      envConfig.performance = {
        maxMemoryMb: process.env.FAST_CONTEXT_MAX_MEMORY_MB ? 
          parseInt(process.env.FAST_CONTEXT_MAX_MEMORY_MB, 10) : 1024,
        timeoutMs: process.env.FAST_CONTEXT_TIMEOUT_MS ? 
          parseInt(process.env.FAST_CONTEXT_TIMEOUT_MS, 10) : 30000,
        workerThreads: process.env.FAST_CONTEXT_WORKER_THREADS ? 
          parseInt(process.env.FAST_CONTEXT_WORKER_THREADS, 10) : 4,
        chunkSize: process.env.FAST_CONTEXT_CHUNK_SIZE ? 
          parseInt(process.env.FAST_CONTEXT_CHUNK_SIZE, 10) : 100
      };
    }

    return this.mergeWithDefaults(envConfig);
  }

  /**
   * Load configuration from file
   */
  static async loadFromFile(filePath: string): Promise<Result<AnalysisConfig, ConfigurationError>> {
    try {
      const absolutePath = join(process.cwd(), filePath);
      const fileContent = readFileSync(absolutePath, 'utf-8');
      
      let config: unknown;
      if (filePath.endsWith('.json')) {
        config = JSON.parse(fileContent);
      } else {
        return Err(new ConfigurationError(`Unsupported configuration file format: ${filePath}`));
      }

      const validation = this.validate(config);
      if (!validation.success) {
        return Err(new ConfigurationError(
          `Invalid configuration in ${filePath}`,
          { validationError: validation.error }
        ));
      }

      return Ok(validation.data);
    } catch (error) {
      return Err(new ConfigurationError(
        `Failed to load configuration from ${filePath}`,
        { originalError: error }
      ));
    }
  }

  /**
   * Get predefined performance preset
   */
  static getPreset(name: PresetName): AnalysisConfig {
    const preset = this.PRESETS[name];
    return this.mergeWithDefaults(preset);
  }

  /**
   * Create custom preset and save it
   */
  static createCustomPreset(_name: string, config: Partial<AnalysisConfig>): Result<AnalysisConfig, ValidationError> {
    const fullConfig = this.mergeWithDefaults(config);
    const validation = this.validate(fullConfig);
    
    if (!validation.success) {
      return validation;
    }

    // In a real implementation, this would save to a configuration store
    // For now, we just return the validated config
    return Ok(validation.data);
  }

  /**
   * Merge configuration with defaults
   */
  private static mergeWithDefaults(config: Partial<AnalysisConfig>): AnalysisConfig {
    return {
      ...this.DEFAULT_CONFIG,
      ...config,
      performance: {
        ...this.DEFAULT_CONFIG.performance,
        ...config.performance
      }
    } as AnalysisConfig;
  }

  /**
   * Validate business rules beyond schema validation
   */
  private static validateBusinessRules(config: AnalysisConfig): Result<AnalysisConfig, ValidationError> {
    const errors: string[] = [];

    // Validate project root exists
    try {
      const fs = require('fs');
      if (!fs.existsSync(config.projectRoot)) {
        errors.push(`Project root does not exist: ${config.projectRoot}`);
      }
    } catch {
      errors.push(`Cannot access project root: ${config.projectRoot}`);
    }

    // Validate performance settings
    if (config.performance) {
      const { maxMemoryMb, timeoutMs, workerThreads } = config.performance;
      
      if (maxMemoryMb && (maxMemoryMb < 64 || maxMemoryMb > 16384)) {
        errors.push('Memory limit must be between 64MB and 16GB');
      }
      
      if (timeoutMs && (timeoutMs < 1000 || timeoutMs > 600000)) {
        errors.push('Timeout must be between 1 second and 10 minutes');
      }
      
      if (workerThreads && (workerThreads < 1 || workerThreads > 32)) {
        errors.push('Worker threads must be between 1 and 32');
      }
    }

    // Validate language specifications
    if (config.languages) {
      const supportedLanguages = [
        'javascript', 'typescript', 'python', 'rust', 'java', 'c', 'cpp',
        'go', 'ruby', 'php', 'swift', 'kotlin', 'scala', 'csharp'
      ];
      
      const unsupported = config.languages.filter(lang => 
        !supportedLanguages.includes(lang.toLowerCase())
      );
      
      if (unsupported.length > 0) {
        errors.push(`Unsupported languages: ${unsupported.join(', ')}`);
      }
    }

    if (errors.length > 0) {
      const zodError = new z.ZodError([
        {
          code: 'custom',
          message: errors.join('; '),
          path: []
        }
      ]);
      return Err(new ValidationError('Business rule validation failed', zodError));
    }

    return Ok(config);
  }

  /**
   * Get configuration summary for debugging
   */
  static getConfigSummary(config: AnalysisConfig): string {
    return `Fast-Context Configuration:
  Project Root: ${config.projectRoot}
  Languages: ${config.languages?.join(', ') ?? 'auto-detect'}
  Caching: ${config.enableCaching ? config.cachePolicy : 'disabled'}
  Watching: ${config.enableWatching ? 'enabled' : 'disabled'}
  Parallel Processing: ${config.parallelProcessing ? 'enabled' : 'disabled'}
  Max Files: ${config.maxFiles ?? 'unlimited'}
  Performance:
    Memory Limit: ${config.performance?.maxMemoryMb ?? 1024}MB
    Timeout: ${config.performance?.timeoutMs ?? 30000}ms
    Worker Threads: ${config.performance?.workerThreads ?? 4}
    Chunk Size: ${config.performance?.chunkSize ?? 100}`;
  }
}
