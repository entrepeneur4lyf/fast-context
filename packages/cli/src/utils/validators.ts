/**
 * Input validation utilities
 */

import { access, stat } from 'fs/promises';
import { resolve } from 'path';
import chalk from 'chalk';

export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ValidationError';
  }
}

/**
 * Validate that a path exists and is accessible
 */
export async function validatePath(path: string): Promise<string> {
  if (!path || typeof path !== 'string') {
    throw new ValidationError('Path is required and must be a string');
  }
  
  const resolvedPath = resolve(path);
  
  try {
    await access(resolvedPath);
    const stats = await stat(resolvedPath);
    
    if (!stats.isDirectory()) {
      throw new ValidationError(`Path "${path}" is not a directory`);
    }
    
    return resolvedPath;
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      throw new ValidationError(`Path "${path}" does not exist`);
    } else if (error.code === 'EACCES') {
      throw new ValidationError(`Path "${path}" is not accessible (permission denied)`);
    } else if (error instanceof ValidationError) {
      throw error;
    } else {
      throw new ValidationError(`Invalid path "${path}": ${error.message}`);
    }
  }
}

/**
 * Validate file path for output
 */
export async function validateOutputPath(path: string): Promise<string> {
  if (!path || typeof path !== 'string') {
    throw new ValidationError('Output path is required and must be a string');
  }
  
  const resolvedPath = resolve(path);
  
  try {
    // Check if parent directory exists
    const parentDir = resolve(resolvedPath, '..');
    await access(parentDir);
    
    return resolvedPath;
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      throw new ValidationError(`Parent directory of "${path}" does not exist`);
    } else if (error.code === 'EACCES') {
      throw new ValidationError(`Cannot write to "${path}" (permission denied)`);
    } else {
      throw new ValidationError(`Invalid output path "${path}": ${error.message}`);
    }
  }
}

/**
 * Validate language list
 */
export function validateLanguages(languages: string[]): string[] {
  const supportedLanguages = [
    'typescript', 'javascript', 'python', 'rust', 'go', 'java', 'c', 'cpp',
    'csharp', 'php', 'ruby', 'swift', 'kotlin', 'scala', 'html', 'css',
    'json', 'yaml', 'xml', 'markdown', 'bash', 'lua'
  ];
  
  const validLanguages: string[] = [];
  const invalidLanguages: string[] = [];
  
  for (const lang of languages) {
    const normalizedLang = lang.toLowerCase().trim();
    if (supportedLanguages.includes(normalizedLang)) {
      validLanguages.push(normalizedLang);
    } else {
      invalidLanguages.push(lang);
    }
  }
  
  if (invalidLanguages.length > 0) {
    console.warn(chalk.yellow('Warning: Unsupported languages ignored:'), invalidLanguages.join(', '));
    console.warn(chalk.gray('Supported languages:'), supportedLanguages.join(', '));
  }
  
  return validLanguages;
}

/**
 * Validate ignore patterns
 */
export function validateIgnorePatterns(patterns: string[]): string[] {
  return patterns.filter(pattern => {
    if (!pattern || typeof pattern !== 'string') {
      console.warn(chalk.yellow('Warning: Invalid ignore pattern ignored:'), pattern);
      return false;
    }
    return true;
  });
}

/**
 * Validate numeric options
 */
export function validateNumber(
  value: string | number,
  name: string,
  min?: number,
  max?: number
): number {
  const num = typeof value === 'string' ? parseInt(value, 10) : value;
  
  if (isNaN(num)) {
    throw new ValidationError(`${name} must be a valid number`);
  }
  
  if (min !== undefined && num < min) {
    throw new ValidationError(`${name} must be at least ${min}`);
  }
  
  if (max !== undefined && num > max) {
    throw new ValidationError(`${name} must be at most ${max}`);
  }
  
  return num;
}

/**
 * Validate output format
 */
export function validateFormat(format: string): 'table' | 'json' | 'yaml' | 'markdown' {
  const validFormats = ['table', 'json', 'yaml', 'markdown'] as const;
  const normalizedFormat = format.toLowerCase().trim();
  
  if (!validFormats.includes(normalizedFormat as any)) {
    throw new ValidationError(
      `Invalid format "${format}". Supported formats: ${validFormats.join(', ')}`
    );
  }
  
  return normalizedFormat as 'table' | 'json' | 'yaml' | 'markdown';
}

/**
 * Validate configuration object
 */
export function validateConfig(config: any): void {
  if (!config || typeof config !== 'object') {
    throw new ValidationError('Configuration must be an object');
  }
  
  // Validate required fields
  if (!config.projectRoot) {
    throw new ValidationError('Configuration must include projectRoot');
  }
  
  // Validate optional numeric fields
  if (config.maxDepth !== undefined) {
    config.maxDepth = validateNumber(config.maxDepth, 'maxDepth', 1, 100);
  }
  
  if (config.maxFiles !== undefined) {
    config.maxFiles = validateNumber(config.maxFiles, 'maxFiles', 1, 1000000);
  }
  
  // Validate arrays
  if (config.languages !== undefined) {
    if (!Array.isArray(config.languages)) {
      throw new ValidationError('languages must be an array');
    }
    config.languages = validateLanguages(config.languages);
  }
  
  if (config.ignorePatterns !== undefined) {
    if (!Array.isArray(config.ignorePatterns)) {
      throw new ValidationError('ignorePatterns must be an array');
    }
    config.ignorePatterns = validateIgnorePatterns(config.ignorePatterns);
  }
  
  // Validate boolean fields
  const booleanFields = ['enableCaching', 'parallelProcessing', 'includeTests', 'includeDocs'];
  for (const field of booleanFields) {
    if (config[field] !== undefined && typeof config[field] !== 'boolean') {
      throw new ValidationError(`${field} must be a boolean`);
    }
  }
}

/**
 * Validate symbol name
 */
export function validateSymbolName(name: string): string {
  if (!name || typeof name !== 'string') {
    throw new ValidationError('Symbol name is required and must be a string');
  }
  
  const trimmed = name.trim();
  if (trimmed.length === 0) {
    throw new ValidationError('Symbol name cannot be empty');
  }
  
  // Basic validation for symbol names (allow most characters)
  if (trimmed.length > 200) {
    throw new ValidationError('Symbol name is too long (max 200 characters)');
  }
  
  return trimmed;
}

/**
 * Validate search query
 */
export function validateSearchQuery(query: string): string {
  if (!query || typeof query !== 'string') {
    throw new ValidationError('Search query is required and must be a string');
  }
  
  const trimmed = query.trim();
  if (trimmed.length === 0) {
    throw new ValidationError('Search query cannot be empty');
  }
  
  if (trimmed.length < 2) {
    throw new ValidationError('Search query must be at least 2 characters long');
  }
  
  if (trimmed.length > 500) {
    throw new ValidationError('Search query is too long (max 500 characters)');
  }
  
  return trimmed;
}
