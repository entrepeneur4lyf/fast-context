#!/usr/bin/env python3
"""
Fast-Context Security Module

This module provides comprehensive security controls for the Fast-Context MCP server,
including path validation, authentication, resource limits, and access control.

Security Features:
- Path validation and sandboxing
- Authentication and authorization
- Resource limits and rate limiting
- Secure file access controls
- Input validation and sanitization
- Audit logging
"""

import os
import re
import hashlib
import hmac
import time
import logging
from pathlib import Path
from typing import Optional, List, Set, Dict, Any, Union
from dataclasses import dataclass, field
from functools import wraps
from enum import Enum

logger = logging.getLogger(__name__)


class SecurityLevel(Enum):
    """Security levels for access control."""
    READ_ONLY = "read_only"
    ANALYSIS = "analysis"
    FULL_ACCESS = "full_access"


@dataclass
class SecurityConfig:
    """Security configuration for the MCP server."""
    # Authentication
    api_keys: Set[str] = field(default_factory=set)
    enable_auth: bool = True
    
    # Path validation
    allowed_base_paths: List[Path] = field(default_factory=list)
    blocked_paths: List[Path] = field(default_factory=list)
    max_file_size: int = 10 * 1024 * 1024  # 10MB
    allowed_extensions: Set[str] = field(default_factory=lambda: {
        '.py', '.js', '.ts', '.jsx', '.tsx', '.java', '.cpp', '.c', '.h', '.hpp',
        '.rs', '.go', '.rb', '.php', '.swift', '.kt', '.scala', '.cs', '.html',
        '.css', '.scss', '.sass', '.json', '.yaml', '.yml', '.toml', '.xml',
        '.md', '.txt', '.sh', '.bash', '.zsh', '.fish'
    })
    
    # Resource limits
    max_analysis_files: int = 1000
    max_memory_mb: int = 512
    max_analysis_time_seconds: int = 300
    max_concurrent_requests: int = 5
    
    # Rate limiting
    rate_limit_requests: int = 100
    rate_limit_window_seconds: int = 60
    
    # Audit logging
    enable_audit_log: bool = True
    audit_log_file: Optional[Path] = None


class SecurityValidator:
    """Handles security validation for MCP server operations."""
    
    def __init__(self, config: SecurityConfig):
        self.config = config
        self.request_times: Dict[str, List[float]] = {}
        self.active_requests: int = 0
        
        # Initialize blocked system paths
        self._init_blocked_paths()
    
    def _init_blocked_paths(self):
        """Initialize blocked system paths for security."""
        system_paths = [
            '/etc', '/usr', '/bin', '/sbin', '/lib', '/lib64', '/proc', '/sys',
            '/boot', '/dev', '/root', '/var/log', '/var/cache', '/tmp'
        ]
        
        user_paths = [
            '~/.ssh', '~/.gnupg', '~/.config', '~/.local/share',
            '~/.cache', '~/.npm', '~/.cargo', '~/.pip'
        ]
        
        for path in system_paths:
            self.config.blocked_paths.append(Path(path).resolve())
        
        for path in user_paths:
            self.config.blocked_paths.append(Path(path).expanduser().resolve())
    
    def validate_api_key(self, api_key: str) -> bool:
        """Validate API key for authentication."""
        if not self.config.enable_auth:
            return True
        
        if not api_key:
            return False
        
        # Use constant-time comparison to prevent timing attacks
        for valid_key in self.config.api_keys:
            if hmac.compare_digest(api_key, valid_key):
                return True
        
        return False
    
    def validate_path(self, path_str: str, require_exists: bool = True) -> Path:
        """
        Validate and sanitize a file path.
        
        Args:
            path_str: Input path string
            require_exists: Whether the path must exist
            
        Returns:
            Validated and resolved Path object
            
        Raises:
            ValueError: If path is invalid or blocked
        """
        try:
            path = Path(path_str).resolve()
            
            # Check if path is blocked
            for blocked_path in self.config.blocked_paths:
                try:
                    if str(path).startswith(str(blocked_path)):
                        raise ValueError(f"Access to blocked path denied: {path}")
                except (ValueError, OSError):
                    continue
            
            # Check if path is within allowed base paths
            if self.config.allowed_base_paths:
                is_allowed = False
                for base_path in self.config.allowed_base_paths:
                    try:
                        if str(path).startswith(str(base_path.resolve())):
                            is_allowed = True
                            break
                    except (ValueError, OSError):
                        continue
                
                if not is_allowed:
                    raise ValueError(f"Path outside allowed directories: {path}")
            
            # Check if path exists (if required)
            if require_exists and not path.exists():
                raise ValueError(f"Path does not exist: {path}")
            
            # Check if path is a file (for file operations)
            if path.is_file() and not self._is_allowed_file_type(path):
                raise ValueError(f"File type not allowed: {path.suffix}")
            
            # Check file size
            if path.is_file():
                try:
                    file_size = path.stat().st_size
                    if file_size > self.config.max_file_size:
                        raise ValueError(f"File too large: {file_size} bytes (max: {self.config.max_file_size})")
                except OSError:
                    pass  # File might not be accessible
            
            return path
            
        except (OSError, ValueError) as e:
            raise ValueError(f"Invalid path: {path_str} - {str(e)}")
    
    def _is_allowed_file_type(self, path: Path) -> bool:
        """Check if file type is allowed."""
        if not path.suffix:
            return True  # Allow files without extensions
        
        return path.suffix.lower() in self.config.allowed_extensions
    
    def check_rate_limit(self, client_id: str) -> bool:
        """Check if client has exceeded rate limit."""
        now = time.time()
        
        if client_id not in self.request_times:
            self.request_times[client_id] = []
        
        # Clean up old requests
        window_start = now - self.config.rate_limit_window_seconds
        self.request_times[client_id] = [
            req_time for req_time in self.request_times[client_id]
            if req_time > window_start
        ]
        
        # Check if rate limit exceeded
        if len(self.request_times[client_id]) >= self.config.rate_limit_requests:
            return False
        
        # Add current request
        self.request_times[client_id].append(now)
        return True
    
    def check_resource_limits(self, operation: str, **kwargs) -> bool:
        """Check if operation is within resource limits."""
        # Check concurrent requests
        if self.active_requests >= self.config.max_concurrent_requests:
            return False
        
        # Check operation-specific limits
        if operation == "analyze":
            file_count = kwargs.get("file_count", 0)
            if file_count > self.config.max_analysis_files:
                return False
        
        return True
    
    def start_request(self):
        """Start tracking a request."""
        self.active_requests += 1
    
    def end_request(self):
        """End tracking a request."""
        self.active_requests = max(0, self.active_requests - 1)
    
    def safe_read_file(self, file_path: Union[str, Path], max_size: Optional[int] = None) -> str:
        """
        Safely read a file with size limits and content validation.
        
        Args:
            file_path: Path to the file
            max_size: Maximum file size in bytes (overrides config)
            
        Returns:
            File content as string
            
        Raises:
            ValueError: If file cannot be read safely
        """
        if max_size is None:
            max_size = self.config.max_file_size
        
        path = self.validate_path(str(file_path), require_exists=True)
        
        try:
            with open(path, 'r', encoding='utf-8', errors='strict') as f:
                content = f.read(max_size + 1)  # Read one extra byte to check size
                
                if len(content) > max_size:
                    raise ValueError(f"File content exceeds maximum size: {len(content)} bytes")
                
                return content
                
        except (OSError, UnicodeDecodeError) as e:
            raise ValueError(f"Cannot read file safely: {path} - {str(e)}")
    
    def sanitize_input(self, input_str: str, max_length: int = 1000) -> str:
        """
        Sanitize user input string.
        
        Args:
            input_str: Input string to sanitize
            max_length: Maximum allowed length
            
        Returns:
            Sanitized string
        """
        if not isinstance(input_str, str):
            raise ValueError("Input must be a string")
        
        # Remove null bytes and control characters
        sanitized = re.sub(r'[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]', '', input_str)
        
        # Limit length
        if len(sanitized) > max_length:
            sanitized = sanitized[:max_length]
        
        return sanitized.strip()
    
    def log_audit_event(self, event_type: str, client_id: str, details: Dict[str, Any]):
        """Log security audit event."""
        if not self.config.enable_audit_log:
            return
        
        event = {
            "timestamp": time.time(),
            "event_type": event_type,
            "client_id": client_id,
            "details": details
        }
        
        log_message = f"AUDIT: {event_type} - Client: {client_id} - {details}"
        
        if self.config.audit_log_file:
            try:
                with open(self.config.audit_log_file, 'a', encoding='utf-8') as f:
                    f.write(f"{time.isoformat()} {log_message}\n")
            except OSError:
                logger.error("Failed to write to audit log")
        
        logger.info(log_message)


def require_auth(func):
    """Decorator to require authentication for MCP tools."""
    @wraps(func)
    async def wrapper(*args, **kwargs):
        # Get security validator from first argument (self)
        if hasattr(args[0], 'security_validator'):
            validator = args[0].security_validator
            
            # Extract API key from kwargs (FastMCP passes it as a parameter)
            api_key = kwargs.get('api_key') or kwargs.get('authorization')
            
            if not validator.validate_api_key(api_key or ''):
                return json.dumps({
                    "error": "Authentication required",
                    "code": "AUTH_REQUIRED"
                })
            
            # Check rate limiting
            client_id = kwargs.get('client_id') or api_key or 'anonymous'
            if not validator.check_rate_limit(client_id):
                return json.dumps({
                    "error": "Rate limit exceeded",
                    "code": "RATE_LIMIT_EXCEEDED"
                })
            
            # Log audit event
            validator.log_audit_event(
                "tool_call",
                client_id,
                {"tool": func.__name__, "args": kwargs}
            )
        
        return await func(*args, **kwargs)
    
    return wrapper


def require_resource_limits(operation: str):
    """Decorator to enforce resource limits for MCP tools."""
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            if hasattr(args[0], 'security_validator'):
                validator = args[0].security_validator
                
                # Check resource limits
                if not validator.check_resource_limits(operation, **kwargs):
                    return json.dumps({
                        "error": "Resource limit exceeded",
                        "code": "RESOURCE_LIMIT_EXCEEDED"
                    })
                
                # Start request tracking
                validator.start_request()
                
                try:
                    return await func(*args, **kwargs)
                finally:
                    # End request tracking
                    validator.end_request()
            
            return await func(*args, **kwargs)
        
        return wrapper
    return decorator


# Default security configuration
def create_default_security_config() -> SecurityConfig:
    """Create default security configuration."""
    config = SecurityConfig()
    
    # Add current working directory as allowed base path
    config.allowed_base_paths.append(Path.cwd())
    
    # Generate a default API key for development
    config.api_keys.add("fast-context-dev-key-123456")
    
    return config


# Global security validator instance
_security_validator: Optional[SecurityValidator] = None


def get_security_validator() -> SecurityValidator:
    """Get or create the global security validator."""
    global _security_validator
    if _security_validator is None:
        config = create_default_security_config()
        _security_validator = SecurityValidator(config)
    return _security_validator


def configure_security(config: SecurityConfig):
    """Configure security with custom settings."""
    global _security_validator
    _security_validator = SecurityValidator(config)