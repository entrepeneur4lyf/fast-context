/**
 * Configuration loading and management
 */

import { readFile, access } from 'fs/promises';
import { resolve, join } from 'path';
import { homedir } from 'os';
import yaml from 'js-yaml';
import chalk from 'chalk';
import { validateConfig } from '../utils/validators.js';

export interface FastContextConfig {
  projectRoot: string;
  languages?: string[];
  ignorePatterns?: string[];
  maxDepth?: number;
  maxFiles?: number;
  enableCaching?: boolean;
  parallelProcessing?: boolean;
  includeTests?: boolean;
  includeDocs?: boolean;
  outputFormat?: 'table' | 'json' | 'yaml' | 'markdown';
  verbose?: boolean;
  debug?: boolean;
}

export interface ConfigLoadOptions {
  configPath?: string;
  projectRoot?: string;
  override?: Partial<FastContextConfig>;
}

const DEFAULT_CONFIG: Partial<FastContextConfig> = {
  languages: [],
  ignorePatterns: [
    'node_modules',
    'dist',
    'build',
    '.git',
    '.svn',
    '.hg',
    'coverage',
    '.nyc_output',
    'target',
    'bin',
    'obj',
    '*.log',
    '*.tmp',
    '*.temp'
  ],
  maxDepth: 10,
  maxFiles: 10000,
  enableCaching: true,
  parallelProcessing: true,
  includeTests: false,
  includeDocs: false,
  outputFormat: 'table',
  verbose: false,
  debug: false
};

/**
 * Load configuration from various sources
 */
export async function loadConfig(
  configPath?: string,
  overrides?: Partial<FastContextConfig>
): Promise<FastContextConfig> {
  let config: Partial<FastContextConfig> = { ...DEFAULT_CONFIG };
  
  // 1. Load from global config file
  const globalConfig = await loadGlobalConfig();
  if (globalConfig) {
    config = { ...config, ...globalConfig };
  }
  
  // 2. Load from project config file
  const projectConfig = await loadProjectConfig(overrides?.projectRoot);
  if (projectConfig) {
    config = { ...config, ...projectConfig };
  }
  
  // 3. Load from specified config file
  if (configPath) {
    const fileConfig = await loadConfigFile(configPath);
    if (fileConfig) {
      config = { ...config, ...fileConfig };
    }
  }
  
  // 4. Apply command-line overrides
  if (overrides) {
    config = { ...config, ...overrides };
  }
  
  // 5. Validate final configuration
  validateConfig(config);
  
  return config as FastContextConfig;
}

/**
 * Load global configuration from user's home directory
 */
async function loadGlobalConfig(): Promise<Partial<FastContextConfig> | null> {
  const globalConfigPaths = [
    join(homedir(), '.fast-context.json'),
    join(homedir(), '.fast-context.yaml'),
    join(homedir(), '.fast-context.yml'),
    join(homedir(), '.config', 'fast-context', 'config.json'),
    join(homedir(), '.config', 'fast-context', 'config.yaml')
  ];
  
  for (const configPath of globalConfigPaths) {
    try {
      await access(configPath);
      return await loadConfigFile(configPath);
    } catch {
      // File doesn't exist, try next
      continue;
    }
  }
  
  return null;
}

/**
 * Load project-specific configuration
 */
async function loadProjectConfig(projectRoot?: string): Promise<Partial<FastContextConfig> | null> {
  if (!projectRoot) return null;
  
  const projectConfigPaths = [
    join(projectRoot, '.fast-context.json'),
    join(projectRoot, '.fast-context.yaml'),
    join(projectRoot, '.fast-context.yml'),
    join(projectRoot, 'fast-context.config.json'),
    join(projectRoot, 'fast-context.config.js'),
    join(projectRoot, 'package.json') // Check for fast-context section
  ];
  
  for (const configPath of projectConfigPaths) {
    try {
      await access(configPath);
      
      if (configPath.endsWith('package.json')) {
        return await loadPackageJsonConfig(configPath);
      } else {
        return await loadConfigFile(configPath);
      }
    } catch {
      // File doesn't exist, try next
      continue;
    }
  }
  
  return null;
}

/**
 * Load configuration from a specific file
 */
async function loadConfigFile(filePath: string): Promise<Partial<FastContextConfig> | null> {
  try {
    const content = await readFile(filePath, 'utf-8');
    
    if (filePath.endsWith('.json')) {
      return JSON.parse(content);
    } else if (filePath.endsWith('.yaml') || filePath.endsWith('.yml')) {
      return yaml.load(content) as Partial<FastContextConfig>;
    } else if (filePath.endsWith('.js')) {
      // Dynamic import for JS config files
      const configModule = await import(resolve(filePath));
      return configModule.default || configModule;
    }
    
    return null;
  } catch (error: any) {
    console.warn(chalk.yellow(`Warning: Failed to load config from ${filePath}: ${error.message}`));
    return null;
  }
}

/**
 * Load configuration from package.json fast-context section
 */
async function loadPackageJsonConfig(packageJsonPath: string): Promise<Partial<FastContextConfig> | null> {
  try {
    const content = await readFile(packageJsonPath, 'utf-8');
    const packageJson = JSON.parse(content);
    
    return packageJson['fast-context'] || packageJson.fastContext || null;
  } catch (error: any) {
    console.warn(chalk.yellow(`Warning: Failed to load config from package.json: ${error.message}`));
    return null;
  }
}

/**
 * Save configuration to a file
 */
export async function saveConfig(
  config: Partial<FastContextConfig>,
  filePath: string,
  format: 'json' | 'yaml' = 'json'
): Promise<void> {
  const { writeFile } = await import('fs/promises');
  
  let content: string;
  
  if (format === 'yaml') {
    content = yaml.dump(config, { indent: 2 });
  } else {
    content = JSON.stringify(config, null, 2);
  }
  
  await writeFile(filePath, content, 'utf-8');
}

/**
 * Get default configuration file path for the current project
 */
export function getDefaultConfigPath(projectRoot: string, format: 'json' | 'yaml' = 'json'): string {
  const extension = format === 'yaml' ? 'yaml' : 'json';
  return join(projectRoot, `.fast-context.${extension}`);
}

/**
 * Get global configuration file path
 */
export function getGlobalConfigPath(format: 'json' | 'yaml' = 'json'): string {
  const extension = format === 'yaml' ? 'yaml' : 'json';
  return join(homedir(), `.fast-context.${extension}`);
}

/**
 * Merge configurations with proper precedence
 */
export function mergeConfigs(
  base: Partial<FastContextConfig>,
  override: Partial<FastContextConfig>
): Partial<FastContextConfig> {
  const merged = { ...base };
  
  for (const [key, value] of Object.entries(override)) {
    if (value !== undefined) {
      if (Array.isArray(value) && Array.isArray(merged[key as keyof FastContextConfig])) {
        // For arrays, concatenate and deduplicate
        const baseArray = merged[key as keyof FastContextConfig] as string[];
        merged[key as keyof FastContextConfig] = [...new Set([...baseArray, ...value])] as any;
      } else {
        // For other values, override completely
        merged[key as keyof FastContextConfig] = value as any;
      }
    }
  }
  
  return merged;
}

/**
 * Create a configuration preset
 */
export function createPreset(name: string): Partial<FastContextConfig> {
  const presets: { [key: string]: Partial<FastContextConfig> } = {
    'typescript': {
      languages: ['typescript', 'javascript'],
      ignorePatterns: ['node_modules', 'dist', 'build', '.next', 'coverage'],
      includeTests: true
    },
    
    'react': {
      languages: ['typescript', 'javascript', 'html', 'css'],
      ignorePatterns: ['node_modules', 'build', 'dist', '.next', 'coverage', 'public'],
      includeTests: true,
      includeDocs: true
    },
    
    'node': {
      languages: ['javascript', 'typescript'],
      ignorePatterns: ['node_modules', 'dist', 'coverage', 'logs'],
      includeTests: true
    },
    
    'python': {
      languages: ['python'],
      ignorePatterns: ['__pycache__', '*.pyc', 'venv', '.venv', 'env', '.env', 'dist', 'build'],
      includeTests: true
    },
    
    'rust': {
      languages: ['rust'],
      ignorePatterns: ['target', 'Cargo.lock'],
      includeTests: true
    },
    
    'go': {
      languages: ['go'],
      ignorePatterns: ['vendor', 'bin'],
      includeTests: true
    },
    
    'minimal': {
      languages: [],
      ignorePatterns: ['.git', 'node_modules'],
      maxFiles: 1000,
      includeTests: false,
      includeDocs: false
    },
    
    'comprehensive': {
      languages: [],
      ignorePatterns: ['.git'],
      maxFiles: 100000,
      includeTests: true,
      includeDocs: true,
      verbose: true
    }
  };
  
  return presets[name] || {};
}
