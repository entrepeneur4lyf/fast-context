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

// 🏗️ NEW MODULAR ARCHITECTURE - Proper separation of concerns
pub mod analyzer;    // FastContextAnalyzer implementation
pub mod graph;       // Graph algorithms and data structures
pub mod utils;       // Utility functions
pub mod domains;     // Domain separation for architectural harmony

// 📦 CORE MODULES - Well-organized functionality
pub mod analysis;    // Code analysis and graph construction
pub mod cache;       // Intelligent caching system
pub mod error_tracking; // Error tracking and reporting system
pub mod export;      // Export & serialization system
pub mod parsers;     // Tree-sitter language parsers
pub mod query;       // Query interface for AI assistants
pub mod symbols;     // Symbol extraction and management
pub mod watcher;     // File system monitoring

// 🎯 RE-EXPORTS - Clean public API
pub use analyzer::{FastContextAnalyzer, AnalyzerConfig, AnalysisResultJs, QueryResultJs};
pub use graph::{RustworkxGraph, RustworkxDiGraph};
pub use utils::{get_version, get_supported_languages, detect_language, check_configuration};
