//! # LSP-Compatible Export
//!
//! Language Server Protocol compatible export format for seamless integration
//! with IDEs, editors, and development tools that support LSP.

use super::{ExportData, ExportOptions, UniversalExporter};
use crate::analysis::AnalysisResult;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LSP Symbol Information (LSP specification compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspSymbolInformation {
    /// The name of this symbol
    pub name: String,

    /// The kind of this symbol
    pub kind: LspSymbolKind,

    /// Indicates if this symbol is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// The location of this symbol
    pub location: LspLocation,

    /// The name of the symbol containing this symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    /// Tags for this symbol (LSP 3.16+)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<LspSymbolTag>,
}

/// LSP Document Symbol (hierarchical symbol information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDocumentSymbol {
    /// The name of this symbol
    pub name: String,

    /// More detail for this symbol, e.g. the signature of a function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// The kind of this symbol
    pub kind: LspSymbolKind,

    /// Tags for this symbol
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<LspSymbolTag>,

    /// Indicates if this symbol is deprecated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// The range enclosing this symbol not including leading/trailing whitespace
    pub range: LspRange,

    /// The range that should be selected and revealed when this symbol is being picked
    pub selection_range: LspRange,

    /// Children of this symbol, e.g. properties of a class
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<LspDocumentSymbol>,
}

/// LSP Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspLocation {
    pub uri: String,
    pub range: LspRange,
}

/// LSP Range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspRange {
    /// The range's start position
    pub start: LspPosition,

    /// The range's end position
    pub end: LspPosition,
}

/// LSP Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspPosition {
    /// Line position in a document (zero-based)
    pub line: u32,

    /// Character offset on a line in a document (zero-based)
    pub character: u32,
}

/// LSP Symbol Kind enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LspSymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// LSP Symbol Tags
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LspSymbolTag {
    /// Render a symbol as obsolete, usually using a strike-out
    Deprecated = 1,
}

/// LSP Definition Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDefinitionResult {
    pub definitions: Vec<LspLocation>,
}

/// LSP References Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspReferencesResult {
    pub references: Vec<LspLocation>,
}

/// LSP Hover Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspHover {
    /// The hover's content
    pub contents: LspMarkupContent,

    /// An optional range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<LspRange>,
}

/// LSP Markup Content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspMarkupContent {
    /// The type of the markup
    pub kind: LspMarkupKind,

    /// The content itself
    pub value: String,
}

/// LSP Markup Kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LspMarkupKind {
    #[serde(rename = "plaintext")]
    PlainText,
    #[serde(rename = "markdown")]
    Markdown,
}

/// Complete LSP Export containing all LSP-compatible data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspExport {
    /// Workspace symbols
    pub workspace_symbols: Vec<LspSymbolInformation>,

    /// Document symbols by file URI
    pub document_symbols: HashMap<String, Vec<LspDocumentSymbol>>,

    /// Definition locations by symbol
    pub definitions: HashMap<String, LspDefinitionResult>,

    /// Reference locations by symbol
    pub references: HashMap<String, LspReferencesResult>,

    /// Hover information by symbol
    pub hover_info: HashMap<String, LspHover>,

    /// Export metadata
    pub metadata: LspExportMetadata,
}

/// LSP Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspExportMetadata {
    pub format_version: String,
    pub exported_at: u64,
    pub total_symbols: usize,
    pub total_files: usize,
    pub languages: Vec<String>,
}

/// LSP-compatible exporter
pub struct LspExporter {
    exporter: UniversalExporter,
}

impl LspExporter {
    /// Create a new LSP exporter
    pub fn new(analysis: AnalysisResult, project_root: String) -> Self {
        Self {
            exporter: UniversalExporter::new(analysis, project_root),
        }
    }

    /// Export complete LSP data
    pub fn export_lsp_data(&self, options: &ExportOptions) -> LspExport {
        let export_data = self.exporter.create_export_data(options);

        let workspace_symbols = self.create_workspace_symbols(&export_data);
        let document_symbols = self.create_document_symbols(&export_data);
        let definitions = self.create_definitions(&export_data);
        let references = self.create_references(&export_data);
        let hover_info = self.create_hover_info(&export_data);

        LspExport {
            workspace_symbols,
            document_symbols,
            definitions,
            references,
            hover_info,
            metadata: LspExportMetadata {
                format_version: "1.0.0".to_string(),
                exported_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                total_symbols: export_data.symbols.len(),
                total_files: export_data.files.len(),
                languages: export_data.project.languages,
            },
        }
    }

    /// Export workspace symbols only
    pub fn export_workspace_symbols(&self, options: &ExportOptions) -> Vec<LspSymbolInformation> {
        let export_data = self.exporter.create_export_data(options);
        self.create_workspace_symbols(&export_data)
    }

    /// Export document symbols for a specific file
    pub fn export_document_symbols(
        &self,
        file_uri: &str,
        options: &ExportOptions,
    ) -> Vec<LspDocumentSymbol> {
        let export_data = self.exporter.create_export_data(options);
        let document_symbols = self.create_document_symbols(&export_data);

        document_symbols.get(file_uri).cloned().unwrap_or_default()
    }

    /// Export to JSON string
    pub fn export_to_json(&self, options: &ExportOptions) -> Result<String, serde_json::Error> {
        let lsp_data = self.export_lsp_data(options);
        serde_json::to_string_pretty(&lsp_data)
    }

    /// Create workspace symbols
    fn create_workspace_symbols(&self, export_data: &ExportData) -> Vec<LspSymbolInformation> {
        export_data
            .symbols
            .iter()
            .map(|symbol| {
                LspSymbolInformation {
                    name: symbol.name.clone(),
                    kind: self.convert_symbol_kind(&symbol.kind),
                    deprecated: None, // Could be derived from tags
                    location: LspLocation {
                        uri: self.path_to_uri(&symbol.file_path),
                        range: LspRange {
                            start: LspPosition {
                                line: symbol.location.start_line.saturating_sub(1) as u32, // LSP is 0-based
                                character: symbol.location.start_column as u32,
                            },
                            end: LspPosition {
                                line: symbol.location.end_line.saturating_sub(1) as u32,
                                character: symbol.location.end_column as u32,
                            },
                        },
                    },
                    container_name: if symbol.scope_chain.is_empty() {
                        None
                    } else {
                        Some(symbol.scope_chain.join("::"))
                    },
                    tags: self.convert_symbol_tags(&symbol.tags),
                }
            })
            .collect()
    }

    /// Create document symbols organized by file
    fn create_document_symbols(
        &self,
        export_data: &ExportData,
    ) -> HashMap<String, Vec<LspDocumentSymbol>> {
        let mut document_symbols: HashMap<String, Vec<LspDocumentSymbol>> = HashMap::new();

        // Group symbols by file
        let mut symbols_by_file: HashMap<String, Vec<&super::ExportSymbol>> = HashMap::new();
        for symbol in &export_data.symbols {
            symbols_by_file
                .entry(symbol.file_path.clone())
                .or_default()
                .push(symbol);
        }

        // Create hierarchical document symbols for each file
        for (file_path, file_symbols) in symbols_by_file {
            let uri = self.path_to_uri(&file_path);
            let doc_symbols = self.create_hierarchical_symbols(file_symbols);
            document_symbols.insert(uri, doc_symbols);
        }

        document_symbols
    }

    /// Create hierarchical symbols from flat symbol list
    fn create_hierarchical_symbols(
        &self,
        symbols: Vec<&super::ExportSymbol>,
    ) -> Vec<LspDocumentSymbol> {
        let mut root_symbols = Vec::new();
        let mut symbol_map: HashMap<String, LspDocumentSymbol> = HashMap::new();

        // First pass: create all symbols
        for symbol in &symbols {
            let doc_symbol = LspDocumentSymbol {
                name: symbol.name.clone(),
                detail: symbol.signature.clone(),
                kind: self.convert_symbol_kind(&symbol.kind),
                tags: self.convert_symbol_tags(&symbol.tags),
                deprecated: None,
                range: LspRange {
                    start: LspPosition {
                        line: symbol.location.start_line.saturating_sub(1) as u32,
                        character: symbol.location.start_column as u32,
                    },
                    end: LspPosition {
                        line: symbol.location.end_line.saturating_sub(1) as u32,
                        character: symbol.location.end_column as u32,
                    },
                },
                selection_range: LspRange {
                    start: LspPosition {
                        line: symbol.location.start_line.saturating_sub(1) as u32,
                        character: symbol.location.start_column as u32,
                    },
                    end: LspPosition {
                        line: symbol.location.start_line.saturating_sub(1) as u32,
                        character: symbol.location.start_column as u32 + symbol.name.len() as u32,
                    },
                },
                children: Vec::new(),
            };

            symbol_map.insert(symbol.qualified_name.clone(), doc_symbol);
        }

        // Second pass: build hierarchy
        for symbol in symbols {
            if symbol.scope_chain.is_empty() {
                // Root level symbol
                if let Some(doc_symbol) = symbol_map.remove(&symbol.qualified_name) {
                    root_symbols.push(doc_symbol);
                }
            } else {
                // Child symbol - find parent
                let parent_scope = symbol.scope_chain.join("::");
                let child_qualified_name = symbol.qualified_name.clone();
                if symbol_map.contains_key(&parent_scope)
                    && symbol_map.contains_key(&child_qualified_name)
                {
                    if let Some(child_symbol) = symbol_map.remove(&child_qualified_name) {
                        if let Some(parent) = symbol_map.get_mut(&parent_scope) {
                            parent.children.push(child_symbol);
                        }
                    }
                }
            }
        }

        // Add any remaining symbols as root symbols
        root_symbols.extend(symbol_map.into_values());

        // Sort by position
        root_symbols.sort_by(|a, b| {
            a.range
                .start
                .line
                .cmp(&b.range.start.line)
                .then_with(|| a.range.start.character.cmp(&b.range.start.character))
        });

        root_symbols
    }

    /// Create definition mappings
    fn create_definitions(&self, export_data: &ExportData) -> HashMap<String, LspDefinitionResult> {
        let mut definitions = HashMap::new();

        for symbol in &export_data.symbols {
            let location = LspLocation {
                uri: self.path_to_uri(&symbol.file_path),
                range: LspRange {
                    start: LspPosition {
                        line: symbol.location.start_line.saturating_sub(1) as u32,
                        character: symbol.location.start_column as u32,
                    },
                    end: LspPosition {
                        line: symbol.location.end_line.saturating_sub(1) as u32,
                        character: symbol.location.end_column as u32,
                    },
                },
            };

            definitions.insert(
                symbol.qualified_name.clone(),
                LspDefinitionResult {
                    definitions: vec![location],
                },
            );
        }

        definitions
    }

    /// Create reference mappings
    fn create_references(&self, export_data: &ExportData) -> HashMap<String, LspReferencesResult> {
        let mut references = HashMap::new();

        // Build references from relationships
        for relationship in &export_data.relationships {
            let _from_symbol = &relationship.from_symbol;
            let to_symbol = &relationship.to_symbol;

            // Find the symbol that's being referenced (to_symbol)
            if let Some(symbol) = export_data.symbols.iter().find(|s| s.id == *to_symbol) {
                let reference_location = LspLocation {
                    uri: self.path_to_uri(&symbol.file_path),
                    range: LspRange {
                        start: LspPosition {
                            line: symbol.location.start_line.saturating_sub(1) as u32,
                            character: symbol.location.start_column as u32,
                        },
                        end: LspPosition {
                            line: symbol.location.end_line.saturating_sub(1) as u32,
                            character: symbol.location.end_column as u32,
                        },
                    },
                };

                // Add reference to the target symbol
                references
                    .entry(symbol.qualified_name.clone())
                    .or_insert_with(|| LspReferencesResult {
                        references: Vec::new(),
                    })
                    .references
                    .push(reference_location);
            }
        }

        references
    }

    /// Create hover information
    fn create_hover_info(&self, export_data: &ExportData) -> HashMap<String, LspHover> {
        let mut hover_info = HashMap::new();

        for symbol in &export_data.symbols {
            let mut content_parts = Vec::new();

            // Add signature if available
            if let Some(ref signature) = symbol.signature {
                content_parts.push(format!(
                    "```{}\n{}\n```",
                    symbol.language.to_lowercase(),
                    signature
                ));
            }

            // Add documentation if available
            if let Some(ref docs) = symbol.documentation {
                content_parts.push(docs.clone());
            }

            // Add complexity information
            if symbol.complexity > 0 {
                content_parts.push(format!("**Complexity:** {}", symbol.complexity));
            }

            // Add dependency information
            if !symbol.dependencies.is_empty() || !symbol.dependents.is_empty() {
                content_parts.push(format!(
                    "**Dependencies:** {} | **Dependents:** {}",
                    symbol.dependencies.len(),
                    symbol.dependents.len()
                ));
            }

            if !content_parts.is_empty() {
                let hover = LspHover {
                    contents: LspMarkupContent {
                        kind: LspMarkupKind::Markdown,
                        value: content_parts.join("\n\n"),
                    },
                    range: Some(LspRange {
                        start: LspPosition {
                            line: symbol.location.start_line.saturating_sub(1) as u32,
                            character: symbol.location.start_column as u32,
                        },
                        end: LspPosition {
                            line: symbol.location.end_line.saturating_sub(1) as u32,
                            character: symbol.location.end_column as u32,
                        },
                    }),
                };

                hover_info.insert(symbol.qualified_name.clone(), hover);
            }
        }

        hover_info
    }

    /// Convert internal symbol kind to LSP symbol kind
    fn convert_symbol_kind(&self, kind: &str) -> LspSymbolKind {
        match kind {
            "Function" => LspSymbolKind::Function,
            "Class" => LspSymbolKind::Class,
            "Interface" => LspSymbolKind::Interface,
            "Struct" => LspSymbolKind::Struct,
            "Enum" => LspSymbolKind::Enum,
            "Variable" => LspSymbolKind::Variable,
            "Constant" => LspSymbolKind::Constant,
            "Method" => LspSymbolKind::Method,
            "Property" => LspSymbolKind::Property,
            "Field" => LspSymbolKind::Field,
            "Constructor" => LspSymbolKind::Constructor,
            "Module" => LspSymbolKind::Module,
            "Namespace" => LspSymbolKind::Namespace,
            "Package" => LspSymbolKind::Package,
            "EnumMember" => LspSymbolKind::EnumMember,
            "Event" => LspSymbolKind::Event,
            "Operator" => LspSymbolKind::Operator,
            "TypeParameter" => LspSymbolKind::TypeParameter,
            _ => LspSymbolKind::Variable, // Default fallback
        }
    }

    /// Convert symbol tags to LSP tags
    fn convert_symbol_tags(&self, tags: &[String]) -> Vec<LspSymbolTag> {
        let mut lsp_tags = Vec::new();

        if tags.contains(&"deprecated".to_string()) {
            lsp_tags.push(LspSymbolTag::Deprecated);
        }

        lsp_tags
    }

    /// Convert file path to URI
    fn path_to_uri(&self, path: &str) -> String {
        if path.starts_with("file://") {
            path.to_string()
        } else if path.starts_with('/') {
            format!("file://{path}")
        } else {
            format!("file:///{}", path.replace('\\', "/"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{AnalysisResult, CodeMetrics, CodeNode};
    use crate::parsers::LanguageId;
    use crate::symbols::{Location, Symbol, SymbolKind};
    use petgraph::Graph;

    fn create_test_analysis() -> AnalysisResult {
        let mut graph = Graph::new();

        let symbol = Symbol {
            name: "test_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "/test/project/src/main.rs".to_string(),
                start_line: 5,
                start_column: 0,
                end_line: 10,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: Some("Test function documentation".to_string()),
            modifiers: vec!["pub".to_string()],
            signature: Some("fn test_function() -> i32".to_string()),
        };

        let node_data = CodeNode {
            symbol,
            file_path: "/test/project/src/main.rs".to_string(),
            metrics: CodeMetrics {
                cyclomatic_complexity: 3,
                ..Default::default()
            },
        };

        graph.add_node(node_data);

        AnalysisResult {
            graph,
            file_count: 1,
            symbol_count: 1,
            relationship_count: 0,
            languages: vec![LanguageId::Rust],
        }
    }

    #[test]
    fn test_lsp_workspace_symbols() {
        let analysis = create_test_analysis();
        let exporter = LspExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();

        let workspace_symbols = exporter.export_workspace_symbols(&options);

        assert_eq!(workspace_symbols.len(), 1);
        let symbol = &workspace_symbols[0];
        assert_eq!(symbol.name, "test_function");
        assert!(matches!(symbol.kind, LspSymbolKind::Function));
        assert_eq!(symbol.location.range.start.line, 4); // 0-based
        assert_eq!(symbol.container_name, Some("main".to_string()));
    }

    #[test]
    fn test_lsp_document_symbols() {
        let analysis = create_test_analysis();
        let exporter = LspExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();

        let doc_symbols =
            exporter.export_document_symbols("file:///test/project/src/main.rs", &options);

        assert_eq!(doc_symbols.len(), 1);
        let symbol = &doc_symbols[0];
        assert_eq!(symbol.name, "test_function");
        assert!(matches!(symbol.kind, LspSymbolKind::Function));
        assert_eq!(symbol.detail, Some("fn test_function() -> i32".to_string()));
    }

    #[test]
    fn test_lsp_complete_export() {
        let analysis = create_test_analysis();
        let exporter = LspExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();

        let lsp_data = exporter.export_lsp_data(&options);

        assert_eq!(lsp_data.workspace_symbols.len(), 1);
        assert_eq!(lsp_data.document_symbols.len(), 1);
        assert_eq!(lsp_data.definitions.len(), 1);
        assert_eq!(lsp_data.hover_info.len(), 1);
        assert_eq!(lsp_data.metadata.total_symbols, 1);
    }

    #[test]
    fn test_path_to_uri_conversion() {
        let analysis = create_test_analysis();
        let exporter = LspExporter::new(analysis, "/test/project".to_string());

        assert_eq!(
            exporter.path_to_uri("/absolute/path"),
            "file:///absolute/path"
        );
        assert_eq!(
            exporter.path_to_uri("relative/path"),
            "file:///relative/path"
        );
        assert_eq!(
            exporter.path_to_uri("file:///already/uri"),
            "file:///already/uri"
        );
    }

    #[test]
    fn test_json_export() {
        let analysis = create_test_analysis();
        let exporter = LspExporter::new(analysis, "/test/project".to_string());
        let options = ExportOptions::default();

        let json = exporter.export_to_json(&options).unwrap();
        assert!(json.contains("test_function"));
        assert!(json.contains("workspace_symbols"));
        assert!(json.contains("document_symbols"));
    }
}
