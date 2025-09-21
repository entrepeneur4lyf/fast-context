package filewatch

import (
	"context"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/fast-context/go-sdk/fastcontext"
)

// Watcher provides file system monitoring capabilities
type Watcher struct {
	projectRoot    string
	ignorePatterns []string
	includePatterns []string
	recursive      bool
	debounceDelay  time.Duration
	eventChan      chan *FileEvent
	errorChan      chan error
	ctx            context.Context
	cancelFunc     context.CancelFunc
	wg             sync.WaitGroup
	isWatching     bool
	watchMutex     sync.RWMutex
	fileStates     map[string]fileState
	stats          *WatchStats
}

// FileEvent represents a file system event
type FileEvent struct {
	Type      FileEventType `json:"type"`
	Path      string        `json:"path"`
	OldPath   string        `json:"oldPath,omitempty"`
	Timestamp time.Time     `json:"timestamp"`
	Size      int64         `json:"size,omitempty"`
	Checksum  string        `json:"checksum,omitempty"`
	Metadata  interface{}   `json:"metadata,omitempty"`
}

// FileEventType represents the type of file event
type FileEventType int

const (
	FileCreated FileEventType = iota
	FileModified
	FileDeleted
	FileRenamed
	FileMoved
	FileAccessed
)

// WatchOptions defines options for file watching
type WatchOptions struct {
	IgnorePatterns   []string        `json:"ignorePatterns"`
	IncludePatterns  []string        `json:"includePatterns"`
	Recursive        bool            `json:"recursive"`
	DebounceDelay    time.Duration   `json:"debounceDelay"`
	BufferSize       int             `json:"bufferSize"`
	EnableStats      bool            `json:"enableStats"`
	WatchAccess      bool            `json:"watchAccess"`
	WatchMetadata    bool            `json:"watchMetadata"`
	ThrottleEvents   bool            `json:"throttleEvents"`
	MaxEventsPerSec  int             `json:"maxEventsPerSec"`
	RescanInterval   time.Duration   `json:"rescanInterval"`
	CallbackMode     string          `json:"callbackMode"` // "sync", "async"
}

// fileState tracks the state of a watched file
type fileState struct {
	path      string
	size      int64
	modTime   time.Time
	checksum  string
	isDir     bool
	// lastEvent *FileEvent // Commented out as unused
}

// WatchStats contains statistics for the file watcher
type WatchStats struct {
	StartTime          time.Time     `json:"startTime"`
	EndTime            time.Time     `json:"endTime,omitempty"`
	TotalDuration      time.Duration `json:"totalDuration"`
	EventsProcessed    int64         `json:"eventsProcessed"`
	CreatedEvents      int64         `json:"createdEvents"`
	ModifiedEvents     int64         `json:"modifiedEvents"`
	DeletedEvents      int64         `json:"deletedEvents"`
	RenamedEvents      int64         `json:"renamedEvents"`
	IgnoredEvents      int64         `json:"ignoredEvents"`
	ErrorCount         int           `json:"errorCount"`
	FilesWatched       int           `json:"filesWatched"`
	DirectoriesWatched int           `json:"directoriesWatched"`
	EventsPerSecond    float64       `json:"eventsPerSecond"`
	PeakEventsPerSec   float64       `json:"peakEventsPerSec"`
}

// ChangeHandler defines the interface for handling file changes
type ChangeHandler interface {
	OnFileChange(event *FileEvent) error
	OnFileCreate(event *FileEvent) error
	OnFileDelete(event *FileEvent) error
	OnFileRename(event *FileEvent) error
	OnError(err error)
}

// EventHandler is a simple function-based event handler
type EventHandler func(event *FileEvent) error

// DefaultChangeHandler provides a default implementation of ChangeHandler
type DefaultChangeHandler struct {
	onChange    EventHandler
	onCreate    EventHandler
	onDelete    EventHandler
	onRename    EventHandler
	onError     func(err error)
	analyzer    *fastcontext.Analyzer
}

// NewWatcher creates a new file watcher
func NewWatcher(projectRoot string, opts *WatchOptions) (*Watcher, error) {
	if projectRoot == "" {
		return nil, fastcontext.NewFastContextError(fastcontext.ErrInvalidInput, "project root cannot be empty")
	}

	if opts == nil {
		opts = &WatchOptions{
			IgnorePatterns: []string{".git", "node_modules", "*.tmp", "*.log"},
			Recursive:      true,
			DebounceDelay:  100 * time.Millisecond,
			BufferSize:     1000,
			EnableStats:    true,
			WatchAccess:    false,
			WatchMetadata:  true,
			ThrottleEvents: true,
			MaxEventsPerSec: 1000,
			RescanInterval: 30 * time.Second,
			CallbackMode:   "async",
		}
	}

	ctx, cancel := context.WithCancel(context.Background())

	return &Watcher{
		projectRoot:    projectRoot,
		ignorePatterns: opts.IgnorePatterns,
		includePatterns: opts.IncludePatterns,
		recursive:      opts.Recursive,
		debounceDelay:  opts.DebounceDelay,
		eventChan:      make(chan *FileEvent, opts.BufferSize),
		errorChan:      make(chan error, 100),
		ctx:            ctx,
		cancelFunc:     cancel,
		fileStates:     make(map[string]fileState),
		stats:          &WatchStats{StartTime: time.Now()},
	}, nil
}

// StartWatching starts watching the file system for changes
func (w *Watcher) StartWatching() error {
	w.watchMutex.Lock()
	if w.isWatching {
		w.watchMutex.Unlock()
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "watcher is already running")
	}
	w.isWatching = true
	w.watchMutex.Unlock()

	// Initialize file states
	if err := w.initializeFileStates(); err != nil {
		return err
	}

	// Start the watching loop
	w.wg.Add(1)
	go w.watchLoop()

	return nil
}

// StopWatching stops the file watcher
func (w *Watcher) StopWatching() error {
	w.watchMutex.Lock()
	defer w.watchMutex.Unlock()

	if !w.isWatching {
		return nil
	}

	w.cancelFunc()
	w.isWatching = false
	w.stats.EndTime = time.Now()
	w.stats.TotalDuration = w.stats.EndTime.Sub(w.stats.StartTime)

	// Calculate final statistics
	if w.stats.TotalDuration > 0 {
		w.stats.EventsPerSecond = float64(w.stats.EventsProcessed) / w.stats.TotalDuration.Seconds()
	}

	// Close channels
	close(w.eventChan)
	close(w.errorChan)

	w.wg.Wait()

	return nil
}

// GetEvents returns the event channel
func (w *Watcher) GetEvents() <-chan *FileEvent {
	return w.eventChan
}

// GetErrors returns the error channel
func (w *Watcher) GetErrors() <-chan error {
	return w.errorChan
}

// IsWatching returns whether the watcher is active
func (w *Watcher) IsWatching() bool {
	w.watchMutex.RLock()
	defer w.watchMutex.RUnlock()
	return w.isWatching
}

// GetStats returns the current statistics
func (w *Watcher) GetStats() *WatchStats {
	statsCopy := *w.stats
	return &statsCopy
}

// AddIgnorePattern adds a new ignore pattern
func (w *Watcher) AddIgnorePattern(pattern string) {
	w.watchMutex.Lock()
	defer w.watchMutex.Unlock()
	w.ignorePatterns = append(w.ignorePatterns, pattern)
}

// RemoveIgnorePattern removes an ignore pattern
func (w *Watcher) RemoveIgnorePattern(pattern string) {
	w.watchMutex.Lock()
	defer w.watchMutex.Unlock()
	
	newPatterns := []string{}
	for _, p := range w.ignorePatterns {
		if p != pattern {
			newPatterns = append(newPatterns, p)
		}
	}
	w.ignorePatterns = newPatterns
}

// WatchDirectory adds a directory to watch
func (w *Watcher) WatchDirectory(dirPath string) error {
	if !w.IsWatching() {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "watcher is not running")
	}

	// Initialize file states for the new directory
	return w.initializeDirectoryStates(dirPath)
}

// UnwatchDirectory removes a directory from watching
func (w *Watcher) UnwatchDirectory(dirPath string) error {
	w.watchMutex.Lock()
	defer w.watchMutex.Unlock()

	// Remove all file states for this directory
	for path := range w.fileStates {
		if strings.HasPrefix(path, dirPath) {
			delete(w.fileStates, path)
		}
	}

	return nil
}

// GetWatchedFiles returns a list of currently watched files
func (w *Watcher) GetWatchedFiles() []string {
	w.watchMutex.RLock()
	defer w.watchMutex.RUnlock()

	files := make([]string, 0, len(w.fileStates))
	for path := range w.fileStates {
		files = append(files, path)
	}

	return files
}

// ForceRescan forces a rescan of the watched directories
func (w *Watcher) ForceRescan() error {
	if !w.IsWatching() {
		return fastcontext.NewFastContextError(fastcontext.ErrInvalidConfiguration, "watcher is not running")
	}

	// Reinitialize file states
	return w.initializeFileStates()
}

// SetHandler sets a change handler for file events
func (w *Watcher) SetHandler(handler ChangeHandler) {
	w.wg.Add(1)
	go w.handleEvents(handler)
}

// initializeFileStates initializes the file states for all files in the project
func (w *Watcher) initializeFileStates() error {
	return w.initializeDirectoryStates(w.projectRoot)
}

// initializeDirectoryStates initializes file states for a specific directory
func (w *Watcher) initializeDirectoryStates(dirPath string) error {
	// In a real implementation, this would use fsnotify or similar
	// For now, we'll simulate with a simple file discovery
	
	// Mock implementation - would walk the directory tree
	mockFiles := []string{
		filepath.Join(dirPath, "main.go"),
		filepath.Join(dirPath, "config.go"),
		filepath.Join(dirPath, "utils", "helpers.go"),
		filepath.Join(dirPath, "README.md"),
	}

	for _, file := range mockFiles {
		if !w.shouldIgnore(file) {
			state := fileState{
				path:     file,
				size:     1024, // Mock size
				modTime:  time.Now(),
				checksum: "mock_checksum",
				isDir:    false,
			}
			w.fileStates[file] = state
		}
	}

	w.stats.FilesWatched = len(w.fileStates)
	w.stats.DirectoriesWatched = 1 // Mock

	return nil
}

// watchLoop is the main watching loop
func (w *Watcher) watchLoop() {
	defer w.wg.Done()

	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()

	// Simulate file changes
	changes := 0
	
	for {
		select {
		case <-w.ctx.Done():
			return
		case <-ticker.C:
			// Simulate periodic file changes for demo
			if changes < 5 { // Simulate 5 changes
				w.simulateFileChange()
				changes++
			}
		}
	}
}

// simulateFileChange simulates a file change for demonstration
func (w *Watcher) simulateFileChange() {
	// Pick a random file to modify
	files := w.GetWatchedFiles()
	if len(files) == 0 {
		return
	}

	file := files[0] // Just use first file for demo
	
	event := &FileEvent{
		Type:      FileModified,
		Path:      file,
		Timestamp: time.Now(),
		Size:      2048, // New size
	}

	// Update file state
	if state, exists := w.fileStates[file]; exists {
		state.size = event.Size
		state.modTime = event.Timestamp
		w.fileStates[file] = state
	}

	// Update statistics
	w.stats.EventsProcessed++
	w.stats.ModifiedEvents++

	// Send event
	select {
	case w.eventChan <- event:
	default:
		// Channel full, drop event
		w.stats.IgnoredEvents++
	}
}

// shouldIgnore checks if a file should be ignored based on patterns
func (w *Watcher) shouldIgnore(path string) bool {
	// Check ignore patterns
	for _, pattern := range w.ignorePatterns {
		matched, err := filepath.Match(pattern, filepath.Base(path))
		if err == nil && matched {
			return true
		}
		
		// Check if path contains pattern
		if strings.Contains(path, pattern) {
			return true
		}
	}

	// Check include patterns if specified
	if len(w.includePatterns) > 0 {
		included := false
		for _, pattern := range w.includePatterns {
			matched, err := filepath.Match(pattern, filepath.Base(path))
			if err == nil && matched {
				included = true
				break
			}
		}
		return !included
	}

	return false
}

// handleEvents processes events using the provided handler
func (w *Watcher) handleEvents(handler ChangeHandler) {
	defer w.wg.Done()

	for {
		select {
		case <-w.ctx.Done():
			return
		case event := <-w.eventChan:
			if handler == nil {
				continue
			}

			var err error
			switch event.Type {
			case FileCreated:
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onCreate != nil {
					err = h.onCreate(event)
				} else if h, ok := handler.(interface{ OnFileCreate(event *FileEvent) error }); ok {
					err = h.OnFileCreate(event)
				}
			case FileModified:
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onChange != nil {
					err = h.onChange(event)
				} else if h, ok := handler.(interface{ OnFileChange(event *FileEvent) error }); ok {
					err = h.OnFileChange(event)
				}
			case FileDeleted:
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onDelete != nil {
					err = h.onDelete(event)
				} else if h, ok := handler.(interface{ OnFileDelete(event *FileEvent) error }); ok {
					err = h.OnFileDelete(event)
				}
			case FileRenamed:
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onRename != nil {
					err = h.onRename(event)
				} else if h, ok := handler.(interface{ OnFileRename(event *FileEvent) error }); ok {
					err = h.OnFileRename(event)
				}
			}

			if err != nil {
				w.errorChan <- err
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onError != nil {
					h.onError(err)
				} else if h, ok := handler.(interface{ OnError(err error) }); ok {
					h.OnError(err)
				}
			}
		case err := <-w.errorChan:
			if handler != nil {
				if h, ok := handler.(*DefaultChangeHandler); ok && h.onError != nil {
					h.onError(err)
				} else if h, ok := handler.(interface{ OnError(err error) }); ok {
					h.OnError(err)
				}
			}
		}
	}
}

// NewDefaultChangeHandler creates a new default change handler
func NewDefaultChangeHandler() *DefaultChangeHandler {
	return &DefaultChangeHandler{}
}

// SetChangeHandler sets the change handler for specific event types
func (h *DefaultChangeHandler) SetChangeHandler(eventType FileEventType, handler EventHandler) {
	switch eventType {
	case FileModified:
		h.onChange = handler
	case FileCreated:
		h.onCreate = handler
	case FileDeleted:
		h.onDelete = handler
	case FileRenamed:
		h.onRename = handler
	}
}

// SetErrorHandler sets the error handler
func (h *DefaultChangeHandler) SetErrorHandler(handler func(err error)) {
	h.onError = handler
}

// SetAnalyzer sets the analyzer for automatic analysis on file changes
func (h *DefaultChangeHandler) SetAnalyzer(analyzer *fastcontext.Analyzer) {
	h.analyzer = analyzer
}

// Implement ChangeHandler interface
func (h *DefaultChangeHandler) OnFileChange(event *FileEvent) error {
	if h.onChange != nil {
		return h.onChange(event)
	}
	return nil
}

func (h *DefaultChangeHandler) OnFileCreate(event *FileEvent) error {
	if h.onCreate != nil {
		return h.onCreate(event)
	}
	return nil
}

func (h *DefaultChangeHandler) OnFileDelete(event *FileEvent) error {
	if h.onDelete != nil {
		return h.onDelete(event)
	}
	return nil
}

func (h *DefaultChangeHandler) OnFileRename(event *FileEvent) error {
	if h.onRename != nil {
		return h.onRename(event)
	}
	return nil
}

func (h *DefaultChangeHandler) OnError(err error) {
	if h.onError != nil {
		h.onError(err)
	}
}

// String representation of FileEventType
func (et FileEventType) String() string {
	switch et {
	case FileCreated:
		return "created"
	case FileModified:
		return "modified"
	case FileDeleted:
		return "deleted"
	case FileRenamed:
		return "renamed"
	case FileMoved:
		return "moved"
	case FileAccessed:
		return "accessed"
	default:
		return "unknown"
	}
}