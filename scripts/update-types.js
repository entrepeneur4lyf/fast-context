#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Script to merge auto-generated TypeScript types with the main index.d.ts file
 * Run after: cargo test generate_typescript_types
 */

const PROJECT_ROOT = path.join(__dirname, '..');
const GENERATED_TYPES_PATH = path.join(PROJECT_ROOT, 'types', 'generated.d.ts');
const INDEX_DTS_PATH = path.join(PROJECT_ROOT, 'index.d.ts');

function updateIndexDts() {
    console.log('🔄 Updating TypeScript definitions...');

    // Check if generated types exist
    if (!fs.existsSync(GENERATED_TYPES_PATH)) {
        console.error('❌ Generated types not found. Run: cargo test generate_typescript_types');
        process.exit(1);
    }

    // Read generated types
    const generatedTypes = fs.readFileSync(GENERATED_TYPES_PATH, 'utf8');
    
    // Read current index.d.ts
    const currentIndex = fs.readFileSync(INDEX_DTS_PATH, 'utf8');

    // Find the insertion point (after the NAPI-RS auto-generated section)
    const napiEndMarker = 'export declare function getSupportedLanguages(): Array<string>';
    const insertionPoint = currentIndex.indexOf(napiEndMarker);
    
    if (insertionPoint === -1) {
        console.error('❌ Could not find insertion point in index.d.ts');
        process.exit(1);
    }

    // Find the end of the line after napiEndMarker
    const lineEnd = currentIndex.indexOf('\n', insertionPoint) + 1;
    
    // Check if auto-generated section already exists
    const autoGenStartMarker = '\n// ========================================\n// Auto-generated types from Rust structs\n// ========================================\n';
    const existingAutoGenStart = currentIndex.indexOf(autoGenStartMarker);
    
    let beforeInsertion, afterInsertion;
    
    if (existingAutoGenStart !== -1) {
        // Find the end of existing auto-generated section
        const autoGenEnd = currentIndex.indexOf('\n/** Detect language from file extension */');
        if (autoGenEnd !== -1) {
            beforeInsertion = currentIndex.substring(0, existingAutoGenStart);
            afterInsertion = currentIndex.substring(autoGenEnd);
        } else {
            // Fallback: replace everything after the marker
            beforeInsertion = currentIndex.substring(0, existingAutoGenStart);
            afterInsertion = '';
        }
    } else {
        // No existing auto-generated section
        beforeInsertion = currentIndex.substring(0, lineEnd);
        afterInsertion = currentIndex.substring(lineEnd);
    }

    // Clean up generated types - remove the header and format properly
    const cleanedTypes = generatedTypes
        .replace(/^\/\/ Auto-generated TypeScript types for Fast-Context[\s\S]*?\n\n/, '')
        .split('\n')
        .map(line => {
            // Fix property naming (convert snake_case to camelCase)
            return line
                .replace(/project_root:/g, 'projectRoot:')
                .replace(/ignore_patterns:/g, 'ignorePatterns:')
                .replace(/enable_caching:/g, 'enableCaching:')
                .replace(/cache_policy:/g, 'cachePolicy:')
                .replace(/enable_watching:/g, 'enableWatching:')
                .replace(/max_files:/g, 'maxFiles:')
                .replace(/parallel_processing:/g, 'parallelProcessing:')
                .replace(/file_count:/g, 'fileCount:')
                .replace(/symbol_count:/g, 'symbolCount:')
                .replace(/relationship_count:/g, 'relationshipCount:')
                .replace(/duration_ms:/g, 'durationMs:')
                .replace(/memory_usage_mb:/g, 'memoryUsageMb:')
                .replace(/total_results:/g, 'totalResults:')
                .replace(/qualified_name:/g, 'qualifiedName:')
                .replace(/file_path:/g, 'filePath:')
                .replace(/start_line:/g, 'startLine:')
                .replace(/end_line:/g, 'endLine:')
                .replace(/total_symbols:/g, 'totalSymbols:')
                .replace(/files_involved:/g, 'filesInvolved:')
                .replace(/complexity_score:/g, 'complexityScore:')
                .replace(/architectural_patterns:/g, 'architecturalPatterns:')
                .replace(/potential_issues:/g, 'potentialIssues:')
                .replace(/pretty_print:/g, 'prettyPrint:')
                .replace(/include_details:/g, 'includeDetails:')
                .replace(/include_relationships:/g, 'includeRelationships:')
                .replace(/max_symbols:/g, 'maxSymbols:')
                .replace(/page_size:/g, 'pageSize:')
                .replace(/sort_field:/g, 'sortField:')
                .replace(/sort_direction:/g, 'sortDirection:')
                .replace(/symbol_kinds:/g, 'symbolKinds:')
                .replace(/file_patterns:/g, 'filePatterns:')
                .replace(/min_complexity:/g, 'minComplexity:')
                .replace(/max_complexity:/g, 'maxComplexity:')
                .replace(/documented_only:/g, 'documentedOnly:')
                .replace(/change_type:/g, 'changeType:')
                .replace(/old_path:/g, 'oldPath:')
                .replace(/affects_analysis:/g, 'affectsAnalysis:')
                .replace(/change_count:/g, 'changeCount:')
                .replace(/batch_timestamp:/g, 'batchTimestamp:')
                .replace(/requires_reanalysis:/g, 'requiresReanalysis:')
                .replace(/impact_level:/g, 'impactLevel:')
                .replace(/chunk_size:/g, 'chunkSize:')
                .replace(/include_progress:/g, 'includeProgress:')
                .replace(/chunk_timeout_ms:/g, 'chunkTimeoutMs:')
                .replace(/chunk_index:/g, 'chunkIndex:')
                .replace(/total_chunks:/g, 'totalChunks:')
                .replace(/is_last:/g, 'isLast:')
                .replace(/processing_time_ms:/g, 'processingTimeMs:');
        })
        .join('\n');

    // Add proper spacing and comments
    const formattedTypes = '\n// ========================================\n' +
                          '// Auto-generated types from Rust structs\n' +
                          '// ========================================\n\n' +
                          cleanedTypes;

    // Combine everything
    const updatedContent = beforeInsertion + formattedTypes + afterInsertion;

    // Write back to index.d.ts
    fs.writeFileSync(INDEX_DTS_PATH, updatedContent);

    console.log('✅ Successfully updated index.d.ts with generated types');
    console.log(`📄 Generated types from: ${GENERATED_TYPES_PATH}`);
    console.log(`📄 Updated file: ${INDEX_DTS_PATH}`);
}

function main() {
    try {
        updateIndexDts();
    } catch (error) {
        console.error('❌ Error updating TypeScript definitions:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = { updateIndexDts };