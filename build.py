#!/usr/bin/env python3
"""
Build script for Fast-Context Python package
"""

import os
import sys
import subprocess
import platform
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and return the result"""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    print(result.stdout)
    return result

def build_wheels():
    """Build wheels for current platform"""
    print("Building wheels...")
    run_command(["maturin", "build", "--release", "--out", "dist"])
    
def build_sdist():
    """Build source distribution"""
    print("Building source distribution...")
    run_command(["maturin", "sdist", "--out", "dist"])
    
def install_dev():
    """Install in development mode"""
    print("Installing in development mode...")
    run_command(["maturin", "develop", "--release"])
    
def run_tests():
    """Run tests"""
    print("Running tests...")
    run_command(["python", "-m", "pytest", "tests/", "-v"])
    
def lint_code():
    """Run linting"""
    print("Running linting...")
    run_command(["ruff", "check", "python/", "tests/"])
    run_command(["black", "--check", "python/", "tests/"])
    run_command(["mypy", "python/"])

def format_code():
    """Format code"""
    print("Formatting code...")
    run_command(["black", "python/", "tests/"])
    run_command(["ruff", "check", "--fix", "python/", "tests/"])

def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        print("Usage: python build.py <command>")
        print("Commands:")
        print("  build      - Build wheels and sdist")
        print("  wheels     - Build wheels only")
        print("  sdist      - Build source distribution only")
        print("  install    - Install in development mode")
        print("  test       - Run tests")
        print("  lint       - Run linting")
        print("  format     - Format code")
        print("  all        - Build, test, and lint")
        sys.exit(1)
    
    command = sys.argv[1]
    
    if command == "build":
        build_wheels()
        build_sdist()
    elif command == "wheels":
        build_wheels()
    elif command == "sdist":
        build_sdist()
    elif command == "install":
        install_dev()
    elif command == "test":
        run_tests()
    elif command == "lint":
        lint_code()
    elif command == "format":
        format_code()
    elif command == "all":
        install_dev()
        run_tests()
        lint_code()
        build_wheels()
        build_sdist()
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)
    
    print("Done!")

if __name__ == "__main__":
    main()