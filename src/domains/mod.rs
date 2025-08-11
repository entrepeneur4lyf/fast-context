//! # Domain Architecture
//!
//! This module implements the core domain separation for architectural harmony:
//! 
//! - **Graph Domain**: Pure graph algorithms and data structures
//! - **Analysis Domain**: Codebase analysis and intelligence features
//! - **Core Domain**: Shared utilities and abstractions
//!
//! Each domain is self-contained with clear interfaces and minimal coupling.

pub mod core;
pub mod graph;
pub mod analysis;

// Re-export domain APIs for unified access
pub use core::*;
pub use graph::GraphEngine;
pub use analysis::AnalysisEngine;

/// Domain trait for consistent behavior across all domains
pub trait Domain {
    type Config;
    type Error;
    
    /// Initialize the domain with configuration
    fn initialize(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
    
    /// Get domain name for logging and debugging
    fn domain_name(&self) -> &'static str;
    
    /// Check if domain is healthy and operational
    fn health_check(&self) -> Result<(), Self::Error>;
}

/// Plugin trait for extensible architecture
pub trait Plugin {
    type Config;
    type Error;
    
    /// Plugin identifier
    fn plugin_id(&self) -> &'static str;
    
    /// Initialize plugin with configuration
    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    
    /// Check if plugin is compatible with current system
    fn is_compatible(&self) -> bool;
}

/// Event system for domain communication
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Graph domain events
    GraphCreated { graph_id: String },
    GraphModified { graph_id: String },
    GraphDeleted { graph_id: String },
    
    /// Analysis domain events
    AnalysisStarted { project_path: String },
    AnalysisCompleted { project_path: String, duration_ms: u64 },
    AnalysisError { project_path: String, error: String },
    
    /// Core system events
    CacheUpdated { key: String },
    ConfigChanged { domain: String },
}

/// Event handler trait for domain communication
pub trait EventHandler {
    fn handle_event(&mut self, event: DomainEvent) -> Result<(), Box<dyn std::error::Error>>;
}

/// Registry for managing domain instances and plugins
pub struct DomainRegistry {
    graph_engine: Option<GraphEngine>,
    analysis_engine: Option<AnalysisEngine>,
    event_handlers: Vec<Box<dyn EventHandler>>,
}

impl DomainRegistry {
    pub fn new() -> Self {
        Self {
            graph_engine: None,
            analysis_engine: None,
            event_handlers: Vec::new(),
        }
    }
    
    /// Register graph engine
    pub fn register_graph_engine(&mut self, engine: GraphEngine) {
        self.graph_engine = Some(engine);
    }
    
    /// Register analysis engine
    pub fn register_analysis_engine(&mut self, engine: AnalysisEngine) {
        self.analysis_engine = Some(engine);
    }
    
    /// Add event handler
    pub fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.event_handlers.push(handler);
    }
    
    /// Get graph engine reference
    pub fn graph_engine(&self) -> Option<&GraphEngine> {
        self.graph_engine.as_ref()
    }
    
    /// Get analysis engine reference
    pub fn analysis_engine(&self) -> Option<&AnalysisEngine> {
        self.analysis_engine.as_ref()
    }
    
    /// Broadcast event to all handlers
    pub fn broadcast_event(&mut self, event: DomainEvent) {
        for handler in &mut self.event_handlers {
            if let Err(e) = handler.handle_event(event.clone()) {
                eprintln!("Event handler error: {e}");
            }
        }
    }

    /// Get count of active graphs
    pub fn get_active_graph_count(&self) -> u32 {
        if let Some(graph_engine) = &self.graph_engine {
            graph_engine.get_graph_count()
        } else {
            0
        }
    }

    /// Get count of active analysis sessions
    pub fn get_active_analysis_count(&self) -> u32 {
        if let Some(analysis_engine) = &self.analysis_engine {
            analysis_engine.get_session_count()
        } else {
            0
        }
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}
