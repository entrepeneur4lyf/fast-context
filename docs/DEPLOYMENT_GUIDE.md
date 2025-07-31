# Fast-Context Production Deployment Guide

## Overview

This guide covers deploying Fast-Context in production environments, including configuration, monitoring, scaling, and troubleshooting.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation](#installation)
- [Production Configuration](#production-configuration)
- [Environment Variables](#environment-variables)
- [Performance Tuning](#performance-tuning)
- [Monitoring and Logging](#monitoring-and-logging)
- [Scaling Strategies](#scaling-strategies)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements

- **CPU**: 2 cores, 2.0 GHz
- **Memory**: 4GB RAM
- **Storage**: 1GB available space
- **OS**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows (10+)
- **Node.js**: 16.x or higher

### Recommended for Production

- **CPU**: 4+ cores, 3.0+ GHz
- **Memory**: 8GB+ RAM
- **Storage**: 10GB+ SSD storage
- **OS**: Linux (Ubuntu 22.04 LTS)
- **Node.js**: 18.x LTS or 20.x LTS

### Large Scale Deployments

- **CPU**: 8+ cores, 3.5+ GHz
- **Memory**: 16GB+ RAM
- **Storage**: 50GB+ NVMe SSD
- **Network**: 1Gbps+ bandwidth
- **Load Balancer**: For horizontal scaling

## Installation

### NPM Installation

```bash
npm install fast-context
```

### From Source

```bash
git clone https://github.com/entrepeneur4lyf/fast-context.git
cd fast-context
npm install
npm run build
```

### Docker Deployment

```dockerfile
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

EXPOSE 3000
CMD ["npm", "start"]
```

## Production Configuration

### Basic Production Config

```typescript
const config = {
  project_root: process.env.PROJECT_ROOT || "/app/codebase",
  include_patterns: [
    "**/*.rs", "**/*.js", "**/*.ts", "**/*.py", 
    "**/*.java", "**/*.go", "**/*.cs"
  ],
  exclude_patterns: [
    "target/**", "node_modules/**", "dist/**", 
    "build/**", ".git/**", "**/*.test.*"
  ],
  max_file_size: 5 * 1024 * 1024, // 5MB
  follow_symlinks: false,
  respect_gitignore: true,
  analysis_timeout_seconds: 120,
  enable_caching: true,
  cache_ttl_seconds: 1800, // 30 minutes
  max_cache_size: 5000,
  enable_incremental: true,
  language_config: {
    rust: { max_complexity_threshold: 15 },
    javascript: { max_complexity_threshold: 10 },
    typescript: { max_complexity_threshold: 10 },
    python: { max_complexity_threshold: 12 }
  }
};
```

### High-Performance Config

```typescript
const highPerfConfig = {
  // ... base config
  analysis_timeout_seconds: 300,
  cache_ttl_seconds: 3600, // 1 hour
  max_cache_size: 10000,
  enable_incremental: true,
  
  // Performance optimizations
  max_file_size: 10 * 1024 * 1024, // 10MB
  parallel_analysis: true,
  memory_limit_mb: 2048,
  
  language_config: {
    rust: { 
      max_complexity_threshold: 20,
      enable_macro_analysis: true,
      cargo_features: ["default"]
    },
    javascript: { 
      max_complexity_threshold: 15,
      enable_jsx: true,
      babel_presets: ["@babel/preset-env"]
    }
  }
};
```

## Environment Variables

### Core Configuration

```bash
# Project settings
PROJECT_ROOT=/path/to/codebase
MAX_FILE_SIZE=5242880  # 5MB in bytes
ANALYSIS_TIMEOUT=120   # seconds

# Caching
ENABLE_CACHING=true
CACHE_TTL=1800        # 30 minutes
MAX_CACHE_SIZE=5000

# Performance
MEMORY_LIMIT_MB=2048
PARALLEL_ANALYSIS=true
MAX_WORKERS=4

# Security
ENABLE_SECURITY_CHECKS=true
BLOCK_PATH_TRAVERSAL=true
SANITIZE_INPUT=true
```

### Monitoring and Logging

```bash
# Logging
LOG_LEVEL=info        # debug, info, warn, error
LOG_FORMAT=json       # json, text
LOG_FILE=/var/log/fast-context.log

# Metrics
ENABLE_METRICS=true
METRICS_PORT=9090
METRICS_PATH=/metrics

# Health checks
HEALTH_CHECK_PORT=8080
HEALTH_CHECK_PATH=/health
```

### Database and Storage

```bash
# Cache storage
CACHE_BACKEND=redis   # memory, redis, file
REDIS_URL=redis://localhost:6379
CACHE_PREFIX=fast-context:

# Persistent storage
STORAGE_BACKEND=file  # file, s3, gcs
STORAGE_PATH=/var/lib/fast-context
```

## Performance Tuning

### Memory Optimization

```typescript
// Memory-efficient configuration
const memoryOptimizedConfig = {
  max_file_size: 2 * 1024 * 1024, // 2MB
  max_cache_size: 1000,
  enable_incremental: true,
  memory_limit_mb: 1024,
  
  // Aggressive garbage collection
  gc_interval_seconds: 300,
  max_memory_usage_percent: 80,
  
  // Streaming analysis for large files
  enable_streaming: true,
  stream_chunk_size: 64 * 1024 // 64KB
};
```

### CPU Optimization

```typescript
// CPU-optimized configuration
const cpuOptimizedConfig = {
  parallel_analysis: true,
  max_workers: Math.min(8, require('os').cpus().length),
  worker_timeout_seconds: 60,
  
  // Analysis optimizations
  enable_fast_mode: true,
  skip_complex_analysis: false,
  complexity_threshold: 50,
  
  // Caching for CPU-intensive operations
  cache_complexity_analysis: true,
  cache_dependency_graphs: true
};
```

### I/O Optimization

```typescript
// I/O optimized configuration
const ioOptimizedConfig = {
  // File system optimizations
  use_memory_mapped_files: true,
  read_buffer_size: 256 * 1024, // 256KB
  max_concurrent_files: 50,
  
  // Network optimizations
  connection_pool_size: 20,
  request_timeout_ms: 30000,
  retry_attempts: 3,
  
  // Storage optimizations
  compress_cache: true,
  cache_compression_level: 6
};
```

## Monitoring and Logging

### Health Check Endpoint

```typescript
// Health check implementation
app.get('/health', (req, res) => {
  const health = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env.npm_package_version,
    uptime: process.uptime(),
    memory: process.memoryUsage(),
    cache: {
      size: analyzer.getCacheSize(),
      hit_rate: analyzer.getCacheHitRate()
    }
  };
  
  res.json(health);
});
```

### Metrics Collection

```typescript
// Prometheus metrics
const prometheus = require('prom-client');

const analysisCounter = new prometheus.Counter({
  name: 'fast_context_analysis_total',
  help: 'Total number of analysis operations',
  labelNames: ['operation', 'status']
});

const analysisDuration = new prometheus.Histogram({
  name: 'fast_context_analysis_duration_seconds',
  help: 'Duration of analysis operations',
  labelNames: ['operation']
});

const cacheHitRate = new prometheus.Gauge({
  name: 'fast_context_cache_hit_rate',
  help: 'Cache hit rate percentage'
});
```

### Structured Logging

```typescript
const winston = require('winston');

const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  transports: [
    new winston.transports.File({ 
      filename: '/var/log/fast-context-error.log', 
      level: 'error' 
    }),
    new winston.transports.File({ 
      filename: '/var/log/fast-context.log' 
    })
  ]
});
```

## Scaling Strategies

### Horizontal Scaling

```yaml
# Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fast-context
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fast-context
  template:
    metadata:
      labels:
        app: fast-context
    spec:
      containers:
      - name: fast-context
        image: fast-context:latest
        ports:
        - containerPort: 3000
        env:
        - name: CACHE_BACKEND
          value: "redis"
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
```

### Load Balancing

```nginx
# Nginx load balancer configuration
upstream fast_context {
    least_conn;
    server app1:3000 max_fails=3 fail_timeout=30s;
    server app2:3000 max_fails=3 fail_timeout=30s;
    server app3:3000 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name api.example.com;
    
    location / {
        proxy_pass http://fast_context;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_timeout 300s;
    }
    
    location /health {
        access_log off;
        proxy_pass http://fast_context;
    }
}
```

## Security Considerations

### Input Validation

- All user inputs are validated and sanitized
- Path traversal attacks are blocked
- SQL injection and XSS attempts are prevented
- File size limits are enforced

### Access Control

```typescript
// API key authentication
const authenticateApiKey = (req, res, next) => {
  const apiKey = req.headers['x-api-key'];
  if (!apiKey || !isValidApiKey(apiKey)) {
    return res.status(401).json({ error: 'Invalid API key' });
  }
  next();
};

// Rate limiting
const rateLimit = require('express-rate-limit');
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP'
});
```

### Network Security

- Use HTTPS in production
- Implement proper CORS policies
- Use secure headers (helmet.js)
- Regular security updates

## Troubleshooting

### Common Issues

#### High Memory Usage

```bash
# Check memory usage
ps aux | grep fast-context
free -h

# Solutions:
# 1. Reduce cache size
# 2. Lower max_file_size
# 3. Enable streaming mode
# 4. Increase memory limits
```

#### Slow Analysis Performance

```bash
# Check CPU usage
top -p $(pgrep fast-context)

# Solutions:
# 1. Enable parallel analysis
# 2. Increase worker count
# 3. Optimize include/exclude patterns
# 4. Enable incremental mode
```

#### Cache Issues

```bash
# Check cache status
curl http://localhost:8080/health

# Solutions:
# 1. Clear cache: DELETE /cache
# 2. Restart service
# 3. Check Redis connectivity
# 4. Verify cache configuration
```

### Debug Mode

```bash
# Enable debug logging
export LOG_LEVEL=debug
export DEBUG=fast-context:*

# Run with profiling
node --prof app.js

# Analyze profile
node --prof-process isolate-*.log > profile.txt
```

### Performance Monitoring

```bash
# Monitor key metrics
curl http://localhost:9090/metrics | grep fast_context

# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/api/symbols

# Monitor resource usage
iostat -x 1
vmstat 1
```
