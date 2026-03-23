//! Intelligent Caching System for Python SDK
//!
//! Provides multi-level caching with predictive capabilities,
//! adaptive strategies, and comprehensive cache management

#![allow(non_local_definitions)]

use crate::python_bindings::{AnalysisResult, PySymbol};
use crate::python_bindings_config::{PyAdvancedAnalyzerConfig, PyCachePolicy};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
// Removed unused chrono imports

/// Cache entry metadata
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyCacheEntry {
    #[pyo3(get)]
    pub key: String,

    #[pyo3(get)]
    pub data: String, // JSON-serialized data

    #[pyo3(get)]
    pub created_at: u64,

    #[pyo3(get)]
    pub accessed_at: u64,

    #[pyo3(get)]
    pub access_count: u32,

    #[pyo3(get)]
    pub size_bytes: u64,

    #[pyo3(get)]
    pub ttl_seconds: u64,

    #[pyo3(get)]
    pub compression_ratio: f64,

    #[pyo3(get)]
    pub tags: Vec<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyCacheEntry {
    #[new]
    pub fn new(key: String, data: String, ttl_seconds: u64, tags: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let size_bytes = data.len() as u64;

        Self {
            key,
            data,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            size_bytes,
            ttl_seconds,
            compression_ratio: 1.0,
            tags,
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now.saturating_sub(self.created_at) > self.ttl_seconds
    }

    /// Update access information
    pub fn update_access(&mut self) {
        self.accessed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.access_count += 1;
    }

    /// Calculate priority for eviction (lower = higher priority)
    pub fn eviction_priority(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let _age = now.saturating_sub(self.created_at) as f64;
        let idle_time = now.saturating_sub(self.accessed_at) as f64;
        let frequency = self.access_count as f64;

        // Priority based on frequency, recency, and size
        (idle_time + 1.0) / (frequency + 1.0) * (self.size_bytes as f64 / 1024.0)
    }
}

/// Cache statistics
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyCacheStatistics {
    #[pyo3(get)]
    pub total_entries: usize,

    #[pyo3(get)]
    pub total_memory_bytes: u64,

    #[pyo3(get)]
    pub total_disk_bytes: u64,

    #[pyo3(get)]
    pub hits: u64,

    #[pyo3(get)]
    pub misses: u64,

    #[pyo3(get)]
    pub evictions: u64,

    #[pyo3(get)]
    pub hit_rate: f64,

    #[pyo3(get)]
    pub compression_ratio: f64,

    #[pyo3(get)]
    pub average_access_time_ms: f64,

    #[pyo3(get)]
    pub prediction_accuracy: f64,
}

#[cfg(feature = "python")]
impl PyCacheStatistics {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            total_memory_bytes: 0,
            total_disk_bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            hit_rate: 0.0,
            compression_ratio: 1.0,
            average_access_time_ms: 0.0,
            prediction_accuracy: 0.0,
        }
    }

    pub fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
    }
}

#[cfg(feature = "python")]
impl Default for PyCacheStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-level cache implementation
#[cfg(feature = "python")]
#[pyclass]
pub struct PyMultiLevelCache {
    l1_cache: Arc<Mutex<HashMap<String, PyCacheEntry>>>, // Memory cache
    l2_cache_path: PathBuf,                              // Disk cache path
    policy: PyCachePolicy,
    statistics: Arc<Mutex<PyCacheStatistics>>,
    access_log: Arc<Mutex<VecDeque<(String, u64, bool)>>>, // (key, timestamp, hit)
    prediction_model: Arc<Mutex<HashMap<String, f64>>>,    // Simple prediction model
}

#[cfg(feature = "python")]
#[pymethods]
impl PyMultiLevelCache {
    #[new]
    pub fn new(cache_dir: String, policy: PyCachePolicy) -> PyResult<Self> {
        let cache_path = PathBuf::from(cache_dir);
        fs::create_dir_all(&cache_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        Ok(Self {
            l1_cache: Arc::new(Mutex::new(HashMap::new())),
            l2_cache_path: cache_path,
            policy,
            statistics: Arc::new(Mutex::new(PyCacheStatistics::new())),
            access_log: Arc::new(Mutex::new(VecDeque::new())),
            prediction_model: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get value from cache
    pub fn get(&self, key: String) -> PyResult<Option<PyCacheEntry>> {
        let start_time = std::time::Instant::now();

        // Try L1 cache first
        {
            let mut l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            if let Some(entry) = l1.get_mut(&key) {
                if !entry.is_expired() {
                    entry.update_access();
                    self.record_access(key.clone(), true);

                    let mut stats = self
                        .statistics
                        .lock()
                        .expect("Failed to acquire cache mutex - possible poisoning");
                    stats.hits += 1;
                    stats.update_hit_rate();

                    let access_time = start_time.elapsed().as_millis() as f64;
                    stats.average_access_time_ms =
                        (stats.average_access_time_ms * (stats.hits - 1) as f64 + access_time)
                            / stats.hits as f64;

                    return Ok(Some(entry.clone()));
                } else {
                    // Remove expired entry
                    l1.remove(&key);
                }
            }
        }

        // Try L2 cache (disk)
        let disk_entry = self.get_from_disk(&key)?;
        if let Some(mut entry) = disk_entry {
            if !entry.is_expired() {
                entry.update_access();
                self.record_access(key.clone(), true);

                // Promote to L1 cache
                {
                    let mut l1 = self
                        .l1_cache
                        .lock()
                        .expect("Failed to acquire cache mutex - possible poisoning");
                    l1.insert(key.clone(), entry.clone());
                    self.enforce_memory_limits();
                }

                let mut stats = self
                    .statistics
                    .lock()
                    .expect("Failed to acquire cache mutex - possible poisoning");
                stats.hits += 1;
                stats.update_hit_rate();

                return Ok(Some(entry));
            } else {
                // Remove expired file
                self.remove_from_disk(&key)?;
            }
        }

        self.record_access(key, false);
        let mut stats = self
            .statistics
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        stats.misses += 1;
        stats.update_hit_rate();

        Ok(None)
    }

    /// Put value in cache
    pub fn put(&self, key: String, entry: PyCacheEntry) -> PyResult<()> {
        // Update L1 cache
        {
            let mut l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            l1.insert(key.clone(), entry.clone());
            self.enforce_memory_limits();
        }

        // Update L2 cache
        self.put_to_disk(&key, &entry)?;

        // Update statistics
        {
            let mut stats = self
                .statistics
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            stats.total_entries += 1;
            stats.total_memory_bytes += entry.size_bytes;
            stats.total_disk_bytes += entry.size_bytes;
            stats.compression_ratio = entry.compression_ratio;
        }

        Ok(())
    }

    /// Remove value from cache
    pub fn remove(&self, key: String) -> PyResult<bool> {
        let mut removed = false;

        // Remove from L1
        {
            let mut l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            if l1.remove(&key).is_some() {
                removed = true;
            }
        }

        // Remove from L2
        if self.remove_from_disk(&key)? {
            removed = true;
        }

        if removed {
            let mut stats = self
                .statistics
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            stats.total_entries = stats.total_entries.saturating_sub(1);
        }

        Ok(removed)
    }

    /// Clear all cache entries
    pub fn clear(&self) -> PyResult<()> {
        // Clear L1 cache
        {
            let mut l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            l1.clear();
        }

        // Clear L2 cache
        if self.l2_cache_path.exists() {
            fs::remove_dir_all(&self.l2_cache_path)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
            fs::create_dir_all(&self.l2_cache_path)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        }

        // Reset statistics
        {
            let mut stats = self
                .statistics
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            *stats = PyCacheStatistics::new();
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> PyResult<PyCacheStatistics> {
        let stats = self
            .statistics
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        Ok(stats.clone())
    }

    /// Run cache maintenance (cleanup expired entries, enforce limits)
    pub fn run_maintenance(&self) -> PyResult<()> {
        self.cleanup_expired_entries();
        self.enforce_memory_limits();
        self.enforce_disk_limits();
        self.update_prediction_model();

        Ok(())
    }

    /// Prefetch predicted entries
    pub fn prefetch(&self, keys: Vec<String>) -> PyResult<u32> {
        let mut prefetched = 0;

        for key in keys {
            if self.get(key.clone()).unwrap().is_none() {
                // Simulate prefetch by creating empty entry
                let entry = PyCacheEntry::new(
                    key,
                    "prefetched".to_string(),
                    self.policy.ttl_seconds,
                    vec!["prefetched".to_string()],
                );
                self.put(entry.key.clone(), entry)?;
                prefetched += 1;
            }
        }

        Ok(prefetched)
    }

    /// Get cache keys with pattern matching
    pub fn get_keys(&self, pattern: String) -> PyResult<Vec<String>> {
        let regex = regex::Regex::new(&pattern).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid regex: {}", e))
        })?;

        let mut keys = Vec::new();

        // Check L1 cache
        {
            let l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            for key in l1.keys() {
                if regex.is_match(key) {
                    keys.push(key.clone());
                }
            }
        }

        // Check L2 cache
        if let Ok(entries) = fs::read_dir(&self.l2_cache_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    let key = file_name.replace(".cache", "");
                    if regex.is_match(&key) && !keys.contains(&key) {
                        keys.push(key);
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Get cache entries by tag
    pub fn get_entries_by_tag(&self, tag: String) -> PyResult<Vec<PyCacheEntry>> {
        let mut entries = Vec::new();

        // Check L1 cache
        {
            let l1 = self
                .l1_cache
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            for entry in l1.values() {
                if entry.tags.contains(&tag) {
                    entries.push(entry.clone());
                }
            }
        }

        // Check L2 cache
        if let Ok(cache_files) = fs::read_dir(&self.l2_cache_path) {
            for cache_file in cache_files.flatten() {
                if let Ok(entry) = self.load_from_disk(&cache_file.path()) {
                    if entry.tags.contains(&tag) {
                        entries.push(entry);
                    }
                }
            }
        }

        Ok(entries)
    }
}

// Private implementation methods
#[cfg(feature = "python")]
impl PyMultiLevelCache {
    fn get_from_disk(&self, key: &str) -> PyResult<Option<PyCacheEntry>> {
        let file_path = self.l2_cache_path.join(format!("{}.cache", key));

        if !file_path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(&file_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        let entry: PyCacheEntry = serde_json::from_str(&data)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Some(entry))
    }

    fn put_to_disk(&self, key: &str, entry: &PyCacheEntry) -> PyResult<()> {
        let file_path = self.l2_cache_path.join(format!("{}.cache", key));

        let data = serde_json::to_string(entry)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        fs::write(&file_path, data)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        Ok(())
    }

    fn remove_from_disk(&self, key: &str) -> PyResult<bool> {
        let file_path = self.l2_cache_path.join(format!("{}.cache", key));

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn load_from_disk(&self, file_path: &Path) -> PyResult<PyCacheEntry> {
        let data = fs::read_to_string(file_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

        serde_json::from_str(&data)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn enforce_memory_limits(&self) {
        let mut l1 = self
            .l1_cache
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        let mut total_size: u64 = l1.values().map(|e| e.size_bytes).sum();

        while total_size > self.policy.max_memory_mb as u64 * 1024 * 1024 {
            if let Some((key_to_remove, _)) = l1
                .iter()
                .min_by_key(|(_, entry)| entry.eviction_priority() as i64)
            {
                let key = key_to_remove.clone();
                if let Some(removed) = l1.remove(&key) {
                    total_size -= removed.size_bytes;
                    let mut stats = self
                        .statistics
                        .lock()
                        .expect("Failed to acquire cache mutex - possible poisoning");
                    stats.evictions += 1;
                }
            } else {
                break;
            }
        }
    }

    fn enforce_disk_limits(&self) {
        let mut total_size: u64 = 0;
        let mut entries = Vec::new();

        if let Ok(cache_files) = fs::read_dir(&self.l2_cache_path) {
            for cache_file in cache_files.flatten() {
                if let Ok(metadata) = cache_file.metadata() {
                    total_size += metadata.len();
                    entries.push((
                        cache_file.path(),
                        metadata.len(),
                        metadata.modified().unwrap_or(UNIX_EPOCH),
                    ));
                }
            }
        }

        let max_disk_bytes = (self.policy.max_disk_gb * 1024.0 * 1024.0 * 1024.0) as u64;

        while total_size > max_disk_bytes && !entries.is_empty() {
            // Find oldest entry
            entries.sort_by(|a, b| a.2.cmp(&b.2));

            if let Some((path, size, _)) = entries.first() {
                if fs::remove_file(path).is_ok() {
                    total_size -= *size;
                    let mut stats = self
                        .statistics
                        .lock()
                        .expect("Failed to acquire cache mutex - possible poisoning");
                    stats.evictions += 1;
                }
                entries.remove(0);
            } else {
                break;
            }
        }
    }

    fn cleanup_expired_entries(&self) {
        let mut l1 = self
            .l1_cache
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        let expired_keys: Vec<String> = l1
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            l1.remove(&key);
        }

        // Clean up disk cache
        if let Ok(cache_files) = fs::read_dir(&self.l2_cache_path) {
            for cache_file in cache_files.flatten() {
                if let Ok(entry) = self.load_from_disk(&cache_file.path()) {
                    if entry.is_expired() {
                        let _ = fs::remove_file(cache_file.path());
                    }
                }
            }
        }
    }

    fn record_access(&self, key: String, hit: bool) {
        let mut log = self
            .access_log
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        log.push_back((key, now, hit));

        // Keep only last 1000 entries
        if log.len() > 1000 {
            log.pop_front();
        }
    }

    fn update_prediction_model(&self) {
        let log = self
            .access_log
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");
        let mut model = self
            .prediction_model
            .lock()
            .expect("Failed to acquire cache mutex - possible poisoning");

        // Simple frequency-based prediction
        let mut key_counts = HashMap::new();
        let mut total_accesses = 0;

        for (key, _, hit) in log.iter() {
            if *hit {
                *key_counts.entry(key.clone()).or_insert(0) += 1;
                total_accesses += 1;
            }
        }

        // Update prediction scores
        for (key, count) in key_counts {
            let score = count as f64 / total_accesses as f64;
            model.insert(key, score);
        }

        // Calculate actual prediction accuracy based on recent performance
        {
            let mut stats = self
                .statistics
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");

            // Calculate prediction accuracy by comparing predicted vs actual hits
            let log = self
                .access_log
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");
            let model = self
                .prediction_model
                .lock()
                .expect("Failed to acquire cache mutex - possible poisoning");

            if log.len() > 10 {
                let mut correct_predictions = 0;
                let mut total_predictions = 0;

                // Use recent log entries to evaluate prediction accuracy
                for (key, _, hit) in log.iter().rev().take(50) {
                    if let Some(predicted_score) = model.get(key) {
                        // Consider prediction correct if score > 0.5 and it was a hit,
                        // or score <= 0.5 and it was a miss
                        if (*predicted_score > 0.5 && *hit) || (*predicted_score <= 0.5 && !*hit) {
                            correct_predictions += 1;
                        }
                        total_predictions += 1;
                    }
                }

                stats.prediction_accuracy = if total_predictions > 0 {
                    correct_predictions as f64 / total_predictions as f64
                } else {
                    0.0
                };
            } else {
                // Not enough data yet, use default value
                stats.prediction_accuracy = 0.5;
            }
        }
    }
}

/// Analysis cache for storing analysis results
#[cfg(feature = "python")]
#[pyclass]
pub struct PyAnalysisCache {
    cache: PyMultiLevelCache,
    config: PyAdvancedAnalyzerConfig,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyAnalysisCache {
    #[new]
    pub fn new(cache_dir: String, config: PyAdvancedAnalyzerConfig) -> PyResult<Self> {
        let cache = PyMultiLevelCache::new(cache_dir, config.cache_policy.clone())?;

        Ok(Self { cache, config })
    }

    /// Get cached analysis result for a file
    pub fn get_analysis_result(&self, file_path: String) -> PyResult<Option<AnalysisResult>> {
        let key = format!("analysis:{}", file_path);

        if let Some(entry) = self.cache.get(key)? {
            let result: AnalysisResult = serde_json::from_str(&entry.data)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Store analysis result for a file
    pub fn put_analysis_result(&self, file_path: String, result: AnalysisResult) -> PyResult<()> {
        let key = format!("analysis:{}", file_path);
        let data = serde_json::to_string(&result)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let entry = PyCacheEntry::new(
            key,
            data,
            self.config.cache_policy.ttl_seconds,
            vec!["analysis".to_string()],
        );

        self.cache.put(entry.key.clone(), entry)?;
        Ok(())
    }

    /// Get cached symbols for a file
    pub fn get_symbols(&self, file_path: String) -> PyResult<Option<Vec<PySymbol>>> {
        let key = format!("symbols:{}", file_path);

        if let Some(entry) = self.cache.get(key)? {
            let symbols: Vec<PySymbol> = serde_json::from_str(&entry.data)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(Some(symbols))
        } else {
            Ok(None)
        }
    }

    /// Store symbols for a file
    pub fn put_symbols(&self, file_path: String, symbols: Vec<PySymbol>) -> PyResult<()> {
        let key = format!("symbols:{}", file_path);
        let data = serde_json::to_string(&symbols)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let entry = PyCacheEntry::new(
            key,
            data,
            self.config.cache_policy.ttl_seconds,
            vec!["symbols".to_string()],
        );

        self.cache.put(entry.key.clone(), entry)?;
        Ok(())
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> PyResult<PyCacheStatistics> {
        self.cache.get_statistics()
    }

    /// Clear all cached analysis results
    pub fn clear_analysis_cache(&self) -> PyResult<()> {
        let analysis_keys = self.cache.get_keys("analysis:*".to_string())?;
        let _cleared = 0;

        for key in analysis_keys {
            let _ = self.cache.remove(key)?;
        }

        Ok(())
    }

    /// Run predictive prefetching
    pub fn run_predictive_prefetch(&self, file_paths: Vec<String>) -> PyResult<u32> {
        if !self.config.cache_policy.prediction_enabled {
            return Ok(0);
        }

        let mut prefetch_keys = Vec::new();

        // Predict which files will be accessed based on patterns
        for file_path in file_paths {
            // Add related files (same directory, similar extensions)
            if let Some(parent) = Path::new(&file_path).parent() {
                if let Ok(entries) = fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        if let Some(path) = entry.path().to_str() {
                            if path != file_path {
                                prefetch_keys.push(format!("analysis:{}", path));
                                prefetch_keys.push(format!("symbols:{}", path));
                            }
                        }
                    }
                }
            }
        }

        // Limit prefetching to policy constraints
        prefetch_keys.truncate(100); // Max 100 prefetch operations

        self.cache.prefetch(prefetch_keys)
    }

    /// Get cache health metrics
    pub fn get_health_metrics(&self) -> PyResult<PyCacheHealthMetrics> {
        let stats = self.cache.get_statistics()?;
        let total_capacity = self.config.cache_policy.max_memory_mb as f64 * 1024.0 * 1024.0;
        let usage_ratio = stats.total_memory_bytes as f64 / total_capacity;

        Ok(PyCacheHealthMetrics {
            memory_usage_ratio: usage_ratio,
            hit_rate: stats.hit_rate,
            average_access_time_ms: stats.average_access_time_ms,
            total_entries: stats.total_entries,
            eviction_rate: stats.evictions as f64 / (stats.hits + stats.misses + 1) as f64,
            compression_efficiency: stats.compression_ratio,
            prediction_accuracy: stats.prediction_accuracy,
            health_score: self.calculate_health_score(&stats, usage_ratio),
        })
    }
}

/// Cache health metrics
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCacheHealthMetrics {
    #[pyo3(get)]
    pub memory_usage_ratio: f64,

    #[pyo3(get)]
    pub hit_rate: f64,

    #[pyo3(get)]
    pub average_access_time_ms: f64,

    #[pyo3(get)]
    pub total_entries: usize,

    #[pyo3(get)]
    pub eviction_rate: f64,

    #[pyo3(get)]
    pub compression_efficiency: f64,

    #[pyo3(get)]
    pub prediction_accuracy: f64,

    #[pyo3(get)]
    pub health_score: f64,
}

// Private implementation for PyAnalysisCache
#[cfg(feature = "python")]
impl PyAnalysisCache {
    fn calculate_health_score(&self, stats: &PyCacheStatistics, usage_ratio: f64) -> f64 {
        let mut score = 0.0;

        // Hit rate (0-40 points)
        score += (stats.hit_rate * 40.0).min(40.0);

        // Memory usage efficiency (0-30 points)
        let optimal_usage = 0.7; // 70% is optimal
        let usage_efficiency = 1.0 - (usage_ratio - optimal_usage).abs();
        score += (usage_efficiency * 30.0).min(30.0);

        // Access time (0-20 points)
        let access_score = (10.0 / (stats.average_access_time_ms + 1.0)).min(20.0);
        score += access_score;

        // Prediction accuracy (0-10 points)
        score += (stats.prediction_accuracy * 10.0).min(10.0);

        score
    }
}
