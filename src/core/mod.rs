// Shared Send + Sync core analyzer used by both Python and Node bindings

use crate::errors::FastContextResult;

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

impl CoreAnalyzer {
    pub fn new(project_root: String, languages: Option<Vec<String>>, ignore_patterns: Option<Vec<String>>) -> Self {
        Self {
            project_root,
            languages: languages.unwrap_or_else(|| vec![
                "rust".to_string(), "javascript".to_string(), "typescript".to_string(), "python".to_string()
            ]),
            ignore_patterns: ignore_patterns.unwrap_or_else(|| vec![
                "node_modules/**".to_string(), "target/**".to_string(), ".git/**".to_string()
            ]),
        }
    }

    #[cfg(feature = "python")]
    pub fn analyze(&self) -> FastContextResult<crate::python_bindings::AnalysisResult> {
        use crate::parsers::ParserFactory;
        use crate::symbols::{SymbolExtractorFactory, Symbol};
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;

        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();
        let mut file_count: u32 = 0;
        let mut symbol_count: u32 = 0;
        let mut languages: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut relationships: Vec<crate::symbols::Dependency> = Vec::new();

        'outer: for entry in WalkDir::new(&self.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().to_string();
                if crate::utils::should_ignore_file(&path_str, &self.ignore_patterns) {
                    continue;
                }
                if !self.languages.is_empty() {
                    if let Some(ext) = std::path::Path::new(&path_str).extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            let allow = self.languages.iter().any(|s| crate::parsers::LanguageId::from_string(s).map(|l| l == lang).unwrap_or(false));
                            if !allow { continue 'outer; }
                        }
                    }
                }
                if let Ok(content) = fs::read_to_string(&path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        let syms: Vec<Symbol> = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                        let deps = dep_factory.extract_dependencies(&parse.tree, &parse.source, syms.clone(), &path_str, parse.language);
                        symbol_count += syms.len() as u32;
                        relationships.extend(deps);
                        languages.insert(format!("{:?}", parse.language));
                        file_count += 1;
                    }
                }
            }
        }

        Ok(crate::python_bindings::AnalysisResult {
            file_count,
            symbol_count,
            languages: languages.into_iter().collect(),
            duration_ms: 0,
            relationships: relationships.into_iter().map(crate::python_bindings::PyDependency::from).collect(),
        })
    }

    #[cfg(not(feature = "python"))]
    pub fn analyze(&self) -> FastContextResult<CoreAnalysisSummary> {
        use crate::parsers::ParserFactory;
        use crate::symbols::{SymbolExtractorFactory, Symbol};
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;

        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();
        let mut file_count: u32 = 0;
        let mut symbol_count: u32 = 0;
        let mut languages: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut relationships: Vec<crate::symbols::Dependency> = Vec::new();

        'outer: for entry in WalkDir::new(&self.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().to_string();
                if crate::utils::should_ignore_file(&path_str, &self.ignore_patterns) {
                    continue;
                }
                // Filter by selected languages if provided
                if !self.languages.is_empty() {
                    if let Some(ext) = std::path::Path::new(&path_str).extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            let allow = self.languages.iter().any(|s| crate::parsers::LanguageId::from_string(s).map(|l| l == lang).unwrap_or(false));
                            if !allow { continue 'outer; }
                        }
                    }
                }
                if let Ok(content) = fs::read_to_string(&path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        let syms: Vec<Symbol> = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                        let deps = dep_factory.extract_dependencies(&parse.tree, &parse.source, syms.clone(), &path_str, parse.language);
                        symbol_count += syms.len() as u32;
                        relationships.extend(deps);
                        languages.insert(format!("{:?}", parse.language));
                        file_count += 1;
                    }
                }
            }
        }

        Ok(CoreAnalysisSummary {
            file_count,
            symbol_count,
            languages: languages.into_iter().collect(),
            duration_ms: 0,
            relationships,
        })
    }

    pub fn find_symbols_by_kind(&self, symbol_kind: String) -> FastContextResult<Vec<String>> {
        use crate::parsers::ParserFactory;
        use crate::symbols::SymbolExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;

        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let mut results = Vec::new();

'outer2: for entry in WalkDir::new(&self.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().to_string();
                if crate::utils::should_ignore_file(&path_str, &self.ignore_patterns) {
                    continue;
                }
                // Filter by selected languages
                if !self.languages.is_empty() {
                    if let Some(ext) = std::path::Path::new(&path_str).extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            let allow = self.languages.iter().any(|s| crate::parsers::LanguageId::from_string(s).map(|l| l == lang).unwrap_or(false));
                            if !allow { continue 'outer2; }
                        }
                    }
                }
                if let Ok(content) = fs::read_to_string(&path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        use crate::symbols::SymbolKind;
                        let syms = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
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
        }
        Ok(results)
    }

    pub fn find_symbols_in_file(&self, file_path: String) -> FastContextResult<Vec<String>> {
        use crate::parsers::ParserFactory;
        use crate::symbols::{SymbolExtractorFactory, SymbolKind};
        use std::fs;

        let content = fs::read_to_string(&file_path)?;
        let mut parser_factory = ParserFactory::new();
        if let Some(parse) = parser_factory.parse_file(&content, &file_path) {
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
        use crate::parsers::ParserFactory;
        use crate::symbols::{SymbolExtractorFactory};
        use crate::symbols::dependency_extractor::DependencyExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;

        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let dep_factory = DependencyExtractorFactory::new();
        let mut results: Vec<String> = Vec::new();

'outer3: for entry in WalkDir::new(&self.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().to_string();
                if crate::utils::should_ignore_file(&path_str, &self.ignore_patterns) {
                    continue;
                }
                // Filter by selected languages
                if !self.languages.is_empty() {
                    if let Some(ext) = std::path::Path::new(&path_str).extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            let allow = self.languages.iter().any(|s| crate::parsers::LanguageId::from_string(s).map(|l| l == lang).unwrap_or(false));
                            if !allow { continue 'outer3; }
                        }
                    }
                }
                if let Ok(content) = fs::read_to_string(&path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        let symbols = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                        let deps = dep_factory.extract_dependencies(&parse.tree, &parse.source, symbols, &path_str, parse.language);
                        for d in deps {
                            if d.to_symbol.contains(&symbol_name) || d.from_symbol.contains(&symbol_name) {
                                results.push(format!("{} -> {} ({:?}) @ {}:{}", d.from_symbol, d.to_symbol, d.relationship_type, d.file_path, d.location.start_line));
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn find_complex_symbols(&self, threshold: u32) -> FastContextResult<Vec<String>> {
        use crate::parsers::ParserFactory;
        use crate::symbols::SymbolExtractorFactory;
        use walkdir::WalkDir;
        use std::fs;

        let mut parser_factory = ParserFactory::new();
        let extractor_factory = SymbolExtractorFactory::new();
        let mut complex: Vec<String> = Vec::new();

'outer4: for entry in WalkDir::new(&self.project_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path_str = entry.path().to_string_lossy().to_string();
                if crate::utils::should_ignore_file(&path_str, &self.ignore_patterns) { continue; }
                // Filter by selected languages
                if !self.languages.is_empty() {
                    if let Some(ext) = std::path::Path::new(&path_str).extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = crate::parsers::LanguageId::from_extension(ext) {
                            let allow = self.languages.iter().any(|s| crate::parsers::LanguageId::from_string(s).map(|l| l == lang).unwrap_or(false));
                            if !allow { continue 'outer4; }
                        }
                    }
                }
                if let Ok(content) = fs::read_to_string(&path_str) {
                    if let Some(parse) = parser_factory.parse_file(&content, &path_str) {
                        // naive complexity: number of control-flow tokens in file + function count
                        let syms = extractor_factory.extract_symbols(&parse.tree, &parse.source, &path_str, parse.language);
                        let func_count = syms.iter().filter(|s| matches!(s.kind, crate::symbols::SymbolKind::Function | crate::symbols::SymbolKind::Method)).count() as u32;
                        let cf_tokens = ["if ", "else if ", "for ", "while ", "match ", "switch ", "catch ", "except "];
                        let mut score = func_count;
                        for tok in cf_tokens.iter() { score += content.matches(tok).count() as u32; }
                        if score >= threshold {
                            complex.push(format!("{path_str} (complexity: {score})"));
                        }
                    }
                }
            }
        }
        Ok(complex)
    }
}

