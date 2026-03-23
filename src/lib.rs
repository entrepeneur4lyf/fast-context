//! # Fast-Context: Intelligent Codebase Analysis Engine
//!
//! Fast-Context transforms complex codebases into comprehensive knowledge graphs that empower
//! coding assistants with deep semantic understanding, causal analysis, and real-time intelligence.
//!
//! ## Core Architecture
//!
//! ### Graph Algorithm Foundation (80+ Algorithms)
//! The comprehensive graph algorithm suite provides the computational engine for code analysis:
//!
//! **Shortest Path Algorithms**: A*, Bellman-Ford, K-shortest paths, all paths enumeration
//! **Centrality Measures**: Betweenness, eigenvector, Katz centrality for code importance analysis
//! **Graph Operations**: Union, complement, tensor/cartesian products for code relationship modeling
//! **Traversal Algorithms**: Complete BFS/DFS suites for dependency tracing and impact analysis
//! **Specialized Algorithms**: SCC condensation, ancestors/descendants for call graph analysis
//! **Performance Optimizations**: Parallel algorithms, memory-efficient streaming, intelligent caching
//!
//! ### Codebase Analysis Engine (In Development)
//! - **Multi-language Parsing**: 20+ programming languages via Tree-sitter
//! - **Symbol Extraction**: Functions, classes, variables, imports with full context
//! - **Dependency Graphs**: Call graphs, import graphs, data flow analysis
//! - **Real-time Updates**: File watching with incremental graph updates
//! - **Intelligent Caching**: Adaptive caching strategies from small projects to large monorepos
//! - **AI Assistant APIs**: Query interfaces designed for coding assistants and LLMs
//!
//! ## Intelligent Caching Strategy
//!
//! | Project Size | Files | Memory | Disk Cache | Features |
//! |--------------|-------|--------|------------|----------|
//! | Small | <1K | <200MB | <100MB | LRU + selective disk |
//! | Medium | 1K-10K | <500MB | <500MB | Multi-level cache |
//! | Large | >10K | <1GB | <1GB | Basic disk persistence |
//!
//! ## Use Cases
//!
//! - **Impact Analysis**: Trace how code changes propagate through the codebase
//! - **Semantic Search**: Find symbols, references, and usage patterns across languages
//! - **Dependency Visualization**: Understand complex code relationships and architecture
//! - **Refactoring Safety**: Identify all affected code before making changes
//! - **Code Intelligence**: Power AI assistants with deep codebase understanding
//!
//! The existing graph algorithms serve as the high-performance foundation that enables
//! sophisticated code relationship modeling, dependency analysis, and impact assessment.

// All imports moved to respective modules for better organization

pub mod types;
pub mod errors;      // Comprehensive error management system
pub mod validation;  // Input validation and security

// NEW MODULAR ARCHITECTURE - Proper separation of concerns
#[cfg(feature = "nodejs")]
pub mod analyzer;    // FastContextAnalyzer implementation
pub mod graph;       // Graph algorithms and data structures
pub mod utils;
mod test_display;       // Utility functions
pub mod domains;     // Domain separation for architectural harmony

// CORE MODULES - Well-organized functionality
pub mod analysis;    // Code analysis and graph construction
pub mod cache;       // Intelligent caching system
// error_tracking is now part of the errors module
pub mod export;      // Export & serialization system
pub mod parsers;     // Tree-sitter language parsers
pub mod query;       // Query interface for AI assistants
pub mod symbols;     // Symbol extraction and management
pub mod watcher;     // File system monitoring
pub mod core;        // Shared Send + Sync CoreAnalyzer

// PYTHON BINDINGS - Optional Python integration
#[cfg(feature = "python")]
pub mod python_bindings;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_util.rs"]
pub mod python_bindings_util;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_graph.rs"]
pub mod python_bindings_graph;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_export.rs"]
pub mod python_bindings_export;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_query.rs"]
pub mod python_bindings_query;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_config.rs"]
pub mod python_bindings_config;
#[cfg(feature = "python")]
#[path = "python_bindings/python_bindings_cache.rs"]
pub mod python_bindings_cache;

// RE-EXPORTS - Clean public API
#[cfg(feature = "nodejs")]
pub use analyzer::{FastContextAnalyzer, AnalyzerConfig, AnalysisResultJs, QueryResultJs};
#[cfg(feature = "nodejs")]
pub use graph::{RustworkxGraph, RustworkxDiGraph};
#[cfg(feature = "nodejs")]
pub use utils::{get_version, get_supported_languages, detect_language, check_configuration, should_ignore_file_default};

// Additional re-exports for testing and advanced usage
pub use core::CoreAnalyzer;
pub use errors::{FastContextError, FastContextResult};
pub use parsers::LanguageId;
