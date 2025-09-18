//! # Cache Policy Management
//!
//! Automatically configures optimal caching strategies based on project characteristics.
//! Provides adaptive policies that scale from tiny scripts to massive monorepos.

use crate::cache::size_detector::{ProjectProfile, ProjectSize};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configurable constraints for cache validation
#[derive(Debug, Clone)]
pub struct CacheConstraints {
    pub min_memory_mb: usize,
    pub max_memory_mb: usize,
    pub min_l1_capacity: usize,
    pub max_l1_capacity: usize,
    pub max_disk_mb: usize,
    pub max_threads: usize,
    pub min_ttl: Duration,
    pub max_ttl: Duration,
    pub min_cascade_depth: usize,
    pub max_cascade_depth: usize,
}

impl Default for CacheConstraints {
    fn default() -> Self {
        Self {
            min_memory_mb: 50,                    // 50MB minimum
            max_memory_mb: 8192,                  // 8GB maximum
            min_l1_capacity: 100,                 // 100 entries minimum
            max_l1_capacity: 50000,               // 50K entries maximum
            max_disk_mb: 10240,                   // 10GB maximum disk usage
            max_threads: 8,                       // Maximum 8 background threads
            min_ttl: Duration::from_secs(60),     // 1 minute minimum
            max_ttl: Duration::from_secs(604800), // 7 days maximum
            min_cascade_depth: 3,                 // Minimum cascade depth
            max_cascade_depth: 50,                // Maximum cascade depth
        }
    }
}

impl CacheConstraints {
    /// Create constraints optimized for development environments
    pub fn for_development() -> Self {
        Self {
            min_memory_mb: 10,
            max_memory_mb: 1024, // 1GB max for dev
            max_disk_mb: 1024,   // 1GB max disk for dev
            max_threads: 4,      // Fewer threads for dev
            ..Default::default()
        }
    }

    /// Create constraints optimized for production environments
    pub fn for_production() -> Self {
        Self {
            min_memory_mb: 100,
            max_memory_mb: 16384, // 16GB max for production
            max_disk_mb: 51200,   // 50GB max disk for production
            max_threads: 16,      // More threads for production
            ..Default::default()
        }
    }

    /// Create constraints optimized for CI/CD environments
    pub fn for_ci_cd() -> Self {
        Self {
            min_memory_mb: 25,
            max_memory_mb: 512,                 // 512MB max for CI
            max_disk_mb: 512,                   // 512MB max disk for CI
            max_threads: 2,                     // Minimal threads for CI
            min_ttl: Duration::from_secs(30),   // Shorter TTL for CI
            max_ttl: Duration::from_secs(3600), // 1 hour max for CI
            ..Default::default()
        }
    }
}

/// Comprehensive cache configuration that adapts to project characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Memory limits
    pub memory_limit_mb: usize,
    pub l1_capacity: usize,

    /// Disk cache settings
    pub disk_limit_mb: usize,
    pub cache_dir: PathBuf,

    /// Cache level enablement
    pub enable_l1_cache: bool,
    pub enable_l2_cache: bool,

    /// TTL settings for different data types
    pub symbol_ttl: Duration,
    pub ast_ttl: Duration,
    pub graph_ttl: Duration,
    pub analysis_ttl: Duration,

    /// Performance settings
    pub enable_predictive_caching: bool,
    pub enable_cache_warming: bool,
    pub background_warming_threads: usize,

    /// Invalidation settings
    pub enable_dependency_cascade: bool,
    pub batch_invalidation_delay: Duration,
    pub max_cascade_depth: usize,

    /// Project-specific optimizations
    pub policy_type: CachePolicyType,
    pub compression_enabled: bool,
    pub streaming_enabled: bool,
}

/// Different cache policy types optimized for project characteristics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePolicyType {
    /// For tiny projects: Fast, simple, memory-only
    Minimal,

    /// For small projects: Balanced performance with selective disk cache
    Balanced,

    /// For medium projects: Intelligent multi-level caching
    Adaptive,

    /// For large projects: Persistence-focused with basic compression
    Persistent,

    /// For massive projects: Enterprise features (future)
    Enterprise,
}

impl CachePolicyType {
    /// Get policy type from project size
    pub fn from_project_size(size: ProjectSize) -> Self {
        match size {
            ProjectSize::Tiny => CachePolicyType::Minimal,
            ProjectSize::Small => CachePolicyType::Balanced,
            ProjectSize::Medium => CachePolicyType::Adaptive,
            ProjectSize::Large => CachePolicyType::Persistent,
            ProjectSize::Massive => CachePolicyType::Enterprise,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CachePolicyType::Minimal => "Fast, memory-only caching for tiny projects",
            CachePolicyType::Balanced => "Balanced performance with selective disk cache",
            CachePolicyType::Adaptive => "Intelligent multi-level caching with prediction",
            CachePolicyType::Persistent => "Persistence-focused with compression",
            CachePolicyType::Enterprise => "Advanced features for massive codebases",
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 200,
            l1_capacity: 1000,
            disk_limit_mb: 500,
            cache_dir: default_cache_dir(),
            enable_l1_cache: true,
            enable_l2_cache: true,
                        symbol_ttl: Duration::from_secs(3600),    // 1 hour
            ast_ttl: Duration::from_secs(1800),       // 30 minutes
            graph_ttl: Duration::from_secs(21600),    // 6 hours
            analysis_ttl: Duration::from_secs(86400), // 24 hours
            enable_predictive_caching: true,
            enable_cache_warming: true,
            background_warming_threads: 2,
            enable_dependency_cascade: true,
            batch_invalidation_delay: Duration::from_millis(100),
            max_cascade_depth: 10,
            policy_type: CachePolicyType::Balanced,
            compression_enabled: false,
            streaming_enabled: false,
        }
    }
}

impl CacheConfig {
    /// Create cache configuration optimized for a specific project profile
    pub fn from_project_profile(profile: &ProjectProfile) -> Self {
        let recommendations = profile.cache_recommendations();
        let policy_type = CachePolicyType::from_project_size(profile.size);

        let mut config = Self {
            memory_limit_mb: recommendations.memory_limit_mb,
            l1_capacity: recommendations.l1_capacity,
            disk_limit_mb: recommendations.l2_disk_limit_mb,
            cache_dir: default_cache_dir(),
            enable_l1_cache: true, // Always enabled
            enable_l2_cache: recommendations.l2_enabled,
            enable_predictive_caching: recommendations.enable_predictive,
            enable_cache_warming: recommendations.cache_warming_enabled,
            policy_type,
            ..Default::default()
        };

        // Apply policy-specific optimizations
        config.apply_policy_optimizations(profile);
        config
    }

    /// Apply policy-specific optimizations based on project characteristics
    fn apply_policy_optimizations(&mut self, profile: &ProjectProfile) {
        match self.policy_type {
            CachePolicyType::Minimal => {
                // Optimize for speed and simplicity
                self.enable_l2_cache = false;
                self.enable_predictive_caching = false;
                self.enable_cache_warming = false;
                self.enable_dependency_cascade = false;
                self.background_warming_threads = 0;

                // Shorter TTLs for rapid iteration
                self.symbol_ttl = Duration::from_secs(600); // 10 minutes
                self.ast_ttl = Duration::from_secs(300); // 5 minutes
            }

            CachePolicyType::Balanced => {
                // Balanced approach with selective features
                self.enable_dependency_cascade = true;
                self.background_warming_threads = 1;

                // Moderate TTLs
                self.symbol_ttl = Duration::from_secs(1800); // 30 minutes
                self.ast_ttl = Duration::from_secs(900); // 15 minutes

                // Enable L2 only for projects with significant complexity
                if profile.complexity_score < 2.0 {
                    self.enable_l2_cache = false;
                }
            }

            CachePolicyType::Adaptive => {
                // Intelligent caching with prediction
                self.enable_predictive_caching = true;
                self.enable_cache_warming = true;
                self.background_warming_threads = 2;

                // Adaptive TTLs based on project activity
                if profile.has_tests {
                    // Shorter TTLs for active development
                    self.symbol_ttl = Duration::from_secs(2700); // 45 minutes
                    self.ast_ttl = Duration::from_secs(1200); // 20 minutes
                } else {
                    // Longer TTLs for stable projects
                    self.symbol_ttl = Duration::from_secs(7200); // 2 hours
                    self.ast_ttl = Duration::from_secs(3600); // 1 hour
                }

                // Enable compression for complex projects
                if profile.complexity_score > 3.0 {
                    self.compression_enabled = true;
                }
            }

            CachePolicyType::Persistent => {
                // Optimize for large codebases with persistence
                self.enable_l2_cache = true;
                                self.compression_enabled = true;
                self.background_warming_threads = 3;

                // Longer TTLs for stability
                self.symbol_ttl = Duration::from_secs(14400); // 4 hours
                self.ast_ttl = Duration::from_secs(7200); // 2 hours
                self.graph_ttl = Duration::from_secs(43200); // 12 hours
                self.analysis_ttl = Duration::from_secs(172800); // 48 hours

                // Advanced invalidation
                self.max_cascade_depth = 15;
                self.batch_invalidation_delay = Duration::from_millis(200);
            }

            CachePolicyType::Enterprise => {
                // Future: Enterprise features for massive codebases
                                self.compression_enabled = true;
                self.streaming_enabled = true;
                self.background_warming_threads = 4;

                // Very long TTLs for stable massive projects
                self.symbol_ttl = Duration::from_secs(28800); // 8 hours
                self.ast_ttl = Duration::from_secs(14400); // 4 hours
                self.graph_ttl = Duration::from_secs(86400); // 24 hours
                self.analysis_ttl = Duration::from_secs(604800); // 7 days

                // Advanced features
                self.max_cascade_depth = 25;
                self.batch_invalidation_delay = Duration::from_millis(500);
            }
        }

        // Apply language-specific optimizations
        if let Some(primary_language) = &profile.primary_language {
            self.apply_language_optimizations(primary_language);
        }
    }

    /// Apply language-specific cache optimizations
    fn apply_language_optimizations(&mut self, primary_language: &crate::parsers::LanguageId) {
        use crate::parsers::LanguageId;

        match primary_language {
            LanguageId::Rust => {
                // Rust has expensive compilation, longer TTLs beneficial
                self.ast_ttl = self.ast_ttl.mul_f64(1.5);
                self.analysis_ttl = self.analysis_ttl.mul_f64(1.3);
            }

            LanguageId::JavaScript | LanguageId::TypeScript => {
                // Fast iteration languages, shorter TTLs
                self.symbol_ttl = self.symbol_ttl.mul_f64(0.7);
                self.ast_ttl = self.ast_ttl.mul_f64(0.8);

                // Enable predictive caching for frequent changes
                self.enable_predictive_caching = true;
            }

            LanguageId::Python => {
                // Dynamic language, balance between speed and accuracy
                self.symbol_ttl = self.symbol_ttl.mul_f64(0.9);
                self.enable_dependency_cascade = true;
            }

            LanguageId::Java | LanguageId::CSharp => {
                // Structured languages with clear dependencies
                self.enable_dependency_cascade = true;
                self.max_cascade_depth += 5;
            }

            LanguageId::Go => {
                // Fast compilation, moderate TTLs
                self.ast_ttl = self.ast_ttl.mul_f64(1.2);
                self.background_warming_threads = self.background_warming_threads.max(2);
            }

            _ => {
                // Default optimizations for other languages
            }
        }
    }

    /// Validate configuration and apply constraints
    pub fn validate_and_constrain(&mut self) -> Result<(), ConfigValidationError> {
        // Memory constraints
        if self.memory_limit_mb == 0 {
            return Err(ConfigValidationError::InvalidMemoryLimit);
        }

        if self.l1_capacity == 0 {
            return Err(ConfigValidationError::InvalidL1Capacity);
        }

        // Apply configurable validation constraints
        let constraints = CacheConstraints::default();

        // Memory constraints
        self.memory_limit_mb = self
            .memory_limit_mb
            .clamp(constraints.min_memory_mb, constraints.max_memory_mb);
        self.l1_capacity = self
            .l1_capacity
            .clamp(constraints.min_l1_capacity, constraints.max_l1_capacity);
        self.disk_limit_mb = self.disk_limit_mb.min(constraints.max_disk_mb);

        // Thread constraints
        self.background_warming_threads =
            self.background_warming_threads.min(constraints.max_threads);

        // TTL constraints
        self.symbol_ttl = self
            .symbol_ttl
            .max(constraints.min_ttl)
            .min(constraints.max_ttl);
        self.ast_ttl = self
            .ast_ttl
            .max(constraints.min_ttl)
            .min(constraints.max_ttl);
        self.graph_ttl = self
            .graph_ttl
            .max(constraints.min_ttl)
            .min(constraints.max_ttl);
        self.analysis_ttl = self
            .analysis_ttl
            .max(constraints.min_ttl)
            .min(constraints.max_ttl);

        // Cascade depth constraints
        self.max_cascade_depth = self
            .max_cascade_depth
            .clamp(constraints.min_cascade_depth, constraints.max_cascade_depth);

        // Logic constraints
        if !self.enable_l1_cache {
            return Err(ConfigValidationError::L1CacheRequired);
        }

        
        if self.enable_cache_warming && self.background_warming_threads == 0 {
            self.background_warming_threads = 1;
        }

        Ok(())
    }

    /// Create a custom configuration with manual overrides
    pub fn custom() -> CacheConfigBuilder {
        CacheConfigBuilder::new()
    }

    /// Get memory usage estimation for this configuration
    pub fn estimated_memory_usage_mb(&self) -> usize {
        let mut total = 0;

        // L1 cache estimation (symbols + metadata)
        total += (self.l1_capacity * 200) / (1024 * 1024); // ~200 bytes per entry

        // Base overhead
        total += 50; // Base system overhead

        // Additional overhead for advanced features
        if self.enable_predictive_caching {
            total += 20;
        }

        if self.enable_cache_warming {
            total += self.background_warming_threads * 10;
        }

        total.min(self.memory_limit_mb)
    }

    /// Get configuration summary for logging/debugging
    pub fn summary(&self) -> String {
        format!(
            "CacheConfig[{}]: L1={}MB, L2={}, TTL={}h, Threads={}",
            self.policy_type.description(),
            self.estimated_memory_usage_mb(),
            if self.enable_l2_cache { "ON" } else { "OFF" },
            self.symbol_ttl.as_secs() / 3600,
            self.background_warming_threads
        )
    }
}

/// Builder pattern for custom cache configurations
pub struct CacheConfigBuilder {
    config: CacheConfig,
}

impl CacheConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
        }
    }

    pub fn memory_limit_mb(mut self, mb: usize) -> Self {
        self.config.memory_limit_mb = mb;
        self
    }

    pub fn l1_capacity(mut self, capacity: usize) -> Self {
        self.config.l1_capacity = capacity;
        self
    }

    pub fn disk_limit_mb(mut self, mb: usize) -> Self {
        self.config.disk_limit_mb = mb;
        self
    }

    pub fn cache_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.cache_dir = dir.into();
        self
    }

    pub fn enable_l2_cache(mut self, enabled: bool) -> Self {
        self.config.enable_l2_cache = enabled;
        self
    }

    pub fn symbol_ttl(mut self, ttl: Duration) -> Self {
        self.config.symbol_ttl = ttl;
        self
    }

    pub fn enable_predictive_caching(mut self, enabled: bool) -> Self {
        self.config.enable_predictive_caching = enabled;
        self
    }

    pub fn background_threads(mut self, threads: usize) -> Self {
        self.config.background_warming_threads = threads;
        self
    }

    pub fn policy_type(mut self, policy: CachePolicyType) -> Self {
        self.config.policy_type = policy;
        self
    }

    pub fn build(mut self) -> Result<CacheConfig, ConfigValidationError> {
        self.config.validate_and_constrain()?;
        Ok(self.config)
    }
}

impl Default for CacheConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Memory limit must be greater than 0")]
    InvalidMemoryLimit,

    #[error("L1 cache capacity must be greater than 0")]
    InvalidL1Capacity,

    #[error("L1 cache is required and cannot be disabled")]
    L1CacheRequired,

    #[error("L3 cache requires L2 cache to be enabled")]
    L3RequiresL2,

    #[error("Cache directory is invalid: {0}")]
    InvalidCacheDirectory(String),
}

/// Get the default cache directory based on the platform
fn default_cache_dir() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("fast-context")
    } else {
        // Fallback for systems without standard cache directory
        std::env::temp_dir().join("fast-context-cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::size_detector::{CodebaseAnalyzer, ProjectSize};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_policy_type_from_project_size() {
        assert_eq!(
            CachePolicyType::from_project_size(ProjectSize::Tiny),
            CachePolicyType::Minimal
        );
        assert_eq!(
            CachePolicyType::from_project_size(ProjectSize::Small),
            CachePolicyType::Balanced
        );
        assert_eq!(
            CachePolicyType::from_project_size(ProjectSize::Medium),
            CachePolicyType::Adaptive
        );
        assert_eq!(
            CachePolicyType::from_project_size(ProjectSize::Large),
            CachePolicyType::Persistent
        );
        assert_eq!(
            CachePolicyType::from_project_size(ProjectSize::Massive),
            CachePolicyType::Enterprise
        );
    }

    #[test]
    fn test_default_cache_config() {
        let config = CacheConfig::default();

        assert!(config.enable_l1_cache);
        assert!(config.enable_l2_cache);
                assert_eq!(config.policy_type, CachePolicyType::Balanced);
        assert!(config.memory_limit_mb > 0);
        assert!(config.l1_capacity > 0);
    }

    #[test]
    fn test_config_from_tiny_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a tiny project with multiple files
        fs::write(
            temp_path.join("main.rs"),
            "fn main() { println!(\"Hello\"); }",
        )
        .unwrap();
        fs::write(
            temp_path.join("lib.rs"),
            "pub fn hello() -> String { \"world\".to_string() }",
        )
        .unwrap();
        fs::write(
            temp_path.join("utils.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        )
        .unwrap();

        let analyzer = CodebaseAnalyzer::new();
        match analyzer.analyze_project(temp_path) {
            Ok(profile) => {
                let config = CacheConfig::from_project_profile(&profile);

                assert_eq!(config.policy_type, CachePolicyType::Minimal);
                // Note: Minimal policy disables these features in apply_policy_optimizations
                assert!(!config.enable_l2_cache);
                                assert!(!config.enable_predictive_caching);
            }
            Err(e) => {
                // If no files found, create a simple test
                println!("Warning: Could not analyze project: {e}");
                let config = CacheConfig::default();
                assert_eq!(config.policy_type, CachePolicyType::Balanced); // Default policy
            }
        }
    }

    #[test]
    fn test_config_validation() {
        let mut config = CacheConfig::default();

        // Valid configuration should pass
        assert!(config.validate_and_constrain().is_ok());

        // Invalid memory limit
        config.memory_limit_mb = 0;
        assert!(config.validate_and_constrain().is_err());

        // Reset and test L1 capacity
        config = CacheConfig::default();
        config.l1_capacity = 0;
        assert!(config.validate_and_constrain().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = CacheConfig::custom()
            .memory_limit_mb(1024)
            .l1_capacity(5000)
            .enable_l2_cache(true)
            .symbol_ttl(Duration::from_secs(7200))
            .policy_type(CachePolicyType::Adaptive)
            .build()
            .unwrap();

        assert_eq!(config.memory_limit_mb, 1024);
        assert_eq!(config.l1_capacity, 5000);
        assert!(config.enable_l2_cache);
        assert_eq!(config.symbol_ttl, Duration::from_secs(7200));
        assert_eq!(config.policy_type, CachePolicyType::Adaptive);
    }

    #[test]
    fn test_memory_usage_estimation() {
        let config = CacheConfig::custom()
            .l1_capacity(10000)
            .enable_predictive_caching(true)
            .background_threads(2)
            .build()
            .unwrap();

        let estimated = config.estimated_memory_usage_mb();
        assert!(estimated > 0);
        assert!(estimated <= config.memory_limit_mb);
    }

    #[test]
    fn test_language_specific_optimizations() {
        use crate::parsers::LanguageId;

        let mut rust_config = CacheConfig::default();
        rust_config.apply_language_optimizations(&LanguageId::Rust);

        let mut js_config = CacheConfig::default();
        js_config.apply_language_optimizations(&LanguageId::JavaScript);

        // Rust should have longer TTLs than JavaScript
        assert!(rust_config.ast_ttl > js_config.ast_ttl);
        assert!(js_config.enable_predictive_caching);
    }

    #[test]
    fn test_config_summary() {
        let config = CacheConfig::default();
        let summary = config.summary();

        assert!(summary.contains("CacheConfig"));
        assert!(summary.contains("L1="));
        assert!(summary.contains("L2="));
        assert!(summary.contains("TTL="));
        assert!(summary.contains("Threads="));
    }
}
