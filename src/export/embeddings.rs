//! # Embedding-Friendly Export
//! 
//! Specialized export format optimized for AI embeddings and machine learning
//! applications, providing rich contextual information for code understanding.

use super::{ExportData, UniversalExporter, ExportOptions};
use crate::analysis::AnalysisResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Symbol embedding representation optimized for ML/AI applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEmbedding {
    /// Unique identifier
    pub id: String,
    
    /// Core symbol information
    pub symbol: EmbeddingSymbolInfo,
    
    /// Rich contextual information
    pub context: CodeContext,
    
    /// Structural relationships
    pub relationships: EmbeddingRelationships,
    
    /// Semantic features
    pub semantic_features: SemanticFeatures,
    
    /// Embedding-specific metadata
    pub embedding_metadata: EmbeddingMetadata,
}

/// Core symbol information for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingSymbolInfo {
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub language: String,
    pub file_path: String,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub scope_chain: Vec<String>,
    pub modifiers: Vec<String>,
    pub location: EmbeddingLocation,
}

/// Location information optimized for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingLocation {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub byte_offset: Option<usize>,
    pub byte_length: Option<usize>,
    pub relative_position: f32, // Position in file (0.0 - 1.0)
}

/// Rich code context for better understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Surrounding code snippet
    pub surrounding_code: Option<String>,
    
    /// Function/class body content
    pub body_content: Option<String>,
    
    /// Leading comments
    pub leading_comments: Vec<String>,
    
    /// Inline comments
    pub inline_comments: Vec<String>,
    
    /// File-level context
    pub file_context: FileContext,
    
    /// Project-level context
    pub project_context: ProjectContext,
}

/// File-level contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    /// File summary/description
    pub summary: Option<String>,
    
    /// Other symbols in the same file
    pub sibling_symbols: Vec<String>,
    
    /// Import statements in the file
    pub imports: Vec<String>,
    
    /// File-level tags/annotations
    pub file_tags: Vec<String>,
    
    /// File complexity metrics
    pub complexity_metrics: FileComplexityMetrics,
}

/// Project-level contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Project type/domain
    pub project_type: Option<String>,
    
    /// Primary programming languages
    pub primary_languages: Vec<String>,
    
    /// Project tags/characteristics
    pub project_tags: Vec<String>,
    
    /// Related projects/dependencies
    pub related_projects: Vec<String>,
    
    /// Project complexity level
    pub complexity_level: String,
}

/// File complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComplexityMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub symbol_count: usize,
    pub average_complexity: f32,
    pub max_complexity: u32,
}

/// Structural relationships for graph embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRelationships {
    /// Direct dependencies (what this symbol uses)
    pub dependencies: Vec<EmbeddingRelationship>,
    
    /// Direct dependents (what uses this symbol)
    pub dependents: Vec<EmbeddingRelationship>,
    
    /// Hierarchical relationships (parent/child)
    pub hierarchy: Vec<EmbeddingRelationship>,
    
    /// Semantic relationships (similar/related symbols)
    pub semantic_links: Vec<EmbeddingRelationship>,
    
    /// Cross-file relationships
    pub cross_file_links: Vec<EmbeddingRelationship>,
}

/// Individual relationship for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRelationship {
    pub target_symbol: String,
    pub relationship_type: String,
    pub strength: f32, // Relationship strength (0.0 - 1.0)
    pub context: Option<String>,
    pub location: Option<EmbeddingLocation>,
}

/// Semantic features for ML applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFeatures {
    /// Abstract syntax tree features
    pub ast_features: AstFeatures,
    
    /// Textual features
    pub text_features: TextFeatures,
    
    /// Structural features
    pub structural_features: StructuralFeatures,
    
    /// Behavioral features
    pub behavioral_features: BehavioralFeatures,
    
    /// Language-specific features
    pub language_features: LanguageFeatures,
}

/// AST-based features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstFeatures {
    /// Node types in the AST
    pub node_types: Vec<String>,
    
    /// AST depth
    pub depth: usize,
    
    /// Number of children
    pub child_count: usize,
    
    /// AST structural patterns
    pub patterns: Vec<String>,
}

/// Text-based features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFeatures {
    /// Token count
    pub token_count: usize,
    
    /// Unique token count
    pub unique_tokens: usize,
    
    /// Text length in characters
    pub char_count: usize,
    
    /// Keywords used
    pub keywords: Vec<String>,
    
    /// Identifiers used
    pub identifiers: Vec<String>,
    
    /// Literals used
    pub literals: Vec<String>,
}

/// Structural code features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralFeatures {
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    
    /// Nesting depth
    pub nesting_depth: usize,
    
    /// Number of parameters
    pub parameter_count: usize,
    
    /// Number of local variables
    pub local_variable_count: usize,
    
    /// Number of return statements
    pub return_count: usize,
    
    /// Control flow complexity
    pub control_flow_complexity: u32,
}

/// Behavioral code features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralFeatures {
    /// Function calls made
    pub calls_made: Vec<String>,
    
    /// Variables accessed
    pub variables_accessed: Vec<String>,
    
    /// Side effects detected
    pub side_effects: Vec<String>,
    
    /// I/O operations
    pub io_operations: Vec<String>,
    
    /// Memory operations
    pub memory_operations: Vec<String>,
}

/// Language-specific features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFeatures {
    /// Language-specific constructs used
    pub constructs: Vec<String>,
    
    /// Design patterns detected
    pub patterns: Vec<String>,
    
    /// Idioms used
    pub idioms: Vec<String>,
    
    /// Framework-specific features
    pub framework_features: Vec<String>,
}

/// Embedding-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// Vector dimensions (if pre-computed)
    pub vector_dimensions: Option<usize>,
    
    /// Embedding model used
    pub model_name: Option<String>,
    
    /// Confidence scores
    pub confidence_scores: HashMap<String, f32>,
    
    /// Feature importance weights
    pub feature_weights: HashMap<String, f32>,
    
    /// Preprocessing flags
    pub preprocessing: PreprocessingFlags,
    
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
}

/// Preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingFlags {
    /// Normalize identifier names
    pub normalize_identifiers: bool,
    
    /// Remove comments
    pub remove_comments: bool,
    
    /// Tokenize content
    pub tokenize: bool,
    
    /// Apply stemming
    pub stem_tokens: bool,
    
    /// Remove stop words
    pub remove_stop_words: bool,
}

/// Quality metrics for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Completeness score (0.0 - 1.0)
    pub completeness: f32,
    
    /// Context richness score (0.0 - 1.0)
    pub context_richness: f32,
    
    /// Relationship density (0.0 - 1.0)
    pub relationship_density: f32,
    
    /// Information content score (0.0 - 1.0)
    pub information_content: f32,
}

/// Complete embedding export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingExport {
    /// All symbol embeddings
    pub embeddings: Vec<SymbolEmbedding>,
    
    /// Global context information
    pub global_context: GlobalContext,
    
    /// Export configuration
    pub export_config: EmbeddingExportConfig,
    
    /// Export metadata
    pub metadata: EmbeddingExportMetadata,
}

/// Global project context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContext {
    /// Project vocabulary (all unique tokens)
    pub vocabulary: Vec<String>,
    
    /// Global symbol index
    pub symbol_index: HashMap<String, usize>,
    
    /// File index
    pub file_index: HashMap<String, Vec<usize>>,
    
    /// Language distribution
    pub language_distribution: HashMap<String, f32>,
    
    /// Complexity distribution
    pub complexity_distribution: HashMap<String, usize>,
    
    /// Common patterns
    pub common_patterns: Vec<String>,
}

/// Embedding export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingExportConfig {
    /// Include code context
    pub include_context: bool,
    
    /// Include semantic features  
    pub include_semantic_features: bool,
    
    /// Include relationships
    pub include_relationships: bool,
    
    /// Maximum context length
    pub max_context_length: usize,
    
    /// Feature extraction options
    pub feature_options: FeatureExtractionOptions,
    
    /// Preprocessing options
    pub preprocessing: PreprocessingFlags,
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureExtractionOptions {
    /// Extract AST features
    pub ast_features: bool,
    
    /// Extract text features
    pub text_features: bool,
    
    /// Extract structural features
    pub structural_features: bool,
    
    /// Extract behavioral features
    pub behavioral_features: bool,
    
    /// Extract language-specific features
    pub language_features: bool,
}

/// Embedding export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingExportMetadata {
    pub format_version: String,
    pub exported_at: u64,
    pub total_embeddings: usize,
    pub feature_extraction_time_ms: u64,
    pub export_config: EmbeddingExportConfig,
}

/// Embedding-optimized exporter
pub struct EmbeddingExporter {
    exporter: UniversalExporter,
}

impl EmbeddingExporter {
    /// Create a new embedding exporter
    pub fn new(analysis: AnalysisResult, project_root: String) -> Self {
        Self {
            exporter: UniversalExporter::new(analysis, project_root),
        }
    }
    
    /// Export symbol embeddings
    pub fn export_embeddings(&self, options: &ExportOptions, config: &EmbeddingExportConfig) -> EmbeddingExport {
        let export_data = self.exporter.create_export_data(options);
        
        let symbol_count = export_data.symbols.len();
        let embeddings: Vec<SymbolEmbedding> = export_data.symbols.iter()
            .map(|symbol| self.create_symbol_embedding(symbol.clone(), &export_data, config))
            .collect();
        
        let global_context = self.create_global_context(&embeddings);
        
        EmbeddingExport {
            embeddings,
            global_context,
            export_config: config.clone(),
            metadata: EmbeddingExportMetadata {
                format_version: "1.0.0".to_string(),
                exported_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                total_embeddings: symbol_count,
                feature_extraction_time_ms: 0, // Would be measured in real implementation
                export_config: config.clone(),
            },
        }
    }
    
    /// Create a single symbol embedding
    fn create_symbol_embedding(
        &self,
        symbol: super::ExportSymbol,
        export_data: &ExportData,
        config: &EmbeddingExportConfig,
    ) -> SymbolEmbedding {
        let embedding_symbol = EmbeddingSymbolInfo {
            name: symbol.name.clone(),
            qualified_name: symbol.qualified_name.clone(),
            kind: symbol.kind.clone(),
            language: symbol.language.clone(),
            file_path: symbol.file_path.clone(),
            signature: symbol.signature.clone(),
            documentation: symbol.documentation.clone(),
            scope_chain: symbol.scope_chain.clone(),
            modifiers: symbol.modifiers.clone(),
            location: EmbeddingLocation {
                start_line: symbol.location.start_line,
                end_line: symbol.location.end_line,
                start_column: symbol.location.start_column,
                end_column: symbol.location.end_column,
                byte_offset: symbol.location.byte_offset,
                byte_length: symbol.location.byte_length,
                relative_position: self.calculate_relative_position(&symbol, export_data),
            },
        };
        
        let context = if config.include_context {
            self.create_code_context(&symbol, export_data)
        } else {
            CodeContext {
                surrounding_code: None,
                body_content: None,
                leading_comments: Vec::new(),
                inline_comments: Vec::new(),
                file_context: FileContext {
                    summary: None,
                    sibling_symbols: Vec::new(),
                    imports: Vec::new(),
                    file_tags: Vec::new(),
                    complexity_metrics: FileComplexityMetrics {
                        total_lines: 0,
                        code_lines: 0,
                        comment_lines: 0,
                        blank_lines: 0,
                        symbol_count: 0,
                        average_complexity: 0.0,
                        max_complexity: 0,
                    },
                },
                project_context: self.create_project_context(export_data),
            }
        };
        
        let relationships = if config.include_relationships {
            self.create_embedding_relationships(&symbol, export_data)
        } else {
            EmbeddingRelationships {
                dependencies: Vec::new(),
                dependents: Vec::new(),
                hierarchy: Vec::new(),
                semantic_links: Vec::new(),
                cross_file_links: Vec::new(),
            }
        };
        
        let semantic_features = if config.include_semantic_features {
            self.extract_semantic_features(&symbol, config)
        } else {
            SemanticFeatures {
                ast_features: AstFeatures {
                    node_types: Vec::new(),
                    depth: 0,
                    child_count: 0,
                    patterns: Vec::new(),
                },
                text_features: TextFeatures {
                    token_count: 0,
                    unique_tokens: 0,
                    char_count: 0,
                    keywords: Vec::new(),
                    identifiers: Vec::new(),
                    literals: Vec::new(),
                },
                structural_features: StructuralFeatures {
                    cyclomatic_complexity: symbol.complexity,
                    nesting_depth: 0,
                    parameter_count: 0,
                    local_variable_count: 0,
                    return_count: 0,
                    control_flow_complexity: 0,
                },
                behavioral_features: BehavioralFeatures {
                    calls_made: symbol.dependencies.clone(),
                    variables_accessed: Vec::new(),
                    side_effects: Vec::new(),
                    io_operations: Vec::new(),
                    memory_operations: Vec::new(),
                },
                language_features: LanguageFeatures {
                    constructs: Vec::new(),
                    patterns: Vec::new(),
                    idioms: Vec::new(),
                    framework_features: Vec::new(),
                },
            }
        };
        
        let embedding_metadata = EmbeddingMetadata {
            vector_dimensions: None,
            model_name: None,
            confidence_scores: HashMap::new(),
            feature_weights: HashMap::new(),
            preprocessing: config.preprocessing.clone(),
            quality_metrics: self.calculate_quality_metrics(&symbol, &context, &relationships),
        };
        
        SymbolEmbedding {
            id: symbol.id,
            symbol: embedding_symbol,
            context,
            relationships,
            semantic_features,
            embedding_metadata,
        }
    }
    
    /// Calculate relative position of symbol in file
    fn calculate_relative_position(&self, symbol: &super::ExportSymbol, export_data: &ExportData) -> f32 {
        // Find the file and calculate relative position
        if let Some(file_info) = export_data.files.iter().find(|f| f.path == symbol.file_path) {
            if file_info.line_count > 0 {
                symbol.location.start_line as f32 / file_info.line_count as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Create rich code context
    fn create_code_context(&self, symbol: &super::ExportSymbol, export_data: &ExportData) -> CodeContext {
        // Find sibling symbols in the same file
        let sibling_symbols: Vec<String> = export_data.symbols.iter()
            .filter(|s| s.file_path == symbol.file_path && s.id != symbol.id)
            .map(|s| s.qualified_name.clone())
            .collect();
        
        // Calculate file complexity metrics
        let file_symbols: Vec<&super::ExportSymbol> = export_data.symbols.iter()
            .filter(|s| s.file_path == symbol.file_path)
            .collect();
        
        let complexity_metrics = FileComplexityMetrics {
            total_lines: 0, // Would need file system access
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            symbol_count: file_symbols.len(),
            average_complexity: if !file_symbols.is_empty() {
                file_symbols.iter().map(|s| s.complexity as f32).sum::<f32>() / file_symbols.len() as f32
            } else {
                0.0
            },
            max_complexity: file_symbols.iter().map(|s| s.complexity).max().unwrap_or(0),
        };
        
        let file_context = FileContext {
            summary: None, // Could be generated from file content
            sibling_symbols,
            imports: Vec::new(), // Would need to be extracted from AST
            file_tags: Vec::new(),
            complexity_metrics,
        };
        
        CodeContext {
            surrounding_code: None, // Would need file system access to extract
            body_content: None,
            leading_comments: Vec::new(),
            inline_comments: Vec::new(),
            file_context,
            project_context: self.create_project_context(export_data),
        }
    }
    
    /// Create project context
    fn create_project_context(&self, export_data: &ExportData) -> ProjectContext {
        ProjectContext {
            project_type: None, // Could be inferred from dependencies/files
            primary_languages: export_data.project.languages.clone(),
            project_tags: Vec::new(),
            related_projects: Vec::new(),
            complexity_level: if export_data.metrics.average_complexity < 5.0 {
                "Low".to_string()
            } else if export_data.metrics.average_complexity < 15.0 {
                "Medium".to_string()
            } else {
                "High".to_string()
            },
        }
    }
    
    /// Create embedding relationships
    fn create_embedding_relationships(&self, symbol: &super::ExportSymbol, _export_data: &ExportData) -> EmbeddingRelationships {
        let dependencies: Vec<EmbeddingRelationship> = symbol.dependencies.iter()
            .map(|dep| EmbeddingRelationship {
                target_symbol: dep.clone(),
                relationship_type: "uses".to_string(),
                strength: 1.0, // Could be calculated based on usage frequency
                context: None,
                location: None,
            })
            .collect();
        
        let dependents: Vec<EmbeddingRelationship> = symbol.dependents.iter()
            .map(|dep| EmbeddingRelationship {
                target_symbol: dep.clone(),
                relationship_type: "used_by".to_string(),
                strength: 1.0,
                context: None,
                location: None,
            })
            .collect();
        
        // Find cross-file relationships
        let cross_file_links: Vec<EmbeddingRelationship> = symbol.related_files.iter()
            .map(|file| EmbeddingRelationship {
                target_symbol: file.clone(),
                relationship_type: "cross_file".to_string(),
                strength: 0.5,
                context: Some(format!("Related to file: {file}")),
                location: None,
            })
            .collect();
        
        EmbeddingRelationships {
            dependencies,
            dependents,
            hierarchy: Vec::new(), // Would need hierarchical analysis
            semantic_links: Vec::new(), // Would need semantic similarity computation
            cross_file_links,
        }
    }
    
    /// Extract semantic features
    fn extract_semantic_features(&self, symbol: &super::ExportSymbol, config: &EmbeddingExportConfig) -> SemanticFeatures {
        let text_features = if config.feature_options.text_features {
            let signature_text = symbol.signature.as_deref().unwrap_or("");
            let doc_text = symbol.documentation.as_deref().unwrap_or("");
            let combined_text = format!("{signature_text} {doc_text}");
            
            TextFeatures {
                token_count: combined_text.split_whitespace().count(),
                unique_tokens: combined_text.split_whitespace()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                char_count: combined_text.len(),
                keywords: self.extract_keywords(&combined_text, &symbol.language),
                identifiers: vec![symbol.name.clone()],
                literals: Vec::new(),
            }
        } else {
            TextFeatures {
                token_count: 0,
                unique_tokens: 0,
                char_count: 0,
                keywords: Vec::new(),
                identifiers: Vec::new(),
                literals: Vec::new(),
            }
        };
        
        SemanticFeatures {
            ast_features: AstFeatures {
                node_types: Vec::new(),
                depth: 0,
                child_count: 0,
                patterns: Vec::new(),
            },
            text_features,
            structural_features: StructuralFeatures {
                cyclomatic_complexity: symbol.complexity,
                nesting_depth: symbol.scope_chain.len(),
                parameter_count: 0, // Would need AST analysis
                local_variable_count: 0,
                return_count: 0,
                control_flow_complexity: symbol.complexity,
            },
            behavioral_features: BehavioralFeatures {
                calls_made: symbol.dependencies.clone(),
                variables_accessed: Vec::new(),
                side_effects: Vec::new(),
                io_operations: Vec::new(),
                memory_operations: Vec::new(),
            },
            language_features: LanguageFeatures {
                constructs: Vec::new(),
                patterns: Vec::new(),
                idioms: Vec::new(),
                framework_features: Vec::new(),
            },
        }
    }
    
    /// Extract keywords from text based on language
    fn extract_keywords(&self, text: &str, language: &str) -> Vec<String> {
        let common_keywords = match language.to_lowercase().as_str() {
            "rust" => vec!["fn", "struct", "enum", "impl", "trait", "pub", "mut", "let", "const"],
            "python" => vec!["def", "class", "import", "from", "if", "for", "while", "try", "except"],
            "javascript" | "typescript" => vec!["function", "class", "const", "let", "var", "if", "for", "while", "try", "catch"],
            "java" => vec!["public", "private", "class", "interface", "extends", "implements", "static", "final"],
            _ => vec!["function", "class", "public", "private", "static"],
        };
        
        text.split_whitespace()
            .filter(|word| common_keywords.contains(&word.to_lowercase().as_str()))
            .map(|word| word.to_string())
            .collect()
    }
    
    /// Calculate quality metrics
    fn calculate_quality_metrics(
        &self,
        symbol: &super::ExportSymbol,
        context: &CodeContext,
        relationships: &EmbeddingRelationships,
    ) -> QualityMetrics {
        let completeness = if symbol.signature.is_some() && symbol.documentation.is_some() {
            1.0
        } else if symbol.signature.is_some() || symbol.documentation.is_some() {
            0.7
        } else {
            0.3
        };
        
        let context_richness = if context.surrounding_code.is_some() {
            0.9
        } else if !context.file_context.sibling_symbols.is_empty() {
            0.6
        } else {
            0.2
        };
        
        let relationship_density = {
            let total_relationships = relationships.dependencies.len() + 
                                    relationships.dependents.len() + 
                                    relationships.cross_file_links.len();
            (total_relationships as f32 / 10.0).min(1.0) // Normalize to 0-1
        };
        
        let information_content = (completeness + context_richness + relationship_density) / 3.0;
        
        QualityMetrics {
            completeness,
            context_richness,
            relationship_density,
            information_content,
        }
    }
    
    /// Create global context
    fn create_global_context(&self, embeddings: &[SymbolEmbedding]) -> GlobalContext {
        let mut vocabulary = std::collections::HashSet::new();
        let mut symbol_index = HashMap::new();
        let mut file_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut language_counts = HashMap::new();
        let mut complexity_counts = HashMap::new();
        
        for (idx, embedding) in embeddings.iter().enumerate() {
            // Build vocabulary
            for keyword in &embedding.semantic_features.text_features.keywords {
                vocabulary.insert(keyword.clone());
            }
            for identifier in &embedding.semantic_features.text_features.identifiers {
                vocabulary.insert(identifier.clone());
            }
            
            // Build symbol index
            symbol_index.insert(embedding.symbol.qualified_name.clone(), idx);
            
            // Build file index
            file_index
                .entry(embedding.symbol.file_path.clone())
                .or_default()
                .push(idx);
            
            // Language distribution
            *language_counts.entry(embedding.symbol.language.clone()).or_insert(0) += 1;
            
            // Complexity distribution
            let complexity_range = match embedding.semantic_features.structural_features.cyclomatic_complexity {
                0..=5 => "Low",
                6..=10 => "Medium",
                11..=20 => "High",
                _ => "Very High",
            };
            *complexity_counts.entry(complexity_range.to_string()).or_insert(0) += 1;
        }
        
        let total_embeddings = embeddings.len() as f32;
        let language_distribution = language_counts.into_iter()
            .map(|(lang, count)| (lang, count as f32 / total_embeddings))
            .collect();
        
        GlobalContext {
            vocabulary: vocabulary.into_iter().collect(),
            symbol_index,
            file_index,
            language_distribution,
            complexity_distribution: complexity_counts,
            common_patterns: Vec::new(), // Would need pattern analysis
        }
    }
}

impl Default for EmbeddingExportConfig {
    fn default() -> Self {
        Self {
            include_context: true,
            include_semantic_features: true,
            include_relationships: true,
            max_context_length: 1000,
            feature_options: FeatureExtractionOptions {
                ast_features: true,
                text_features: true,
                structural_features: true,
                behavioral_features: true,
                language_features: true,
            },
            preprocessing: PreprocessingFlags {
                normalize_identifiers: false,
                remove_comments: false,
                tokenize: true,
                stem_tokens: false,
                remove_stop_words: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbols::{Symbol, SymbolKind, Location};
    use crate::parsers::LanguageId;
    use crate::analysis::{CodeNode, CodeMetrics, AnalysisResult};
    use petgraph::Graph;

    fn create_test_analysis() -> AnalysisResult {
        let mut graph = Graph::new();
        
        let symbol = Symbol {
            name: "test_function".to_string(),
            kind: SymbolKind::Function,
            location: Location {
                file_path: "/test/main.rs".to_string(),
                start_line: 5,
                start_column: 0,
                end_line: 15,
                end_column: 1,
            },
            scope_chain: vec![],
            language: LanguageId::Rust,
            documentation: Some("Test function for embeddings".to_string()),
            modifiers: vec!["pub".to_string()],
            signature: Some("fn test_function(x: i32) -> String".to_string()),
        };
        
        let node_data = CodeNode {
            symbol,
            file_path: "/test/main.rs".to_string(),
            metrics: CodeMetrics {
                cyclomatic_complexity: 8,
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
    fn test_embedding_export() {
        let analysis = create_test_analysis();
        let exporter = EmbeddingExporter::new(analysis, "/test".to_string());
        let options = ExportOptions::default();
        let config = EmbeddingExportConfig::default();
        
        let embedding_export = exporter.export_embeddings(&options, &config);
        
        assert_eq!(embedding_export.embeddings.len(), 1);
        let embedding = &embedding_export.embeddings[0];
        
        assert_eq!(embedding.symbol.name, "test_function");
        assert_eq!(embedding.semantic_features.structural_features.cyclomatic_complexity, 8);
        assert!(embedding.embedding_metadata.quality_metrics.completeness > 0.5);
        
        // Check that semantic features were extracted
        assert!(embedding.semantic_features.text_features.token_count > 0);
        assert!(!embedding.semantic_features.text_features.keywords.is_empty());
    }

    #[test]
    fn test_quality_metrics() {
        let analysis = create_test_analysis();
        let exporter = EmbeddingExporter::new(analysis, "/test".to_string());
        let options = ExportOptions::default();
        let config = EmbeddingExportConfig::default();
        
        let embedding_export = exporter.export_embeddings(&options, &config);
        let embedding = &embedding_export.embeddings[0];
        
        let metrics = &embedding.embedding_metadata.quality_metrics;
        
        // Should have high completeness due to signature and documentation
        assert!(metrics.completeness > 0.9);
        assert!(metrics.information_content > 0.0);
    }

    #[test]
    fn test_global_context() {
        let analysis = create_test_analysis();
        let exporter = EmbeddingExporter::new(analysis, "/test".to_string());
        let options = ExportOptions::default();
        let config = EmbeddingExportConfig::default();
        
        let embedding_export = exporter.export_embeddings(&options, &config);
        
        assert!(!embedding_export.global_context.vocabulary.is_empty());
        assert_eq!(embedding_export.global_context.symbol_index.len(), 1);
        assert_eq!(embedding_export.global_context.file_index.len(), 1);
        assert!(embedding_export.global_context.language_distribution.contains_key("Rust"));
    }
}