//! Advanced Configuration Management for Python SDK
//!
//! Provides sophisticated configuration management with cache policies,
//! performance tuning, and dynamic configuration updates

use crate::python_bindings::{PyLocation, PyScope};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cache policy configuration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyCachePolicy {
    #[pyo3(get, set)]
    pub policy_type: String,
    
    #[pyo3(get, set)]
    pub max_memory_mb: u32,
    
    #[pyo3(get, set)]
    pub max_disk_gb: f64,
    
    #[pyo3(get, set)]
    pub ttl_seconds: u64,
    
    #[pyo3(get, set)]
    pub enable_compression: bool,
    
    #[pyo3(get, set)]
    pub enable_incremental: bool,
    
    #[pyo3(get, set)]
    pub prediction_enabled: bool,
    
    #[pyo3(get, set)]
    pub prefetch_patterns: Vec<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyCachePolicy {
    #[new]
    #[pyo3(signature = (policy_type="conservative".to_string(), max_memory_mb=512, max_disk_gb=1.0, ttl_seconds=3600, enable_compression=true, enable_incremental=true, prediction_enabled=false, prefetch_patterns=Vec::new()))]
    pub fn new(
        policy_type: String,
        max_memory_mb: u32,
        max_disk_gb: f64,
        ttl_seconds: u64,
        enable_compression: bool,
        enable_incremental: bool,
        prediction_enabled: bool,
        prefetch_patterns: Vec<String>,
    ) -> Self {
        Self {
            policy_type,
            max_memory_mb,
            max_disk_gb,
            ttl_seconds,
            enable_compression,
            enable_incremental,
            prediction_enabled,
            prefetch_patterns,
        }
    }
    
    /// Create conservative cache policy (saves memory)
    #[staticmethod]
    #[pyo3(name = "conservative")]
    pub fn conservative() -> Self {
        Self {
            policy_type: "conservative".to_string(),
            max_memory_mb: 256,
            max_disk_gb: 0.5,
            ttl_seconds: 7200,
            enable_compression: true,
            enable_incremental: true,
            prediction_enabled: false,
            prefetch_patterns: Vec::new(),
        }
    }
    
    /// Create aggressive cache policy (maximizes performance)
    #[staticmethod]
    #[pyo3(name = "aggressive")]
    pub fn aggressive() -> Self {
        Self {
            policy_type: "aggressive".to_string(),
            max_memory_mb: 2048,
            max_disk_gb: 5.0,
            ttl_seconds: 14400,
            enable_compression: false,
            enable_incremental: true,
            prediction_enabled: true,
            prefetch_patterns: vec![
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.rs".to_string(),
            ],
        }
    }
    
    /// Create adaptive cache policy (balances memory and performance)
    #[staticmethod]
    #[pyo3(name = "adaptive")]
    pub fn adaptive() -> Self {
        Self {
            policy_type: "adaptive".to_string(),
            max_memory_mb: 1024,
            max_disk_gb: 2.0,
            ttl_seconds: 10800,
            enable_compression: true,
            enable_incremental: true,
            prediction_enabled: true,
            prefetch_patterns: vec![
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
            ],
        }
    }
    
    /// Create minimal cache policy (for testing/debugging)
    #[staticmethod]
    #[pyo3(name = "minimal")]
    pub fn minimal() -> Self {
        Self {
            policy_type: "minimal".to_string(),
            max_memory_mb: 64,
            max_disk_gb: 0.1,
            ttl_seconds: 600,
            enable_compression: false,
            enable_incremental: false,
            prediction_enabled: false,
            prefetch_patterns: Vec::new(),
        }
    }
}

/// Performance configuration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyPerformanceConfig {
    #[pyo3(get, set)]
    pub parallel_processing: bool,
    
    #[pyo3(get, set)]
    pub worker_threads: u32,
    
    #[pyo3(get, set)]
    pub chunk_size: usize,
    
    #[pyo3(get, set)]
    pub enable_rayon: bool,
    
    #[pyo3(get, set)]
    pub memory_limit_mb: u32,
    
    #[pyo3(get, set)]
    pub enable_gc_optimization: bool,
    
    #[pyo3(get, set)]
    pub io_timeout_ms: u64,
    
    #[pyo3(get, set)]
    pub enable_async_io: bool,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPerformanceConfig {
    #[new]
    #[pyo3(signature = (parallel_processing=true, worker_threads=4, chunk_size=1000, enable_rayon=true, memory_limit_mb=4096, enable_gc_optimization=true, io_timeout_ms=30000, enable_async_io=true))]
    pub fn new(
        parallel_processing: bool,
        worker_threads: u32,
        chunk_size: usize,
        enable_rayon: bool,
        memory_limit_mb: u32,
        enable_gc_optimization: bool,
        io_timeout_ms: u64,
        enable_async_io: bool,
    ) -> Self {
        Self {
            parallel_processing,
            worker_threads,
            chunk_size,
            enable_rayon,
            memory_limit_mb,
            enable_gc_optimization,
            io_timeout_ms,
            enable_async_io,
        }
    }
    
    /// Create performance config for small projects
    #[staticmethod]
    #[pyo3(name = "for_small_project")]
    pub fn for_small_project() -> Self {
        Self {
            parallel_processing: false,
            worker_threads: 2,
            chunk_size: 500,
            enable_rayon: false,
            memory_limit_mb: 1024,
            enable_gc_optimization: true,
            io_timeout_ms: 15000,
            enable_async_io: false,
        }
    }
    
    /// Create performance config for medium projects
    #[staticmethod]
    #[pyo3(name = "for_medium_project")]
    pub fn for_medium_project() -> Self {
        Self {
            parallel_processing: true,
            worker_threads: 4,
            chunk_size: 1000,
            enable_rayon: true,
            memory_limit_mb: 2048,
            enable_gc_optimization: true,
            io_timeout_ms: 30000,
            enable_async_io: true,
        }
    }
    
    /// Create performance config for large projects
    #[staticmethod]
    #[pyo3(name = "for_large_project")]
    pub fn for_large_project() -> Self {
        Self {
            parallel_processing: true,
            worker_threads: 8,
            chunk_size: 2000,
            enable_rayon: true,
            memory_limit_mb: 8192,
            enable_gc_optimization: true,
            io_timeout_ms: 60000,
            enable_async_io: true,
        }
    }
}

/// Advanced analyzer configuration
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyAdvancedAnalyzerConfig {
    #[pyo3(get, set)]
    pub project_root: String,
    
    #[pyo3(get, set)]
    pub languages: Vec<String>,
    
    #[pyo3(get, set)]
    pub ignore_patterns: Vec<String>,
    
    #[pyo3(get, set)]
    pub cache_policy: PyCachePolicy,
    
    #[pyo3(get, set)]
    pub performance_config: PyPerformanceConfig,
    
    #[pyo3(get, set)]
    pub enable_experimental_architecture: bool,
    
    #[pyo3(get, set)]
    pub enable_ai_assistant_integration: bool,
    
    #[pyo3(get, set)]
    pub enable_real_time_updates: bool,
    
    #[pyo3(get, set)]
    pub enable_cross_language_analysis: bool,
    
    #[pyo3(get, set)]
    pub max_file_size_mb: u32,
    
    #[pyo3(get, set)]
    pub enable_syntax_validation: bool,
    
    #[pyo3(get, set)]
    pub enable_semantic_analysis: bool,
    
    #[pyo3(get, set)]
    pub custom_extractors: HashMap<String, String>,
    
    #[pyo3(get, set)]
    pub environment_variables: HashMap<String, String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyAdvancedAnalyzerConfig {
    #[new]
    #[pyo3(signature = (project_root, languages, ignore_patterns, cache_policy=None, performance_config=None, enable_experimental_architecture=false, enable_ai_assistant_integration=false, enable_real_time_updates=false, enable_cross_language_analysis=true, max_file_size_mb=10, enable_syntax_validation=true, enable_semantic_analysis=true, custom_extractors=HashMap::new(), environment_variables=HashMap::new()))]
    pub fn new(
        project_root: String,
        languages: Vec<String>,
        ignore_patterns: Vec<String>,
        cache_policy: Option<PyCachePolicy>,
        performance_config: Option<PyPerformanceConfig>,
        enable_experimental_architecture: bool,
        enable_ai_assistant_integration: bool,
        enable_real_time_updates: bool,
        enable_cross_language_analysis: bool,
        max_file_size_mb: u32,
        enable_syntax_validation: bool,
        enable_semantic_analysis: bool,
        custom_extractors: HashMap<String, String>,
        environment_variables: HashMap<String, String>,
    ) -> Self {
        Self {
            project_root,
            languages,
            ignore_patterns,
            cache_policy: cache_policy.unwrap_or_else(PyCachePolicy::adaptive),
            performance_config: performance_config.unwrap_or_else(PyPerformanceConfig::for_medium_project),
            enable_experimental_architecture,
            enable_ai_assistant_integration,
            enable_real_time_updates,
            enable_cross_language_analysis,
            max_file_size_mb,
            enable_syntax_validation,
            enable_semantic_analysis,
            custom_extractors,
            environment_variables,
        }
    }
    
    /// Create config from directory (auto-detect settings)
    #[staticmethod]
    #[pyo3(name = "from_directory")]
    pub fn from_directory(project_root: String) -> PyResult<Self> {
        let project_path = PathBuf::from(&project_root);
        
        // Auto-detect project type and settings
        let languages = Self::detect_languages(&project_path)?;
        let ignore_patterns = Self::detect_ignore_patterns(&project_path)?;
        
        // Estimate project size for optimal configuration
        let estimated_size = Self::estimate_project_size(&project_path)?;
        let (cache_policy, performance_config) = Self::recommend_config(estimated_size);
        
        Ok(Self {
            project_root,
            languages,
            ignore_patterns,
            cache_policy,
            performance_config,
            enable_experimental_architecture: false,
            enable_ai_assistant_integration: true,
            enable_real_time_updates: false,
            enable_cross_language_analysis: true,
            max_file_size_mb: 10,
            enable_syntax_validation: true,
            enable_semantic_analysis: true,
            custom_extractors: HashMap::new(),
            environment_variables: HashMap::new(),
        })
    }
    
    /// Validate configuration
    pub fn validate(&self) -> PyResult<()> {
        // Validate project root exists
        if !std::path::Path::new(&self.project_root).exists() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Project root does not exist: {}", self.project_root)
            ));
        }
        
        // Validate memory limits
        if self.cache_policy.max_memory_mb > self.performance_config.memory_limit_mb {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cache memory limit cannot exceed overall memory limit"
            ));
        }
        
        // Validate worker threads
        if self.performance_config.worker_threads == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Worker threads must be at least 1"
            ));
        }
        
        // Validate languages
        if self.languages.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "At least one language must be specified"
            ));
        }
        
        Ok(())
    }
    
    /// Get estimated memory usage
    pub fn estimate_memory_usage(&self) -> u32 {
        let base_usage = 100; // Base memory in MB
        let cache_usage = self.cache_policy.max_memory_mb;
        let analysis_usage = self.languages.len() as u32 * 50; // 50MB per language
        let parallel_usage = if self.performance_config.parallel_processing {
            self.performance_config.worker_threads * 100
        } else {
            0
        };
        
        base_usage + cache_usage + analysis_usage + parallel_usage
    }
    
    /// Optimize configuration for current system
    pub fn optimize_for_system(&mut self) -> PyResult<()> {
        // Detect system capabilities
        let available_memory = Self::get_system_memory()?;
        let cpu_cores = Self::get_cpu_cores()?;
        
        // Optimize memory usage
        let recommended_memory = (available_memory as f32 * 0.7) as u32;
        if self.performance_config.memory_limit_mb > recommended_memory {
            self.performance_config.memory_limit_mb = recommended_memory;
            
            // Adjust cache memory proportionally
            let cache_ratio = self.cache_policy.max_memory_mb as f32 / 
                             (self.performance_config.memory_limit_mb as f32 + self.cache_policy.max_memory_mb as f32);
            self.cache_policy.max_memory_mb = (recommended_memory as f32 * cache_ratio) as u32;
        }
        
        // Optimize worker threads
        let optimal_threads = (cpu_cores / 2).max(1).min(8);
        self.performance_config.worker_threads = optimal_threads;
        
        // Enable parallel processing for multi-core systems
        self.performance_config.parallel_processing = cpu_cores > 1;
        
        Ok(())
    }
    
    /// Convert to basic analyzer config (for backward compatibility)
    pub fn to_basic_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("project_root".to_string(), self.project_root.clone());
        config.insert("languages".to_string(), self.languages.join(","));
        config.insert("ignore_patterns".to_string(), self.ignore_patterns.join(","));
        config
    }
    
    /// Create optimized config for specific use case
    #[staticmethod]
    #[pyo3(name = "for_ai_assistant")]
    pub fn for_ai_assistant(project_root: String) -> PyResult<Self> {
        let mut config = Self::from_directory(project_root)?;
        
        // Optimize for AI assistant usage
        config.enable_ai_assistant_integration = true;
        config.enable_cross_language_analysis = true;
        config.enable_semantic_analysis = true;
        config.cache_policy.prediction_enabled = true;
        config.cache_policy.prefetch_patterns = vec![
            "**/*.py".to_string(),
            "**/*.js".to_string(),
            "**/*.ts".to_string(),
            "**/*.md".to_string(),
            "**/*.json".to_string(),
        ];
        
        config.optimize_for_system()?;
        Ok(config)
    }
    
    #[staticmethod]
    #[pyo3(name = "for_ci_cd")]
    pub fn for_ci_cd(project_root: String) -> PyResult<Self> {
        let mut config = Self::from_directory(project_root)?;
        
        // Optimize for CI/CD pipeline
        config.enable_real_time_updates = false;
        config.performance_config.parallel_processing = true;
        config.performance_config.worker_threads = Self::get_cpu_cores()?;
        config.cache_policy = PyCachePolicy::aggressive();
        
        config.optimize_for_system()?;
        Ok(config)
    }
    
    #[staticmethod]
    #[pyo3(name = "for_development")]
    pub fn for_development(project_root: String) -> PyResult<Self> {
        let mut config = Self::from_directory(project_root)?;
        
        // Optimize for development workflow
        config.enable_real_time_updates = true;
        config.enable_ai_assistant_integration = true;
        config.cache_policy = PyCachePolicy::adaptive();
        config.performance_config.io_timeout_ms = 10000; // Faster I/O for development
        
        config.optimize_for_system()?;
        Ok(config)
    }
}

// Private helper methods
#[cfg(feature = "python")]
impl PyAdvancedAnalyzerConfig {
    fn detect_languages(project_path: &PathBuf) -> PyResult<Vec<String>> {
        let mut languages = Vec::new();
        
        // Look for common language indicators
        if project_path.join("package.json").exists() {
            languages.push("javascript".to_string());
            languages.push("typescript".to_string());
        }
        
        if project_path.join("requirements.txt").exists() || 
           project_path.join("pyproject.toml").exists() ||
           project_path.join("setup.py").exists() {
            languages.push("python".to_string());
        }
        
        if project_path.join("Cargo.toml").exists() {
            languages.push("rust".to_string());
        }
        
        if project_path.join("pom.xml").exists() || 
           project_path.join("build.gradle").exists() {
            languages.push("java".to_string());
        }
        
        if languages.is_empty() {
            // Default to common languages
            languages.extend(["python", "javascript", "typescript"].iter().map(|s| s.to_string()));
        }
        
        Ok(languages)
    }
    
    fn detect_ignore_patterns(project_path: &PathBuf) -> PyResult<Vec<String>> {
        let mut patterns = vec![
            "node_modules/**".to_string(),
            "*.pyc".to_string(),
            "__pycache__/**".to_string(),
            ".git/**".to_string(),
            "target/**".to_string(),
            "dist/**".to_string(),
            "build/**".to_string(),
            "*.min.js".to_string(),
            "*.min.css".to_string(),
        ];
        
        // Read .gitignore if it exists
        if let Ok(gitignore) = std::fs::read_to_string(project_path.join(".gitignore")) {
            for line in gitignore.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_string());
                }
            }
        }
        
        Ok(patterns)
    }
    
    fn estimate_project_size(project_path: &PathBuf) -> PyResult<u64> {
        let mut total_size = 0;
        let mut file_count = 0;
        
        if let Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = path.metadata() {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                }
            }
        }
        
        // Return estimated size in MB
        Ok(total_size / (1024 * 1024))
    }
    
    fn recommend_config(project_size_mb: u64) -> (PyCachePolicy, PyPerformanceConfig) {
        match project_size_mb {
            0..=100 => (PyCachePolicy::conservative(), PyPerformanceConfig::for_small_project()),
            101..=1000 => (PyCachePolicy::adaptive(), PyPerformanceConfig::for_medium_project()),
            _ => (PyCachePolicy::aggressive(), PyPerformanceConfig::for_large_project()),
        }
    }
    
    fn get_system_memory() -> PyResult<u32> {
        // This is a simplified implementation
        // In a real implementation, you'd use sysinfo or similar
        Ok(8192) // Default to 8GB
    }
    
    fn get_cpu_cores() -> PyResult<u32> {
        // This is a simplified implementation
        // In a real implementation, you'd use num_cpus or similar
        Ok(4) // Default to 4 cores
    }
}

/// Configuration profile management
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct PyConfigProfileManager {
    profiles: HashMap<String, PyAdvancedAnalyzerConfig>,
    current_profile: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyConfigProfileManager {
    #[new]
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        
        // Add default profiles
        profiles.insert("development".to_string(), 
            PyAdvancedAnalyzerConfig::for_development("./".to_string()).unwrap_or_default());
        profiles.insert("ci_cd".to_string(), 
            PyAdvancedAnalyzerConfig::for_ci_cd("./".to_string()).unwrap_or_default());
        profiles.insert("ai_assistant".to_string(), 
            PyAdvancedAnalyzerConfig::for_ai_assistant("./".to_string()).unwrap_or_default());
        
        Self {
            profiles,
            current_profile: "development".to_string(),
        }
    }
    
    /// Get current configuration
    pub fn get_current_config(&self) -> PyResult<PyAdvancedAnalyzerConfig> {
        self.profiles.get(&self.current_profile)
            .cloned()
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(
                format!("Profile '{}' not found", self.current_profile)
            ))
    }
    
    /// Switch to a different profile
    pub fn switch_profile(&mut self, profile_name: String) -> PyResult<()> {
        if self.profiles.contains_key(&profile_name) {
            self.current_profile = profile_name;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyKeyError::new_err(
                format!("Profile '{}' not found", profile_name)
            ))
        }
    }
    
    /// Add a new profile
    pub fn add_profile(&mut self, name: String, config: PyAdvancedAnalyzerConfig) -> PyResult<()> {
        self.profiles.insert(name, config);
        Ok(())
    }
    
    /// Remove a profile
    pub fn remove_profile(&mut self, name: String) -> PyResult<()> {
        if name == "development" || name == "ci_cd" || name == "ai_assistant" {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot remove default profiles"
            ));
        }
        
        if self.profiles.remove(&name).is_none() {
            return Err(pyo3::exceptions::PyKeyError::new_err(
                format!("Profile '{}' not found", name)
            ));
        }
        
        // Switch to development if removing current profile
        if self.current_profile == name {
            self.current_profile = "development".to_string();
        }
        
        Ok(())
    }
    
    /// List all available profiles
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
    
    /// Export profile to file
    pub fn export_profile(&self, profile_name: String, file_path: String) -> PyResult<()> {
        let config = self.profiles.get(&profile_name)
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(
                format!("Profile '{}' not found", profile_name)
            ))?;
        
        let json_data = serde_json::to_string_pretty(config)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        std::fs::write(&file_path, json_data)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        
        Ok(())
    }
    
    /// Import profile from file
    pub fn import_profile(&mut self, profile_name: String, file_path: String) -> PyResult<()> {
        let json_data = std::fs::read_to_string(&file_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        
        let config: PyAdvancedAnalyzerConfig = serde_json::from_str(&json_data)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        config.validate()?;
        
        self.profiles.insert(profile_name, config);
        Ok(())
    }
    
    /// Save all profiles to directory
    pub fn save_profiles(&self, directory: String) -> PyResult<()> {
        std::fs::create_dir_all(&directory)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        
        for (name, config) in &self.profiles {
            let file_path = std::path::Path::new(&directory).join(format!("{}.json", name));
            let json_data = serde_json::to_string_pretty(config)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            
            std::fs::write(file_path, json_data)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Load all profiles from directory
    pub fn load_profiles(&mut self, directory: String) -> PyResult<()> {
        let dir_path = std::path::Path::new(&directory);
        if !dir_path.exists() {
            return Ok(()); // Directory doesn't exist, no profiles to load
        }
        
        for entry in std::fs::read_dir(dir_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
        {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(config) = Self::load_profile_from_file(&path) {
                            self.profiles.insert(stem.to_string(), config);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

// Private helper methods for PyConfigProfileManager
#[cfg(feature = "python")]
impl PyConfigProfileManager {
    fn load_profile_from_file(file_path: &std::path::Path) -> PyResult<PyAdvancedAnalyzerConfig> {
        let json_data = std::fs::read_to_string(file_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        
        serde_json::from_str(&json_data)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

/// Default implementation for config creation
#[cfg(feature = "python")]
impl Default for PyAdvancedAnalyzerConfig {
    fn default() -> Self {
        Self {
            project_root: "./".to_string(),
            languages: vec!["python".to_string(), "javascript".to_string()],
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                "*.pyc".to_string(),
                "__pycache__/**".to_string(),
                ".git/**".to_string(),
            ],
            cache_policy: PyCachePolicy::adaptive(),
            performance_config: PyPerformanceConfig::for_medium_project(),
            enable_experimental_architecture: false,
            enable_ai_assistant_integration: false,
            enable_real_time_updates: false,
            enable_cross_language_analysis: true,
            max_file_size_mb: 10,
            enable_syntax_validation: true,
            enable_semantic_analysis: true,
            custom_extractors: HashMap::new(),
            environment_variables: HashMap::new(),
        }
    }
}