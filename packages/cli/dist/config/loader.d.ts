/**
 * Configuration loading and management
 */
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
/**
 * Load configuration from various sources
 */
export declare function loadConfig(configPath?: string, overrides?: Partial<FastContextConfig>): Promise<FastContextConfig>;
/**
 * Save configuration to a file
 */
export declare function saveConfig(config: Partial<FastContextConfig>, filePath: string, format?: 'json' | 'yaml'): Promise<void>;
/**
 * Get default configuration file path for the current project
 */
export declare function getDefaultConfigPath(projectRoot: string, format?: 'json' | 'yaml'): string;
/**
 * Get global configuration file path
 */
export declare function getGlobalConfigPath(format?: 'json' | 'yaml'): string;
/**
 * Merge configurations with proper precedence
 */
export declare function mergeConfigs(base: Partial<FastContextConfig>, override: Partial<FastContextConfig>): Partial<FastContextConfig>;
/**
 * Create a configuration preset
 */
export declare function createPreset(name: string): Partial<FastContextConfig>;
//# sourceMappingURL=loader.d.ts.map