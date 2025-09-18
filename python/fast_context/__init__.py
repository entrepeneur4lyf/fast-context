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
    ]

__version__ = get_version()
