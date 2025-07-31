//! # Export Filtering and Pagination
//! 
//! Provides powerful filtering and pagination capabilities for large codebase exports,
//! enabling efficient data retrieval and reducing memory overhead.

use serde::{Deserialize, Serialize};


/// Paginated result container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    /// The data items for this page
    pub items: Vec<T>,
    
    /// Pagination metadata
    pub pagination: PaginationInfo,
    
    /// Applied filters
    pub filters: Option<ResultFilter>,
    
    /// Result statistics
    pub stats: ResultStats,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current page number (0-based)
    pub page: usize,
    
    /// Number of items per page
    pub page_size: usize,
    
    /// Total number of items across all pages
    pub total_items: usize,
    
    /// Total number of pages
    pub total_pages: usize,
    
    /// Whether there is a next page
    pub has_next: bool,
    
    /// Whether there is a previous page
    pub has_previous: bool,
    
    /// Index of the first item on this page
    pub start_index: usize,
    
    /// Index of the last item on this page
    pub end_index: usize,
}

/// Pagination options for export requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    /// Page number to retrieve (0-based)
    pub page: usize,
    
    /// Number of items per page
    pub page_size: usize,
    
    /// Maximum items to return (overrides page_size if smaller)
    pub max_items: Option<usize>,
    
    /// Sort criteria
    pub sort: Option<SortOptions>,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 100,
            max_items: None,
            sort: None,
        }
    }
}

/// Sort options for paginated results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    /// Field to sort by
    pub field: SortField,
    
    /// Sort direction
    pub direction: SortDirection,
    
    /// Secondary sort field (for ties)
    pub secondary: Option<Box<SortOptions>>,
}

/// Available sort fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    Name,
    QualifiedName,
    Kind,
    FilePath,
    Language,
    Complexity,
    DependencyCount,
    DependentCount,
    LineNumber,
    LastModified,
    Size,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Comprehensive filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ResultFilter {
    /// Filter by symbol names (substring match)
    pub name_patterns: Option<Vec<String>>,
    
    /// Filter by qualified names (exact or pattern match)
    pub qualified_name_patterns: Option<Vec<String>>,
    
    /// Filter by symbol kinds
    pub symbol_kinds: Option<Vec<String>>,
    
    /// Filter by programming languages
    pub languages: Option<Vec<String>>,
    
    /// Filter by file paths (glob patterns supported)
    pub file_patterns: Option<Vec<String>>,
    
    /// Filter by complexity range
    pub complexity_range: Option<ComplexityRange>,
    
    /// Filter by dependency count range
    pub dependency_count_range: Option<CountRange>,
    
    /// Filter by dependent count range  
    pub dependent_count_range: Option<CountRange>,
    
    /// Filter by tags (any of these tags)
    pub tags: Option<Vec<String>>,
    
    /// Filter by modifiers (any of these modifiers)
    pub modifiers: Option<Vec<String>>,
    
    /// Include only documented symbols
    pub documented_only: Option<bool>,
    
    /// Include only symbols with signatures
    pub with_signatures_only: Option<bool>,
    
    /// Exclude symbols by patterns
    pub exclude_patterns: Option<ExcludePatterns>,
    
    /// Date range filters
    pub date_range: Option<DateRange>,
}

/// Complexity range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Count range filter (for dependencies, dependents, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountRange {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

/// Exclude patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcludePatterns {
    /// Exclude symbols with these name patterns
    pub names: Option<Vec<String>>,
    
    /// Exclude files with these path patterns
    pub files: Option<Vec<String>>,
    
    /// Exclude these symbol kinds
    pub kinds: Option<Vec<String>>,
    
    /// Exclude test-related symbols
    pub tests: Option<bool>,
    
    /// Exclude generated code
    pub generated: Option<bool>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<u64>, // Unix timestamp
    pub end: Option<u64>,   // Unix timestamp
}

/// Result statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultStats {
    /// Total items before filtering
    pub total_before_filter: usize,
    
    /// Total items after filtering
    pub total_after_filter: usize,
    
    /// Items filtered out
    pub filtered_out: usize,
    
    /// Filter efficiency (0.0 - 1.0)
    pub filter_efficiency: f64,
    
    /// Breakdown by category
    pub category_breakdown: CategoryBreakdown,
}

/// Breakdown of results by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    /// Count by symbol kind
    pub by_kind: std::collections::HashMap<String, usize>,
    
    /// Count by language
    pub by_language: std::collections::HashMap<String, usize>,
    
    /// Count by complexity range
    pub by_complexity: std::collections::HashMap<String, usize>,
    
    /// Count by file type
    pub by_file_type: std::collections::HashMap<String, usize>,
}

/// Filter engine for applying filters and pagination
pub struct FilterEngine;

impl FilterEngine {
    /// Apply filters to export symbols
    pub fn filter_symbols(
        symbols: Vec<super::ExportSymbol>,
        filter: &Option<ResultFilter>,
    ) -> Vec<super::ExportSymbol> {
        if let Some(filter) = filter {
            symbols.into_iter()
                .filter(|symbol| Self::matches_symbol_filter(symbol, filter))
                .collect()
        } else {
            symbols
        }
    }
    
    /// Apply pagination to filtered results
    pub fn paginate_symbols(
        symbols: Vec<super::ExportSymbol>,
        pagination: &PaginationOptions,
        original_count: usize,
    ) -> PagedResult<super::ExportSymbol> {
        let mut symbols = symbols;
        
        // Apply sorting
        if let Some(ref sort) = pagination.sort {
            Self::sort_symbols(&mut symbols, sort);
        }
        
        let total_items = symbols.len();
        let total_pages = if pagination.page_size > 0 {
            total_items.div_ceil(pagination.page_size)
        } else {
            1
        };
        
        let start_index = pagination.page * pagination.page_size;
        let mut end_index = start_index + pagination.page_size;
        
        // Apply max_items limit
        if let Some(max_items) = pagination.max_items {
            end_index = end_index.min(start_index + max_items);
        }
        
        end_index = end_index.min(total_items);
        
        let items = if start_index < total_items {
            symbols.into_iter()
                .skip(start_index)
                .take(end_index - start_index)
                .collect()
        } else {
            Vec::new()
        };
        
        let pagination_info = PaginationInfo {
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
            total_pages,
            has_next: pagination.page + 1 < total_pages,
            has_previous: pagination.page > 0,
            start_index,
            end_index: if items.is_empty() { start_index } else { end_index - 1 },
        };
        
        let stats = Self::calculate_stats(&items, total_items, original_count);
        
        PagedResult {
            items,
            pagination: pagination_info,
            filters: None, // Will be set by caller
            stats,
        }
    }
    
    /// Check if a symbol matches the filter criteria
    fn matches_symbol_filter(symbol: &super::ExportSymbol, filter: &ResultFilter) -> bool {
        // Name pattern matching
        if let Some(ref patterns) = filter.name_patterns {
            if !patterns.iter().any(|pattern| {
                symbol.name.contains(pattern) || 
                Self::glob_match(&symbol.name, pattern)
            }) {
                return false;
            }
        }
        
        // Qualified name pattern matching
        if let Some(ref patterns) = filter.qualified_name_patterns {
            if !patterns.iter().any(|pattern| {
                symbol.qualified_name.contains(pattern) ||
                Self::glob_match(&symbol.qualified_name, pattern)
            }) {
                return false;
            }
        }
        
        // Symbol kind filtering
        if let Some(ref kinds) = filter.symbol_kinds {
            if !kinds.contains(&symbol.kind) {
                return false;
            }
        }
        
        // Language filtering
        if let Some(ref languages) = filter.languages {
            if !languages.contains(&symbol.language) {
                return false;
            }
        }
        
        // File pattern filtering
        if let Some(ref patterns) = filter.file_patterns {
            if !patterns.iter().any(|pattern| {
                symbol.file_path.contains(pattern) ||
                Self::glob_match(&symbol.file_path, pattern)
            }) {
                return false;
            }
        }
        
        // Complexity range filtering
        if let Some(ref range) = filter.complexity_range {
            if let Some(min) = range.min {
                if symbol.complexity < min {
                    return false;
                }
            }
            if let Some(max) = range.max {
                if symbol.complexity > max {
                    return false;
                }
            }
        }
        
        // Dependency count filtering
        if let Some(ref range) = filter.dependency_count_range {
            if let Some(min) = range.min {
                if symbol.dependencies.len() < min {
                    return false;
                }
            }
            if let Some(max) = range.max {
                if symbol.dependencies.len() > max {
                    return false;
                }
            }
        }
        
        // Dependent count filtering
        if let Some(ref range) = filter.dependent_count_range {
            if let Some(min) = range.min {
                if symbol.dependents.len() < min {
                    return false;
                }
            }
            if let Some(max) = range.max {
                if symbol.dependents.len() > max {
                    return false;
                }
            }
        }
        
        // Tag filtering
        if let Some(ref filter_tags) = filter.tags {
            if !filter_tags.iter().any(|tag| symbol.tags.contains(tag)) {
                return false;
            }
        }
        
        // Modifier filtering
        if let Some(ref filter_modifiers) = filter.modifiers {
            if !filter_modifiers.iter().any(|modifier| symbol.modifiers.contains(modifier)) {
                return false;
            }
        }
        
        // Documentation filtering
        if let Some(documented_only) = filter.documented_only {
            if documented_only && symbol.documentation.is_none() {
                return false;
            }
        }
        
        // Signature filtering
        if let Some(with_signatures_only) = filter.with_signatures_only {
            if with_signatures_only && symbol.signature.is_none() {
                return false;
            }
        }
        
        // Exclude patterns
        if let Some(ref exclude) = filter.exclude_patterns {
            if Self::matches_exclude_patterns(symbol, exclude) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if symbol matches exclude patterns
    fn matches_exclude_patterns(symbol: &super::ExportSymbol, exclude: &ExcludePatterns) -> bool {
        // Exclude by name patterns
        if let Some(ref patterns) = exclude.names {
            if patterns.iter().any(|pattern| {
                symbol.name.contains(pattern) ||
                Self::glob_match(&symbol.name, pattern)
            }) {
                return true;
            }
        }
        
        // Exclude by file patterns
        if let Some(ref patterns) = exclude.files {
            if patterns.iter().any(|pattern| {
                symbol.file_path.contains(pattern) ||
                Self::glob_match(&symbol.file_path, pattern)
            }) {
                return true;
            }
        }
        
        // Exclude by kinds
        if let Some(ref kinds) = exclude.kinds {
            if kinds.contains(&symbol.kind) {
                return true;
            }
        }
        
        // Exclude tests
        if let Some(exclude_tests) = exclude.tests {
            if exclude_tests && Self::is_test_symbol(symbol) {
                return true;
            }
        }
        
        // Exclude generated code
        if let Some(exclude_generated) = exclude.generated {
            if exclude_generated && Self::is_generated_symbol(symbol) {
                return true;
            }
        }
        
        false
    }
    
    /// Simple glob pattern matching
    fn glob_match(text: &str, pattern: &str) -> bool {
        // Simple implementation - could be enhanced with full glob support
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.starts_with(prefix) && text.ends_with(suffix);
            }
        }
        text.contains(pattern)
    }
    
    /// Check if symbol is test-related
    fn is_test_symbol(symbol: &super::ExportSymbol) -> bool {
        let name_lower = symbol.name.to_lowercase();
        let file_lower = symbol.file_path.to_lowercase();
        
        name_lower.contains("test") ||
        name_lower.starts_with("test_") ||
        name_lower.ends_with("_test") ||
        file_lower.contains("test") ||
        file_lower.contains("spec") ||
        symbol.tags.iter().any(|tag| tag.to_lowercase().contains("test"))
    }
    
    /// Check if symbol is generated code
    fn is_generated_symbol(symbol: &super::ExportSymbol) -> bool {
        let file_lower = symbol.file_path.to_lowercase();
        
        file_lower.contains("generated") ||
        file_lower.contains(".gen.") ||
        file_lower.contains("_gen.") ||
        file_lower.ends_with(".gen") ||
        symbol.tags.iter().any(|tag| tag.to_lowercase().contains("generated"))
    }
    
    /// Sort symbols based on sort options
    fn sort_symbols(symbols: &mut [super::ExportSymbol], sort: &SortOptions) {
        symbols.sort_by(|a, b| {
            let primary_cmp = Self::compare_symbols(a, b, &sort.field, &sort.direction);
            
            if primary_cmp == std::cmp::Ordering::Equal {
                if let Some(ref secondary) = sort.secondary {
                    Self::compare_symbols(a, b, &secondary.field, &secondary.direction)
                } else {
                    // Default secondary sort by name
                    a.name.cmp(&b.name)
                }
            } else {
                primary_cmp
            }
        });
    }
    
    /// Compare two symbols based on field and direction
    fn compare_symbols(
        a: &super::ExportSymbol,
        b: &super::ExportSymbol,
        field: &SortField,
        direction: &SortDirection,
    ) -> std::cmp::Ordering {
        let cmp = match field {
            SortField::Name => a.name.cmp(&b.name),
            SortField::QualifiedName => a.qualified_name.cmp(&b.qualified_name),
            SortField::Kind => a.kind.cmp(&b.kind),
            SortField::FilePath => a.file_path.cmp(&b.file_path),
            SortField::Language => a.language.cmp(&b.language),
            SortField::Complexity => a.complexity.cmp(&b.complexity),
            SortField::DependencyCount => a.dependencies.len().cmp(&b.dependencies.len()),
            SortField::DependentCount => a.dependents.len().cmp(&b.dependents.len()),
            SortField::LineNumber => a.location.start_line.cmp(&b.location.start_line),
            SortField::LastModified => std::cmp::Ordering::Equal, // Would need file metadata
            SortField::Size => std::cmp::Ordering::Equal, // Would need file metadata
        };
        
        match direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    }
    
    /// Calculate result statistics
    fn calculate_stats(
        items: &[super::ExportSymbol],
        total_after_filter: usize,
        total_before_filter: usize,
    ) -> ResultStats {
        let filtered_out = total_before_filter.saturating_sub(total_after_filter);
        let filter_efficiency = if total_before_filter > 0 {
            total_after_filter as f64 / total_before_filter as f64
        } else {
            1.0
        };
        
        let mut by_kind = std::collections::HashMap::new();
        let mut by_language = std::collections::HashMap::new();
        let mut by_complexity = std::collections::HashMap::new();
        let mut by_file_type = std::collections::HashMap::new();
        
        for symbol in items {
            *by_kind.entry(symbol.kind.clone()).or_insert(0) += 1;
            *by_language.entry(symbol.language.clone()).or_insert(0) += 1;
            
            let complexity_range = match symbol.complexity {
                0..=5 => "Low (0-5)",
                6..=10 => "Medium (6-10)",
                11..=20 => "High (11-20)",
                _ => "Very High (20+)",
            };
            *by_complexity.entry(complexity_range.to_string()).or_insert(0) += 1;
            
            if let Some(ext) = std::path::Path::new(&symbol.file_path).extension() {
                if let Some(ext_str) = ext.to_str() {
                    *by_file_type.entry(ext_str.to_string()).or_insert(0) += 1;
                }
            }
        }
        
        ResultStats {
            total_before_filter,
            total_after_filter,
            filtered_out,
            filter_efficiency,
            category_breakdown: CategoryBreakdown {
                by_kind,
                by_language,
                by_complexity,
                by_file_type,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_symbol(name: &str, kind: &str, complexity: u32) -> crate::export::ExportSymbol {
        crate::export::ExportSymbol {
            id: format!("sym_{}", name),
            name: name.to_string(),
            qualified_name: format!("test::{}", name),
            kind: kind.to_string(),
            file_path: format!("src/{}.rs", name),
            language: "Rust".to_string(),
            location: crate::export::ExportLocation {
                start_line: 1,
                start_column: 0,
                end_line: 5,
                end_column: 1,
                byte_offset: None,
                byte_length: None,
            },
            scope_chain: vec!["test".to_string()],
            modifiers: vec![],
            signature: Some(format!("fn {}()", name)),
            documentation: Some(format!("{} documentation", name)),
            complexity,
            dependencies: vec![],
            dependents: vec![],
            related_files: vec![],
            tags: vec!["function".to_string()],
            confidence: 1.0,
        }
    }

    #[test]
    fn test_symbol_filtering() {
        let symbols = vec![
            create_test_symbol("low_complexity", "Function", 2),
            create_test_symbol("high_complexity", "Function", 15),
            create_test_symbol("test_function", "Function", 5),
        ];

        let mut filter = ResultFilter {
            complexity_range: Some(ComplexityRange {
                min: Some(1),
                max: Some(10),
            }),
            ..Default::default()
        };

        let filtered = FilterEngine::filter_symbols(symbols, &Some(filter));
        assert_eq!(filtered.len(), 2); // Should exclude high_complexity
        
        // Test exclude patterns
        filter = ResultFilter {
            exclude_patterns: Some(ExcludePatterns {
                tests: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let symbols = vec![
            create_test_symbol("normal_function", "Function", 2),
            create_test_symbol("test_function", "Function", 5),
        ];

        let filtered = FilterEngine::filter_symbols(symbols, &Some(filter));
        assert_eq!(filtered.len(), 1); // Should exclude test_function
    }

    #[test]
    fn test_pagination() {
        let symbols = vec![
            create_test_symbol("func_a", "Function", 1),
            create_test_symbol("func_b", "Function", 2),
            create_test_symbol("func_c", "Function", 3),
            create_test_symbol("func_d", "Function", 4),
            create_test_symbol("func_e", "Function", 5),
        ];

        let pagination = PaginationOptions {
            page: 1,
            page_size: 2,
            max_items: None,
            sort: None,
        };

        let result = FilterEngine::paginate_symbols(symbols, &pagination, 5);
        
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.pagination.total_items, 5);
        assert_eq!(result.pagination.total_pages, 3);
        assert_eq!(result.pagination.page, 1);
        assert!(result.pagination.has_next);
        assert!(result.pagination.has_previous);
    }

    #[test]
    fn test_sorting() {
        let mut symbols = vec![
            create_test_symbol("charlie", "Function", 15),
            create_test_symbol("alpha", "Function", 5),
            create_test_symbol("bravo", "Function", 10),
        ];

        let sort = SortOptions {
            field: SortField::Name,
            direction: SortDirection::Ascending,
            secondary: None,
        };

        FilterEngine::sort_symbols(&mut symbols, &sort);
        
        assert_eq!(symbols[0].name, "alpha");
        assert_eq!(symbols[1].name, "bravo");
        assert_eq!(symbols[2].name, "charlie");
        
        let sort = SortOptions {
            field: SortField::Complexity,
            direction: SortDirection::Descending,
            secondary: None,
        };

        FilterEngine::sort_symbols(&mut symbols, &sort);
        
        assert_eq!(symbols[0].complexity, 15);
        assert_eq!(symbols[1].complexity, 10);
        assert_eq!(symbols[2].complexity, 5);
    }
}

