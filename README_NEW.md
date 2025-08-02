# Fast-Context: Harmonious Codebase Analysis Engine

[![npm version](https://badge.fury.io/js/fast-context.svg)](https://badge.fury.io/js/fast-context)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://github.com/entrepeneur4lyf/fast-context/workflows/CI/badge.svg)](https://github.com/entrepeneur4lyf/fast-context/actions)

A **harmonious, architecturally sound** codebase analysis engine that provides both graph algorithms and code intelligence through a unified, modular API. Built in Rust with Node.js bindings for maximum performance and developer experience.

## 🏗️ Architectural Harmony

Fast-Context achieves architectural harmony through **clear separation of concerns**:

### 🎯 **Domain Separation**
- **Graph Domain**: Pure graph algorithms and data structures
- **Analysis Domain**: Codebase analysis and intelligence features
- **Core Domain**: Shared utilities and abstractions

### 🔌 **Plugin Architecture**
- Extensible design for adding new features
- Clear interfaces for domain integration
- Optional functionality through feature flags

### 🎨 **Unified API**
- Single entry point with consistent patterns
- Domain-specific functionality accessible when needed
- Graceful degradation when features are disabled

## ✨ Key Benefits

- **🚀 Performance**: Rust-powered engine with intelligent caching
- **🎯 Focused**: Use only the features you need
- **🔧 Extensible**: Plugin architecture for future growth
- **📚 Consistent**: Unified API patterns across all functionality
- **🛡️ Reliable**: Comprehensive error handling and validation

## 🚀 Quick Start

### Installation

```bash
npm install fast-context
```

### Basic Usage

```javascript
const { FastContext } = require('fast-context');

// Create with selective features
const context = new FastContext({
  core: {
    enable_graph: true,
    enable_analysis: true,
    enable_caching: true
  },
  analysis: {
    project_root: './my-project',
    languages: ['javascript', 'typescript'],
    ignore_patterns: ['node_modules/**', '.git/**']
  }
});

// Initialize the system
await context.initialize();

// Check system health
const health = await context.health_check();
console.log('System healthy:', health.healthy);

// Use graph functionality
await context.create_graph('dependencies', 'Project Dependencies', true);

// Use analysis functionality
const sessionId = await context.analyze_project('./my-project');
const results = await context.query('find all functions');
```

### Feature-Specific Usage

#### Graph-Only Usage
```javascript
const context = new FastContext({
  core: { 
    enable_graph: true, 
    enable_analysis: false 
  }
});

await context.initialize();
await context.create_graph('my-graph', 'My Graph', false);
```

#### Analysis-Only Usage
```javascript
const context = new FastContext({
  core: { 
    enable_graph: false, 
    enable_analysis: true 
  },
  analysis: {
    project_root: './src',
    languages: ['rust', 'javascript']
  }
});

await context.initialize();
const results = await context.analyze_project('./src');
```

## 📋 Configuration Options

### Core Configuration
```javascript
{
  core: {
    enable_graph: boolean,      // Enable graph functionality
    enable_analysis: boolean,   // Enable analysis functionality
    enable_caching: boolean,    // Enable intelligent caching
    max_memory_mb: number,      // Maximum memory usage
    worker_threads: number      // Number of worker threads
  }
}
```

### Graph Configuration
```javascript
{
  graph: {
    enable_parallel: boolean,   // Enable parallel algorithms
    max_nodes: number,          // Maximum nodes per graph
    max_edges: number          // Maximum edges per graph
  }
}
```

### Analysis Configuration
```javascript
{
  analysis: {
    project_root: string,           // Project root directory
    languages: string[],            // Languages to analyze
    ignore_patterns: string[],      // File patterns to ignore
    enable_watching: boolean,       // Enable file watching
    max_file_size_mb: number       // Maximum file size to analyze
  }
}
```

## 🔄 Migration from Legacy API

### Old Pattern (Deprecated)
```javascript
const { FastContextAnalyzer } = require('fast-context');
const analyzer = new FastContextAnalyzer({ 
  project_root: './project' 
});
await analyzer.analyze();
const results = await analyzer.query('find functions');
```

### New Pattern (Recommended)
```javascript
const { FastContext } = require('fast-context');
const context = new FastContext({
  analysis: { project_root: './project' }
});
await context.initialize();
await context.analyze_project('./project');
const results = await context.query('find functions');
```

### Migration Benefits
- ✅ **Clear separation** between graph and analysis features
- ✅ **Better performance** through selective feature loading
- ✅ **Consistent API** patterns across all functionality
- ✅ **Plugin architecture** for future extensibility
- ✅ **Improved error handling** and validation

## 🎯 Supported Languages

- **Rust** - Complete support with advanced analysis
- **JavaScript/TypeScript** - Full ES6+ and TypeScript support
- **Python** - Python 3.x with type hints
- **Java** - Java 8+ with modern language features
- **Go** - Go modules and modern Go features
- **C/C++** - C11 and C++17 standards
- **C#** - .NET Core and Framework support
- **PHP** - PHP 7.4+ with modern features
- **Ruby** - Ruby 2.7+ support
- **Swift** - Swift 5+ support

## 📊 Performance

Real-world performance on a typical project:
- **86 files, 26,529 symbols, 6,026 relationships** analyzed in **667ms**
- **15MB memory usage** during analysis
- **Sub-second** incremental updates with file watching
- **90%+ cache hit rate** on subsequent analyses

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Unified API Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   FastContext   │  │  Health Check   │  │  Features   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     Domain Layer                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │  Graph Domain   │  │ Analysis Domain │  │ Core Domain │ │
│  │                 │  │                 │  │             │ │
│  │ • Algorithms    │  │ • Parsing       │  │ • Config    │ │
│  │ • Data Structs  │  │ • Symbols       │  │ • Metrics   │ │
│  │ • Operations    │  │ • Queries       │  │ • Errors    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │     Caching     │  │   File System   │  │   Metrics   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 🤝 Contributing

We welcome contributions! This project uses:
- **Rust** for core analysis engine
- **NAPI-RS** for Node.js bindings
- **Tree-sitter** for parsing
- **Tokio** for async runtime

## 📄 License

Apache-2.0

## 📞 Support

- **GitHub Issues**: [fast-context/fast-context](https://github.com/entrepeneur4lyf/fast-context)
- **Documentation**: [https://docs.fast-context.dev](https://docs.fast-context.dev)

---

**Fast-Context** - Harmonious architecture for the next generation of code analysis tools.
