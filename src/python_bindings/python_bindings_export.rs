//! Export system for Python SDK
//!
//! Provides JSON, LSP, and embedding export capabilities for AI/ML workflows

#![allow(non_local_definitions)]

use crate::python_bindings::AnalysisResult;
use pyo3::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use chrono;

/// Export options configuration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyExportOptions {
    #[pyo3(get, set)]
    pub format: String,
    
    #[pyo3(get, set)]
    pub output_path: Option<String>,
    
    #[pyo3(get, set)]
    pub include_source: bool,
    
    #[pyo3(get, set)]
    pub include_docs: bool,
    
    #[pyo3(get, set)]
    pub minify: bool,
    
    #[pyo3(get, set)]
    pub include_relationships: bool,
    
    #[pyo3(get, set)]
    pub include_embeddings: bool,
    
    #[pyo3(get, set)]
    pub embedding_format: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyExportOptions {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (format, output_path=None, include_source=false, include_docs=false, minify=false, include_relationships=true, include_embeddings=false, embedding_format="numpy".to_string()))]
    pub fn new(
        format: String,
        output_path: Option<String>,
        include_source: bool,
        include_docs: bool,
        minify: bool,
        include_relationships: bool,
        include_embeddings: bool,
        embedding_format: String,
    ) -> Self {
        Self {
            format,
            output_path,
            include_source,
            include_docs,
            minify,
            include_relationships,
            include_embeddings,
            embedding_format,
        }
    }
}

/// JSON Exporter for structured data export
#[cfg(feature = "python")]
#[pyclass]
pub struct PyJsonExporter {
    options: PyExportOptions,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyJsonExporter {
    #[new]
    pub fn new(options: PyExportOptions) -> Self {
        Self { options }
    }
    
    /// Export analysis results to JSON format
    pub fn export_analysis(&self, analysis: &AnalysisResult) -> PyResult<String> {
        let mut export_data = serde_json::Map::new();
        
        // Basic analysis metadata
        export_data.insert("file_count".to_string(), json!(analysis.file_count));
        export_data.insert("symbol_count".to_string(), json!(analysis.symbol_count));
        export_data.insert("languages".to_string(), json!(analysis.languages));
        export_data.insert("duration_ms".to_string(), json!(analysis.duration_ms));
        
        // Convert relationships to JSON
        if self.options.include_relationships {
            let relationships_json: Vec<Value> = analysis.relationships
                .iter()
                .map(|rel| {
                    let mut rel_map = serde_json::Map::new();
                    rel_map.insert("from_symbol".to_string(), json!(rel.from_symbol));
                    rel_map.insert("to_symbol".to_string(), json!(rel.to_symbol));
                    rel_map.insert("relationship_type".to_string(), json!(rel.relationship_type));
                    rel_map.insert("strength".to_string(), json!(rel.strength));
                    rel_map.insert("is_conditional".to_string(), json!(rel.is_conditional));
                    json!(rel_map)
                })
                .collect();
            export_data.insert("relationships".to_string(), json!(relationships_json));
        }
        
        // Generate embeddings if requested
        if self.options.include_embeddings {
            let embeddings_str = self.generate_embeddings(analysis);
            if let Ok(embeddings_value) = serde_json::from_str(&embeddings_str) {
                export_data.insert("embeddings".to_string(), embeddings_value);
            }
        }
        
        // Add metadata
        let mut metadata = serde_json::Map::new();
        metadata.insert("export_format".to_string(), json!("json"));
        metadata.insert("export_version".to_string(), json!("1.0"));
        metadata.insert("generated_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        export_data.insert("metadata".to_string(), json!(metadata));
        
        let json_str = if self.options.minify {
            serde_json::to_string(&export_data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
        } else {
            serde_json::to_string_pretty(&export_data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
        };
        
        Ok(json_str)
    }
    
    /// Export analysis results to file
    pub fn export_to_file(&self, analysis: &AnalysisResult, output_path: String) -> PyResult<()> {
        let json_data = self.export_analysis(analysis)?;
        let mut file = File::create(&output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(())
    }
    
    /// Generate feature-based embeddings for symbols
    fn generate_embeddings(&self, analysis: &AnalysisResult) -> String {
        let mut embeddings = serde_json::Map::new();
        
        // Generate simple feature-based embeddings for each symbol
        let mut symbol_embeddings = HashMap::new();
        for rel in &analysis.relationships {
            let features = vec![
                rel.strength,
                if rel.is_conditional { 1.0 } else { 0.0 },
                rel.from_symbol.len() as f32 / 100.0,
                rel.to_symbol.len() as f32 / 100.0,
            ];
            symbol_embeddings.insert(rel.from_symbol.clone(), features);
        }
        
        embeddings.insert("symbol_embeddings".to_string(), json!(symbol_embeddings));
        embeddings.insert("embedding_format".to_string(), json!(self.options.embedding_format));
        embeddings.insert("embedding_size".to_string(), json!(4));
        
        serde_json::to_string(&json!(embeddings)).unwrap_or_else(|_| "{}".to_string())
    }
}

/// LSP Symbol Information for Language Server Protocol integration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyLspSymbolInformation {
    #[pyo3(get)]
    pub name: String,
    
    #[pyo3(get)]
    pub kind: u32, // LSP SymbolKind enum
    
    #[pyo3(get)]
    pub location: PyLspLocation,
    
    #[pyo3(get)]
    pub container_name: Option<String>,
    
    #[pyo3(get)]
    pub documentation: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLspSymbolInformation {
    #[new]
    #[pyo3(signature = (name, kind, location, container_name=None, documentation=None))]
    pub fn new(
        name: String,
        kind: u32,
        location: PyLspLocation,
        container_name: Option<String>,
        documentation: Option<String>,
    ) -> Self {
        Self {
            name,
            kind,
            location,
            container_name,
            documentation,
        }
    }
}

/// LSP Location for Language Server Protocol integration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyLspLocation {
    #[pyo3(get)]
    pub uri: String,
    
    #[pyo3(get)]
    pub range: PyLspRange,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLspLocation {
    #[new]
    pub fn new(uri: String, range: PyLspRange) -> Self {
        Self { uri, range }
    }
}

/// LSP Range for Language Server Protocol integration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyLspRange {
    #[pyo3(get)]
    pub start: PyLspPosition,
    
    #[pyo3(get)]
    pub end: PyLspPosition,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLspRange {
    #[new]
    pub fn new(start: PyLspPosition, end: PyLspPosition) -> Self {
        Self { start, end }
    }
}

/// LSP Position for Language Server Protocol integration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyLspPosition {
    #[pyo3(get)]
    pub line: u32,
    
    #[pyo3(get)]
    pub character: u32,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLspPosition {
    #[new]
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// LSP Exporter for Language Server Protocol integration
#[cfg(feature = "python")]
#[pyclass]
pub struct PyLspExporter {
    #[allow(dead_code)]
    options: PyExportOptions,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLspExporter {
    #[new]
    pub fn new(options: PyExportOptions) -> Self {
        Self { options }
    }
    
    /// Export analysis results to LSP SymbolInformation format
    pub fn export_symbols(&self, analysis: &AnalysisResult) -> PyResult<Vec<PyLspSymbolInformation>> {
        let mut symbols = Vec::new();
        
        // Convert relationships to LSP symbols (simplified)
        for rel in &analysis.relationships {
            let location = PyLspLocation {
                uri: format!("file://{}", rel.file_path),
                range: PyLspRange {
                    start: PyLspPosition { line: 0, character: 0 },
                    end: PyLspPosition { line: 0, character: 0 },
                },
            };
            
            let symbol = PyLspSymbolInformation {
                name: rel.from_symbol.clone(),
                kind: 12, // LSP SymbolKind::Function
                location,
                container_name: None,
                documentation: None,
            };
            
            symbols.push(symbol);
        }
        
        Ok(symbols)
    }
    
    /// Export as LSP workspace symbols response
    pub fn export_workspace_symbols(&self, analysis: &AnalysisResult) -> PyResult<String> {
        let symbols = self.export_symbols(analysis)?;
        let symbols_json: Vec<Value> = symbols
            .iter()
            .map(|sym| {
                json!({
                    "name": sym.name,
                    "kind": sym.kind,
                    "location": {
                        "uri": sym.location.uri,
                        "range": {
                            "start": {
                                "line": sym.location.range.start.line,
                                "character": sym.location.range.start.character
                            },
                            "end": {
                                "line": sym.location.range.end.line,
                                "character": sym.location.range.end.character
                            }
                        }
                    },
                    "containerName": sym.container_name
                })
            })
            .collect();
        
        serde_json::to_string_pretty(&json!({ "symbols": symbols_json }))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

/// Embedding Exporter for AI/ML workflows
#[cfg(feature = "python")]
#[pyclass]
pub struct PyEmbeddingExporter {
    options: PyExportOptions,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyEmbeddingExporter {
    #[new]
    pub fn new(options: PyExportOptions) -> Self {
        Self { options }
    }
    
    /// Export analysis results as embeddings
    pub fn export_embeddings(&self, analysis: &AnalysisResult) -> PyResult<String> {
        let mut embeddings = serde_json::Map::new();
        
        // Generate simple statistical embeddings
        let mut feature_vectors = HashMap::new();
        
        // Basic statistics features
        let stats_features = vec![
            analysis.file_count as f32,
            analysis.symbol_count as f32,
            analysis.duration_ms as f32,
            analysis.relationships.len() as f32,
        ];
        
        feature_vectors.insert("statistics".to_string(), stats_features);
        
        // Language distribution features
        let mut lang_features = HashMap::new();
        for lang in &analysis.languages {
            *lang_features.entry(lang.clone()).or_insert(0) += 1;
        }
        
        embeddings.insert("feature_vectors".to_string(), json!(feature_vectors));
        embeddings.insert("language_distribution".to_string(), json!(lang_features));
        embeddings.insert("embedding_format".to_string(), json!(self.options.embedding_format));
        embeddings.insert("metadata".to_string(), json!({
            "total_symbols": analysis.symbol_count,
            "total_relationships": analysis.relationships.len(),
            "analysis_duration_ms": analysis.duration_ms
        }));
        
        serde_json::to_string_pretty(&embeddings)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

/// Export factory for creating exporters
#[cfg(feature = "python")]
#[pyclass]
pub struct PyExportFactory;

#[cfg(feature = "python")]
impl Default for PyExportFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyExportFactory {
    #[new]
    pub fn new() -> Self {
        Self
    }
    
    /// Create JSON exporter
    pub fn create_json_exporter(&self, options: PyExportOptions) -> PyJsonExporter {
        PyJsonExporter::new(options)
    }
    
    /// Create LSP exporter
    pub fn create_lsp_exporter(&self, options: PyExportOptions) -> PyLspExporter {
        PyLspExporter::new(options)
    }
    
    /// Create embedding exporter
    pub fn create_embedding_exporter(&self, options: PyExportOptions) -> PyEmbeddingExporter {
        PyEmbeddingExporter::new(options)
    }
    
    /// Get supported export formats
    pub fn get_supported_formats(&self) -> Vec<String> {
        vec![
            "json".to_string(),
            "lsp".to_string(),
            "embeddings".to_string(),
            "numpy".to_string(),
        ]
    }
}
