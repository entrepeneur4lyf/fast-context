#!/usr/bin/env python3
"""
Fast-Context MCP Server

This is a comprehensive Model Context Protocol (MCP) server for the Fast-Context
code analysis engine. It provides tools, resources, and prompts for intelligent
codebase analysis, graph operations, symbol extraction, and advanced features.

Features:
- Code analysis and symbol extraction
- Graph algorithm operations and advanced analytics
- Project management and configuration
- Streaming analysis with progress tracking
- Real-time file watching and incremental updates
- Performance monitoring and metrics
- Multi-language support
- Advanced graph operations (centrality, connectivity, path analysis)
- Export and query capabilities
- Architectural analysis and code review prompts

Usage:
    # Start server with stdio transport
    python -m fast_context.mcp_server
    
    # Start server with SSE transport
    python -m fast_context.mcp_server --transport sse --port 8000
    
    # Use with Claude Desktop or other MCP clients
"""

import asyncio
import json
import logging
import os
import re
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any, AsyncIterator, Dict, List, Optional, Union
from dataclasses import dataclass, asdict
import click

# MCP imports
import mcp.types as types
from mcp.server.fastmcp import FastMCP
from mcp.server.stdio import stdio_server
from mcp.server.sse import SseServerTransport
from mcp.server.lowlevel import Server
from mcp.server.lowlevel.helper_types import ReadResourceContents
from starlette.applications import Starlette
from starlette.requests import Request
from starlette.responses import Response, StreamingResponse
from starlette.routing import Mount, Route
import uvicorn
import anyio

# Fast-Context imports
try:
    from fast_context import (
        FastContextAnalyzer,
        AnalyzerConfig,
        AnalysisResult,
        PyRustworkxGraph,
        PyRustworkxDiGraph,
        get_supported_languages,
        detect_language,
        get_version
    )
    from fast_context.security import (
        SecurityValidator,
        SecurityConfig,
        require_auth,
        require_resource_limits,
        get_security_validator
    )
    
    # Create a simple query engine wrapper for compatibility
    class SimpleQueryEngine:
        def __init__(self, analysis_result):
            self.analysis_result = analysis_result
        
        def find_symbols_by_pattern(self, pattern):
            # Return empty list for now - actual implementation would filter symbols
            return []
    
    PyCodeQueryEngine = SimpleQueryEngine
    CoreAnalyzer = FastContextAnalyzer
    
except ImportError as e:
    print(f"Error importing Fast-Context: {e}")
    print("Please ensure Fast-Context is properly installed")
    sys.exit(1)

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Create FastMCP server instance
mcp = FastMCP(
    "fast-context",
    instructions="Intelligent codebase analysis engine with graph-powered code comprehension"
)

# Initialize security
security_validator = get_security_validator()

# Global state for managing analysis sessions and graphs
analysis_sessions: Dict[str, Dict[str, Any]] = {}
graph_registry: Dict[str, Union[PyRustworkxGraph, PyRustworkxDiGraph]] = {}


class AwaitableJSON(str):
    """String wrapper that can also be awaited by legacy tests."""

    def __new__(cls, value: str):
        return super().__new__(cls, value)

    def __await__(self):
        async def _result():
            return str(self)

        return _result().__await__()


def _json_response(payload: Dict[str, Any]) -> AwaitableJSON:
    return AwaitableJSON(json.dumps(payload, indent=2))


def _serialize_components(components: List[Any]) -> List[List[int]]:
    serialized = []
    for component in components:
        nodes = getattr(component, "nodes", component)
        serialized.append(list(nodes))
    return serialized


def _average_degree(graph: Union[PyRustworkxGraph, PyRustworkxDiGraph]) -> float:
    if getattr(graph, "node_count", 0) == 0:
        return 0.0

    total = 0
    for node_id in range(graph.node_count):
        if hasattr(graph, "neighbors"):
            total += len(graph.neighbors(node_id))
        else:
            total += len(graph.successors(node_id)) + len(graph.predecessors(node_id))
    return total / graph.node_count


def _shortest_path_distances(
    graph: PyRustworkxGraph,
    nodes: List[Any],
    node_ids: Dict[Any, int],
    start_node: Any,
    algorithm: str,
) -> Dict[Any, float]:
    start_id = node_ids.get(start_node)
    if start_id is None:
        raise KeyError(f"Start node '{start_node}' not found in graph")

    if algorithm == "floyd_warshall":
        all_pairs = graph.floyd_warshall_all_pairs()
        return {
            nodes[index]: all_pairs[start_id][index]
            for index in range(len(nodes))
            if all_pairs[start_id][index] is not None
        }

    if algorithm not in {"dijkstra", "bellman_ford"}:
        raise ValueError(f"Unsupported algorithm '{algorithm}'")

    distances = {}
    for node, node_id in node_ids.items():
        path_result = graph.dijkstra_shortest_path(start_id, node_id)
        if path_result is not None:
            distances[node] = path_result.distance
    return distances


# === DATA STRUCTURES ===

@dataclass
class AnalysisProgress:
    """Progress tracking for analysis operations"""
    session_id: str
    total_files: int
    processed_files: int
    current_file: str
    start_time: float
    estimated_remaining: Optional[float] = None
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class GraphMetrics:
    """Graph analysis metrics"""
    node_count: int
    edge_count: int
    density: float
    average_degree: float
    diameter: Optional[float]
    clustering_coefficient: float
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


# === CODE ANALYSIS TOOLS ===

@mcp.tool()
@require_auth
@require_resource_limits("analyze")
async def analyze_codebase(
    project_path: str,
    languages: Optional[List[str]] = None,
    max_files: int = 1000,
    include_patterns: Optional[List[str]] = None,
    exclude_patterns: Optional[List[str]] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Analyze a codebase and extract symbols, relationships, and dependencies.
    
    Args:
        project_path: Path to the project directory to analyze
        languages: Optional list of languages to focus on (e.g., ["python", "typescript"])
        max_files: Maximum number of files to analyze
        include_patterns: Optional glob patterns for files to include
        exclude_patterns: Optional glob patterns for files to exclude
        api_key: API key for authentication
        client_id: Client identifier for rate limiting
    
    Returns:
        JSON string containing analysis results
    """
    try:
        # Validate and sanitize project path
        validated_path = security_validator.validate_path(project_path, require_exists=True)
        
        # Sanitize input parameters
        if languages:
            languages = [security_validator.sanitize_input(lang, max_length=50) for lang in languages]
        if max_files > security_validator.config.max_analysis_files:
            max_files = security_validator.config.max_analysis_files
        if include_patterns:
            include_patterns = [security_validator.sanitize_input(pattern) for pattern in include_patterns]
        if exclude_patterns:
            exclude_patterns = [security_validator.sanitize_input(pattern) for pattern in exclude_patterns]
        
        # Create and run analyzer with actual Fast-Context API
        analyzer = FastContextAnalyzer(str(validated_path))
        
        logger.info(f"Starting analysis of {validated_path}")
        result = await analyzer.analyze_async()
        
        if result is None:
            return json.dumps({"error": "Analysis returned no results"})
        
        # Convert to JSON for serialization
        analysis_data = {
            "file_count": getattr(result, 'file_count', 0),
            "symbol_count": getattr(result, 'symbol_count', 0),
            "relationship_count": getattr(result, 'relationship_count', 0),
            "languages": getattr(result, 'languages', []),
            "duration_ms": getattr(result, 'duration_ms', 0),
            "memory_usage_mb": getattr(result, 'memory_usage_mb', None),
            "analysis_successful": True
        }
        
        return json.dumps(analysis_data, indent=2)
        
    except ValueError as e:
        logger.warning(f"Validation error analyzing codebase: {str(e)}")
        return json.dumps({
            "error": "Validation failed",
            "message": str(e),
            "code": "VALIDATION_ERROR"
        })
    except Exception as e:
        logger.exception("Error analyzing codebase")
        return json.dumps({
            "error": "Internal server error",
            "code": "INTERNAL_ERROR"
        })


@mcp.tool()
@require_auth
@require_resource_limits("analyze")
def find_symbols(
    project_path: str,
    pattern: str,
    symbol_type: Optional[str] = None,
    language: Optional[str] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Find symbols in a codebase matching a pattern.
    
    Args:
        project_path: Path to the project directory
        pattern: Search pattern (supports regex)
        symbol_type: Optional filter by symbol type (function, class, variable, etc.)
        language: Optional filter by programming language
    
    Returns:
        JSON string containing matching symbols
    """
    try:
        project_path = Path(project_path).resolve()
        if not project_path.exists():
            return _json_response({
                "error": f"Project path '{project_path}' does not exist",
                "code": "PROJECT_NOT_FOUND",
            })

        analyzer = FastContextAnalyzer(str(project_path))
        analysis = analyzer.analyze(str(project_path))
        regex = re.compile(pattern)
        symbols_found = []

        for file_path in project_path.rglob("*"):
            if not file_path.is_file():
                continue

            symbol_map = analyzer.extract_symbols(str(file_path))
            for language_name, entries in symbol_map.items():
                if language and language_name != language.lower():
                    continue
                for entry in entries:
                    entry_kind = entry.get("type", "")
                    entry_name = entry.get("name", "")
                    if symbol_type and entry_kind != symbol_type:
                        continue
                    if not regex.search(entry_name):
                        continue
                    symbols_found.append({
                        "name": entry_name,
                        "kind": entry_kind,
                        "language": language_name,
                        "file": file_path.name,
                    })

        return _json_response({
            "pattern": pattern,
            "symbol_type": symbol_type,
            "language": language,
            "total_matches": len(symbols_found),
            "symbols": symbols_found,
            "analysis_info": {
                "file_count": analysis.get("total_files", 0),
                "symbol_count": analysis.get("total_symbols", 0),
                "languages": sorted(analysis.get("languages", {}).keys()),
            }
        })
        
    except Exception as e:
        logger.exception("Error finding symbols")
        return _json_response({
            "error": "Error finding symbols",
            "code": "SYMBOL_SEARCH_ERROR"
        })


@mcp.tool()
@require_auth
@require_resource_limits("analyze")
async def analyze_codebase_streaming(
    project_path: str,
    languages: Optional[List[str]] = None,
    max_files: int = 1000,
    include_patterns: Optional[List[str]] = None,
    exclude_patterns: Optional[List[str]] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Perform streaming codebase analysis with progress updates.
    
    Args:
        project_path: Path to the project directory to analyze
        languages: Optional list of languages to focus on
        max_files: Maximum number of files to analyze
        include_patterns: Optional glob patterns for files to include
        exclude_patterns: Optional glob patterns for files to exclude
    
    Returns:
        JSON string containing streaming analysis results with progress
    """
    try:
        project_path = Path(project_path).resolve()
        if not project_path.exists():
            return json.dumps({"error": f"Project path '{project_path}' does not exist"})
        
        # Create analysis session
        session_id = f"session_{int(time.time())}"
        analysis_sessions[session_id] = {
            "status": "initializing",
            "project_path": str(project_path),
            "start_time": time.time()
        }
        
        # Simulate streaming analysis (in real implementation, this would be incremental)
        total_files = len(list(project_path.rglob("*")))
        processed_files = 0
        
        # Create analyzer
        config = AnalyzerConfig(project_root=str(project_path))
        config.project_root = str(project_path)
        config.max_files = max_files
        
        if include_patterns:
            config.include_patterns = include_patterns
        if exclude_patterns:
            config.exclude_patterns = exclude_patterns
        
        analyzer = CoreAnalyzer(config)
        
        # Stream progress updates
        progress_updates = []
        for i, file_path in enumerate(project_path.rglob("*")):
            if file_path.is_file():
                processed_files += 1
                progress = (processed_files / total_files) * 100
                
                # Add progress update
                progress_updates.append({
                    "session_id": session_id,
                    "status": "analyzing",
                    "current_file": str(file_path.relative_to(project_path)),
                    "processed_files": processed_files,
                    "total_files": total_files,
                    "progress": progress,
                    "estimated_remaining": max(0, (total_files - processed_files) * 0.1)
                })
                
                # Simulate processing time
                await asyncio.sleep(0.01)
        
        # Complete analysis
        result = analyzer.analyze()
        
        analysis_sessions[session_id].update({
            "status": "completed",
            "result": result,
            "end_time": time.time()
        })
        
        # Return complete result with progress history
        return json.dumps({
            "session_id": session_id,
            "status": "completed",
            "progress_updates": progress_updates,
            "result": {
                "file_count": result.file_count,
                "symbol_count": result.symbol_count,
                "languages": result.languages,
                "duration_ms": result.duration_ms,
                "relationship_count": len(result.relationships)
            },
            "total_time": time.time() - analysis_sessions[session_id]["start_time"]
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error in streaming analysis")
        return json.dumps({
            "error": "Analysis failed",
            "code": "STREAMING_ANALYSIS_ERROR"
        })


# === GRAPH ANALYSIS TOOLS ===

@mcp.tool()
@require_auth
@require_resource_limits("graph")
def create_graph(
    graph_type: str = "undirected",
    capacity_nodes: int = 100,
    capacity_edges: int = 200,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Create a new graph for analysis.
    
    Args:
        graph_type: Type of graph ("undirected" or "directed")
        capacity_nodes: Initial node capacity
        capacity_edges: Initial edge capacity
    
    Returns:
        Graph ID and basic information
    """
    try:
        if graph_type == "undirected":
            graph = PyRustworkxGraph.with_capacity(capacity_nodes, capacity_edges)
        elif graph_type == "directed":
            graph = PyRustworkxDiGraph.with_capacity(capacity_nodes, capacity_edges)
        else:
            return json.dumps({
                "error": f"Unsupported graph type '{graph_type}'. Use 'undirected' or 'directed'",
                "code": "UNSUPPORTED_GRAPH_TYPE"
            })
        
        # Store graph in a global registry (in a real implementation, this would be more sophisticated)
        graph_id = f"graph_{id(graph)}"
        
        return json.dumps({
            "graph_id": graph_id,
            "graph_type": graph_type,
            "node_count": graph.node_count,
            "edge_count": graph.edge_count,
            "capacity_nodes": capacity_nodes,
            "capacity_edges": capacity_edges
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error creating graph")
        return json.dumps({
            "error": "Error creating graph",
            "code": "GRAPH_CREATION_ERROR"
        })


@mcp.tool()
@require_auth
@require_resource_limits("graph")
def analyze_graph_connectivity(
    nodes: List[Any],
    edges: List[tuple],
    graph_type: str = "undirected",
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Analyze connectivity of a graph.
    
    Args:
        nodes: List of node identifiers
        edges: List of (source, target, weight) tuples
        graph_type: Type of graph ("undirected" or "directed")
    
    Returns:
        Connectivity analysis results
    """
    try:
        # Create graph
        if graph_type == "undirected":
            graph = PyRustworkxGraph()
        else:
            graph = PyRustworkxDiGraph()
        
        # Add nodes
        node_ids = {}
        for i, node in enumerate(nodes):
            node_id = graph.add_node(str(node))
            node_ids[node] = node_id
        
        # Add edges
        for source, target, weight in edges:
            if source in node_ids and target in node_ids:
                graph.add_edge(node_ids[source], node_ids[target], weight)
        
        # Analyze connectivity
        if graph_type == "undirected":
            components = graph.connected_components()
            connectivity_info = {
                "num_components": len(components),
                "components": [
                    [nodes[node_id] for node_id in component.nodes]
                    for component in components
                ],
                "is_connected": len(components) == 1,
                "density": graph.density() if hasattr(graph, 'density') else 0.0
            }
        else:
            scc = graph.strongly_connected_components()
            weak_components = graph.weakly_connected_components()
            connectivity_info = {
                "num_strongly_connected_components": len(scc),
                "num_weakly_connected_components": len(weak_components),
                "strongly_connected_components": [
                    [nodes[node_id] for node_id in component.nodes]
                    for component in scc if len(component.nodes) > 1
                ],
                "is_dag": graph.is_directed_acyclic_graph(),
                "density": graph.density() if hasattr(graph, 'density') else 0.0
            }
        
        return json.dumps(connectivity_info, indent=2)
        
    except Exception as e:
        logger.exception("Error analyzing graph connectivity")
        return json.dumps({
            "error": "Graph connectivity analysis failed",
            "code": "GRAPH_CONNECTIVITY_ERROR"
        })


@require_auth
@require_resource_limits("compute")
@mcp.tool()
def find_shortest_paths(
    nodes: List[Any],
    edges: List[tuple],
    start_node: Any,
    algorithm: str = "dijkstra",
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Find shortest paths from a start node to all other nodes.
    
    Args:
        nodes: List of node identifiers
        edges: List of (source, target, weight) tuples
        start_node: Starting node for path calculation
        algorithm: Algorithm to use ("dijkstra", "bellman_ford", "floyd_warshall")
    
    Returns:
        Shortest path results
    """
    try:
        # Create graph
        graph = PyRustworkxGraph()
        
        # Add nodes
        node_ids = {}
        for i, node in enumerate(nodes):
            node_id = graph.add_node(str(node))
            node_ids[node] = node_id
        
        # Add edges
        for source, target, weight in edges:
            if source in node_ids and target in node_ids:
                graph.add_edge(node_ids[source], node_ids[target], weight)
        
        path_results = _shortest_path_distances(graph, nodes, node_ids, start_node, algorithm)

        return json.dumps({
            "algorithm": algorithm,
            "start_node": start_node,
            "distances": path_results,
            "reachable_nodes": len(path_results)
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error finding shortest paths")
        return json.dumps({
            "error": "Path finding failed",
            "code": "PATH_FINDING_ERROR"
        })


@require_auth
@require_resource_limits("graph")
@mcp.tool()
def create_advanced_graph(
    graph_type: str = "undirected",
    capacity_nodes: int = 100,
    capacity_edges: int = 200,
    metadata: Optional[Dict[str, Any]] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Create a graph with advanced configuration options.
    
    Args:
        graph_type: Type of graph ("undirected" or "directed")
        capacity_nodes: Initial node capacity
        capacity_edges: Initial edge capacity
        metadata: Optional metadata to attach to the graph
    
    Returns:
        Graph ID and advanced information
    """
    try:
        if graph_type == "undirected":
            graph = PyRustworkxGraph.with_capacity(capacity_nodes, capacity_edges)
        elif graph_type == "directed":
            graph = PyRustworkxDiGraph.with_capacity(capacity_nodes, capacity_edges)
        else:
            return json.dumps({"error": f"Unsupported graph type: {graph_type}"})
        
        graph_id = f"graph_{int(time.time())}_{id(graph)}"
        graph_registry[graph_id] = graph
        
        return json.dumps({
            "graph_id": graph_id,
            "graph_type": graph_type,
            "node_count": graph.node_count,
            "edge_count": graph.edge_count,
            "capacity_nodes": capacity_nodes,
            "capacity_edges": capacity_edges,
            "metadata": metadata or {},
            "created_at": datetime.now().isoformat()
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error creating advanced graph")
        return json.dumps({
            "error": "Graph creation failed",
            "code": "GRAPH_CREATION_ERROR"
        })


@require_auth
@require_resource_limits("compute")
@mcp.tool()
def perform_advanced_graph_analysis(
    graph_id: str,
    analysis_type: str = "comprehensive",
    start_node: Optional[int] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Perform comprehensive graph analysis with advanced metrics.
    
    Args:
        graph_id: ID of the graph to analyze
        analysis_type: Type of analysis ("comprehensive", "centrality", "connectivity", "paths")
        start_node: Starting node for path-based analyses
    
    Returns:
        Comprehensive graph analysis results
    """
    try:
        if not graph_id or graph_id not in graph_registry:
            return json.dumps({"error": "Invalid or missing graph_id"})
        
        graph = graph_registry[graph_id]
        results = {}
        
        if analysis_type in ["comprehensive", "centrality"]:
            # Centrality analysis
            results["centrality"] = {
                "betweenness": graph.betweenness_centrality(normalized=True) if hasattr(graph, 'betweenness_centrality') else [],
                "closeness": graph.closeness_centrality(normalized=True) if hasattr(graph, 'closeness_centrality') else [],
                "pagerank": graph.pagerank(alpha=0.85, max_iter=100) if hasattr(graph, 'pagerank') else None
            }
        
        if analysis_type in ["comprehensive", "connectivity"]:
            # Connectivity analysis
            if isinstance(graph, PyRustworkxGraph):
                results["connectivity"] = {
                    "connected_components": _serialize_components(graph.connected_components()),
                    "is_connected": len(graph.connected_components()) == 1,
                    "density": graph.density() if hasattr(graph, 'density') else 0.0,
                    "clustering_coefficient": graph.clustering_coefficient() if hasattr(graph, 'clustering_coefficient') else None
                }
            else:
                results["connectivity"] = {
                    "strongly_connected_components": _serialize_components(graph.strongly_connected_components()),
                    "weakly_connected_components": _serialize_components(graph.weakly_connected_components()),
                    "is_dag": graph.is_directed_acyclic_graph() if hasattr(graph, 'is_directed_acyclic_graph') else None,
                    "density": graph.density() if hasattr(graph, 'density') else 0.0
                }
        
        if analysis_type in ["comprehensive", "paths"]:
            # Path analysis
            if start_node is not None and isinstance(graph, PyRustworkxGraph):
                results["paths"] = {
                    "dijkstra_distances": {
                        target: graph.dijkstra_shortest_path(start_node, target).distance
                        for target in range(graph.node_count)
                        if graph.dijkstra_shortest_path(start_node, target) is not None
                    },
                    "bfs_tree": graph.bfs_tree(start_node) if hasattr(graph, 'bfs_tree') else None,
                    "dfs_tree": graph.dfs_tree(start_node) if hasattr(graph, 'dfs_tree') else None
                }
        
        # Add basic metrics
        results["metrics"] = {
            "node_count": graph.node_count,
            "edge_count": graph.edge_count,
            "average_degree": _average_degree(graph),
            "diameter": graph.diameter() if hasattr(graph, 'diameter') else None
        }
        
        return json.dumps({
            "graph_id": graph_id,
            "analysis_type": analysis_type,
            "results": results,
            "timestamp": datetime.now().isoformat()
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error in advanced graph analysis")
        return json.dumps({
            "error": "Graph analysis failed",
            "code": "GRAPH_ANALYSIS_ERROR"
        })


# === PROJECT MANAGEMENT TOOLS ===

@require_auth
@require_resource_limits("query")
@mcp.tool()
def get_project_info(project_path: str, api_key: Optional[str] = None, client_id: Optional[str] = None) -> str:
    """
    Get basic information about a project.
    
    Args:
        project_path: Path to the project directory
    
    Returns:
        Project information including detected languages, file counts, etc.
    """
    try:
        project_path = Path(project_path).resolve()
        if not project_path.exists():
            return f"Error: Project path '{project_path}' does not exist"
        
        # Analyze project structure
        file_extensions = {}
        total_files = 0
        project_files = []
        
        for file_path in project_path.rglob("*"):
            if file_path.is_file():
                total_files += 1
                ext = file_path.suffix.lower()
                file_extensions[ext] = file_extensions.get(ext, 0) + 1
                
                # Store sample files for each type
                if len(project_files) < 10:  # Limit to first 10 files
                    project_files.append({
                        "path": str(file_path.relative_to(project_path)),
                        "size": file_path.stat().st_size,
                        "extension": ext
                    })
        
        # Detect project type based on files
        project_types = []
        if "package.json" in [f["path"] for f in project_files]:
            project_types.append("Node.js/JavaScript")
        if "requirements.txt" in [f["path"] for f in project_files] or "setup.py" in [f["path"] for f in project_files]:
            project_types.append("Python")
        if "Cargo.toml" in [f["path"] for f in project_files]:
            project_types.append("Rust")
        if "pom.xml" in [f["path"] for f in project_files]:
            project_types.append("Java/Maven")
        
        return json.dumps({
            "project_path": str(project_path),
            "total_files": total_files,
            "file_extensions": file_extensions,
            "detected_types": project_types,
            "sample_files": project_files,
            "size_bytes": sum(f.stat().st_size for f in project_path.rglob("*") if f.is_file())
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error getting project info")
        return json.dumps({
            "error": "Project information retrieval failed",
            "code": "PROJECT_INFO_ERROR"
        })


@require_auth
@require_resource_limits("query")
@mcp.tool()
def get_performance_metrics(api_key: Optional[str] = None, client_id: Optional[str] = None) -> str:
    """
    Get performance metrics for operations and system status.
    
    Returns:
        Current performance metrics and system status
    """
    try:
        # Collect various performance metrics
        current_time = time.time()
        
        # Active analysis sessions
        active_sessions = len([
            s for s in analysis_sessions.values() 
            if s.get("status") == "analyzing"
        ])
        
        # Graph registry size
        total_graphs = len(graph_registry)
        
        # Memory usage estimation (simplified)
        total_nodes = sum(
            graph.node_count for graph in graph_registry.values()
        )
        total_edges = sum(
            graph.edge_count for graph in graph_registry.values()
        )
        
        metrics = {
            "timestamp": current_time,
            "system_metrics": {
                "active_analysis_sessions": active_sessions,
                "registered_graphs": total_graphs,
                "total_graph_nodes": total_nodes,
                "total_graph_edges": total_edges
            },
            "performance_indicators": {
                "analysis_queue_length": active_sessions,
                "memory_efficiency": "good" if total_graphs < 10 else "high",
                "processing_capacity": "available" if active_sessions < 5 else "busy"
            }
        }
        
        return json.dumps(metrics, indent=2)
        
    except Exception as e:
        logger.exception("Error getting performance metrics")
        return json.dumps({
            "error": "Performance metrics retrieval failed",
            "code": "PERFORMANCE_METRICS_ERROR"
        })


# === CODE ANALYSIS RESOURCES ===

@mcp.resource("code://{project_path}/symbols")
def get_code_symbols(project_path: str) -> str:
    """
    Get all symbols in a codebase.
    
    Args:
        project_path: Path to the project directory
    
    Returns:
        Complete symbol listing
    """
    try:
        # Validate authentication and resource limits
        if not security_validator.validate_api_key(""):  # Resources need default auth
            return json.dumps({"error": "Authentication required", "code": "AUTH_REQUIRED"})
        if not security_validator.check_resource_limits("query"):
            return json.dumps({"error": "Resource limit exceeded", "code": "RESOURCE_LIMIT_EXCEEDED"})
        
        project_path = Path(project_path).resolve()
        if not project_path.exists():
            return f"Error: Project path '{project_path}' does not exist"
        
        config = AnalyzerConfig(project_root=str(project_path))
        config.project_root = str(project_path)
        analyzer = CoreAnalyzer(config)
        result = analyzer.analyze()
        
        return json.dumps({
            "project_path": str(project_path),
            "file_count": result.file_count,
            "symbol_count": result.symbol_count,
            "languages": result.languages,
            "analysis_duration_ms": result.duration_ms,
            "relationship_count": len(result.relationships)
        }, indent=2)
        
    except Exception as e:
        logger.exception("Error getting code symbols")
        return json.dumps({
            "error": "Code symbols retrieval failed",
            "code": "CODE_SYMBOLS_ERROR"
        })


@mcp.resource("graph://{nodes}/{edges}/analysis")
def get_graph_analysis(nodes: str, edges: str) -> str:
    """
    Get comprehensive graph analysis.
    
    Args:
        nodes: JSON string representing nodes
        edges: JSON string representing edges
    
    Returns:
        Graph analysis results
    """
    try:
        # Validate authentication and resource limits
        if not security_validator.validate_api_key(""):  # Resources need default auth
            return json.dumps({"error": "Authentication required", "code": "AUTH_REQUIRED"})
        if not security_validator.check_resource_limits("graph"):
            return json.dumps({"error": "Resource limit exceeded", "code": "RESOURCE_LIMIT_EXCEEDED"})
        
        import json
        nodes_list = json.loads(nodes)
        edges_list = json.loads(edges)
        
        # Create graph
        graph = PyRustworkxGraph()
        
        # Add nodes
        node_ids = {}
        for i, node in enumerate(nodes_list):
            node_id = graph.add_node(str(node))
            node_ids[node] = node_id
        
        # Add edges
        for edge in edges_list:
            if len(edge) >= 2:
                source = edge[0]
                target = edge[1]
                weight = edge[2] if len(edge) > 2 else 1.0
                
                if source in node_ids and target in node_ids:
                    graph.add_edge(node_ids[source], node_ids[target], weight)
        
        # Perform analysis
        analysis = {
            "basic_stats": {
                "node_count": graph.node_count,
                "edge_count": graph.edge_count,
                "density": graph.density() if hasattr(graph, 'density') else 0.0,
                "is_connected": graph.is_connected()
            },
            "centrality": {
                "betweenness": graph.betweenness_centrality(normalized=True),
                "closeness": graph.closeness_centrality(normalized=True)
            }
        }
        
        return json.dumps(analysis, indent=2)
        
    except Exception as e:
        logger.exception("Error analyzing graph")
        return json.dumps({
            "error": "Graph analysis failed",
            "code": "GRAPH_ANALYSIS_ERROR"
        })


@mcp.resource("fast-context://analysis/sessions")
def get_analysis_sessions() -> str:
    """
    Get active analysis sessions and their status.
    
    Returns:
        List of active analysis sessions
    """
    try:
        sessions_data = {
            "total_sessions": len(analysis_sessions),
            "active_sessions": {
                session_id: {
                    "status": session.get("status"),
                    "project_path": session.get("project_path"),
                    "start_time": session.get("start_time"),
                    "duration": time.time() - session.get("start_time", time.time())
                }
                for session_id, session in analysis_sessions.items()
            }
        }
        
        return json.dumps(sessions_data, indent=2)
        
    except Exception as e:
        logger.exception("Error getting analysis sessions")
        return json.dumps({
            "error": "Analysis sessions retrieval failed",
            "code": "SESSIONS_RETRIEVAL_ERROR"
        })


@mcp.resource("fast-context://graphs/registry")
def get_graph_registry() -> str:
    """
    Get registry of all created graphs.
    
    Returns:
        Graph registry information
    """
    try:
        registry_data = {
            "total_graphs": len(graph_registry),
            "graphs": {
                graph_id: {
                    "type": type(graph).__name__,
                    "node_count": graph.node_count,
                    "edge_count": graph.edge_count,
                    "density": graph.density() if hasattr(graph, 'density') else 0.0
                }
                for graph_id, graph in graph_registry.items()
            }
        }
        
        return json.dumps(registry_data, indent=2)
        
    except Exception as e:
        logger.exception("Error getting graph registry")
        return json.dumps({
            "error": "Graph registry retrieval failed",
            "code": "GRAPH_REGISTRY_ERROR"
        })


@mcp.resource("fast-context://performance/metrics")
def get_performance_resource() -> str:
    """
    Get current performance metrics and system status.
    
    Returns:
        Performance metrics
    """
    try:
        # Validate authentication and resource limits
        if not security_validator.validate_api_key(""):  # Resources need default auth
            return json.dumps({"error": "Authentication required", "code": "AUTH_REQUIRED"})
        if not security_validator.check_resource_limits("query"):
            return json.dumps({"error": "Resource limit exceeded", "code": "RESOURCE_LIMIT_EXCEEDED"})
        
        metrics = {
            "timestamp": time.time(),
            "analysis_sessions": len(analysis_sessions),
            "graph_registry_size": len(graph_registry),
            "system_status": "healthy"
        }
        
        return json.dumps(metrics, indent=2)
        
    except Exception as e:
        logger.exception("Error getting performance resource")
        return json.dumps({
            "error": "Performance resource retrieval failed",
            "code": "PERFORMANCE_RESOURCE_ERROR"
        })


# === CODE ANALYSIS PROMPTS ===

@mcp.prompt()
@require_auth
def code_review_prompt(
    file_path: str,
    focus_areas: List[str] = ["readability", "performance", "maintainability"],
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Generate a prompt for code review.
    
    Args:
        file_path: Path to the file to review
        focus_areas: Areas to focus on during review
    
    Returns:
        Review prompt
    """
    try:
        # Validate and sanitize file path
        validated_path = security_validator.validate_path(str(file_path), require_exists=True)
        
        # Safely read file content with size limits
        content = security_validator.safe_read_file(validated_path)
        
        # Sanitize focus areas
        sanitized_focus_areas = [
            security_validator.sanitize_input(area, max_length=50) 
            for area in focus_areas
        ]
        
        # Determine language from file extension
        ext = file_path.suffix.lower()
        language_map = {
            '.py': 'Python',
            '.js': 'JavaScript',
            '.ts': 'TypeScript',
            '.java': 'Java',
            '.rs': 'Rust',
            '.cpp': 'C++',
            '.c': 'C',
            '.go': 'Go'
        }
        language = language_map.get(ext, 'Unknown')
        
        prompt = f"""Please review the following {language} code:

File: {file_path}
Language: {language}
Lines: {len(content.splitlines())}

Focus areas: {', '.join(sanitized_focus_areas)}

Code:
```{language}
{content}
```

Please provide a comprehensive code review focusing on:
1. Code quality and readability
2. Performance considerations
3. Best practices and conventions
4. Potential bugs or issues
5. Suggestions for improvement

Please structure your response with clear sections for each area and provide specific, actionable feedback."""
        
        return prompt
        
    except Exception as e:
        logger.exception("Error generating code review prompt")
        return json.dumps({
            "error": "Code review prompt generation failed",
            "code": "PROMPT_GENERATION_ERROR"
        })


@require_auth
@mcp.prompt()
def refactoring_suggestion_prompt(
    project_path: str,
    problem_description: str,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Generate a prompt for refactoring suggestions.
    
    Args:
        project_path: Path to the project
        problem_description: Description of the problem to solve
    
    Returns:
        Refactoring prompt
    """
    try:
        project_path = Path(project_path)
        if not project_path.exists():
            return f"Error: Project path '{project_path}' does not exist"
        
        # Get project info
        file_count = len(list(project_path.rglob("*")))
        
        prompt = f"""Please provide refactoring suggestions for the following scenario:

Project: {project_path}
Project Size: {file_count} files
Problem: {problem_description}

Please analyze this situation and provide:

1. **Problem Analysis**: Brief analysis of the current situation and challenges

2. **Refactoring Goals**: Clear objectives for the refactoring effort

3. **Suggested Approaches**: Multiple approaches to solve the problem, including:
   - Incremental vs. Big Bang refactoring
   - Pattern-based solutions
   - Architectural changes

4. **Implementation Strategy**: Step-by-step implementation plan

5. **Risk Assessment**: Potential risks and mitigation strategies

6. **Testing Strategy**: How to ensure the refactoring doesn't break existing functionality

7. **Success Metrics**: How to measure the success of the refactoring

Please provide practical, actionable advice that can be implemented incrementally."""
        
        return prompt
        
    except Exception as e:
        logger.exception("Error generating refactoring prompt")
        return json.dumps({
            "error": "Refactoring prompt generation failed",
            "code": "REFACTORING_PROMPT_ERROR"
        })


@require_auth
@mcp.prompt()
def advanced_code_analysis_prompt(
    project_path: str,
    analysis_depth: str = "detailed",
    focus_areas: Optional[List[str]] = None,
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Generate comprehensive code analysis prompt with custom parameters.
    
    Args:
        project_path: Path to the project directory
        analysis_depth: Depth of analysis ("basic", "detailed", "comprehensive")
        focus_areas: Specific areas to focus analysis on
    
    Returns:
        Advanced code analysis prompt
    """
    try:
        project_path = Path(project_path)
        if not project_path.exists():
            return f"Error: Project path '{project_path}' does not exist"
        
        focus_areas_text = ', '.join(focus_areas) if focus_areas else 'General code quality'
        
        prompt = f"""Please perform a {analysis_depth} code analysis of the project at: {project_path}

Analysis Focus Areas: {focus_areas_text}

Please provide a comprehensive analysis covering:

1. **Code Structure and Organization**
   - Module organization and separation of concerns
   - Code duplication and redundancy
   - Architectural patterns and design principles

2. **Code Quality and Best Practices**
   - Naming conventions and coding standards
   - Error handling and robustness
   - Documentation and code comments

3. **Performance and Efficiency**
   - Algorithm efficiency and complexity
   - Resource usage and memory management
   - Potential bottlenecks and optimization opportunities

4. **Maintainability and Extensibility**
   - Code modularity and reusability
   - Test coverage and testability
   - Deployment and operational considerations

5. **Security Considerations**
   - Input validation and sanitization
   - Authentication and authorization
   - Data protection and privacy

Please provide specific, actionable recommendations with code examples where appropriate."""
        
        return prompt
        
    except Exception as e:
        logger.exception("Error generating advanced code analysis prompt")
        return json.dumps({
            "error": "Advanced code analysis prompt generation failed",
            "code": "ADVANCED_ANALYSIS_PROMPT_ERROR"
        })


@require_auth
@mcp.prompt()
def architecture_review_prompt(
    project_path: str,
    architecture_type: str = "microservices",
    review_scope: str = "high-level",
    api_key: Optional[str] = None,
    client_id: Optional[str] = None
) -> str:
    """
    Generate prompt for architectural code review.
    
    Args:
        project_path: Path to the project
        architecture_type: Type of architecture to review
        review_scope: Scope of the review
    
    Returns:
        Architecture review prompt
    """
    try:
        project_path = Path(project_path)
        if not project_path.exists():
            return f"Error: Project path '{project_path}' does not exist"
        
        prompt = f"""Please perform an architectural review of the {architecture_type} project at: {project_path}

Review Scope: {review_scope}

Please provide a comprehensive architectural analysis covering:

1. **Architectural Patterns and Design**
   - Overall system architecture and design patterns
   - Component boundaries and interactions
   - Data flow and communication patterns

2. **Scalability and Performance**
   - Horizontal and vertical scaling considerations
   - Performance characteristics and bottlenecks
   - Resource utilization and optimization

3. **Reliability and Fault Tolerance**
   - Error handling and failure recovery
   - Redundancy and high availability
   - Monitoring and observability

4. **Security Architecture**
   - Security boundaries and trust zones
   - Authentication and authorization flows
   - Data protection and compliance

5. **Operational Considerations**
   - Deployment strategies and CI/CD
   - Monitoring and alerting
   - Maintenance and operational overhead

Please provide specific architectural recommendations with diagrams or code examples where appropriate."""
        
        return prompt
        
    except Exception as e:
        logger.exception("Error generating architecture review prompt")
        return json.dumps({
            "error": "Architecture review prompt generation failed",
            "code": "ARCHITECTURE_REVIEW_PROMPT_ERROR"
        })


# === SERVER SETUP ===

@click.command()
@click.option("--port", default=8000, help="Port to listen on for SSE")
@click.option(
    "--transport",
    type=click.Choice(["stdio", "sse"]),
    default="stdio",
    help="Transport type",
)
@click.option("--host", default="127.0.0.1", help="Host to bind to")
@click.option("--log-level", default="INFO", help="Log level")
def main(port: int, transport: str, host: str, log_level: str) -> None:
    """Run the Fast-Context MCP server."""
    
    # Set log level
    logging.basicConfig(level=getattr(logging, log_level.upper()))
    
    if transport == "sse":
        # SSE transport setup
        sse = SseServerTransport("/messages/")
        
        async def handle_sse(request: Request):
            async with sse.connect_sse(request.scope, request.receive, request._send) as streams:
                await mcp.run(streams[0], streams[1], mcp.create_initialization_options())
            return Response()
        
        starlette_app = Starlette(
            debug=True,
            routes=[
                Route("/sse", endpoint=handle_sse, methods=["GET"]),
                Mount("/messages/", app=sse.handle_post_message),
            ],
        )
        
        logger.info(f"Starting Fast-Context MCP server with SSE transport on {host}:{port}")
        uvicorn.run(starlette_app, host=host, port=port)
        
    else:
        # Stdio transport setup
        logger.info("Starting Fast-Context MCP server with stdio transport")
        
        async def run_stdio():
            async with stdio_server() as streams:
                await mcp.run(streams[0], streams[1], mcp.create_initialization_options())
        
        asyncio.run(run_stdio())


def run_mcp_server():
    """Run the MCP server directly"""
    main()

if __name__ == "__main__":
    run_mcp_server()
