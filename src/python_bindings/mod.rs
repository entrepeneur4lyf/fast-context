//! # Python Bindings Module
//!
//! Comprehensive Python integration using PyO3 for the Fast-Context analyzer.
//! Provides high-performance Python APIs for all core functionality.

pub mod cache;
pub mod config;
pub mod export;
pub mod graph;
pub mod query;
pub mod util;

// Re-export main Python binding interface
pub use python_bindings::*;