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

__version__ = get_version()

__all__ = [
    "FastContextAnalyzer",
    "AnalyzerConfig", 
    "AnalysisResult",
    "get_supported_languages",
    "detect_language",
    "get_version",
]
