/**
 * Configuration Management System
 * Provides schema validation, environment-based configuration, and performance presets
 */
import { AnalysisConfig, PresetName, Result, ValidationError, ConfigurationError } from '../types';
export declare class ConfigurationManager {
    private static readonly DEFAULT_CONFIG;
    private static readonly PRESETS;
    /**
     * Validate configuration with comprehensive error reporting
     */
    static validate(config: unknown): Result<AnalysisConfig, ValidationError>;
    /**
     * Load configuration from environment variables
     */
    static loadFromEnvironment(): AnalysisConfig;
    /**
     * Load configuration from file
     */
    static loadFromFile(filePath: string): Promise<Result<AnalysisConfig, ConfigurationError>>;
    /**
     * Get predefined performance preset
     */
    static getPreset(name: PresetName): AnalysisConfig;
    /**
     * Create custom preset and save it
     */
    static createCustomPreset(_name: string, config: Partial<AnalysisConfig>): Result<AnalysisConfig, ValidationError>;
    /**
     * Merge configuration with defaults
     */
    private static mergeWithDefaults;
    /**
     * Validate business rules beyond schema validation
     */
    private static validateBusinessRules;
    /**
     * Get configuration summary for debugging
     */
    static getConfigSummary(config: AnalysisConfig): string;
}
//# sourceMappingURL=index.d.ts.map