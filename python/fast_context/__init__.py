"""
Fast-Context: Intelligent codebase analysis engine for coding assistants.

This package provides high-performance codebase analysis capabilities with
graph-powered code comprehension, built in Rust for maximum performance.
"""

import ast
import asyncio
import os
import re
import threading
from pathlib import Path

from .fast_context import (
    FastContextAnalyzer as _RustFastContextAnalyzer,
    AnalyzerConfig as _RustAnalyzerConfig,
    AnalysisResult,
    get_supported_languages,
    detect_language,
    get_version,
)


def _run_awaitable(awaitable):
    """Run an awaitable from synchronous package APIs."""
    try:
        asyncio.get_running_loop()
    except RuntimeError:
        return asyncio.run(awaitable)

    result = {}
    error = {}

    def runner():
        try:
            result["value"] = asyncio.run(awaitable)
        except Exception as exc:  # pragma: no cover - forwarded to caller
            error["exc"] = exc

    thread = threading.Thread(target=runner, daemon=True)
    thread.start()
    thread.join()

    if "exc" in error:
        raise error["exc"]
    return result.get("value")


def AnalyzerConfig(project_root=None, **kwargs):
    """Compatibility factory for the native AnalyzerConfig."""
    if project_root is None:
        project_root = os.getcwd()
    return _RustAnalyzerConfig(project_root=project_root, **kwargs)


def _config_kwargs(config):
    return {
        "languages": list(getattr(config, "languages", [])),
        "ignore_patterns": list(getattr(config, "ignore_patterns", [])),
        "enable_caching": getattr(config, "enable_caching", True),
        "enable_watching": getattr(config, "enable_watching", False),
        "max_files": getattr(config, "max_files", 10000),
        "parallel_processing": getattr(config, "parallel_processing", True),
    }


def _language_key(file_path):
    suffix = Path(file_path).suffix.lower()
    if suffix == ".py":
        return "python"
    if suffix in {".js", ".mjs", ".cjs"}:
        return "javascript"
    if suffix in {".ts", ".tsx"}:
        return "typescript"
    if suffix == ".rs":
        return "rust"
    return (detect_language(str(file_path)) or "unknown").lower()


def _python_symbols_from_source(source):
    tree = ast.parse(source)
    symbols = []
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            symbols.append({"name": node.name, "type": "function", "line": getattr(node, "lineno", 0)})
        elif isinstance(node, ast.ClassDef):
            symbols.append({"name": node.name, "type": "class", "line": getattr(node, "lineno", 0)})
    return symbols


def _python_imports_from_source(source):
    imports = []
    tree = ast.parse(source)
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.extend(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            imports.append(node.module)
    return imports


class FastContextAnalyzer:
    """Python compatibility wrapper around the native analyzer."""

    def __init__(self, config=None, *args, **kwargs):
        if isinstance(config, str) or config is None:
            config = AnalyzerConfig(project_root=config, **kwargs)
        self._config = config
        self._inner = None
        self._last_analysis = None

    def _ensure_inner(self):
        if self._inner is None:
            self._inner = _RustFastContextAnalyzer(self._config)
        return self._inner

    def _reconfigure(self, project_root):
        project_root = str(project_root)
        if getattr(self._config, "project_root", None) == project_root:
            return
        self._config = AnalyzerConfig(project_root=project_root, **_config_kwargs(self._config))
        self._inner = None
        self._last_analysis = None

    def _project_files(self, project_root=None):
        root = Path(project_root or self._config.project_root)
        if not root.exists():
            return []
        return [path for path in root.rglob("*") if path.is_file()]

    def _analysis_to_dict(self, result, project_root):
        languages = {
            language.lower(): True
            for language in getattr(result, "languages", [])
        }
        symbols = {}
        for path in self._project_files(project_root):
            symbol_map = self.extract_symbols(path)
            for language_name, entries in symbol_map.items():
                symbols.setdefault(language_name, []).extend(entries)
        return {
            "project_path": str(project_root),
            "total_files": len(self._project_files(project_root)),
            "total_symbols": getattr(result, "symbol_count", 0),
            "languages": languages,
            "duration_ms": getattr(result, "duration_ms", 0),
            "symbols": symbols,
        }

    @classmethod
    def from_config(cls, config):
        return cls(config)

    async def analyze_async(self, *_args):
        result = await self._ensure_inner().analyze_async()
        self._last_analysis = result
        return result

    def analyze(self, project_root=None):
        if project_root is not None:
            self._reconfigure(project_root)
            result = _run_awaitable(self.analyze_async())
            return self._analysis_to_dict(result, project_root)
        return _run_awaitable(self.analyze_async())

    async def reanalyze_if_dirty_async(self, *_args):
        return await self._ensure_inner().reanalyze_if_dirty_async()

    async def find_symbols_by_kind_async(self, *args):
        symbol_kind = args[-1]
        return await self._ensure_inner().find_symbols_by_kind_async(symbol_kind)

    def find_symbols_by_kind(self, symbol_kind):
        return _run_awaitable(self.find_symbols_by_kind_async(symbol_kind))

    async def find_symbols_in_file_async(self, *args):
        file_path = args[-1]
        return await self._ensure_inner().find_symbols_in_file_async(file_path)

    def find_symbols_in_file(self, file_path):
        return _run_awaitable(self.find_symbols_in_file_async(file_path))

    async def find_dependencies_async(self, *args):
        symbol_name = args[-1]
        return await self._ensure_inner().find_dependencies_async(symbol_name)

    def find_dependencies(self, symbol_name):
        return _run_awaitable(self.find_dependencies_async(symbol_name))

    async def find_complex_symbols_async(self, *args):
        threshold = args[-1]
        return await self._ensure_inner().find_complex_symbols_async(threshold)

    def find_complex_symbols(self, threshold):
        return _run_awaitable(self.find_complex_symbols_async(threshold))

    def get_analysis(self):
        return self._last_analysis

    def extract_symbols(self, file_path):
        path = Path(file_path)
        if not path.exists():
            return {}

        source = path.read_text(encoding="utf-8")
        language = _language_key(path)

        if language == "python":
            symbols = _python_symbols_from_source(source)
        elif language in {"javascript", "typescript", "rust"}:
            symbol_entries = self.find_symbols_in_file(str(path.name))
            symbols = []
            for entry in symbol_entries:
                if ":" not in entry:
                    continue
                kind, name = entry.split(":", 1)
                symbols.append({"name": name.strip(), "type": kind.strip(), "line": 0})
        else:
            symbols = []

        return {language: symbols}

    def analyze_dependencies(self, project_root):
        imports = {}
        for path in self._project_files(project_root):
            if path.suffix.lower() != ".py":
                continue
            try:
                imports[path.name] = _python_imports_from_source(path.read_text(encoding="utf-8"))
            except SyntaxError:
                imports[path.name] = []
        return {"imports": imports}

    def create_dependency_graph(self, project_root):
        nodes = []
        edges = []

        for path in self._project_files(project_root):
            symbol_map = self.extract_symbols(path)
            for entries in symbol_map.values():
                nodes.extend(entries)

            if path.suffix.lower() == ".py":
                try:
                    for module_name in _python_imports_from_source(path.read_text(encoding="utf-8")):
                        edges.append({"source": path.name, "target": module_name})
                except SyntaxError:
                    continue

        return {"nodes": nodes, "edges": edges}

    def __getattr__(self, name):
        return getattr(self._ensure_inner(), name)


def analyze_project(project_root):
    analyzer = FastContextAnalyzer(project_root)
    return analyzer.analyze()


def find_symbols_by_kind(project_root, symbol_kind):
    analyzer = FastContextAnalyzer(project_root)
    analyzer.analyze()
    return analyzer.find_symbols_by_kind(symbol_kind)


def find_symbols_in_file(file_path):
    file_path = Path(file_path)
    analyzer = FastContextAnalyzer(str(file_path.parent))
    analyzer.analyze()
    return analyzer.find_symbols_in_file(file_path.name)


def find_dependencies(project_root, symbol_name):
    analyzer = FastContextAnalyzer(project_root)
    analyzer.analyze()
    return analyzer.find_dependencies(symbol_name)


def find_complex_symbols(project_root, threshold):
    analyzer = FastContextAnalyzer(project_root)
    analyzer.analyze()
    return analyzer.find_complex_symbols(threshold)


class Graph:
    """Compatibility wrapper around the undirected Rust graph binding."""

    def __init__(self, inner=None):
        self._inner = inner or PyRustworkxGraph()

    @classmethod
    def with_capacity(cls, nodes, edges):
        return cls(PyRustworkxGraph.with_capacity(nodes, edges))

    @property
    def node_count(self):
        return _CallableInt(lambda: self._inner.node_count)

    @property
    def edge_count(self):
        return _CallableInt(lambda: self._inner.edge_count)

    def dijkstra(self, source, target):
        return self._inner.dijkstra_shortest_path(source, target)

    def __getattr__(self, name):
        return getattr(self._inner, name)


class DiGraph:
    """Compatibility wrapper around the directed Rust graph binding."""

    def __init__(self, inner=None):
        self._inner = inner or PyRustworkxDiGraph()

    @classmethod
    def with_capacity(cls, nodes, edges):
        return cls(PyRustworkxDiGraph.with_capacity(nodes, edges))

    @property
    def node_count(self):
        return _CallableInt(lambda: self._inner.node_count)

    @property
    def edge_count(self):
        return _CallableInt(lambda: self._inner.edge_count)

    def __getattr__(self, name):
        return getattr(self._inner, name)


class _CallableInt:
    """Integer-like proxy that also supports call syntax for legacy tests."""

    def __init__(self, getter):
        self._getter = getter

    def __call__(self):
        return self._getter()

    def __int__(self):
        return self._getter()

    def __float__(self):
        return float(self._getter())

    def __str__(self):
        return str(self._getter())

    def __repr__(self):
        return repr(self._getter())

    def __eq__(self, other):
        return self._getter() == other

    def __ge__(self, other):
        return self._getter() >= other

    def __gt__(self, other):
        return self._getter() > other

    def __le__(self, other):
        return self._getter() <= other

    def __lt__(self, other):
        return self._getter() < other

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
    
    __all__ = [
        "FastContextAnalyzer",
        "AnalyzerConfig", 
        "AnalysisResult",
        "get_supported_languages",
        "detect_language",
        "get_version",
        "analyze_project",
        "find_symbols_by_kind",
        "find_symbols_in_file",
        "find_dependencies",
        "find_complex_symbols",
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
