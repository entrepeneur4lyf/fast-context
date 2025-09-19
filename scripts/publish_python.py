#!/usr/bin/env python3
"""
PyPI Publishing Script for Fast-Context Python SDK

This script automates the process of building and publishing the Fast-Context
Python package to PyPI using maturin.
"""

import subprocess
import sys
import os
import shutil
from pathlib import Path
import argparse


def run_command(cmd, cwd=None, check=True, env=None):
    """Run a command and return the result."""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True, env=env)
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    return result


def check_dependencies():
    """Check if required tools are installed."""
    required_tools = ["maturin", "twine"]
    missing = []
    
    for tool in required_tools:
        try:
            run_command([tool, "--version"])
        except (subprocess.CalledProcessError, FileNotFoundError):
            missing.append(tool)
    
    if missing:
        print(f"Missing required tools: {', '.join(missing)}")
        print("Install with: pip install maturin twine")
        return False
    
    return True


def clean_build():
    """Clean previous build artifacts."""
    print("🧹 Cleaning build artifacts...")
    
    # Remove build directories
    dirs_to_clean = ["dist", "target/wheels", "python.egg-info", "python/python.egg-info"]
    for dir_path in dirs_to_clean:
        if os.path.exists(dir_path):
            shutil.rmtree(dir_path)
            print(f"Removed {dir_path}")
    
    # Remove compiled Python files
    for root, dirs, files in os.walk("."):
        for file in files:
            if file.endswith((".pyc", ".pyo")):
                os.remove(os.path.join(root, file))
        if "__pycache__" in dirs:
            shutil.rmtree(os.path.join(root, "__pycache__"))


def build_package(release=True):
    """Build the Python package using maturin."""
    print("🔨 Building Python package...")

    cmd = ["maturin", "build"]
    if release:
        cmd.append("--release")

    # Add features
    cmd.extend(["--features", "python"])

    # Set environment for PyO3 compatibility
    env = os.environ.copy()
    env["PYO3_USE_ABI3_FORWARD_COMPATIBILITY"] = "1"

    result = run_command(cmd, env=env)

    if result.returncode == 0:
        print("✅ Package built successfully!")
        return True
    else:
        print("❌ Package build failed!")
        return False


def test_package():
    """Test the built package."""
    print("🧪 Testing built package...")
    
    # Install in development mode
    try:
        run_command(["pip", "install", "-e", ".", "--force-reinstall"])
        
        # Run basic import test
        run_command([sys.executable, "-c", "import fast_context; print(f'Version: {fast_context.__version__}')"])
        
        # Run CLI test
        run_command(["fast-context", "--version"])
        
        print("✅ Package tests passed!")
        return True
        
    except subprocess.CalledProcessError:
        print("❌ Package tests failed!")
        return False


def check_package():
    """Check package with twine."""
    print("🔍 Checking package with twine...")

    # Check both dist/ and target/wheels/ directories
    dist_files = (list(Path("dist").glob("*.whl")) + list(Path("dist").glob("*.tar.gz")) +
                  list(Path("target/wheels").glob("*.whl")))

    if not dist_files:
        print("❌ No distribution files found!")
        return False

    for dist_file in dist_files:
        try:
            run_command(["twine", "check", str(dist_file)])
        except subprocess.CalledProcessError:
            print(f"❌ Package check failed for {dist_file}")
            return False

    print("✅ Package check passed!")
    return True


def publish_package(test_pypi=False):
    """Publish package to PyPI."""
    if test_pypi:
        print("📦 Publishing to Test PyPI...")
        repository = "testpypi"
    else:
        print("📦 Publishing to PyPI...")
        repository = "pypi"

    # Check both dist/ and target/wheels/ directories
    dist_files = (list(Path("dist").glob("*.whl")) + list(Path("dist").glob("*.tar.gz")) +
                  list(Path("target/wheels").glob("*.whl")))

    if not dist_files:
        print("❌ No distribution files found!")
        return False

    cmd = ["twine", "upload", "--repository", repository] + [str(f) for f in dist_files]

    try:
        run_command(cmd)
        print("✅ Package published successfully!")
        return True
    except subprocess.CalledProcessError:
        print("❌ Package publishing failed!")
        return False


def main():
    """Main publishing workflow."""
    parser = argparse.ArgumentParser(description="Publish Fast-Context Python package to PyPI")
    parser.add_argument("--test", action="store_true", help="Publish to Test PyPI instead of PyPI")
    parser.add_argument("--dry-run", action="store_true", help="Build and check package without publishing")
    parser.add_argument("--skip-tests", action="store_true", help="Skip package testing")
    parser.add_argument("--debug", action="store_true", help="Build in debug mode")
    
    args = parser.parse_args()
    
    print("🚀 Fast-Context Python Package Publisher")
    print("=" * 50)
    
    # Check dependencies
    if not check_dependencies():
        sys.exit(1)
    
    # Clean build
    clean_build()
    
    # Build package
    if not build_package(release=not args.debug):
        sys.exit(1)
    
    # Test package
    if not args.skip_tests:
        if not test_package():
            sys.exit(1)
    
    # Check package
    if not check_package():
        sys.exit(1)
    
    # Publish package
    if not args.dry_run:
        if not publish_package(test_pypi=args.test):
            sys.exit(1)
    else:
        print("🏁 Dry run completed successfully!")
        print("To publish, run without --dry-run flag")
    
    print("🎉 Publishing workflow completed successfully!")


if __name__ == "__main__":
    main()
