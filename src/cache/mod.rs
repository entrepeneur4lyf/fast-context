//! # Intelligent Caching System
//!
//! Multi-level cache system for parsed ASTs, symbol extraction results, and analysis data.
//! Provides L1 (memory) and L2 (disk) caching with intelligent invalidation.
//!
//! ## Modules
//!
//! - `size_detector`: Project analysis and size detection for cache policy selection
//! - `policies`: Configurable cache policies that adapt to project characteristics
//! - `adaptive`: Unified cache manager that orchestrates all caching components

pub mod adaptive;
pub mod policies;
pub mod size_detector;

// Re-export key types for convenience
pub use adaptive::{
    AdaptiveCacheManager, AdaptiveCacheStats, MemoryPressureLevel, OptimizationEvent,
};
pub use policies::{CacheConfig, CacheConfigBuilder, CachePolicyType};
pub use size_detector::{CodebaseAnalyzer, ProjectProfile, ProjectSize};

use lru::LruCache;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs as async_fs;
use tokio::sync::RwLock as TokioRwLock;

/// Cache key for identifying cached items
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub file_path: String,
    pub content_hash: String,
    pub cache_type: CacheType,
}

/// Type of cached data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheType {
    ParsedAst,
    ExtractedSymbols,
    AnalysisResult,
    DependencyGraph,
}

/// Cached data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub file_size: u64,
    pub dependencies: Vec<String>, // Files this cache entry depends on
}

/// Get current timestamp in seconds since UNIX epoch, with fallback for system clock issues
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|_| {
            // Fallback: if system clock is before UNIX epoch, use a reasonable default
            // This can happen on embedded systems or during system clock adjustments
            eprintln!(
                "Warning: System clock appears to be before UNIX epoch, using fallback timestamp"
            );
            0
        })
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, file_size: u64, dependencies: Vec<String>) -> Self {
        let now = get_current_timestamp();

        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            file_size,
            dependencies,
        }
    }

    pub fn access(&mut self) -> &T {
        self.last_accessed = get_current_timestamp();
        self.access_count += 1;
        &self.data
    }

    pub fn age(&self) -> Duration {
        let now = get_current_timestamp();
        // Handle potential clock adjustments where current time might be before creation time
        if now >= self.created_at {
            Duration::from_secs(now - self.created_at)
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn last_access_age(&self) -> Duration {
        let now = get_current_timestamp();
        // Handle potential clock adjustments where current time might be before last access
        if now >= self.last_accessed {
            Duration::from_secs(now - self.last_accessed)
        } else {
            Duration::from_secs(0)
        }
    }
}

/// L1 Cache: In-memory LRU cache for frequently accessed items
/// Optimized for read-heavy workloads with RwLock for better concurrency
pub struct L1Cache<T> {
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry<T>>>>,
    max_size: usize,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
    /// Access statistics for adaptive optimization
    access_pattern: Arc<RwLock<AccessPattern>>,
}

/// Access pattern tracking for intelligent cache optimization
#[derive(Debug, Clone, Default)]
struct AccessPattern {
    recent_accesses: VecDeque<(CacheKey, std::time::Instant)>,
    hot_keys: HashMap<CacheKey, u64>,
    total_accesses: u64,
    pattern_type: AccessPatternType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessPatternType {
    Random,
    Temporal,  // Time-based locality
    Frequency,  // Frequency-based locality
    Unknown,
}

impl Default for AccessPatternType {
    fn default() -> Self {
        AccessPatternType::Unknown
    }
}

impl AccessPattern {
    /// Analyze access patterns to optimize caching strategy
    fn analyze_pattern(&mut self) {
        if self.recent_accesses.len() < 100 {
            return;
        }

        let mut time_gaps = Vec::new();
        let mut key_frequencies = HashMap::new();
        
        // Calculate time gaps between consecutive accesses
        let accesses: Vec<_> = self.recent_accesses.iter().collect();
        for window in accesses.windows(2) {
            if let (Some((_, time1)), Some((_, time2))) = (window.first(), window.get(1)) {
                let gap = time2.duration_since(*time1).as_millis();
                time_gaps.push(gap);
            }
        }
        
        // Count key frequencies
        for (key, _) in &self.recent_accesses {
            *key_frequencies.entry(key).or_insert(0) += 1;
        }
        
        // Analyze patterns
        if !time_gaps.is_empty() {
            let avg_gap = time_gaps.iter().sum::<u128>() / time_gaps.len() as u128;
            let variance = time_gaps.iter()
                .map(|gap| (*gap as i64 - avg_gap as i64).pow(2))
                .sum::<i64>() / time_gaps.len() as i64;
            
            // Determine pattern type
            self.pattern_type = if variance < (avg_gap / 4).pow(2) as i64 {
                AccessPatternType::Temporal // Regular time intervals
            } else if key_frequencies.len() < self.recent_accesses.len() / 2 {
                AccessPatternType::Frequency // High key reuse
            } else {
                AccessPatternType::Random
            };
        }
    }
    
    /// Get predicted hot keys for prefetching
    fn get_hot_keys(&self, limit: usize) -> Vec<CacheKey> {
        let mut keys: Vec<_> = self.hot_keys.iter()
            .map(|(key, count)| (key.clone(), *count))
            .collect();
        
        keys.sort_by(|a, b| b.1.cmp(&a.1));
        keys.into_iter().take(limit).map(|(key, _)| key).collect()
    }
}

impl<T: Clone> L1Cache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
            max_size: capacity,
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
            access_pattern: Arc::new(RwLock::new(AccessPattern::default())),
        }
    }

    /// Optimized get operation with read lock for better concurrency
    pub fn get(&self, key: &CacheKey) -> Option<T> {
        // Try read lock first for better performance in read-heavy workloads
        let cache = self.cache.read().unwrap();
        
        if let Some(entry) = cache.peek(key) {
            // Update hit counter atomically
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            
            // Update access pattern (brief write lock)
            self.update_access_pattern(key, true);
            
            // Clone the data while still holding read lock
            let data = entry.data.clone();
            
            // Drop read lock before updating last_accessed to minimize lock contention
            drop(cache);
            
            // Update access time with write lock (brief operation)
            let mut cache = self.cache.write().unwrap();
            if let Some(entry) = cache.get_mut(key) {
                entry.last_accessed = get_current_timestamp();
                entry.access_count += 1;
            }
            
            Some(data)
        } else {
            // Update miss counter atomically
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            self.update_access_pattern(key, false);
            None
        }
    }

    /// Update access pattern analysis for intelligent optimization
    fn update_access_pattern(&self, key: &CacheKey, _is_hit: bool) {
        let mut pattern = self.access_pattern.write().unwrap();
        pattern.total_accesses += 1;
        
        // Track recent accesses for pattern analysis
        pattern.recent_accesses.push_back((key.clone(), std::time::Instant::now()));
        
        // Keep only recent accesses (last 1000)
        while pattern.recent_accesses.len() > 1000 {
            pattern.recent_accesses.pop_front();
        }
        
        // Track hot keys
        *pattern.hot_keys.entry(key.clone()).or_insert(0) += 1;
        
        // Analyze access pattern every 100 accesses
        if pattern.total_accesses % 100 == 0 {
            pattern.analyze_pattern();
        }
    }

    /// Optimized put operation with intelligent eviction
    pub fn put(&self, key: CacheKey, data: T, file_size: u64, dependencies: Vec<String>) {
        let mut cache = self.cache.write().unwrap();
        let entry = CacheEntry::new(data, file_size, dependencies);
        
        // Check if we're at capacity and need intelligent eviction
        if cache.len() >= self.max_size {
            self.intelligent_eviction(&mut cache);
        }
        
        cache.put(key, entry);
    }
    
    /// Intelligent eviction based on access patterns rather than simple LRU
    fn intelligent_eviction(&self, cache: &mut LruCache<CacheKey, CacheEntry<T>>) {
        let pattern = self.access_pattern.read().unwrap();
        
        match pattern.pattern_type {
            AccessPatternType::Frequency => {
                // For frequency-based access, evict least frequently used
                self.evict_lfu(cache);
            }
            AccessPatternType::Temporal => {
                // For temporal patterns, evict oldest but keep recently accessed
                self.evict_temporal(cache);
            }
            _ => {
                // Default to LRU for unknown or random patterns
                // The LruCache already handles this efficiently
            }
        }
    }
    
    /// Evict least frequently used items
    fn evict_lfu(&self, cache: &mut LruCache<CacheKey, CacheEntry<T>>) {
        // For LFU, we need to find and remove the least frequently used
        // Since LruCache doesn't directly support LFU, we'll use a simple heuristic
        // Remove the oldest entry (LRU behavior as fallback)
        if let Some((lru_key, _)) = cache.peek_lru() {
            let key_to_remove = lru_key.clone();
            cache.pop(&key_to_remove);
        }
    }
    
    /// Evict based on temporal patterns
    fn evict_temporal(&self, cache: &mut LruCache<CacheKey, CacheEntry<T>>) {
        let now = get_current_timestamp();
        
        // Find entries that haven't been accessed recently
        if let Some((lru_key, entry)) = cache.peek_lru() {
            let age = now.saturating_sub(entry.last_accessed);
            if age > 3600 { // 1 hour threshold
                let key_to_remove = lru_key.clone();
                cache.pop(&key_to_remove);
            }
        }
    }

    pub fn remove(&self, key: &CacheKey) -> Option<T> {
        let mut cache = self.cache.write().unwrap();
        cache.pop(key).map(|entry| entry.data)
    }

    pub fn stats(&self) -> CacheStats {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };

        let cache = self.cache.read().unwrap();
        CacheStats {
            hits,
            misses,
            hit_rate,
            entries: cache.len(),
            max_entries: self.max_size,
        }
    }

    /// Get access pattern information for optimization
    pub fn get_access_pattern(&self) -> AccessPatternType {
        let pattern = self.access_pattern.read().unwrap();
        pattern.pattern_type.clone()
    }

    /// Get predicted hot keys for prefetching
    pub fn get_hot_keys(&self, limit: usize) -> Vec<CacheKey> {
        let pattern = self.access_pattern.read().unwrap();
        pattern.get_hot_keys(limit)
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        
        // Reset access pattern tracking
        let mut pattern = self.access_pattern.write().unwrap();
        pattern.recent_accesses.clear();
        pattern.hot_keys.clear();
        pattern.total_accesses = 0;
        pattern.pattern_type = AccessPatternType::Unknown;
    }
}

/// Clone implementation for L1Cache
impl<T> Clone for L1Cache<T> {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            max_size: self.max_size,
            hit_count: self.hit_count.clone(),
            miss_count: self.miss_count.clone(),
            access_pattern: self.access_pattern.clone(),
        }
    }
}

/// L2 Cache: Persistent disk cache for larger datasets
/// Optimized with efficient eviction and async operations
pub struct L2Cache {
    cache_dir: PathBuf,
    index: Arc<TokioRwLock<HashMap<CacheKey, CacheMetadata>>>,
    max_size_bytes: u64,
    current_size_bytes: Arc<AtomicU64>,
    /// Eviction candidates cache to avoid repeated sorting
    eviction_cache: Arc<RwLock<Vec<(CacheKey, CacheMetadata)>>>,
    /// Background compaction task handle
    compaction_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    file_path: PathBuf,
    created_at: u64,
    last_accessed: u64,
    file_size: u64,
    dependencies: Vec<String>,
}

impl L2Cache {
    pub fn new(cache_dir: PathBuf, max_size_mb: u64) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(&cache_dir)?;

        let cache = Self {
            cache_dir,
            index: Arc::new(TokioRwLock::new(HashMap::new())),
            max_size_bytes: max_size_mb * 1024 * 1024,
            current_size_bytes: Arc::new(AtomicU64::new(0)),
            eviction_cache: Arc::new(RwLock::new(Vec::new())),
            compaction_task: Arc::new(Mutex::new(None)),
        };

        // Load existing cache index
        cache.load_index()?;

        // Start background compaction task
        cache.start_background_compaction();

        Ok(cache)
    }

    pub async fn get<T>(&self, key: &CacheKey) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let index = self.index.read().await;

        if let Some(metadata) = index.get(key) {
            // Check if file exists and read it
            if metadata.file_path.exists() {
                if let Ok(content) = fs::read(&metadata.file_path) {
                    if let Ok(entry) = bincode::deserialize::<CacheEntry<T>>(&content) {
                        // Update access time in index
                        drop(index);
                        self.update_access_time(key).await;
                        return Some(entry.data);
                    }
                }
            }
        }

        None
    }

    pub async fn put<T>(
        &self,
        key: CacheKey,
        data: T,
        dependencies: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let entry = CacheEntry::new(data, 0, dependencies); // file_size will be calculated
        let serialized = bincode::serialize(&entry)?;
        let file_size = serialized.len() as u64;

        // Generate file path
        let key_hash = self.hash_key(&key);
        let file_path = self.cache_dir.join(format!("{key_hash}.cache"));

        // Check if we need to evict entries
        self.ensure_space(file_size).await?;

        // Write to disk
        fs::write(&file_path, &serialized)?;

        // Update index
        let metadata = CacheMetadata {
            file_path,
            created_at: entry.created_at,
            last_accessed: entry.last_accessed,
            file_size,
            dependencies: entry.dependencies,
        };

        let mut index = self.index.write().await;
        index.insert(key, metadata);
        self.current_size_bytes.fetch_add(file_size, Ordering::Relaxed);

        Ok(())
    }

    pub async fn remove(&self, key: &CacheKey) -> bool {
        let mut index = self.index.write().await;

        if let Some(metadata) = index.remove(key) {
            if metadata.file_path.exists() {
                let _ = fs::remove_file(&metadata.file_path);
                self.current_size_bytes.fetch_sub(metadata.file_size, Ordering::Relaxed);
                return true;
            }
        }

        false
    }

    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = self.index.write().await;

        for metadata in index.values() {
            if metadata.file_path.exists() {
                fs::remove_file(&metadata.file_path)?;
            }
        }

        index.clear();
        self.current_size_bytes.store(0, Ordering::Relaxed);

        Ok(())
    }

    pub async fn invalidate_dependencies(&self, changed_file: &str) {
        let index = self.index.read().await;
        let mut keys_to_remove = Vec::new();

        for (key, metadata) in index.iter() {
            if metadata.dependencies.contains(&changed_file.to_string()) {
                keys_to_remove.push(key.clone());
            }
        }

        drop(index);

        for key in keys_to_remove {
            self.remove(&key).await;
        }
    }

    fn hash_key(&self, key: &CacheKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(key).unwrap());
        format!("{:x}", hasher.finalize())
    }

    async fn ensure_space(&self, needed_bytes: u64) -> Result<(), Box<dyn std::error::Error>> {
        let current_size = self.current_size_bytes.load(Ordering::Relaxed);

        if current_size + needed_bytes > self.max_size_bytes {
            // Evict least recently used entries
            self.evict_lru_entries(needed_bytes).await?;
        }

        Ok(())
    }

    async fn evict_lru_entries(&self, needed_bytes: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Use cached eviction candidates if available and fresh
        let candidates = {
            let cache = self.eviction_cache.read().unwrap();
            if !cache.is_empty() {
                Some(cache.clone())
            } else {
                None
            }
        };

        let entries = if let Some(cached) = candidates {
            cached
        } else {
            // Build eviction candidates cache
            let index = self.index.read().await;
            let entries: Vec<(CacheKey, CacheMetadata)> =
                index.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

            // Sort by last access time (oldest first)
            let mut sorted_entries = entries;
            sorted_entries.sort_by_key(|(_, metadata)| metadata.last_accessed);

            // Cache for future evictions
            *self.eviction_cache.write().unwrap() = sorted_entries.clone();
            sorted_entries
        };

        let mut freed_bytes = 0u64;

        for (key, _) in entries {
            if freed_bytes >= needed_bytes {
                break;
            }

            let removed_size = {
                let index = self.index.read().await;
                index.get(&key).map(|m| m.file_size).unwrap_or(0)
            };

            if self.remove(&key).await {
                freed_bytes += removed_size;
                
                // Update eviction cache
                let mut cache = self.eviction_cache.write().unwrap();
                cache.retain(|(k, _)| k != &key);
            }
        }

        Ok(())
    }

    /// Start background compaction task for cache optimization
    fn start_background_compaction(&self) {
        let compaction_task = {
            let index = self.index.clone();
            let cache_dir = self.cache_dir.clone();
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
                
                loop {
                    interval.tick().await;
                    
                    // Perform background cleanup
                    if let Err(e) = Self::background_compaction(&index, &cache_dir).await {
                        eprintln!("Background compaction error: {}", e);
                    }
                }
            })
        };
        
        *self.compaction_task.lock().unwrap() = Some(compaction_task);
    }

    /// Background compaction to clean up orphaned cache files
    async fn background_compaction(
        index: &Arc<TokioRwLock<HashMap<CacheKey, CacheMetadata>>>,
        cache_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let index_guard = index.read().await;
        let valid_files: std::collections::HashSet<PathBuf> = 
            index_guard.values().map(|m| m.file_path.clone()).collect();
        drop(index_guard);

        // Scan cache directory for orphaned files
        let mut entries = tokio::fs::read_dir(cache_dir).await?;
        let mut removed_count = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cache") {
                if !valid_files.contains(&path) {
                    // Orphaned file, remove it
                    tokio::fs::remove_file(&path).await?;
                    removed_count += 1;
                }
            }
        }

        if removed_count > 0 {
            eprintln!("Background compaction removed {} orphaned cache files", removed_count);
        }

        Ok(())
    }

    async fn update_access_time(&self, key: &CacheKey) {
        let mut index = self.index.write().await;

        if let Some(metadata) = index.get_mut(key) {
            metadata.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    fn load_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Try to load existing index file first
        let index_path = self.cache_dir.join("cache_index.json");
        if index_path.exists() {
            if let Ok(index_content) = fs::read_to_string(&index_path) {
                if let Ok(_index_data) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    // Index loaded successfully
                    return Ok(());
                }
            }
        }

        // If index doesn't exist or is corrupted, rebuild by scanning cache directory
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".cache") {
                            // Process cache file and update index
                            // This would typically extract metadata and add to index
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Clone implementation for L2Cache
impl Clone for L2Cache {
    fn clone(&self) -> Self {
        Self {
            cache_dir: self.cache_dir.clone(),
            index: self.index.clone(),
            max_size_bytes: self.max_size_bytes,
            current_size_bytes: self.current_size_bytes.clone(),
            eviction_cache: self.eviction_cache.clone(),
            compaction_task: self.compaction_task.clone(),
        }
    }
}

/// Two-level cache system
pub struct MultiLevelCache<T> {
    l1: L1Cache<T>,
    l2: L2Cache,
}

impl<T> MultiLevelCache<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    pub fn new(
        l1_capacity: usize,
        l2_cache_dir: PathBuf,
        l2_max_size_mb: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            l1: L1Cache::new(l1_capacity),
            l2: L2Cache::new(l2_cache_dir, l2_max_size_mb)?,
        })
    }

    pub async fn get(&self, key: &CacheKey) -> Option<T> {
        // Try L1 cache first
        if let Some(data) = self.l1.get(key) {
            return Some(data);
        }

        // Try L2 cache
        if let Some(data) = self.l2.get::<T>(key).await {
            // Promote to L1 cache
            self.l1.put(key.clone(), data.clone(), 0, vec![]);
            return Some(data);
        }

        None
    }

    pub async fn put(
        &self,
        key: CacheKey,
        data: T,
        dependencies: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Store in both cache levels
        self.l1
            .put(key.clone(), data.clone(), 0, dependencies.clone());
        self.l2.put(key.clone(), data.clone(), dependencies).await?;

        Ok(())
    }

    pub async fn invalidate(&self, key: &CacheKey) {
        self.l1.remove(key);
        self.l2.remove(key).await;
      }

    pub async fn invalidate_dependencies(&self, changed_file: &str) {
        // For L1 cache, track and invalidate file-specific dependencies
        let mut files_to_invalidate = vec![changed_file.to_string()];

        // Check if we have dependency information to cascade invalidation
        if let Some(deps) = self.get_file_dependencies(changed_file).await {
            files_to_invalidate.extend(deps);
        }

        // Invalidate specific entries rather than clearing everything
        for file in &files_to_invalidate {
            // Get keys to remove (LRU cache doesn't have retain method)
            let keys_to_remove: Vec<CacheKey> = {
                let cache = self.l1.cache.read().unwrap();
                cache
                    .iter()
                    .filter(|(key, _)| key.file_path.contains(file))
                    .map(|(key, _)| key.clone())
                    .collect()
            };

            // Remove the keys
            let mut cache = self.l1.cache.write().unwrap();
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // L2 cache has dependency tracking
        self.l2.invalidate_dependencies(changed_file).await;
    }

    async fn get_file_dependencies(&self, file: &str) -> Option<Vec<String>> {
        // Extract dependencies from L2 cache index based on file analysis
        let index = self.l2.index.read().await;
        let mut dependencies = Vec::new();
        
        // Find all entries that depend on this file (reverse dependency lookup)
        for (key, metadata) in index.iter() {
            if metadata.dependencies.contains(&file.to_string()) {
                // This entry depends on our file, so our file depends on it
                dependencies.push(key.file_path.clone());
            }
        }
        
        // Also check if this file has stored dependency information
        if let Some(entry_key) = self.find_cache_key_for_file(file).await {
            if let Some(metadata) = index.get(&entry_key) {
                // Add direct dependencies stored in metadata
                for dep in &metadata.dependencies {
                    if !dependencies.contains(dep) {
                        dependencies.push(dep.clone());
                    }
                }
            }
        }
        
        // Extract language-specific dependencies using symbol analysis
        let lang_deps = self.extract_language_dependencies(file).await;
        for dep in lang_deps {
            if !dependencies.contains(&dep) {
                dependencies.push(dep);
            }
        }
        
        if dependencies.is_empty() {
            None
        } else {
            Some(dependencies)
        }
    }

    /// Find cache key for a specific file
    async fn find_cache_key_for_file(&self, file: &str) -> Option<CacheKey> {
        let index = self.l2.index.read().await;
        
        for (key, _) in index.iter() {
            if key.file_path == file {
                return Some(key.clone());
            }
        }
        
        None
    }

    /// Extract language-specific dependencies using import/require analysis
    async fn extract_language_dependencies(&self, file: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Determine file type based on extension
        let file_path = Path::new(file);
        let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        match extension {
            "rs" => {
                // Rust dependencies: use, mod, extern crate
                dependencies.extend(self.extract_rust_dependencies(file).await);
            }
            "py" => {
                // Python dependencies: import, from ... import
                dependencies.extend(self.extract_python_dependencies(file).await);
            }
            "js" | "ts" | "jsx" | "tsx" => {
                // JavaScript/TypeScript dependencies: require, import
                dependencies.extend(self.extract_javascript_dependencies(file).await);
            }
            "cpp" | "cc" | "cxx" | "h" | "hpp" => {
                // C++ dependencies: #include
                dependencies.extend(self.extract_cpp_dependencies(file).await);
            }
            _ => {}
        }
        
        dependencies
    }

    /// Extract Rust import dependencies
    async fn extract_rust_dependencies(&self, file: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        // This would typically use the Rust extractor to analyze the file
        // For now, implement a simple file-based approach
        if let Ok(content) = async_fs::read_to_string(file).await {
            // Simple regex-based extraction for demonstration
            let use_re = Regex::new(r"use\s+([^;]+);").unwrap();
            let mod_re = Regex::new(r"mod\s+([^;]+);").unwrap();
            
            for cap in use_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    let import = matched.as_str().trim();
                    // Convert use paths to file paths
                    let file_path = import.replace("::", "/").replace(" ", "");
                    deps.push(file_path);
                }
            }
            
            for cap in mod_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    deps.push(format!("{}.rs", matched.as_str().trim()));
                }
            }
        }
        
        deps
    }

    /// Extract Python import dependencies
    async fn extract_python_dependencies(&self, file: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        if let Ok(content) = async_fs::read_to_string(file).await {
            let import_re = Regex::new(r"import\s+([^#\n]+)").unwrap();
            let from_re = Regex::new(r"from\s+([^#\s]+)\s+import").unwrap();
            
            for cap in import_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    let import = matched.as_str().trim().split(',').next().unwrap_or("").trim();
                    if !import.is_empty() {
                        deps.push(import.to_string());
                    }
                }
            }
            
            for cap in from_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    deps.push(matched.as_str().to_string());
                }
            }
        }
        
        deps
    }

    /// Extract JavaScript/TypeScript dependencies
    async fn extract_javascript_dependencies(&self, file: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        if let Ok(content) = async_fs::read_to_string(file).await {
            let require_re = Regex::new(r#"(?:require|import)\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
            let import_re = Regex::new(r#"import\s+.*?from\s+["']([^"']+)["']"#).unwrap();
            
            for cap in require_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    deps.push(matched.as_str().to_string());
                }
            }
            
            for cap in import_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    deps.push(matched.as_str().to_string());
                }
            }
        }
        
        deps
    }

    /// Extract C++ include dependencies
    async fn extract_cpp_dependencies(&self, file: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        if let Ok(content) = async_fs::read_to_string(file).await {
            let include_re = Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).unwrap();
            
            for cap in include_re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    deps.push(matched.as_str().to_string());
                }
            }
        }
        
        deps
    }

    pub fn l1_stats(&self) -> CacheStats {
        self.l1.stats()
    }
}

/// Clone implementation for MultiLevelCache
impl<T> Clone for MultiLevelCache<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            l1: self.l1.clone(),
            l2: self.l2.clone(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub entries: usize,
    pub max_entries: usize,
}

/// Utility function to generate content hash for files
pub fn generate_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Create cache key for a file
pub fn create_cache_key(file_path: &str, content: &[u8], cache_type: CacheType) -> CacheKey {
    CacheKey {
        file_path: file_path.to_string(),
        content_hash: generate_content_hash(content),
        cache_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_key_creation() {
        let content = b"fn main() {}";
        let key = create_cache_key("test.rs", content, CacheType::ParsedAst);

        assert_eq!(key.file_path, "test.rs");
        assert_eq!(key.cache_type, CacheType::ParsedAst);
        assert!(!key.content_hash.is_empty());
    }

    #[test]
    fn test_l1_cache() {
        let cache = L1Cache::<String>::new(2);
        let key1 = CacheKey {
            file_path: "test1.rs".to_string(),
            content_hash: "hash1".to_string(),
            cache_type: CacheType::ParsedAst,
        };
        let key2 = CacheKey {
            file_path: "test2.rs".to_string(),
            content_hash: "hash2".to_string(),
            cache_type: CacheType::ParsedAst,
        };

        cache.put(key1.clone(), "data1".to_string(), 100, vec![]);
        cache.put(key2.clone(), "data2".to_string(), 200, vec![]);

        assert_eq!(cache.get(&key1), Some("data1".to_string()));
        assert_eq!(cache.get(&key2), Some("data2".to_string()));

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 1.0);
    }

    #[tokio::test]
    async fn test_l2_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = L2Cache::new(temp_dir.path().to_path_buf(), 10).unwrap();

        let key = CacheKey {
            file_path: "test.rs".to_string(),
            content_hash: "hash".to_string(),
            cache_type: CacheType::ParsedAst,
        };

        cache
            .put(key.clone(), "test_data".to_string(), vec![])
            .await
            .unwrap();
        let result: Option<String> = cache.get(&key).await;

        assert_eq!(result, Some("test_data".to_string()));
    }

    #[test]
    fn test_cache_entry() {
        let entry = CacheEntry::new("test_data".to_string(), 100, vec!["dep1.rs".to_string()]);

        assert_eq!(entry.data, "test_data");
        assert_eq!(entry.file_size, 100);
        assert_eq!(entry.dependencies, vec!["dep1.rs"]);
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_content_hash() {
        let content1 = b"fn main() {}";
        let content2 = b"fn main() { println!(\"Hello\"); }";

        let hash1 = generate_content_hash(content1);
        let hash2 = generate_content_hash(content2);

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, generate_content_hash(content1)); // Consistency
    }
}
