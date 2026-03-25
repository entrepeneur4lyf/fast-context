// Shared Send + Sync core analyzer used by both Python and Node bindings

use crate::errors::FastContextResult;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct CoreAnalyzerOptions {
    pub max_files: Option<usize>,
    pub parallel_processing: bool,
}

impl Default for CoreAnalyzerOptions {
    fn default() -> Self {
        Self {
            max_files: None,
            parallel_processing: true,
        }
    }
}

pub struct CoreAnalyzer {
    project_root: String,
    languages: Vec<String>,
    ignore_patterns: Vec<String>,
    options: CoreAnalyzerOptions,
    root_validation_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CoreAnalysisSummary {
    pub file_count: u32,
    pub symbol_count: u32,
    pub languages: Vec<String>,
    pub duration_ms: u32,
    pub relationships: Vec<crate::symbols::Dependency>,
    pub skipped_files: Vec<SkippedFileDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct SkippedFileDiagnostic {
    pub file_path: String,
    pub stage: String,
    pub reason: String,
}

/// Internal analysis result for unified processing
#[derive(Debug, Clone)]
struct InternalAnalysisResult {
    file_count: u32,
    symbol_count: u32,
    languages: Vec<String>,
    duration_ms: u32,
    relationships: Vec<crate::symbols::Dependency>,
    skipped_files: Vec<SkippedFileDiagnostic>,
}

/// Result from analyzing a single file
#[derive(Debug, Clone)]
struct FileAnalysisResult {
    file_count: u32,
    symbol_count: u32,
    language: String,
    relationships: Vec<crate::symbols::Dependency>,
}

#[derive(Debug, Clone)]
enum FileAnalysisOutcome {
    Analyzed(FileAnalysisResult),
    Skipped(SkippedFileDiagnostic),
    Ignored,
}

impl CoreAnalyzer {
    fn is_excluded_source_artifact(path: &std::path::Path) -> bool {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        file_name.ends_with(".d.ts")
            || file_name.ends_with(".d.mts")
            || file_name.ends_with(".d.cts")
    }

    fn default_languages() -> Vec<String> {
        use crate::parsers::LanguageId;

        [
            LanguageId::Rust,
            LanguageId::Python,
            LanguageId::JavaScript,
            LanguageId::TypeScript,
            LanguageId::Java,
            LanguageId::Go,
            LanguageId::CSharp,
            LanguageId::Cpp,
            LanguageId::Swift,
            LanguageId::ObjectiveC,
            LanguageId::PHP,
            LanguageId::Ruby,
            LanguageId::Scala,
            LanguageId::Zig,
            LanguageId::Dart,
            LanguageId::Lua,
            LanguageId::Bash,
            LanguageId::CSS,
            LanguageId::HTML,
            LanguageId::XML,
            LanguageId::JSON,
            LanguageId::YAML,
            LanguageId::Markdown,
            LanguageId::JSDoc,
            LanguageId::Regex,
        ]
        .into_iter()
        .map(|language| language.to_lowercase_string())
        .collect()
    }

    pub fn new(
        project_root: String,
        languages: Option<Vec<String>>,
        ignore_patterns: Option<Vec<String>>,
    ) -> Self {
        Self::with_options(
            project_root,
            languages,
            ignore_patterns,
            CoreAnalyzerOptions::default(),
        )
    }

    pub fn with_options(
        project_root: String,
        languages: Option<Vec<String>>,
        ignore_patterns: Option<Vec<String>>,
        options: CoreAnalyzerOptions,
    ) -> Self {
        // Validate project root path
        let (validated_root, root_validation_error) =
            match crate::validation::validate_directory_path(&project_root) {
                Ok(root) => (root, None),
                Err(e) => (PathBuf::from(project_root.clone()), Some(e.to_string())),
            };

        // Validate languages
        let validated_languages = languages
            .map(|langs| {
                crate::validation::validate_languages(&langs).unwrap_or_else(|e| {
                    eprintln!("Warning: Invalid languages configuration: {}", e);
                    Self::default_languages()
                })
            })
            .unwrap_or_else(Self::default_languages);

        // Validate ignore patterns
        let validated_ignore_patterns = ignore_patterns.map(|patterns| {
            crate::validation::validate_ignore_patterns(&patterns).unwrap_or_else(|e| {
                eprintln!("Warning: Invalid ignore patterns: {}", e);
                crate::utils::default_ignore_patterns()
            })
        });
        let validated_ignore_patterns =
            crate::utils::merged_ignore_patterns(validated_ignore_patterns);

        Self {
            project_root: validated_root.to_string_lossy().to_string(),
            languages: validated_languages,
            ignore_patterns: validated_ignore_patterns,
            options: CoreAnalyzerOptions {
                max_files: options.max_files.filter(|max_files| *max_files > 0),
                parallel_processing: options.parallel_processing,
            },
            root_validation_error,
        }
    }

    fn ensure_valid_project_root(&self) -> FastContextResult<()> {
        if let Some(error) = &self.root_validation_error {
            return Err(format!("Invalid project root '{}': {}", self.project_root, error).into());
        }

        Ok(())
    }

    fn analyze_file(file_path: &std::path::Path) -> FileAnalysisOutcome {
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use crate::symbols::{Symbol, SymbolExtractorFactory};

        let path_str = file_path.to_string_lossy().into_owned();
        let Some(detected_language) = crate::utils::detect_language_id(&path_str) else {
            return FileAnalysisOutcome::Ignored;
        };

        let content = match file_path.metadata() {
            Ok(metadata) if metadata.len() > 1024 * 1024 => {
                match crate::validation::StreamingTextReader::new(
                    file_path,
                    Some(64 * 1024),
                    Some(10 * 1024 * 1024),
                ) {
                    Ok(mut reader) => {
                        let mut content = String::new();
                        loop {
                            match reader.read_next_line() {
                                Ok(Some(line)) => {
                                    content.push_str(&line);
                                    content.push('\n');
                                }
                                Ok(None) => break,
                                Err(err) => {
                                    return FileAnalysisOutcome::Skipped(SkippedFileDiagnostic {
                                        file_path: path_str.clone(),
                                        stage: "read".to_string(),
                                        reason: err.to_string(),
                                    });
                                }
                            }
                        }
                        content
                    }
                    Err(err) => {
                        return FileAnalysisOutcome::Skipped(SkippedFileDiagnostic {
                            file_path: path_str.clone(),
                            stage: "read".to_string(),
                            reason: err.to_string(),
                        });
                    }
                }
            }
            _ => match crate::validation::secure_read_file(file_path) {
                Ok(content) => content,
                Err(err) => {
                    return FileAnalysisOutcome::Skipped(SkippedFileDiagnostic {
                        file_path: path_str.clone(),
                        stage: "read".to_string(),
                        reason: err.to_string(),
                    });
                }
            },
        };

        let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();

        let Some(parse) = scoped_factory.parse_file(&content, &path_str) else {
            return FileAnalysisOutcome::Skipped(SkippedFileDiagnostic {
                file_path: path_str,
                stage: "parse".to_string(),
                reason: format!("Failed to parse {} source file", detected_language),
            });
        };

        let syms: Vec<Symbol> = extractor_factory.extract_symbols(
            &parse.tree,
            &parse.source,
            &path_str,
            parse.language,
        );
        let deps = dep_factory.extract_dependencies(
            &parse.tree,
            &parse.source,
            syms.clone(),
            &path_str,
            parse.language,
        );

        FileAnalysisOutcome::Analyzed(FileAnalysisResult {
            file_count: 1,
            symbol_count: syms.len() as u32,
            language: parse.language.to_string(),
            relationships: deps,
        })
    }

    /// Common analysis logic shared between Python and non-Python implementations
    fn analyze_internal(&self) -> FastContextResult<InternalAnalysisResult> {
        use rayon::prelude::*;
        use std::time::Instant;

        self.ensure_valid_project_root()?;

        let start_time = Instant::now();

        let max_files = self.options.max_files.unwrap_or(usize::MAX);
        let file_paths: Vec<std::path::PathBuf> = self
            .walk_project_files()
            .take(max_files)
            .map(|entry| entry.path().to_path_buf())
            .collect();

        let outcomes: Vec<_> = if self.options.parallel_processing {
            file_paths
                .par_iter()
                .map(|file_path| Self::analyze_file(file_path.as_path()))
                .collect()
        } else {
            file_paths
                .iter()
                .map(|file_path| Self::analyze_file(file_path.as_path()))
                .collect()
        };

        // Aggregate results
        let mut file_count: u32 = 0;
        let mut symbol_count: u32 = 0;
        let mut languages: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut relationships: Vec<crate::symbols::Dependency> = Vec::new();
        let mut skipped_files: Vec<SkippedFileDiagnostic> = Vec::new();

        for outcome in outcomes {
            match outcome {
                FileAnalysisOutcome::Analyzed(result) => {
                    file_count += result.file_count;
                    symbol_count += result.symbol_count;
                    languages.insert(result.language);
                    relationships.extend(result.relationships);
                }
                FileAnalysisOutcome::Skipped(diagnostic) => skipped_files.push(diagnostic),
                FileAnalysisOutcome::Ignored => {}
            }
        }

        Ok(InternalAnalysisResult {
            file_count,
            symbol_count,
            languages: languages.into_iter().collect(),
            duration_ms: start_time.elapsed().as_millis() as u32,
            relationships,
            skipped_files,
        })
    }

    /// Helper method to walk project files with filtering
    fn walk_project_files(&self) -> Box<dyn Iterator<Item = walkdir::DirEntry> + '_> {
        use walkdir::WalkDir;

        Box::new(
            WalkDir::new(&self.project_root)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|entry| entry.file_type().is_file())
                .filter(|entry| {
                    let path_str = entry.path().to_string_lossy();
                    !crate::utils::should_ignore_file(path_str.as_ref(), &self.ignore_patterns)
                })
                .filter(|entry| !Self::is_excluded_source_artifact(entry.path()))
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
                }),
        )
    }

    pub fn analyze_summary(&self) -> FastContextResult<CoreAnalysisSummary> {
        let internal_result = self.analyze_internal()?;

        Ok(CoreAnalysisSummary {
            file_count: internal_result.file_count,
            symbol_count: internal_result.symbol_count,
            languages: internal_result.languages,
            duration_ms: internal_result.duration_ms,
            relationships: internal_result.relationships,
            skipped_files: internal_result.skipped_files,
        })
    }

    #[cfg(feature = "python")]
    pub fn analyze(&self) -> FastContextResult<crate::python_bindings::AnalysisResult> {
        let summary = self.analyze_summary()?;

        Ok(crate::python_bindings::AnalysisResult {
            file_count: summary.file_count,
            symbol_count: summary.symbol_count,
            languages: summary.languages,
            duration_ms: summary.duration_ms,
            relationships: summary
                .relationships
                .into_iter()
                .map(crate::python_bindings::PyDependency::from)
                .collect(),
            skipped_files: summary
                .skipped_files
                .into_iter()
                .map(crate::python_bindings::PySkippedFile::from)
                .collect(),
        })
    }

    #[cfg(not(feature = "python"))]
    pub fn analyze(&self) -> FastContextResult<CoreAnalysisSummary> {
        self.analyze_summary()
    }

    pub fn find_symbols_by_kind(&self, symbol_kind: String) -> FastContextResult<Vec<String>> {
        use crate::symbols::SymbolExtractorFactory;
        use crate::symbols::SymbolKind;

        self.ensure_valid_project_root()?;

        let extractor_factory = SymbolExtractorFactory::new();
        let mut results = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    let syms = extractor_factory.extract_symbols(
                        &parse.tree,
                        &parse.source,
                        path_str.as_ref(),
                        parse.language,
                    );
                    let filtered = syms
                        .into_iter()
                        .filter(|s| {
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
                        })
                        .map(|s| s.name)
                        .collect::<Vec<_>>();
                    results.extend(filtered);
                }
            }
        }
        Ok(results)
    }

    pub fn find_symbols_in_file(&self, file_path: String) -> FastContextResult<Vec<String>> {
        use crate::symbols::{SymbolExtractorFactory, SymbolKind};

        self.ensure_valid_project_root()?;

        // Use secure file reading with path validation
        let content = crate::validation::secure_read_file(std::path::Path::new(&file_path))
            .map_err(|e| format!("Invalid file path '{}': {}", file_path, e))?;
        let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
        if let Some(parse) = scoped_factory.parse_file(&content, &file_path) {
            let extractor_factory = SymbolExtractorFactory::new();
            let syms = extractor_factory.extract_symbols(
                &parse.tree,
                &parse.source,
                &file_path,
                parse.language,
            );
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
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use crate::symbols::SymbolExtractorFactory;

        self.ensure_valid_project_root()?;

        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();
        let mut results: Vec<String> = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    let symbols = extractor_factory.extract_symbols(
                        &parse.tree,
                        &parse.source,
                        path_str.as_ref(),
                        parse.language,
                    );
                    let deps = dep_factory.extract_dependencies(
                        &parse.tree,
                        &parse.source,
                        symbols,
                        path_str.as_ref(),
                        parse.language,
                    );
                    for d in deps {
                        if d.to_symbol.contains(&symbol_name)
                            || d.from_symbol.contains(&symbol_name)
                        {
                            results.push(format!(
                                "{} -> {} ({:?}) @ {}:{}",
                                d.from_symbol,
                                d.to_symbol,
                                d.relationship_type,
                                d.file_path,
                                d.location.start_line
                            ));
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

        self.ensure_valid_project_root()?;

        let extractor_factory = SymbolExtractorFactory::new();
        let mut complex: Vec<String> = Vec::new();

        for entry in self.walk_project_files() {
            let path_str = entry.path().to_string_lossy();
            if let Ok(content) = crate::validation::secure_read_file(entry.path()) {
                let mut scoped_factory = crate::parsers::ScopedParserFactory::new();
                if let Some(parse) = scoped_factory.parse_file(&content, path_str.as_ref()) {
                    // naive complexity: number of control-flow tokens in file + function count
                    let syms = extractor_factory.extract_symbols(
                        &parse.tree,
                        &parse.source,
                        path_str.as_ref(),
                        parse.language,
                    );
                    let func_count = syms
                        .iter()
                        .filter(|s| matches!(s.kind, SymbolKind::Function | SymbolKind::Method))
                        .count() as u32;
                    let cf_tokens = [
                        "if ", "else if ", "for ", "while ", "match ", "switch ", "catch ",
                        "except ",
                    ];
                    let mut score = func_count;
                    for tok in cf_tokens.iter() {
                        score += content.matches(tok).count() as u32;
                    }
                    if score >= threshold {
                        complex.push(format!("{path_str} (complexity: {score})"));
                    }
                }
            }
        }
        Ok(complex)
    }
}
