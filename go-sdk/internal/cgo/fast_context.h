#ifndef FAST_CONTEXT_H
#define FAST_CONTEXT_H

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

// ====================================
// Result Structures
// ====================================

typedef struct {
    char* json_data;
    size_t json_len;
    int error_code;
    char* error_message;
} FastContextResult;

typedef struct {
    char* json_data;
    size_t json_len;
    int progress_phase;
    int progress_current;
    int progress_total;
    double progress_percentage;
    char* progress_message;
    char* progress_current_file;
} FastContextProgress;

// ====================================
// Core Functions
// ====================================

/**
 * Analyze a codebase and return comprehensive results
 * 
 * @param project_root Path to the project root directory
 * @param config_json JSON string containing configuration (can be NULL for defaults)
 * @return FastContextResult* Analysis results or error information
 */
FastContextResult* fast_context_analyze(const char* project_root, const char* config_json);

/**
 * Find symbols of a specific kind in the codebase
 * 
 * @param project_root Path to the project root directory
 * @param symbol_kind String describing the symbol kind (e.g., "function", "class")
 * @return FastContextResult* List of matching symbols or error
 */
FastContextResult* fast_context_find_symbols(const char* project_root, const char* symbol_kind);

/**
 * Find dependencies for a specific symbol
 * 
 * @param project_root Path to the project root directory
 * @param symbol_name Name of the symbol to find dependencies for
 * @return FastContextResult* List of dependencies or error
 */
FastContextResult* fast_context_find_dependencies(const char* project_root, const char* symbol_name);

/**
 * Get the version of the Fast-Context library
 * 
 * @return char* Version string (must be freed with fast_context_free_string)
 */
char* fast_context_get_version(void);

/**
 * Get list of supported programming languages
 * 
 * @return FastContextResult* JSON array of supported languages
 */
FastContextResult* fast_context_get_supported_languages(void);

// ====================================
// File Watching Functions
// ====================================

/**
 * Start watching a project for file changes
 * 
 * @param project_root Path to the project root directory
 * @param callback Function pointer to handle progress updates
 */
void fast_context_start_watching(const char* project_root, void (*callback)(FastContextProgress*));

/**
 * Stop watching the project
 */
void fast_context_stop_watching(void);

// ====================================
// Memory Management
// ====================================

/**
 * Free a FastContextResult structure
 * 
 * @param result Result to free
 */
void fast_context_free_result(FastContextResult* result);

/**
 * Free a FastContextProgress structure
 * 
 * @param progress Progress structure to free
 */
void fast_context_free_progress(FastContextProgress* progress);

/**
 * Free a string returned by Fast-Context functions
 * 
 * @param str String to free
 */
void fast_context_free_string(char* str);

// ====================================
// Configuration Functions
// ====================================

/**
 * Validate a configuration JSON string
 * 
 * @param config_json JSON string containing configuration
 * @return FastContextResult* Validation result
 */
FastContextResult* fast_context_validate_config(const char* config_json);

/**
 * Get default configuration as JSON
 * 
 * @return FastContextResult* Default configuration JSON
 */
FastContextResult* fast_context_get_default_config(void);

// ====================================
// Error Codes
// ====================================

#define FAST_CONTEXT_ERROR_NONE 0
#define FAST_CONTEXT_ERROR_INVALID_CONFIG 1
#define FAST_CONTEXT_ERROR_PROJECT_NOT_FOUND 2
#define FAST_CONTEXT_ERROR_ANALYSIS_FAILED 3
#define FAST_CONTEXT_ERROR_TIMEOUT 4
#define FAST_CONTEXT_ERROR_CANCELLED 5
#define FAST_CONTEXT_ERROR_OUT_OF_MEMORY 6
#define FAST_CONTEXT_ERROR_PERMISSION_DENIED 7
#define FAST_CONTEXT_ERROR_UNSUPPORTED_LANGUAGE 8
#define FAST_CONTEXT_ERROR_INVALID_INPUT 9
#define FAST_CONTEXT_ERROR_INTERNAL 10

#ifdef __cplusplus
}
#endif

#endif // FAST_CONTEXT_H