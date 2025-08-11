"""Type stubs for fast_context Rust extension module."""

from typing import List, Optional

class AnalyzerConfig:
    """Configuration for the FastContextAnalyzer."""
    
    project_root: str
    languages: List[str]
    ignore_patterns: List[str]
    enable_caching: bool
    enable_watching: bool
    max_files: int
    parallel_processing: bool
    
    def __init__(
        self,
        project_root: str,
        languages: Optional[List[str]] = None,
        ignore_patterns: Optional[List[str]] = None,
        enable_caching: bool = True,
        enable_watching: bool = False,
        max_files: int = 10000,
        parallel_processing: bool = True,
    ) -> None: ...

class AnalysisResult:
    """Results from codebase analysis."""
    
    file_count: int
    symbol_count: int
    relationship_count: int
    languages: List[str]
    duration_ms: int
    memory_usage_mb: Optional[float]

class FastContextAnalyzer:
    """High-performance codebase analyzer."""
    
    def __init__(self, config: AnalyzerConfig) -> None: ...
    
    def analyze(self) -> AnalysisResult:
        """Analyze the codebase and return results."""
        ...
    
    def find_symbols_by_kind(self, kind: str) -> List[str]:
        """Find symbols by their kind (function, class, variable, etc.)."""
        ...
    
    def find_symbols_in_file(self, file_path: str) -> List[str]:
        """Find all symbols in a specific file."""
        ...
    
    def find_dependencies(self, symbol_name: str) -> List[str]:
        """Find dependencies of a given symbol."""
        ...
    
    def find_complex_symbols(self, threshold: int) -> List[str]:
        """Find symbols with complexity above the threshold."""
        ...
    
    def start_watching(self) -> None:
        """Start watching the codebase for changes."""
        ...
    
    def stop_watching(self) -> None:
        """Stop watching the codebase for changes."""
        ...
    
    def get_analysis(self) -> Optional[AnalysisResult]:
        """Get the current analysis results if available."""
        ...

def get_supported_languages() -> List[str]:
    """Get list of supported programming languages."""
    ...

def detect_language(file_path: str) -> Optional[str]:
    """Detect the programming language of a file."""
    ...

def get_version() -> str:
    """Get the library version."""
    ...
