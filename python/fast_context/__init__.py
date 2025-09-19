"""
Fast-Context: Intelligent codebase analysis engine for coding assistants.

This package provides high-performance codebase analysis capabilities with
graph-powered code comprehension, built in Rust for maximum performance.
"""

from .fast_context import (
    FastContextAnalyzer,
    AnalyzerConfig,
    AnalysisResult,
    get_supported_languages,
    detect_language,
    get_version,
)

# Import configuration system
try:
    from .config import (
        load_config,
        save_config,
        create_default_config,
        get_config_manager,
        FastContextConfig,
        AnalysisConfig,
        GraphConfig,
        MCPConfig,
        LoggingConfig,
        ConfigManager,
    )
    
    __config_all__ = [
        "load_config",
        "save_config", 
        "create_default_config",
        "get_config_manager",
        "FastContextConfig",
        "AnalysisConfig",
        "GraphConfig",
        "MCPConfig",
        "LoggingConfig",
        "ConfigManager",
    ]
    
except ImportError:
    # Configuration system not available (missing dependencies)
    __config_all__ = []

# Import graph classes from main module
try:
    from .fast_context import (
        PyRustworkxGraph,
        PyRustworkxDiGraph,
        PathResult,
        CentralityResult,
        ConnectedComponent,
    )
    
    # Convenience aliases
    Graph = PyRustworkxGraph
    DiGraph = PyRustworkxDiGraph
    
    __all__ = [
        "FastContextAnalyzer",
        "AnalyzerConfig", 
        "AnalysisResult",
        "get_supported_languages",
        "detect_language",
        "get_version",
        # Graph classes
        "PyRustworkxGraph",
        "PyRustworkxDiGraph", 
        "Graph",
        "DiGraph",
        "PathResult",
        "CentralityResult",
        "ConnectedComponent",
        # Configuration classes
        "load_config",
        "save_config", 
        "create_default_config",
        "get_config_manager",
        "FastContextConfig",
        "AnalysisConfig",
        "GraphConfig",
        "MCPConfig",
        "LoggingConfig",
        "ConfigManager",
    ]
    
except ImportError:
    # Graph bindings not available (compiled without graph support)
    __all__ = [
        "FastContextAnalyzer",
        "AnalyzerConfig", 
        "AnalysisResult",
        "get_supported_languages",
        "detect_language",
        "get_version",
    ] + __config_all__

# Import MCP server (optional dependencies)
try:
    from . import mcp_server
    
    __all__.extend([
        "mcp_server"
    ])
    
except ImportError:
    # MCP server not available (mcp package not installed)
    pass

__version__ = get_version()
