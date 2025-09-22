// Shared Send + Sync core analyzer used by both Python and Node bindings

use crate::errors::FastContextResult;
use std::path::PathBuf;

pub struct CoreAnalyzer {
    project_root: String,
    languages: Vec<String>,
    ignore_patterns: Vec<String>,
}

#[cfg(not(feature = "python"))]
#[derive(Debug, Clone)]
pub struct CoreAnalysisSummary {
    pub file_count: u32,
    pub symbol_count: u32,
    pub languages: Vec<String>,
    pub duration_ms: u32,
    pub relationships: Vec<crate::symbols::Dependency>,
}

/// Internal analysis result for unified processing
#[derive(Debug, Clone)]
struct InternalAnalysisResult {
    file_count: u32,
    symbol_count: u32,
    languages: Vec<String>,
    relationships: Vec<crate::symbols::Dependency>,
}

/// Result from analyzing a single file
#[derive(Debug, Clone)]
struct FileAnalysisResult {
    file_count: u32,
    symbol_count: u32,
    language: String,
    relationships: Vec<crate::symbols::Dependency>,
}

impl CoreAnalyzer {
    pub fn new(project_root: String, languages: Option<Vec<String>>, ignore_patterns: Option<Vec<String>>) -> Self {
        // Validate project root path
        let validated_root = crate::validation::validate_directory_path(&project_root)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Invalid project root path '{}': {}", project_root, e);
                PathBuf::from(".") // Fallback to current directory
            });
        
        // Validate languages
        let validated_languages = languages
            .map(|langs| crate::validation::validate_languages(&langs)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Invalid languages configuration: {}", e);
                    vec!["rust".to_string(), "javascript".to_string(), "typescript".to_string(), "python".to_string()]
                }))
            .unwrap_or_else(|| vec![
                "rust".to_string(), "javascript".to_string(), "typescript".to_string(), "python".to_string()
            ]);
        
        // Validate ignore patterns
        let validated_ignore_patterns = ignore_patterns
            .map(|patterns| crate::validation::validate_ignore_patterns(&patterns)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Invalid ignore patterns: {}", e);
                    vec!["node_modules/**".to_string(), "target/**".to_string(), ".git/**".to_string()]
                }))
            .unwrap_or_else(|| vec![
                "node_modules/**".to_string(), "target/**".to_string(), ".git/**".to_string()
            ]);
        
        Self {
            project_root: validated_root.to_string_lossy().to_string(),
            languages: validated_languages,
            ignore_patterns: validated_ignore_patterns,
        }
    }

    /// Common analysis logic shared between Python and non-Python implementations
    fn analyze_internal(&self) -> FastContextResult<InternalAnalysisResult> {
        use crate::symbols::{SymbolExtractorFactory, Symbol};
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use rayon::prelude::*;
                use std::time::Instant;
        
        let _start_time = Instant::now();
        
        // Collect file paths first for parallel processing
        let file_paths: Vec<std::path::PathBuf> = self.walk_project_files()
            .map(|entry| entry.path().to_path_buf())
            .collect();

        // Process files in parallel with streaming support for large files
        let results: Vec<_> = file_paths.par_iter()
            .filter_map(|file_path| {
                let path_str = file_path.to_string_lossy();
                
                // Check file size to decide between streaming and regular reading
                match file_path.metadata() {
                    Ok(metadata) if metadata.len() > 1024 * 1024 => { // Files > 1MB use streaming
                        match crate::validation::StreamingTextReader::new(file_path, Some(64 * 1024), Some(10 * 1024 * 1024)) {
                            Ok(mut reader) => {
                                // Read file in chunks and reconstruct content
                                let mut content = String::new();
                                while let Ok(Some(line)) = reader.read_next_line() {
                                    content.push_str(&line);
                                    content.push('\n');
                                }
                                Some((file_path.clone(), content, path_str.to_string()))
                            }
                            Err(_) => None,
                        }
                    }
                    _ => {
                        // Small files use regular reading for better performance
                        match crate::validation::secure_read_file(file_path) {
                            Ok(content) => Some((file_path.clone(), content, path_str.to_string())),
                            Err(_) => None,
                        }
                    }
                }
            })
            .filter_map(|(_file_path, content, path_str)| {
                // Use pooled factories for performance optimization
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                let extractor_factory = SymbolExtractorFactory::new();
                let dep_factory = DependencyExtractorFactory::new();

                if let Some(parse) = scoped_factory.parse_file(&content, &path_str) {
                    let syms: Vec<Symbol> = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                    let deps = dep_factory.extract_dependencies(&parse.tree, &parse.source, syms.clone(), &path_str, parse.language);
                    Some(FileAnalysisResult {
                        file_count: 1,
                        symbol_count: syms.len() as u32,
                        language: parse.language.to_string(),
                        relationships: deps,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Aggregate results
        let mut file_count: u32 = 0;
        let mut symbol_count: u32 = 0;
        let mut languages: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut relationships: Vec<crate::symbols::Dependency> = Vec::new();

        for result in results {
            file_count += result.file_count;
            symbol_count += result.symbol_count;
            languages.insert(result.language);
            relationships.extend(result.relationships);
        }

        Ok(InternalAnalysisResult {
            file_count,
            symbol_count,
            languages: languages.into_iter().collect(),
            relationships,
        })
    }

    /// Helper method to walk project files with filtering
    fn walk_project_files(&self) -> Box<dyn Iterator<Item = walkdir::DirEntry> + '_> {
        use walkdir::WalkDir;
        
        Box::new(
            WalkDir::new(&self.project_root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| {
                    let path_str = entry.path().to_string_lossy();
                    !crate::utils::should_ignore_file(path_str.as_ref(), &self.ignore_patterns)
                })
                .filter(|entry| {
                    if self.languages.is_empty() {
                        return true;
                    }
                    
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            return self.languages.iter().any(|s| {
                                crate::parsers::LanguageId::from_string(s)
                                    .map(|l| l == lang)
                                    .unwrap_or(false)
                            });
                        }
                    }
                    false
                })
        )
    }

    #[cfg(feature = "python")]
    pub fn analyze(&self) -> FastContextResult<crate::python_bindings::AnalysisResult> {
        let internal_result = self.analyze_internal()?;
        
        Ok(crate::python_bindings::AnalysisResult {
            file_count: internal_result.file_count,
            symbol_count: internal_result.symbol_count,
            languages: internal_result.languages,
            duration_ms: 0,
            relationships: internal_result.relationships
                .into_iter()
                .map(crate::python_bindings::PyDependency::from)
                .collect(),
        })
    }

    #[cfg(not(feature = "python"))]
    pub fn analyze(&self) -> FastContextResult<CoreAnalysisSummary> {
        let internal_result = self.analyze_internal()?;
        
        Ok(CoreAnalysisSummary {
            file_count: internal_result.file_count,
            symbol_count: internal_result.symbol_count,
            languages: internal_result.languages,
            duration_ms: 0,
            relationships: internal_result.relationships,
        })
    }

    pub fn find_symbols_by_kind(&self, symbol_kind: String) -> FastContextResult<Vec<String>> {
        use crate::symbols::SymbolExtractorFactory;
        use crate::symbols::SymbolKind;
        
        let extractor_factory = SymbolExtractorFactory::new();
        let mut results = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(&entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    let syms = extractor_factory.extract_symbols(&parse.tree, &parse.source, path_str.as_ref(), parse.language);
                    let filtered = syms.into_iter().filter(|s| {
                        let k = match s.kind {
                            SymbolKind::Function => "function",
                            SymbolKind::Method => "method",
                            SymbolKind::Class => "class",
                            SymbolKind::Struct => "struct",
                            SymbolKind::Union => "union",
                            SymbolKind::Interface => "interface",
                            SymbolKind::Enum => "enum",
                            SymbolKind::Trait => "trait",
                            SymbolKind::Variable => "variable",
                            SymbolKind::Constant => "constant",
                            SymbolKind::Field => "field",
                            SymbolKind::Parameter => "parameter",
                            SymbolKind::Module => "module",
                            SymbolKind::Namespace => "namespace",
                            SymbolKind::Import => "import",
                            SymbolKind::Export => "export",
                            SymbolKind::Type => "type",
                            SymbolKind::Macro => "macro",
                        };
                        k.eq_ignore_ascii_case(&symbol_kind)
                    }).map(|s| s.name).collect::<Vec<_>>();
                    results.extend(filtered);
                }
            }
        }
        Ok(results)
    }

    pub fn find_symbols_in_file(&self, file_path: String) -> FastContextResult<Vec<String>> {
        use crate::symbols::{SymbolExtractorFactory, SymbolKind};
        
        // Use secure file reading with path validation
        let content = crate::validation::secure_read_file(&std::path::Path::new(&file_path))
            .map_err(|e| format!("Invalid file path '{}': {}", file_path, e))?;
        let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
        if let Some(parse) = scoped_factory.parse_file(&content, &file_path) {
            let extractor_factory = SymbolExtractorFactory::new();
            let syms = extractor_factory.extract_symbols(&parse.tree, &parse.source, &file_path, parse.language);
            let mut out: Vec<String> = Vec::new();
            for s in syms {
                let kind = match s.kind {
                    SymbolKind::Function => "function",
                    SymbolKind::Method => "method",
                    SymbolKind::Class => "class",
                    SymbolKind::Struct => "struct",
                    SymbolKind::Interface => "interface",
                    SymbolKind::Enum => "enum",
                    SymbolKind::Trait => "trait",
                    SymbolKind::Variable => "variable",
                    SymbolKind::Constant => "constant",
                    SymbolKind::Field => "field",
                    SymbolKind::Module => "module",
                    SymbolKind::Namespace => "namespace",
                    _ => "symbol",
                };
                out.push(format!("{}: {}", kind, s.name));
            }
            Ok(out)
        } else {
            Ok(vec![])
        }
    }

    pub fn find_dependencies(&self, symbol_name: String) -> FastContextResult<Vec<String>> {
        use crate::symbols::{SymbolExtractorFactory};
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        
        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();
        let mut results: Vec<String> = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(&entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    let symbols = extractor_factory.extract_symbols(&parse.tree, &parse.source, path_str.as_ref(), parse.language);
                    let deps = dep_factory.extract_dependencies(&parse.tree, &parse.source, symbols, path_str.as_ref(), parse.language);
                    for d in deps {
                        if d.to_symbol.contains(&symbol_name) || d.from_symbol.contains(&symbol_name) {
                            results.push(format!("{} -> {} ({:?}) @ {}:{}", d.from_symbol, d.to_symbol, d.relationship_type, d.file_path, d.location.start_line));
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn find_complex_symbols(&self, threshold: u32) -> FastContextResult<Vec<String>> {
        use crate::symbols::SymbolExtractorFactory;
        use crate::symbols::SymbolKind;
        
        let extractor_factory = SymbolExtractorFactory::new();
        let mut complex: Vec<String> = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(&entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    // naive complexity: number of control-flow tokens in file + function count
                    let syms = extractor_factory.extract_symbols(&parse.tree, &parse.source, path_str.as_ref(), parse.language);
                    let func_count = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Function | SymbolKind::Method)).count() as u32;
                    let cf_tokens = ["if ", "else if ", "for ", "while ", "match ", "switch ", "catch ", "except "];
                    let mut score = func_count;
                    for tok in cf_tokens.iter() { score += content.matches(tok).count() as u32; }
                    if score >= threshold {
                        complex.push(format!("{path_str} (complexity: {score})"));
                    }
                }
            }
        }
        Ok(complex)
    }
}

