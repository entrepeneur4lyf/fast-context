/**
 * Input validation utilities
 */
export declare class ValidationError extends Error {
    constructor(message: string);
}
/**
 * Validate that a path exists and is accessible
 */
export declare function validatePath(path: string): Promise<string>;
/**
 * Validate file path for output
 */
export declare function validateOutputPath(path: string): Promise<string>;
/**
 * Validate language list
 */
export declare function validateLanguages(languages: string[]): string[];
/**
 * Validate ignore patterns
 */
export declare function validateIgnorePatterns(patterns: string[]): string[];
/**
 * Validate numeric options
 */
export declare function validateNumber(value: string | number, name: string, min?: number, max?: number): number;
/**
 * Validate output format
 */
export declare function validateFormat(format: string): 'table' | 'json' | 'yaml' | 'markdown';
/**
 * Validate configuration object
 */
export declare function validateConfig(config: any): void;
/**
 * Validate symbol name
 */
export declare function validateSymbolName(name: string): string;
/**
 * Validate search query
 */
export declare function validateSearchQuery(query: string): string;
//# sourceMappingURL=validators.d.ts.map