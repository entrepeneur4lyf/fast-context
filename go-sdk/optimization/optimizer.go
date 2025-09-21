package optimization

import (
	"container/list"
	"context"
	"fmt"
	"runtime"
	"sync"
	"time"

	"github.com/fast-context/go-sdk/logging"
)

// Optimizer provides performance optimization utilities
type Optimizer struct {
	logger         *logging.StructuredLogger
	metrics        *logging.MetricsCollector
	cache          *OptimizationCache
	parallelPool   *WorkerPool
	memoryManager  *MemoryManager
	enabled        bool
	config         *OptimizationConfig
	cancel         context.CancelFunc
}

// OptimizationConfig configures optimization behavior
type OptimizationConfig struct {
	EnableCaching           bool
	EnableParallelization   bool
	EnableMemoryOptimization bool
	MaxCacheSize           int64
	MaxWorkerCount         int
	MemoryPressureThreshold float64 // 0.0 to 1.0
	OptimizationLevel      string    // "fast", "balanced", "thorough"
}

// OptimizationResult contains optimization results
type OptimizationResult struct {
	OriginalDuration time.Duration     `json:"originalDuration"`
	OptimizedDuration time.Duration     `json:"optimizedDuration"`
	MemorySaved      int64             `json:"memorySaved"`
	CacheHits        int64             `json:"cacheHits"`
	ParallelSpeedup  float64           `json:"parallelSpeedup"`
	TechniquesUsed   []string          `json:"techniquesUsed"`
	Recommendations  []string          `json:"recommendations"`
}

// OptimizationCache provides intelligent caching for analysis results
type OptimizationCache struct {
	items    map[string]*cacheItem
	mutex    sync.RWMutex
	maxSize  int64
	currentSize int64
	evictionQueue *list.List
}

type cacheItem struct {
	key        string
	value      interface{}
	size       int64
	lastAccess time.Time
	accessCount int64
	element    *list.Element
}

// WorkerPool manages parallel execution
type WorkerPool struct {
	workers    []*worker
	taskQueue  chan task
	wg         sync.WaitGroup
	ctx        context.Context
	// cancel     context.CancelFunc // Commented out as unused
	maxWorkers int
}

type worker struct {
	id     int
	pool   *WorkerPool
	active bool
}

type task struct {
	id       int
	fn       func() (interface{}, error)
	resultCh chan<- taskResult
}

type taskResult struct {
	id     int
	result interface{}
	err    error
}

// MemoryManager monitors and optimizes memory usage
type MemoryManager struct {
	logger           *logging.StructuredLogger
	metrics          *logging.MetricsCollector
	pressureThreshold float64
	gcTicker        *time.Ticker
	stopCh           chan struct{}
}

// NewOptimizer creates a new performance optimizer
func NewOptimizer(config *OptimizationConfig, logger *logging.StructuredLogger) (*Optimizer, error) {
	if config == nil {
		config = &OptimizationConfig{
			EnableCaching:           true,
			EnableParallelization:   true,
			EnableMemoryOptimization: true,
			MaxCacheSize:           100 * 1024 * 1024, // 100MB
			MaxWorkerCount:         runtime.NumCPU(),
			MemoryPressureThreshold: 0.8,
			OptimizationLevel:      "balanced",
		}
	}

	optimizer := &Optimizer{
		logger:  logger,
		metrics: logging.GetMetricsCollector(),
		enabled: true,
		config:  config,
	}

	// Initialize optimization components
	if config.EnableCaching {
		optimizer.cache = NewOptimizationCache(config.MaxCacheSize)
	}

	if config.EnableParallelization {
		ctx, cancel := context.WithCancel(context.Background())
		optimizer.parallelPool = NewWorkerPool(ctx, config.MaxWorkerCount)
		optimizer.cancel = cancel
	}

	if config.EnableMemoryOptimization {
		optimizer.memoryManager = NewMemoryManager(config.MemoryPressureThreshold, logger)
	}

	optimizer.logger.Info("Optimizer initialized", "config", config)
	return optimizer, nil
}

// OptimizeOperation applies optimizations to an operation
func (o *Optimizer) OptimizeOperation(ctx context.Context, operation string, fn func() (interface{}, error)) (interface{}, *OptimizationResult, error) {
	if !o.enabled {
		start := time.Now()
		result, err := fn()
		duration := time.Since(start)
		
		return result, &OptimizationResult{
			OriginalDuration: duration,
			OptimizedDuration: duration,
			TechniquesUsed:   []string{"none"},
		}, err
	}

	originalStart := time.Now()
	var optimizedResult interface{}
	var optimizedErr error
	
	techniquesUsed := []string{}
	cacheHit := false

	// Try cache first
	if o.cache != nil {
		cacheKey := o.generateCacheKey(operation, ctx)
		if cached, found := o.cache.Get(cacheKey); found {
			optimizedResult = cached
			cacheHit = true
			techniquesUsed = append(techniquesUsed, "caching")
			o.metrics.IncrementCounter("cache_hits", "operation", operation)
		}
	}

	// Execute with parallelization if enabled and not cached
	if !cacheHit && o.parallelPool != nil {
		resultCh := make(chan taskResult, 1)
		task := task{
			id:       0,
			fn:       fn,
			resultCh: resultCh,
		}

		if err := o.parallelPool.Submit(task); err == nil {
			select {
			case result := <-resultCh:
				optimizedResult, optimizedErr = result.result, result.err
				techniquesUsed = append(techniquesUsed, "parallelization")
			case <-ctx.Done():
				return nil, nil, ctx.Err()
			}
		} else {
			// Fallback to direct execution
			optimizedResult, optimizedErr = fn()
		}
	} else if !cacheHit {
		optimizedResult, optimizedErr = fn()
	}

	optimizedDuration := time.Since(originalStart)

	// Cache the result if not already cached
	if !cacheHit && o.cache != nil && optimizedErr == nil {
		cacheKey := o.generateCacheKey(operation, ctx)
		o.cache.Set(cacheKey, optimizedResult, 1024) // Estimate size
	}

	// Apply memory optimizations if enabled
	memorySaved := int64(0)
	if o.memoryManager != nil {
		memorySaved = o.memoryManager.OptimizeMemoryUsage()
		if memorySaved > 0 {
			techniquesUsed = append(techniquesUsed, "memory_optimization")
		}
	}

	// Calculate optimization metrics
	parallelSpeedup := 1.0
	if o.parallelPool != nil && !cacheHit {
		parallelSpeedup = o.calculateParallelSpeedup(operation, optimizedDuration)
	}

	optimizationResult := &OptimizationResult{
		OptimizedDuration: optimizedDuration,
		MemorySaved:      memorySaved,
		CacheHits:        o.getCacheHits(operation),
		ParallelSpeedup:  parallelSpeedup,
		TechniquesUsed:   techniquesUsed,
		Recommendations:  o.generateOptimizationRecommendations(optimizedResult, optimizedDuration),
	}

	o.metrics.ObserveHistogram("optimized_operation_duration", optimizedDuration.Seconds(), "operation", operation)
	o.logger.Info("Operation optimized", "operation", operation, "duration", optimizedDuration, "techniques", techniquesUsed)

	return optimizedResult, optimizationResult, optimizedErr
}

// NewOptimizationCache creates a new optimization cache
func NewOptimizationCache(maxSize int64) *OptimizationCache {
	return &OptimizationCache{
		items:        make(map[string]*cacheItem),
		maxSize:      maxSize,
		evictionQueue: list.New(),
	}
}

// Get retrieves an item from cache
func (c *OptimizationCache) Get(key string) (interface{}, bool) {
	c.mutex.RLock()
	defer c.mutex.RUnlock()

	item, exists := c.items[key]
	if !exists {
		return nil, false
	}

	// Update access info
	item.lastAccess = time.Now()
	item.accessCount++
	
	// Move to front of eviction queue (LRU)
	if item.element != nil {
		c.evictionQueue.MoveToFront(item.element)
	}

	return item.value, true
}

// Set adds an item to cache
func (c *OptimizationCache) Set(key string, value interface{}, size int64) {
	c.mutex.Lock()
	defer c.mutex.Unlock()

	// Remove existing item if present
	if existing, exists := c.items[key]; exists {
		c.currentSize -= existing.size
		c.evictionQueue.Remove(existing.element)
		delete(c.items, key)
	}

	// Evict items if necessary
	for c.currentSize+size > c.maxSize && c.evictionQueue.Len() > 0 {
		c.evictOldest()
	}

	// Add new item
	item := &cacheItem{
		key:        key,
		value:      value,
		size:       size,
		lastAccess: time.Now(),
		accessCount: 1,
	}

	element := c.evictionQueue.PushFront(item)
	item.element = element
	c.items[key] = item
	c.currentSize += size
}

// evictOldest removes the least recently used item from cache
func (c *OptimizationCache) evictOldest() {
	element := c.evictionQueue.Back()
	if element == nil {
		return
	}

	item := element.Value.(*cacheItem)
	delete(c.items, item.key)
	c.currentSize -= item.size
	c.evictionQueue.Remove(element)
}

// NewWorkerPool creates a new worker pool
func NewWorkerPool(ctx context.Context, maxWorkers int) *WorkerPool {
	if maxWorkers <= 0 {
		maxWorkers = runtime.NumCPU()
	}

	pool := &WorkerPool{
		taskQueue:  make(chan task, maxWorkers*2),
		maxWorkers: maxWorkers,
		ctx:        ctx,
	}

	// Create workers
	pool.workers = make([]*worker, maxWorkers)
	for i := 0; i < maxWorkers; i++ {
		worker := &worker{
			id:   i,
			pool: pool,
		}
		pool.workers[i] = worker
		go worker.run()
	}

	return pool
}

// Submit adds a task to the worker pool
func (p *WorkerPool) Submit(task task) error {
	select {
	case p.taskQueue <- task:
		p.wg.Add(1)
		return nil
	case <-p.ctx.Done():
		return fmt.Errorf("worker pool is shutting down")
	}
}

// run is the main worker loop
func (w *worker) run() {
	for {
		select {
		case task := <-w.pool.taskQueue:
			w.active = true
			result, err := task.fn()
			w.active = false
			
			if task.resultCh != nil {
				select {
				case task.resultCh <- taskResult{id: task.id, result: result, err: err}:
				case <-w.pool.ctx.Done():
				}
			}
			
			w.pool.wg.Done()
		case <-w.pool.ctx.Done():
			return
		}
	}
}

// NewMemoryManager creates a new memory manager
func NewMemoryManager(pressureThreshold float64, logger *logging.StructuredLogger) *MemoryManager {
	mm := &MemoryManager{
		logger:            logger,
		metrics:           logging.GetMetricsCollector(),
		pressureThreshold: pressureThreshold,
		stopCh:            make(chan struct{}),
		gcTicker:          time.NewTicker(30 * time.Second),
	}

	go mm.monitorMemory()
	return mm
}

// monitorMemory continuously monitors memory usage
func (mm *MemoryManager) monitorMemory() {
	for {
		select {
		case <-mm.gcTicker.C:
			mm.checkMemoryPressure()
		case <-mm.stopCh:
			return
		}
	}
}

// checkMemoryPressure checks if memory pressure is high and takes action
func (mm *MemoryManager) checkMemoryPressure() {
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	pressure := float64(memStats.Alloc) / float64(memStats.Sys)
	mm.metrics.SetGauge("memory_pressure", pressure)

	if pressure > mm.pressureThreshold {
		mm.logger.Warn("High memory pressure detected", "pressure", pressure, "alloc", memStats.Alloc, "sys", memStats.Sys)
		
		// Force garbage collection
		runtime.GC()
		
		// Log after GC
		runtime.ReadMemStats(&memStats)
		newPressure := float64(memStats.Alloc) / float64(memStats.Sys)
		mm.logger.Info("Garbage collection completed", "old_pressure", pressure, "new_pressure", newPressure)
		
		mm.metrics.IncrementCounter("forced_gc_calls")
	}
}

// OptimizeMemoryUsage attempts to optimize memory usage
func (mm *MemoryManager) OptimizeMemoryUsage() int64 {
	var memStatsBefore runtime.MemStats
	runtime.ReadMemStats(&memStatsBefore)

	// Force garbage collection
	runtime.GC()

	var memStatsAfter runtime.MemStats
	runtime.ReadMemStats(&memStatsAfter)

	saved := int64(memStatsBefore.Alloc - memStatsAfter.Alloc)
	if saved > 0 {
		mm.logger.Info("Memory optimization completed", "saved_bytes", saved)
		mm.metrics.IncrementCounter("memory_optimization_saved_bytes", "amount", fmt.Sprintf("%d", saved))
	}

	return saved
}

// Stop stops the memory manager
func (mm *MemoryManager) Stop() {
	close(mm.stopCh)
	mm.gcTicker.Stop()
}

// Helper methods
func (o *Optimizer) generateCacheKey(operation string, ctx context.Context) string {
	// Generate a cache key based on operation and context
	return fmt.Sprintf("%s_%d", operation, time.Now().Truncate(time.Minute).Unix())
}

func (o *Optimizer) getCacheHits(operation string) int64 {
	if o.cache == nil {
		return 0
	}
	value, _ := o.metrics.GetCounter("cache_hits", "operation", operation)
	return value
}

func (o *Optimizer) calculateParallelSpeedup(operation string, duration time.Duration) float64 {
	// This is a simplified calculation
	// In practice, you'd compare against baseline execution time
	return float64(o.config.MaxWorkerCount) * 0.8 // 80% efficiency assumption
}

func (o *Optimizer) generateOptimizationRecommendations(result interface{}, duration time.Duration) []string {
	recommendations := []string{}

	if duration > time.Second {
		recommendations = append(recommendations, "Consider caching results for long operations")
	}

	if o.parallelPool != nil && duration > 500*time.Millisecond {
		recommendations = append(recommendations, "Operation could benefit from parallelization")
	}

	if o.cache != nil {
		recommendations = append(recommendations, "Enable intelligent caching for repeated operations")
	}

	return recommendations
}

// Cleanup cleans up optimizer resources
func (o *Optimizer) Cleanup() error {
	o.enabled = false

	if o.parallelPool != nil {
		if o.cancel != nil {
			o.cancel()
		}
		o.parallelPool.wg.Wait()
	}

	if o.memoryManager != nil {
		o.memoryManager.Stop()
		o.memoryManager = nil
	}

	if o.cache != nil {
		o.cache = nil
	}

	if o.parallelPool != nil {
		o.parallelPool = nil
	}

	o.logger.Info("Optimizer cleaned up")
	return nil
}