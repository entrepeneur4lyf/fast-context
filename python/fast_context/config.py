"""
Configuration management for Fast-Context.

Provides unified configuration loading and validation with support for
multiple formats (TOML, YAML, JSON) and configuration sources.
"""

import os
import json
from pathlib import Path
from typing import Dict, Any, Optional, Union, List
from dataclasses import dataclass, field, asdict
import jsonschema


@dataclass
class AnalysisConfig:
    """Configuration for codebase analysis"""
    max_files: int = 1000
    max_memory_mb: int = 512
    parallel_processing: bool = True
    worker_threads: int = 4
    exclude_patterns: List[str] = field(default_factory=lambda: [
        "*.min.js",
        "*.min.css",
        "node_modules/*",
        ".git/*",
        "__pycache__/*",
        "*.pyc",
        "target/*",
        "build/*",
        "dist/*"
    ])


@dataclass
class GraphConfig:
    """Configuration for graph operations"""
    cache_size: int = 1000
    enable_advanced_algorithms: bool = True
    max_graph_nodes: int = 10000
    max_graph_edges: int = 50000


@dataclass
class MCPConfig:
    """Configuration for MCP server"""
    transport: str = "stdio"
    port: int = 8000
    enable_sse: bool = True
    host: str = "localhost"
    timeout_seconds: int = 30


@dataclass
class LoggingConfig:
    """Configuration for logging"""
    level: str = "INFO"
    format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    enable_file_logging: bool = False
    log_file_path: Optional[str] = None


@dataclass
class FastContextConfig:
    """Main Fast-Context configuration"""
    analysis: AnalysisConfig = field(default_factory=AnalysisConfig)
    graph: GraphConfig = field(default_factory=GraphConfig)
    mcp: MCPConfig = field(default_factory=MCPConfig)
    logging: LoggingConfig = field(default_factory=LoggingConfig)


# JSON Schema for configuration validation
CONFIG_SCHEMA = {
    "type": "object",
    "properties": {
        "analysis": {
            "type": "object",
            "properties": {
                "max_files": {"type": "integer", "minimum": 1},
                "max_memory_mb": {"type": "integer", "minimum": 1},
                "parallel_processing": {"type": "boolean"},
                "worker_threads": {"type": "integer", "minimum": 1},
                "exclude_patterns": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            }
        },
        "graph": {
            "type": "object",
            "properties": {
                "cache_size": {"type": "integer", "minimum": 0},
                "enable_advanced_algorithms": {"type": "boolean"},
                "max_graph_nodes": {"type": "integer", "minimum": 1},
                "max_graph_edges": {"type": "integer", "minimum": 1}
            }
        },
        "mcp": {
            "type": "object",
            "properties": {
                "transport": {"type": "string", "enum": ["stdio", "sse"]},
                "port": {"type": "integer", "minimum": 1, "maximum": 65535},
                "enable_sse": {"type": "boolean"},
                "host": {"type": "string"},
                "timeout_seconds": {"type": "integer", "minimum": 1}
            }
        },
        "logging": {
            "type": "object",
            "properties": {
                "level": {"type": "string", "enum": ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]},
                "format": {"type": "string"},
                "enable_file_logging": {"type": "boolean"},
                "log_file_path": {"type": ["string", "null"]}
            }
        }
    },
    "additionalProperties": False
}


class ConfigManager:
    """Manages Fast-Context configuration loading and validation"""
    
    def __init__(self):
        self.config = FastContextConfig()
        self.config_paths = self._get_default_config_paths()
        self.schema = CONFIG_SCHEMA
    
    def _get_default_config_paths(self) -> List[Path]:
        """Get default configuration file search paths"""
        return [
            Path.home() / ".fast-context" / "config.toml",
            Path.home() / ".fast-context" / "config.yaml",
            Path.home() / ".fast-context" / "config.yml",
            Path.home() / ".fast-context" / "config.json",
            Path.cwd() / "fast-context.toml",
            Path.cwd() / "fast-context.yaml",
            Path.cwd() / "fast-context.yml",
            Path.cwd() / "fast-context.json",
        ]
    
    def load_config(self, config_path: Optional[Union[str, Path]] = None) -> FastContextConfig:
        """Load configuration from file or use defaults"""
        if config_path:
            config_path = Path(config_path)
            if not config_path.exists():
                raise FileNotFoundError(f"Configuration file not found: {config_path}")
            return self._load_from_file(config_path)
        else:
            # Search for configuration in default locations
            for path in self.config_paths:
                if path.exists():
                    return self._load_from_file(path)
            
            # Use default configuration
            return FastContextConfig()
    
    def _load_from_file(self, config_path: Path) -> FastContextConfig:
        """Load configuration from a specific file"""
        try:
            with open(config_path, 'r', encoding='utf-8') as f:
                if config_path.suffix == '.toml':
                    import pytomlpp
                    data = pytomlpp.load(f)
                elif config_path.suffix in ['.yaml', '.yml']:
                    import yaml
                    data = yaml.safe_load(f)
                elif config_path.suffix == '.json':
                    data = json.load(f)
                else:
                    raise ValueError(f"Unsupported configuration format: {config_path.suffix}")
            
            # Validate configuration
            self._validate_config(data)
            
            # Convert to configuration objects
            return self._dict_to_config(data)
            
        except Exception as e:
            raise ValueError(f"Failed to load configuration from {config_path}: {e}")
    
    def _validate_config(self, data: Dict[str, Any]) -> None:
        """Validate configuration against schema"""
        try:
            jsonschema.validate(data, self.schema)
        except jsonschema.ValidationError as e:
            raise ValueError(f"Configuration validation error: {e.message}")
    
    def _dict_to_config(self, data: Dict[str, Any]) -> FastContextConfig:
        """Convert dictionary to configuration objects"""
        # Extract section data with defaults
        analysis_data = data.get('analysis', {})
        graph_data = data.get('graph', {})
        mcp_data = data.get('mcp', {})
        logging_data = data.get('logging', {})
        
        # Create configuration objects
        analysis_config = AnalysisConfig(**analysis_data)
        graph_config = GraphConfig(**graph_data)
        mcp_config = MCPConfig(**mcp_data)
        logging_config = LoggingConfig(**logging_data)
        
        return FastContextConfig(
            analysis=analysis_config,
            graph=graph_config,
            mcp=mcp_config,
            logging=logging_config
        )
    
    def save_config(self, config: FastContextConfig, config_path: Union[str, Path], 
                   format: Optional[str] = None) -> None:
        """Save configuration to file"""
        config_path = Path(config_path)
        
        # Determine format from file extension if not specified
        if format is None:
            if config_path.suffix == '.toml':
                format = 'toml'
            elif config_path.suffix in ['.yaml', '.yml']:
                format = 'yaml'
            elif config_path.suffix == '.json':
                format = 'json'
            else:
                raise ValueError(f"Cannot determine format from file extension: {config_path.suffix}")
        
        # Create parent directories
        config_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Convert configuration to dictionary
        config_dict = asdict(config)
        
        # Save to file
        try:
            with open(config_path, 'w', encoding='utf-8') as f:
                if format == 'toml':
                    import pytomlpp
                    pytomlpp.dump(config_dict, f)
                elif format == 'yaml':
                    import yaml
                    yaml.dump(config_dict, f, default_flow_style=False, sort_keys=False)
                elif format == 'json':
                    json.dump(config_dict, f, indent=2, ensure_ascii=False)
                else:
                    raise ValueError(f"Unsupported format: {format}")
        except Exception as e:
            raise ValueError(f"Failed to save configuration to {config_path}: {e}")
    
    def get_config(self) -> FastContextConfig:
        """Get current configuration"""
        return self.config
    
    def update_config(self, config: FastContextConfig) -> None:
        """Update current configuration"""
        self.config = config
    
    def merge_config(self, override_config: Dict[str, Any]) -> FastContextConfig:
        """Merge override configuration with current configuration"""
        base_dict = asdict(self.config)
        merged_dict = self._deep_merge(base_dict, override_config)
        return self._dict_to_config(merged_dict)
    
    def _deep_merge(self, base: Dict[str, Any], override: Dict[str, Any]) -> Dict[str, Any]:
        """Deep merge two dictionaries"""
        result = base.copy()
        
        for key, value in override.items():
            if key in result and isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = self._deep_merge(result[key], value)
            else:
                result[key] = value
        
        return result
    
    def create_default_config(self, config_path: Union[str, Path], 
                            format: str = 'toml') -> None:
        """Create a default configuration file"""
        default_config = FastContextConfig()
        self.save_config(default_config, config_path, format)
    
    def validate_config_file(self, config_path: Union[str, Path]) -> bool:
        """Validate a configuration file"""
        try:
            config = self.load_config(config_path)
            return True
        except Exception:
            return False
    
    def get_env_overrides(self) -> Dict[str, Any]:
        """Get configuration overrides from environment variables"""
        overrides = {}
        
        # Analysis overrides
        if 'FAST_CONTEXT_MAX_FILES' in os.environ:
            overrides.setdefault('analysis', {})['max_files'] = int(os.environ['FAST_CONTEXT_MAX_FILES'])
        
        if 'FAST_CONTEXT_MAX_MEMORY_MB' in os.environ:
            overrides.setdefault('analysis', {})['max_memory_mb'] = int(os.environ['FAST_CONTEXT_MAX_MEMORY_MB'])
        
        if 'FAST_CONTEXT_WORKER_THREADS' in os.environ:
            overrides.setdefault('analysis', {})['worker_threads'] = int(os.environ['FAST_CONTEXT_WORKER_THREADS'])
        
        # MCP overrides
        if 'FAST_CONTEXT_MCP_PORT' in os.environ:
            overrides.setdefault('mcp', {})['port'] = int(os.environ['FAST_CONTEXT_MCP_PORT'])
        
        if 'FAST_CONTEXT_MCP_TRANSPORT' in os.environ:
            overrides.setdefault('mcp', {})['transport'] = os.environ['FAST_CONTEXT_MCP_TRANSPORT']
        
        # Logging overrides
        if 'FAST_CONTEXT_LOG_LEVEL' in os.environ:
            overrides.setdefault('logging', {})['level'] = os.environ['FAST_CONTEXT_LOG_LEVEL']
        
        return overrides


# Global configuration manager instance
_config_manager = None


def get_config_manager() -> ConfigManager:
    """Get the global configuration manager instance"""
    global _config_manager
    if _config_manager is None:
        _config_manager = ConfigManager()
    return _config_manager


def load_config(config_path: Optional[Union[str, Path]] = None) -> FastContextConfig:
    """Load configuration with optional environment variable overrides"""
    manager = get_config_manager()
    config = manager.load_config(config_path)
    
    # Apply environment variable overrides
    env_overrides = manager.get_env_overrides()
    if env_overrides:
        config = manager.merge_config(env_overrides)
    
    return config


def save_config(config: FastContextConfig, config_path: Union[str, Path], 
                format: Optional[str] = None) -> None:
    """Save configuration to file"""
    manager = get_config_manager()
    manager.save_config(config, config_path, format)


def create_default_config(config_path: Union[str, Path], format: str = 'toml') -> None:
    """Create a default configuration file"""
    manager = get_config_manager()
    manager.create_default_config(config_path, format)