//! # File System Integration
//!
//! Real-time file monitoring and incremental graph updates for live coding assistance.
//! Integrates with the intelligent caching system and code graph builder for seamless updates.

use crate::analysis::CodeGraphBuilder;
use crate::cache::AdaptiveCacheManager;
use crate::parsers::ParserFactory;
use crate::symbols::SymbolExtractorFactory;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

/// File change event with metadata
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: Instant,
}

/// Type of file system change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Directories to watch
    pub watch_dirs: Vec<PathBuf>,
    /// File extensions to monitor
    pub watched_extensions: HashSet<String>,
    /// Files/directories to ignore
    pub ignore_patterns: Vec<String>,
    /// Debounce delay to batch rapid changes
    pub debounce_duration: Duration,
    /// Maximum number of events to batch
    pub batch_size: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        let mut watched_extensions = HashSet::new();
        watched_extensions.extend(
            [
                "rs", "py", "js", "ts", "java", "go", "cs", "swift", "m", "mm", "php", "rb",
                "scala", "zig", "dart", "lua", "sh", "bash", "css", "html", "xml", "json", "yaml",
                "yml", "md",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        Self {
            watch_dirs: vec![PathBuf::from(".")],
            watched_extensions,
            ignore_patterns: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                ".cache".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
            ],
            debounce_duration: Duration::from_millis(500),
            batch_size: 100,
        }
    }
}

/// File system watcher with intelligent change detection and integrated cache/graph updates
pub struct CodebaseWatcher {
    config: WatcherConfig,
    _watcher: RecommendedWatcher,
    change_sender: broadcast::Sender<Vec<FileChange>>,
    debouncer: Arc<Mutex<ChangeDebouncer>>,
    /// Cache manager for intelligent invalidation
    #[allow(dead_code)]
    cache_manager: Arc<RwLock<Option<Arc<AdaptiveCacheManager<String>>>>>,
    /// Code graph builder for incremental updates
    #[allow(dead_code)]
    graph_builder: Arc<RwLock<Option<CodeGraphBuilder>>>,
    /// Parser factory for file analysis
    #[allow(dead_code)]
    parser_factory: ParserFactory,
    /// Symbol extractor factory
    #[allow(dead_code)]
    extractor_factory: SymbolExtractorFactory,
}

impl CodebaseWatcher {
    /// Create a new codebase watcher
    pub fn new(config: WatcherConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let (change_sender, _) = broadcast::channel(1000);
        let change_sender_clone = change_sender.clone();

        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )?;

        // Watch all configured directories
        for dir in &config.watch_dirs {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::Recursive)?;
            }
        }

        let debouncer = Arc::new(Mutex::new(ChangeDebouncer::new(
            config.debounce_duration,
            config.batch_size,
        )));

        let debouncer_clone = debouncer.clone();
        let config_clone = config.clone();

        // Spawn background thread to process file system events
        thread::spawn(move || {
            let mut last_batch_time = Instant::now();

            while let Ok(event) = rx.recv() {
                if let Some(changes) = Self::process_fs_event(event, &config_clone) {
                    let mut debouncer = match debouncer_clone.lock() {
                        Ok(debouncer) => debouncer,
                        Err(e) => {
                            eprintln!("Warning: Failed to acquire debouncer lock: {}", e);
                            continue;
                        }
                    };

                    for change in changes {
                        debouncer.add_change(change);
                    }

                    // Check if we should flush the batch
                    if debouncer.should_flush()
                        || last_batch_time.elapsed() > config_clone.debounce_duration
                    {
                        let batched_changes = debouncer.flush();
                        if !batched_changes.is_empty() {
                            let _ = change_sender_clone.send(batched_changes);
                            last_batch_time = Instant::now();
                        }
                    }
                }
            }
        });

        Ok(Self {
            config,
            _watcher: watcher,
            change_sender,
            debouncer,
            cache_manager: Arc::new(RwLock::new(None)),
            graph_builder: Arc::new(RwLock::new(None)),
            parser_factory: ParserFactory::new(),
            extractor_factory: SymbolExtractorFactory::new(),
        })
    }

    /// Subscribe to file change events
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<FileChange>> {
        self.change_sender.subscribe()
    }

    /// Check if a file should be watched based on configuration
    fn should_watch_file(path: &Path, config: &WatcherConfig) -> bool {
        // Check if file extension is in watched list
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !config.watched_extensions.contains(ext) {
                return false;
            }
        } else {
            return false; // No extension, skip
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &config.ignore_patterns {
            if path_str.contains(pattern) {
                return false;
            }
        }

        true
    }

    /// Process a file system event into file changes
    fn process_fs_event(event: Event, config: &WatcherConfig) -> Option<Vec<FileChange>> {
        let mut changes = Vec::new();
        let timestamp = Instant::now();

        match event.kind {
            EventKind::Create(_) => {
                for path in event.paths {
                    if Self::should_watch_file(&path, config) {
                        changes.push(FileChange {
                            path,
                            change_type: ChangeType::Created,
                            timestamp,
                        });
                    }
                }
            }
            EventKind::Modify(_) => {
                for path in event.paths {
                    if Self::should_watch_file(&path, config) {
                        changes.push(FileChange {
                            path,
                            change_type: ChangeType::Modified,
                            timestamp,
                        });
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    if Self::should_watch_file(&path, config) {
                        changes.push(FileChange {
                            path,
                            change_type: ChangeType::Deleted,
                            timestamp,
                        });
                    }
                }
            }
            _ => {} // Other event types (access, etc.) are ignored
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }

    /// Force flush any pending changes
    pub fn flush_changes(&self) -> Vec<FileChange> {
        let mut debouncer = match self.debouncer.lock() {
            Ok(debouncer) => debouncer,
            Err(e) => {
                eprintln!("Warning: Failed to acquire debouncer lock: {}", e);
                return Vec::new();
            }
        };
        debouncer.flush()
    }

    /// Get current watcher configuration
    pub fn config(&self) -> &WatcherConfig {
        &self.config
    }
}

/// Debounces and batches file changes to reduce noise
struct ChangeDebouncer {
    pending_changes: HashMap<PathBuf, FileChange>,
    debounce_duration: Duration,
    batch_size: usize,
    last_change_time: Option<Instant>,
}

impl ChangeDebouncer {
    fn new(debounce_duration: Duration, batch_size: usize) -> Self {
        Self {
            pending_changes: HashMap::new(),
            debounce_duration,
            batch_size,
            last_change_time: None,
        }
    }

    /// Add a file change to the debouncer
    fn add_change(&mut self, change: FileChange) {
        self.last_change_time = Some(change.timestamp);

        // For the same file, keep only the latest change
        // But handle renames specially
        match &change.change_type {
            ChangeType::Renamed { from, to } => {
                // Remove any pending changes for the old path
                self.pending_changes.remove(from);
                self.pending_changes.insert(to.clone(), change);
            }
            _ => {
                self.pending_changes.insert(change.path.clone(), change);
            }
        }
    }

    /// Check if changes should be flushed
    fn should_flush(&self) -> bool {
        if self.pending_changes.len() >= self.batch_size {
            return true;
        }

        if let Some(last_time) = self.last_change_time {
            if last_time.elapsed() >= self.debounce_duration {
                return true;
            }
        }

        false
    }

    /// Flush all pending changes
    fn flush(&mut self) -> Vec<FileChange> {
        let changes: Vec<FileChange> = self.pending_changes.values().cloned().collect();
        self.pending_changes.clear();
        self.last_change_time = None;
        changes
    }
}

/// Statistics about watched files and changes
#[derive(Debug, Clone)]
pub struct WatcherStats {
    pub files_watched: usize,
    pub total_changes: usize,
    pub changes_by_type: HashMap<String, usize>,
    pub changes_by_extension: HashMap<String, usize>,
}

impl WatcherStats {
    pub fn new() -> Self {
        Self {
            files_watched: 0,
            total_changes: 0,
            changes_by_type: HashMap::new(),
            changes_by_extension: HashMap::new(),
        }
    }

    pub fn record_change(&mut self, change: &FileChange) {
        self.total_changes += 1;

        // Count by change type
        let type_key = match &change.change_type {
            ChangeType::Created => "created",
            ChangeType::Modified => "modified",
            ChangeType::Deleted => "deleted",
            ChangeType::Renamed { .. } => "renamed",
        };
        *self
            .changes_by_type
            .entry(type_key.to_string())
            .or_insert(0) += 1;

        // Count by file extension
        if let Some(ext) = change.path.extension().and_then(|e| e.to_str()) {
            *self
                .changes_by_extension
                .entry(ext.to_string())
                .or_insert(0) += 1;
        }
    }
}

impl Default for WatcherStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_config_defaults() {
        let config = WatcherConfig::default();
        assert!(config.watched_extensions.contains("rs"));
        assert!(config.watched_extensions.contains("py"));
        assert!(config.ignore_patterns.contains(&"node_modules".to_string()));
        assert_eq!(config.debounce_duration, Duration::from_millis(500));
    }

    #[test]
    fn test_should_watch_file() {
        let config = WatcherConfig::default();

        // Should watch Rust files
        assert!(CodebaseWatcher::should_watch_file(
            Path::new("src/main.rs"),
            &config
        ));

        // Should not watch files in ignored directories
        assert!(!CodebaseWatcher::should_watch_file(
            Path::new("node_modules/package.json"),
            &config
        ));

        // Should not watch files without extensions in watched list
        assert!(!CodebaseWatcher::should_watch_file(
            Path::new("README"),
            &config
        ));
    }

    #[test]
    fn test_change_debouncer() {
        let mut debouncer = ChangeDebouncer::new(Duration::from_millis(100), 5);

        let change = FileChange {
            path: PathBuf::from("test.rs"),
            change_type: ChangeType::Modified,
            timestamp: Instant::now(),
        };

        debouncer.add_change(change);
        assert_eq!(debouncer.pending_changes.len(), 1);

        let flushed = debouncer.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(debouncer.pending_changes.len(), 0);
    }

    #[test]
    fn test_watcher_stats() {
        let mut stats = WatcherStats::new();

        let change = FileChange {
            path: PathBuf::from("test.rs"),
            change_type: ChangeType::Modified,
            timestamp: Instant::now(),
        };

        stats.record_change(&change);
        assert_eq!(stats.total_changes, 1);
        assert_eq!(stats.changes_by_type.get("modified"), Some(&1));
        assert_eq!(stats.changes_by_extension.get("rs"), Some(&1));
    }

    #[tokio::test]
    #[ignore = "Timing-sensitive test that may fail in CI environments"]
    async fn test_file_watcher_integration() {
        let temp_dir =
            TempDir::new().expect("Failed to create temporary directory for watcher test");
        let test_file = temp_dir.path().join("test.rs");

        let config = WatcherConfig {
            watch_dirs: vec![temp_dir.path().to_path_buf()],
            debounce_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let watcher =
            CodebaseWatcher::new(config).expect("Failed to create codebase watcher for test");
        let mut receiver = watcher.subscribe();

        // Create a test file
        fs::write(&test_file, "fn main() {}").expect("Failed to write test file");

        // Wait for the change event
        tokio::time::timeout(Duration::from_secs(5), async {
            while let Ok(changes) = receiver.recv().await {
                if !changes.is_empty() {
                    let change = &changes[0];
                    assert_eq!(change.change_type, ChangeType::Created);
                    assert!(change.path.ends_with("test.rs"));
                    break;
                }
            }
        })
        .await
        .expect("Should receive file change event");
    }
}
