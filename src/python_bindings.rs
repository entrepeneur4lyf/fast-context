//! # Python Bindings for Fast-Context - Phase 1: Simple Functions
//!
//! This module provides Python bindings using PyO3 with a simple, stateless API
//! that works immediately without complex thread safety requirements.

#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use std::fs;

#[cfg(feature = "python")]
use crate::core::{CoreAnalyzer, CoreAnalyzerOptions};
#[cfg(feature = "python")]
use crate::python_bindings_cache::{
    PyAnalysisCache, PyCacheEntry, PyCacheHealthMetrics, PyCacheStatistics, PyMultiLevelCache,
};
#[cfg(feature = "python")]
use crate::python_bindings_config::{
    PyAdvancedAnalyzerConfig, PyCachePolicy, PyConfigProfileManager, PyPerformanceConfig,
};
#[cfg(feature = "python")]
use crate::python_bindings_graph::{
    CentralityResult, ConnectedComponent, PathResult, PyRustworkxDiGraph, PyRustworkxGraph,
};
#[cfg(feature = "python")]
use crate::python_bindings_util::extensions_for_language_str;
#[cfg(feature = "python")]
use pyo3_async_runtimes::tokio::future_into_py;
#[cfg(feature = "python")]
use std::collections::HashMap;
#[cfg(feature = "python")]
use std::sync::atomic::AtomicBool;
#[cfg(feature = "python")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "python")]
use walkdir::WalkDir;

#[cfg(feature = "python")]

/// Simple analysis result for Python
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisResult {
    #[pyo3(get)]
    pub file_count: u32,

    #[pyo3(get)]
    pub symbol_count: u32,

    #[pyo3(get)]
    pub languages: Vec<String>,

    #[pyo3(get)]
    pub duration_ms: u32,

    #[pyo3(get)]
    pub relationships: Vec<PyDependency>,

    #[pyo3(get)]
    pub skipped_files: Vec<PySkippedFile>,
}

/// Enhanced analysis result with full symbol information
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct EnhancedAnalysisResult {
    #[pyo3(get)]
    pub file_count: u32,

    #[pyo3(get)]
    pub symbol_count: u32,

    #[pyo3(get)]
    pub languages: Vec<String>,

    #[pyo3(get)]
    pub duration_ms: u32,

    #[pyo3(get)]
    pub relationships: Vec<PyDependency>,

    #[pyo3(get)]
    pub skipped_files: Vec<PySkippedFile>,

    #[pyo3(get)]
    pub symbols: Vec<PySymbol>,

    #[pyo3(get)]
    pub files_analyzed: Vec<String>,
}

/// Python wrapper for Symbol Location
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PyLocation {
    #[pyo3(get)]
    pub file_path: String,

    #[pyo3(get)]
    pub start_line: usize,

    #[pyo3(get)]
    pub start_column: usize,

    #[pyo3(get)]
    pub end_line: usize,

    #[pyo3(get)]
    pub end_column: usize,
}

/// Python wrapper for Symbol Scope
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PyScope {
    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub kind: String,

    #[pyo3(get)]
    pub location: PyLocation,
}

/// Python wrapper for Symbol with full Tree-sitter integration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PySymbol {
    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub kind: String,

    #[pyo3(get)]
    pub location: PyLocation,

    #[pyo3(get)]
    pub scope_chain: Vec<PyScope>,

    #[pyo3(get)]
    pub language: String,

    #[pyo3(get)]
    pub documentation: Option<String>,

    #[pyo3(get)]
    pub modifiers: Vec<String>,

    #[pyo3(get)]
    pub signature: Option<String>,
}

/// Python wrapper for Dependency
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PyDependency {
    #[pyo3(get)]
    pub from_symbol: String,

    #[pyo3(get)]
    pub to_symbol: String,

    #[pyo3(get)]
    pub relationship_type: String,

    #[pyo3(get)]
    pub location: PyLocation,

    #[pyo3(get)]
    pub file_path: String,

    #[pyo3(get)]
    pub language: String,

    #[pyo3(get)]
    pub context: Option<String>,

    #[pyo3(get)]
    pub strength: f32,

    #[pyo3(get)]
    pub is_conditional: bool,
}

/// Python wrapper for skipped file diagnostics
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PySkippedFile {
    #[pyo3(get)]
    pub file_path: String,

    #[pyo3(get)]
    pub stage: String,

    #[pyo3(get)]
    pub reason: String,
}

/// Enhanced analysis result for single file analysis
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct PyEnhancedAnalysisResult {
    #[pyo3(get)]
    pub file_path: String,

    #[pyo3(get)]
    pub symbols: Vec<PySymbol>,

    #[pyo3(get)]
    pub dependencies: Vec<PyDependency>,

    #[pyo3(get)]
    pub language: String,

    #[pyo3(get)]
    pub line_count: u32,

    #[pyo3(get)]
    pub analysis_duration_ms: u32,
}

#[cfg(feature = "python")]
impl From<crate::symbols::Location> for PyLocation {
    fn from(loc: crate::symbols::Location) -> Self {
        Self {
            file_path: loc.file_path,
            start_line: loc.start_line,
            start_column: loc.start_column,
            end_line: loc.end_line,
            end_column: loc.end_column,
        }
    }
}

#[cfg(feature = "python")]
impl From<crate::symbols::Scope> for PyScope {
    fn from(scope: crate::symbols::Scope) -> Self {
        Self {
            name: scope.name,
            kind: format!("{:?}", scope.kind),
            location: scope.location.into(),
        }
    }
}

#[cfg(feature = "python")]
impl From<crate::symbols::Symbol> for PySymbol {
    fn from(symbol: crate::symbols::Symbol) -> Self {
        Self {
            name: symbol.name,
            kind: format!("{:?}", symbol.kind),
            location: symbol.location.into(),
            scope_chain: symbol.scope_chain.into_iter().map(|s| s.into()).collect(),
            language: format!("{:?}", symbol.language),
            documentation: symbol.documentation,
            modifiers: symbol.modifiers,
            signature: symbol.signature,
        }
    }
}

#[cfg(feature = "python")]
impl From<crate::symbols::Dependency> for PyDependency {
    fn from(dep: crate::symbols::Dependency) -> Self {
        Self {
            from_symbol: dep.from_symbol,
            to_symbol: dep.to_symbol,
            relationship_type: format!("{:?}", dep.relationship_type),
            location: dep.location.into(),
            file_path: dep.file_path,
            language: format!("{:?}", dep.language),
            context: dep.context,
            strength: dep.strength,
            is_conditional: dep.is_conditional,
        }
    }
}

#[cfg(feature = "python")]
impl From<crate::core::SkippedFileDiagnostic> for PySkippedFile {
    fn from(diagnostic: crate::core::SkippedFileDiagnostic) -> Self {
        Self {
            file_path: diagnostic.file_path,
            stage: diagnostic.stage,
            reason: diagnostic.reason,
        }
    }
}

#[cfg(feature = "python")]
fn placeholder_symbol(
    name: String,
    file_path: String,
    language: String,
    kind: &str,
    documentation: Option<String>,
    signature: Option<String>,
) -> PySymbol {
    PySymbol {
        name,
        kind: kind.to_string(),
        location: PyLocation {
            file_path,
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
        },
        scope_chain: Vec::new(),
        language,
        documentation,
        modifiers: Vec::new(),
        signature,
    }
}

/// Phase 2: Thread-safe class-based analyzer for Python
#[cfg(feature = "python")]
#[pyclass]
pub struct FastContextAnalyzer {
    project_root: String,
    languages: Vec<String>,
    ignore_patterns: Vec<String>,
    max_files: i32,
    parallel_processing: bool,
    #[allow(dead_code)]
    enable_watching: bool,
    watcher: Option<Arc<Mutex<crate::watcher::CodebaseWatcher>>>,
    // Shared core (Send + Sync scaffold)
    core: Arc<CoreAnalyzer>,
    // Caching/state
    last_analysis: Arc<Mutex<Option<AnalysisResult>>>,
    symbol_cache: Arc<Mutex<HashMap<String, Vec<String>>>>,

    dirty: Arc<AtomicBool>,
}

#[cfg(feature = "python")]
impl FastContextAnalyzer {
    #[allow(dead_code)]
    fn clone_for_async(&self) -> Self {
        Self {
            project_root: self.project_root.clone(),
            languages: self.languages.clone(),
            ignore_patterns: self.ignore_patterns.clone(),
            max_files: self.max_files,
            parallel_processing: self.parallel_processing,
            enable_watching: self.enable_watching,
            watcher: self.watcher.as_ref().map(|w| w.clone()),
            core: self.core.clone(),
            last_analysis: self.last_analysis.clone(),
            symbol_cache: self.symbol_cache.clone(),
            dirty: self.dirty.clone(),
        }
    }

    fn analyzer_options(config: &AnalyzerConfig) -> CoreAnalyzerOptions {
        CoreAnalyzerOptions {
            max_files: usize::try_from(config.max_files)
                .ok()
                .filter(|max_files| *max_files > 0),
            parallel_processing: config.parallel_processing,
        }
    }
}

#[cfg(feature = "python")]
#[allow(non_local_definitions)]
#[pymethods]
impl FastContextAnalyzer {
    /// Create a new thread-safe FastContextAnalyzer from configuration
    #[new]
    pub fn new(config: AnalyzerConfig) -> PyResult<Self> {
        if !std::path::Path::new(&config.project_root).exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                format!("Project root does not exist: {}", config.project_root),
            ));
        }

        let analyzer_options = Self::analyzer_options(&config);

        Ok(Self {
            project_root: config.project_root.clone(),
            languages: config.languages.clone(),
            ignore_patterns: config.ignore_patterns.clone(),
            max_files: config.max_files,
            parallel_processing: config.parallel_processing,
            enable_watching: config.enable_watching,
            watcher: None,
            core: Arc::new(CoreAnalyzer::with_options(
                config.project_root,
                Some(config.languages),
                Some(config.ignore_patterns),
                analyzer_options,
            )),
            last_analysis: Arc::new(Mutex::new(None)),
            symbol_cache: Arc::new(Mutex::new(HashMap::new())),
            dirty: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Get the current configuration as an AnalyzerConfig object
    pub fn get_config(&self) -> AnalyzerConfig {
        AnalyzerConfig {
            project_root: self.project_root.clone(),
            languages: self.languages.clone(),
            ignore_patterns: self.ignore_patterns.clone(),
            enable_caching: true, // Default for existing analyzer
            enable_watching: self.enable_watching,
            max_files: self.max_files,
            parallel_processing: self.parallel_processing,
        }
    }

    /// Analyze the codebase asynchronously (releases GIL) and update cache/state
    pub fn analyze_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core.clone();
        let last_analysis = self.last_analysis.clone();
        let dirty = self.dirty.clone();
        let awaitable = future_into_py(py, async move {
            // Use a blocking section for CPU/IO heavy work to avoid starving Tokio
            let res = tokio::task::spawn_blocking(move || core.analyze())
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    // Update cache/state
                    if let Ok(mut guard) = last_analysis.lock() {
                        *guard = Some(val.clone());
                    }
                    dirty.store(false, std::sync::atomic::Ordering::SeqCst);
                    Ok(val)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Start file watching (marks caches dirty on changes)
    pub fn start_watching(&mut self) -> PyResult<()> {
        if self.watcher.is_some() {
            return Ok(());
        }
        let mut watched_extensions: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for lang in &self.languages {
            if let Some(exts) = extensions_for_language_str(lang) {
                for e in exts {
                    watched_extensions.insert(e.to_string());
                }
            }
        }
        let config = crate::watcher::WatcherConfig {
            watch_dirs: vec![std::path::PathBuf::from(&self.project_root)],
            watched_extensions,
            ignore_patterns: self.ignore_patterns.clone(),
            debounce_duration: std::time::Duration::from_millis(500),
            batch_size: 200,
        };
        let watcher = crate::watcher::CodebaseWatcher::new(config).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to start watcher: {e}"
            ))
        })?;
        let dirty = self.dirty.clone();
        let symbol_cache = self.symbol_cache.clone();
        let last_analysis = self.last_analysis.clone();
        // spawn a thread listening for FS events to invalidate caches
        let mut rx = watcher.subscribe();
        std::thread::spawn(move || {
            while let Ok(_batch) = rx.blocking_recv() {
                dirty.store(true, std::sync::atomic::Ordering::SeqCst);
                if let Ok(mut sc) = symbol_cache.lock() {
                    sc.clear();
                }
                if let Ok(mut la) = last_analysis.lock() {
                    *la = None;
                }
            }
        });
        self.watcher = Some(std::sync::Arc::new(std::sync::Mutex::new(watcher)));
        Ok(())
    }

    /// Stop file watching
    pub fn stop_watching(&mut self) -> PyResult<()> {
        self.watcher = None; // drop watcher
                             // clear caches and mark dirty to force fresh analysis next call
        self.dirty.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut sc) = self.symbol_cache.lock() {
            sc.clear();
        }
        if let Ok(mut la) = self.last_analysis.lock() {
            *la = None;
        }
        Ok(())
    }

    /// Re-analyze only if caches are dirty (returns True if ran)
    pub fn reanalyze_if_dirty_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dirty = self.dirty.clone();
        let core = self.core.clone();
        let last_analysis = self.last_analysis.clone();
        let awaitable = future_into_py(py, async move {
            if !dirty.load(std::sync::atomic::Ordering::SeqCst) {
                return Ok(false);
            }
            let res = tokio::task::spawn_blocking(move || core.analyze())
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    if let Ok(mut guard) = last_analysis.lock() {
                        *guard = Some(val);
                    }
                    dirty.store(false, std::sync::atomic::Ordering::SeqCst);
                    Ok(true)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Find symbols by kind in the codebase (async)
    pub fn find_symbols_by_kind_async<'py>(
        &self,
        py: Python<'py>,
        symbol_kind: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core.clone();
        let cache_key = format!("kind:{}:{}", symbol_kind, self.languages.join(","));
        let symbol_cache = self.symbol_cache.clone();
        let dirty = self.dirty.clone();
        let awaitable = future_into_py(py, async move {
            if !dirty.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(sc) = symbol_cache.lock() {
                    if let Some(cached) = sc.get(&cache_key) {
                        return Ok(cached.clone());
                    }
                }
            }
            let res = tokio::task::spawn_blocking(move || core.find_symbols_by_kind(symbol_kind))
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    if let Ok(mut sc) = symbol_cache.lock() {
                        sc.insert(cache_key, val.clone());
                    }
                    Ok(val)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Find symbols in a specific file relative to project root (async, with per-file cache)
    pub fn find_symbols_in_file_async<'py>(
        &self,
        py: Python<'py>,
        file_path: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let full_path = if std::path::Path::new(&file_path).is_absolute() {
            file_path
        } else {
            format!("{}/{}", self.project_root, file_path)
        };
        let key = format!("file:{}", full_path);
        let symbol_cache = self.symbol_cache.clone();
        let dirty = self.dirty.clone();
        let core = self.core.clone();
        let awaitable = future_into_py(py, async move {
            if !dirty.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(sc) = symbol_cache.lock() {
                    if let Some(cached) = sc.get(&key) {
                        return Ok(cached.clone());
                    }
                }
            }
            let res = tokio::task::spawn_blocking(move || core.find_symbols_in_file(full_path))
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    if let Ok(mut sc) = symbol_cache.lock() {
                        sc.insert(key, val.clone());
                    }
                    Ok(val)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Find dependencies of a symbol across the project (async, simple cache)
    pub fn find_dependencies_async<'py>(
        &self,
        py: Python<'py>,
        symbol_name: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let key = format!("deps:{}", symbol_name);
        let symbol_cache = self.symbol_cache.clone();
        let dirty = self.dirty.clone();
        let core = self.core.clone();
        let awaitable = future_into_py(py, async move {
            if !dirty.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(sc) = symbol_cache.lock() {
                    if let Some(cached) = sc.get(&key) {
                        return Ok(cached.clone());
                    }
                }
            }
            let res = tokio::task::spawn_blocking(move || core.find_dependencies(symbol_name))
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    if let Ok(mut sc) = symbol_cache.lock() {
                        sc.insert(key, val.clone());
                    }
                    Ok(val)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Find complex symbols over threshold (async, simple cache)
    pub fn find_complex_symbols_async<'py>(
        &self,
        py: Python<'py>,
        threshold: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let key = format!("complex:{}", threshold);
        let symbol_cache = self.symbol_cache.clone();
        let dirty = self.dirty.clone();
        let core = self.core.clone();
        let awaitable = future_into_py(py, async move {
            if !dirty.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(sc) = symbol_cache.lock() {
                    if let Some(cached) = sc.get(&key) {
                        return Ok(cached.clone());
                    }
                }
            }
            let res = tokio::task::spawn_blocking(move || core.find_complex_symbols(threshold))
                .await
                .map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Failed to join background analysis task: {e}"
                    ))
                })?;
            match res {
                Ok(val) => {
                    if let Ok(mut sc) = symbol_cache.lock() {
                        sc.insert(key, val.clone());
                    }
                    Ok(val)
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })?;
        Ok(awaitable)
    }

    /// Extract symbols from a specific file with full metadata
    pub fn extract_symbols_from_file(&self, file_path: &str) -> PyResult<PyEnhancedAnalysisResult> {
        // Use existing methods to extract symbols with full metadata
        match self.core.find_symbols_in_file(file_path.to_string()) {
            Ok(symbol_names) => {
                // Create basic symbols from names (in real implementation, this would use full symbol extraction)
                let symbols: Vec<PySymbol> = symbol_names
                    .into_iter()
                    .map(|name| PySymbol {
                        name: name.clone(),
                        kind: "Unknown".to_string(),
                        location: PyLocation {
                            file_path: file_path.to_string(),
                            start_line: 0,
                            start_column: 0,
                            end_line: 0,
                            end_column: 0,
                        },
                        scope_chain: Vec::new(),
                        language: "Unknown".to_string(),
                        documentation: None,
                        modifiers: Vec::new(),
                        signature: None,
                    })
                    .collect();

                Ok(PyEnhancedAnalysisResult {
                    file_path: file_path.to_string(),
                    symbols,
                    dependencies: Vec::new(),
                    language: "Unknown".to_string(),
                    line_count: 0,
                    analysis_duration_ms: 0,
                })
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to analyze file: {}",
                e
            ))),
        }
    }

    /// Get symbol relationships and call graph  
    pub fn get_symbol_relationships(&self, symbol_name: &str) -> PyResult<Vec<PyDependency>> {
        // Use existing find_dependencies method
        match self.core.find_dependencies(symbol_name.to_string()) {
            Ok(dependencies) => {
                // Convert dependency strings to PyDependency objects
                let py_deps: Vec<PyDependency> = dependencies
                    .into_iter()
                    .map(|dep| PyDependency {
                        from_symbol: symbol_name.to_string(),
                        to_symbol: dep,
                        relationship_type: "Calls".to_string(),
                        location: PyLocation {
                            file_path: "unknown".to_string(),
                            start_line: 0,
                            start_column: 0,
                            end_line: 0,
                            end_column: 0,
                        },
                        file_path: "unknown".to_string(),
                        language: "Unknown".to_string(),
                        context: None,
                        strength: 1.0,
                        is_conditional: false,
                    })
                    .collect();
                Ok(py_deps)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to get symbol relationships: {}",
                e
            ))),
        }
    }

    /// Search for symbols by name pattern across all files
    pub fn search_symbols(
        &self,
        _pattern: &str,
        _language: Option<String>,
    ) -> PyResult<Vec<PySymbol>> {
        // Search across the known relationship graph, which is the stable data
        // currently exposed by the Python analysis result.
        match self.core.analyze() {
            Ok(result) => {
                let pattern_lower = _pattern.to_lowercase();
                let language_filter = _language.as_ref().map(|lang| lang.to_lowercase());
                let mut seen = std::collections::HashSet::new();
                let mut filtered_symbols = Vec::new();

                for rel in result.relationships {
                    let rel_language = rel.language.to_lowercase();
                    if let Some(ref lang_filter) = language_filter {
                        if !rel_language.contains(lang_filter) {
                            continue;
                        }
                    }

                    for symbol_name in [&rel.from_symbol, &rel.to_symbol] {
                        if symbol_name.to_lowercase().contains(&pattern_lower) {
                            let key = format!("{}::{}", rel.file_path, symbol_name);
                            if seen.insert(key) {
                                filtered_symbols.push(placeholder_symbol(
                                    symbol_name.clone(),
                                    rel.file_path.clone(),
                                    rel.language.clone(),
                                    "Unknown",
                                    rel.context.clone(),
                                    None,
                                ));
                            }
                        }
                    }
                }

                Ok(filtered_symbols)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to search symbols: {}",
                e
            ))),
        }
    }

    /// Get symbol documentation and metadata
    pub fn get_symbol_documentation(
        &self,
        _symbol_name: &str,
        _file_path: &str,
    ) -> PyResult<Option<String>> {
        // Use relationship metadata as a lightweight documentation source until
        // the richer Python symbol model is reconciled with the Rust core.
        match self.core.analyze() {
            Ok(result) => {
                for rel in result.relationships {
                    let symbol_matches =
                        rel.from_symbol == _symbol_name || rel.to_symbol == _symbol_name;
                    let file_matches =
                        rel.file_path == _file_path || rel.location.file_path == _file_path;
                    if symbol_matches && file_matches {
                        if let Some(context) = rel.context.clone() {
                            return Ok(Some(context));
                        }

                        return Ok(Some(format!(
                            "**{}**\n\nRelationship: {} -> {} ({})\nLocation: {}:{}",
                            _symbol_name,
                            rel.from_symbol,
                            rel.to_symbol,
                            rel.relationship_type,
                            rel.file_path,
                            rel.location.start_line
                        )));
                    }
                }
                Ok(None)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to get symbol documentation: {}",
                e
            ))),
        }
    }

    /// Analyze cross-language dependencies
    pub fn analyze_cross_language_dependencies(&self) -> PyResult<Vec<PyDependency>> {
        // Cross-language detection is based on the relationship language that is
        // already available from the Python analysis result.
        match self.core.analyze() {
            Ok(result) => {
                let mut cross_lang_deps: Vec<PyDependency> = Vec::new();

                for dep in result.relationships {
                    let context = dep.context.clone().unwrap_or_default().to_lowercase();
                    if context.contains("cross-language")
                        || context.contains("cross language")
                        || context.contains("interop")
                    {
                        cross_lang_deps.push(dep);
                    }
                }

                Ok(cross_lang_deps)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to analyze cross-language dependencies: {}",
                e
            ))),
        }
    }

    /// Export symbol relationships as a graph
    pub fn export_relationship_graph(&self, format: &str) -> PyResult<String> {
        match self.core.analyze() {
            Ok(result) => match format {
                "dot" | "graphviz" => {
                    let mut dot_output = String::from("digraph G {\n  node [shape=box];\n");
                    let mut seen_nodes = std::collections::HashSet::new();

                    for dep in &result.relationships {
                        for symbol_name in [&dep.from_symbol, &dep.to_symbol] {
                            if seen_nodes.insert(symbol_name.clone()) {
                                let escaped_name = symbol_name.replace("\"", "\\\"");
                                dot_output.push_str(&format!(
                                    "  \"{}\" [label=\"{}\"];\n",
                                    escaped_name, escaped_name
                                ));
                            }
                        }
                    }

                    for dep in &result.relationships {
                        let from_escaped = dep.from_symbol.replace("\"", "\\\"");
                        let to_escaped = dep.to_symbol.replace("\"", "\\\"");
                        dot_output.push_str(&format!(
                            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                            from_escaped, to_escaped, dep.relationship_type
                        ));
                    }

                    dot_output.push_str("}\n");
                    Ok(dot_output)
                }
                "json" => {
                    let mut nodes = Vec::new();
                    let mut edges = Vec::new();
                    let mut node_ids: std::collections::HashMap<String, usize> =
                        std::collections::HashMap::new();

                    for dep in &result.relationships {
                        for symbol_name in [&dep.from_symbol, &dep.to_symbol] {
                            if !node_ids.contains_key(symbol_name) {
                                let id = node_ids.len();
                                node_ids.insert(symbol_name.clone(), id);
                                nodes.push(serde_json::json!({
                                    "id": id,
                                    "name": symbol_name,
                                    "type": "Unknown",
                                    "language": dep.language,
                                    "file": dep.file_path
                                }));
                            }
                        }
                    }

                    for dep in &result.relationships {
                        if let (Some(src), Some(tgt)) =
                            (node_ids.get(&dep.from_symbol), node_ids.get(&dep.to_symbol))
                        {
                            edges.push(serde_json::json!({
                                "source": src,
                                "target": tgt,
                                "type": dep.relationship_type,
                                "strength": dep.strength
                            }));
                        }
                    }

                    let graph = serde_json::json!({"nodes": nodes, "edges": edges});
                    Ok(serde_json::to_string(&graph)
                        .unwrap_or_else(|_| "{\"nodes\": [], \"edges\": []}".to_string()))
                }
                "mermaid" => {
                    let mut mermaid_output = String::from("graph TD\n");
                    let mut seen_nodes = std::collections::HashSet::new();

                    for dep in &result.relationships {
                        for symbol_name in [&dep.from_symbol, &dep.to_symbol] {
                            if seen_nodes.insert(symbol_name.clone()) {
                                let safe_name = symbol_name.replace(" ", "_").replace("-", "_");
                                mermaid_output
                                    .push_str(&format!("  {}[\"{}\"]\n", safe_name, symbol_name));
                            }
                        }
                    }

                    for dep in &result.relationships {
                        let from_safe = dep.from_symbol.replace(" ", "_").replace("-", "_");
                        let to_safe = dep.to_symbol.replace(" ", "_").replace("-", "_");
                        mermaid_output.push_str(&format!(
                            "  {} -->|{}| {}\n",
                            from_safe, dep.relationship_type, to_safe
                        ));
                    }

                    Ok(mermaid_output)
                }
                _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unsupported export format: {}",
                    format
                ))),
            },
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Failed to export relationship graph: {}",
                e
            ))),
        }
    }

    // ========== EXPORT & INTELLIGENCE SYSTEM METHODS ==========

    /// Create export options for JSON export
    #[pyo3(signature = (_format, output_path=None, include_source=false, include_docs=false))]
    pub fn create_export_options(
        &self,
        _format: &str,
        output_path: Option<String>,
        include_source: bool,
        include_docs: bool,
    ) -> PyResult<crate::python_bindings_export::PyExportOptions> {
        Ok(crate::python_bindings_export::PyExportOptions::new(
            "json".to_string(),
            output_path,
            include_source,
            include_docs,
            false,
            true,
            false,
            "numpy".to_string(),
        ))
    }

    /// Create LSP export options
    pub fn create_lsp_export_options(
        &self,
    ) -> PyResult<crate::python_bindings_export::PyExportOptions> {
        Ok(crate::python_bindings_export::PyExportOptions::new(
            "lsp".to_string(),
            None,
            false,
            false,
            false,
            true,
            false,
            "numpy".to_string(),
        ))
    }

    /// Create embedding export options
    pub fn create_embedding_export_options(
        &self,
        embedding_format: String,
    ) -> PyResult<crate::python_bindings_export::PyExportOptions> {
        Ok(crate::python_bindings_export::PyExportOptions::new(
            "embeddings".to_string(),
            None,
            false,
            false,
            false,
            false,
            true,
            embedding_format,
        ))
    }

    /// Export analysis results to JSON format
    pub fn export_json(
        &self,
        _py: Python,
        analysis: &AnalysisResult,
        output_path: Option<String>,
    ) -> PyResult<String> {
        let options = self.create_export_options("json", output_path, true, false)?;
        let exporter = crate::python_bindings_export::PyJsonExporter::new(options);
        exporter.export_analysis(analysis)
    }

    /// Export analysis results to LSP format
    pub fn export_lsp(&self, _py: Python, analysis: &AnalysisResult) -> PyResult<String> {
        let options = self.create_lsp_export_options()?;
        let exporter = crate::python_bindings_export::PyLspExporter::new(options);
        exporter.export_workspace_symbols(analysis)
    }

    /// Export analysis results as embeddings
    pub fn export_embeddings(
        &self,
        _py: Python,
        analysis: &AnalysisResult,
        embedding_format: String,
    ) -> PyResult<String> {
        let options = self.create_embedding_export_options(embedding_format)?;
        let exporter = crate::python_bindings_export::PyEmbeddingExporter::new(options);
        exporter.export_embeddings(analysis)
    }

    /// Get query engine for advanced code intelligence
    pub fn get_query_engine(
        &self,
        _py: Python,
        analysis: &AnalysisResult,
    ) -> PyResult<crate::python_bindings_query::PyCodeQueryEngine> {
        Ok(crate::python_bindings_query::PyCodeQueryEngine::new(
            analysis.clone(),
        ))
    }

    /// Query symbols by pattern
    pub fn query_symbols_by_pattern(
        &self,
        py: Python,
        analysis: &AnalysisResult,
        pattern: String,
    ) -> PyResult<Vec<PySymbol>> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.find_symbols_by_pattern(pattern)
    }

    /// Find architectural patterns
    pub fn find_architectural_patterns(
        &self,
        py: Python,
        analysis: &AnalysisResult,
    ) -> PyResult<Vec<String>> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.find_architectural_patterns()
    }

    /// Get context for a specific symbol
    pub fn get_symbol_context(
        &self,
        py: Python,
        analysis: &AnalysisResult,
        symbol_name: String,
    ) -> PyResult<crate::python_bindings_query::PyContextInfo> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.get_context_for_symbol(symbol_name)
    }

    /// Detect code smells
    pub fn detect_code_smells(
        &self,
        py: Python,
        analysis: &AnalysisResult,
    ) -> PyResult<Vec<crate::python_bindings_query::PyCodeSmell>> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.detect_code_smells()
    }

    /// Find complex symbols
    pub fn find_complex_symbols(
        &self,
        py: Python,
        analysis: &AnalysisResult,
        threshold: f64,
    ) -> PyResult<Vec<PySymbol>> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.find_complex_symbols(threshold)
    }

    /// Analyze dependencies for a symbol
    pub fn analyze_symbol_dependencies(
        &self,
        py: Python,
        analysis: &AnalysisResult,
        symbol_name: String,
    ) -> PyResult<crate::python_bindings_query::PyDependencyAnalysis> {
        let query_engine = self.get_query_engine(py, analysis)?;
        query_engine.analyze_symbol_dependencies(symbol_name)
    }
}

/// Phase 1: Simple stateless analysis function
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (project_root, languages=None, ignore_patterns=None))]
pub fn analyze_project(
    project_root: String,
    languages: Option<Vec<String>>,
    ignore_patterns: Option<Vec<String>>,
) -> PyResult<AnalysisResult> {
    let start_time = std::time::Instant::now();

    let supported_languages = languages.unwrap_or_else(|| {
        vec![
            "rust".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "python".to_string(),
        ]
    });

    let ignore_patterns = crate::utils::merged_ignore_patterns(ignore_patterns);

    let mut file_count = 0;
    let mut symbol_count = 0;
    let mut detected_languages = std::collections::HashSet::new();

    // Walk through project files
    for entry in WalkDir::new(&project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().to_str() {
                // Skip ignored patterns
                if should_ignore_file(path_str, &ignore_patterns) {
                    continue;
                }

                if let Some(language) = crate::utils::detect_language_id(path_str) {
                    if supported_languages
                        .iter()
                        .any(|l| language.to_lowercase_string().contains(&l.to_lowercase()))
                    {
                        file_count += 1;
                        detected_languages.insert(language);

                        // Count symbols by reading file content
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            symbol_count +=
                                count_symbols_in_content(&content, &language.to_lowercase_string());
                        }
                    }
                }
            }
        }
    }

    let duration = start_time.elapsed();

    // Extract actual relationships using the CoreAnalyzer
    let core = CoreAnalyzer::new(
        project_root.clone(),
        Some(supported_languages.clone()),
        Some(ignore_patterns.clone()),
    );
    let (relationships, skipped_files) = match core.analyze() {
        Ok(analysis_result) => (analysis_result.relationships, analysis_result.skipped_files),
        Err(_) => {
            // If analysis fails, return empty relationships vector
            (Vec::new(), Vec::new())
        }
    };

    Ok(AnalysisResult {
        file_count,
        symbol_count,
        languages: detected_languages
            .into_iter()
            .map(|lang| lang.to_lowercase_string())
            .collect(),
        duration_ms: duration.as_millis() as u32,
        relationships,
        skipped_files,
    })
}

/// Find symbols by kind in a project
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (project_root, symbol_kind, languages=None))]
pub fn find_symbols_by_kind(
    project_root: String,
    symbol_kind: String,
    languages: Option<Vec<String>>,
) -> PyResult<Vec<String>> {
    let core = CoreAnalyzer::new(project_root, languages, None);
    core.find_symbols_by_kind(symbol_kind)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Find symbols in a specific file
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_symbols_in_file(file_path: String) -> PyResult<Vec<String>> {
    if !std::path::Path::new(&file_path).exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("File not found: {file_path}"),
        ));
    }
    let project_root = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    let core = CoreAnalyzer::new(project_root, None, None);
    core.find_symbols_in_file(file_path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Find dependencies of a symbol in a project
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_dependencies(project_root: String, symbol_name: String) -> PyResult<Vec<String>> {
    let core = CoreAnalyzer::new(project_root, None, None);
    core.find_dependencies(symbol_name)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Find complex symbols in a project
#[cfg(feature = "python")]
#[pyfunction]
pub fn find_complex_symbols(project_root: String, threshold: u32) -> PyResult<Vec<String>> {
    let core = CoreAnalyzer::new(project_root, None, None);
    core.find_complex_symbols(threshold)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Utility functions for Python
#[cfg(feature = "python")]
#[pyfunction]
pub fn get_supported_languages() -> Vec<String> {
    vec![
        "rust".to_string(),
        "javascript".to_string(),
        "typescript".to_string(),
        "python".to_string(),
        "java".to_string(),
        "go".to_string(),
        "cpp".to_string(),
        "c".to_string(),
        "csharp".to_string(),
    ]
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn detect_language(file_path: String) -> Option<String> {
    crate::utils::detect_language_id(&file_path).map(|lang| lang.to_lowercase_string())
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Python module definition
#[cfg(feature = "python")]
#[pymodule]
fn fast_context(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core data classes
    m.add_class::<AnalysisResult>()?;
    m.add_class::<PyDependency>()?;
    m.add_class::<PySkippedFile>()?;

    // Modern thread-safe class-based API
    m.add_class::<AnalyzerConfig>()?;
    m.add_class::<FastContextAnalyzer>()?;

    // Enhanced symbol extraction classes
    m.add_class::<PyLocation>()?;
    m.add_class::<PyScope>()?;
    m.add_class::<PySymbol>()?;
    m.add_class::<PyEnhancedAnalysisResult>()?;

    // Utility functions (kept for convenience)
    m.add_function(wrap_pyfunction!(analyze_project, m)?)?;
    m.add_function(wrap_pyfunction!(find_symbols_by_kind, m)?)?;
    m.add_function(wrap_pyfunction!(find_symbols_in_file, m)?)?;
    m.add_function(wrap_pyfunction!(find_dependencies, m)?)?;
    m.add_function(wrap_pyfunction!(find_complex_symbols, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_languages, m)?)?;
    m.add_function(wrap_pyfunction!(detect_language, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;

    // Add graph classes directly to main module
    if let Err(e) = m.add_class::<PyRustworkxGraph>() {
        eprintln!("Failed to add PyRustworkxGraph: {}", e);
    }
    if let Err(e) = m.add_class::<PyRustworkxDiGraph>() {
        eprintln!("Failed to add PyRustworkxDiGraph: {}", e);
    }
    if let Err(e) = m.add_class::<PathResult>() {
        eprintln!("Failed to add PathResult: {}", e);
    }
    if let Err(e) = m.add_class::<CentralityResult>() {
        eprintln!("Failed to add CentralityResult: {}", e);
    }
    if let Err(e) = m.add_class::<ConnectedComponent>() {
        eprintln!("Failed to add ConnectedComponent: {}", e);
    }

    // Export & Intelligence System classes
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyExportOptions>() {
        eprintln!("Failed to add PyExportOptions: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyJsonExporter>() {
        eprintln!("Failed to add PyJsonExporter: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyLspExporter>() {
        eprintln!("Failed to add PyLspExporter: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyEmbeddingExporter>() {
        eprintln!("Failed to add PyEmbeddingExporter: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyExportFactory>() {
        eprintln!("Failed to add PyExportFactory: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyLspLocation>() {
        eprintln!("Failed to add PyLspLocation: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyLspRange>() {
        eprintln!("Failed to add PyLspRange: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyLspPosition>() {
        eprintln!("Failed to add PyLspPosition: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_export::PyLspSymbolInformation>() {
        eprintln!("Failed to add PyLspSymbolInformation: {}", e);
    }

    // Query engine classes
    if let Err(e) = m.add_class::<crate::python_bindings_query::PyCodeQueryEngine>() {
        eprintln!("Failed to add PyCodeQueryEngine: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_query::PyContextInfo>() {
        eprintln!("Failed to add PyContextInfo: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_query::PyCodeSmell>() {
        eprintln!("Failed to add PyCodeSmell: {}", e);
    }
    if let Err(e) = m.add_class::<crate::python_bindings_query::PyDependencyAnalysis>() {
        eprintln!("Failed to add PyDependencyAnalysis: {}", e);
    }

    // Advanced Configuration System classes
    if let Err(e) = m.add_class::<PyCachePolicy>() {
        eprintln!("Failed to add PyCachePolicy: {}", e);
    }
    if let Err(e) = m.add_class::<PyPerformanceConfig>() {
        eprintln!("Failed to add PyPerformanceConfig: {}", e);
    }
    if let Err(e) = m.add_class::<PyAdvancedAnalyzerConfig>() {
        eprintln!("Failed to add PyAdvancedAnalyzerConfig: {}", e);
    }
    if let Err(e) = m.add_class::<PyConfigProfileManager>() {
        eprintln!("Failed to add PyConfigProfileManager: {}", e);
    }

    // Intelligent Caching System classes
    if let Err(e) = m.add_class::<PyCacheEntry>() {
        eprintln!("Failed to add PyCacheEntry: {}", e);
    }
    if let Err(e) = m.add_class::<PyCacheStatistics>() {
        eprintln!("Failed to add PyCacheStatistics: {}", e);
    }
    if let Err(e) = m.add_class::<PyMultiLevelCache>() {
        eprintln!("Failed to add PyMultiLevelCache: {}", e);
    }
    if let Err(e) = m.add_class::<PyAnalysisCache>() {
        eprintln!("Failed to add PyAnalysisCache: {}", e);
    }
    if let Err(e) = m.add_class::<PyCacheHealthMetrics>() {
        eprintln!("Failed to add PyCacheHealthMetrics: {}", e);
    }

    Ok(())
}

#[cfg(feature = "python")]
pub(crate) fn should_ignore_file(path: &str, ignore_patterns: &[String]) -> bool {
    for pattern in ignore_patterns {
        if pattern.ends_with('/') {
            if path.contains(pattern) {
                return true;
            }
        } else if let Some(ext) = pattern.strip_prefix("*.") {
            if path.ends_with(ext) {
                return true;
            }
        } else if path.contains(pattern) {
            return true;
        }
    }
    false
}

#[cfg(feature = "python")]
pub(crate) fn count_symbols_in_content(content: &str, language: &str) -> u32 {
    let mut count = 0;
    match language {
        "Rust" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("enum ")
                    || trimmed.starts_with("pub enum ")
                {
                    count += 1;
                }
            }
        }
        "JavaScript" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("function ") || trimmed.starts_with("class ") {
                    count += 1;
                }
            }
        }
        "Python" => {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                    count += 1;
                }
            }
        }
        _ => {}
    }
    count
}

// ============================================================================
// PHASE 2: Thread-Safe Class-Based Python API - Configuration
// ============================================================================

/// Configuration for the Python FastContextAnalyzer
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct AnalyzerConfig {
    #[pyo3(get, set)]
    pub project_root: String,

    #[pyo3(get, set)]
    pub languages: Vec<String>,

    #[pyo3(get, set)]
    pub ignore_patterns: Vec<String>,

    #[pyo3(get, set)]
    pub enable_caching: bool,

    #[pyo3(get, set)]
    pub enable_watching: bool,

    #[pyo3(get, set)]
    pub max_files: i32,

    #[pyo3(get, set)]
    pub parallel_processing: bool,
}

#[cfg(feature = "python")]
#[pymethods]
impl AnalyzerConfig {
    #[new]
    #[pyo3(signature = (project_root, languages=None, ignore_patterns=None, enable_caching=true, enable_watching=false, max_files=10000, parallel_processing=true))]
    pub fn new(
        project_root: String,
        languages: Option<Vec<String>>,
        ignore_patterns: Option<Vec<String>>,
        enable_caching: bool,
        enable_watching: bool,
        max_files: i32,
        parallel_processing: bool,
    ) -> Self {
        Self {
            project_root,
            languages: languages.unwrap_or_else(|| {
                vec![
                    "rust".to_string(),
                    "python".to_string(),
                    "javascript".to_string(),
                    "typescript".to_string(),
                ]
            }),
            ignore_patterns: ignore_patterns.unwrap_or_else(crate::utils::default_ignore_patterns),
            enable_caching,
            enable_watching,
            max_files,
            parallel_processing,
        }
    }
}
