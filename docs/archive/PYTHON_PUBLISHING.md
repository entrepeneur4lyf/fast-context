# Fast-Context Python SDK - PyPI Publishing Guide

This guide covers how to prepare and publish the Fast-Context Python SDK to PyPI.

## 📦 Package Overview

The Fast-Context Python SDK is a high-performance codebase analysis engine with:

- **Rust Core**: Built with PyO3 for maximum performance
- **Multi-Language Support**: 20+ programming languages via Tree-sitter
- **Graph Analysis**: Advanced dependency analysis using rustworkx
- **AI Integration**: Built-in MCP server for AI assistants
- **CLI Tools**: Comprehensive command-line interface

## 🚀 Quick Publishing

### Prerequisites

1. **Install Publishing Tools**
   ```bash
   pip install maturin twine
   ```

2. **PyPI Account**: Ensure you have PyPI and Test PyPI accounts
   ```bash
   # Configure PyPI credentials
   pip install keyring
   keyring set https://upload.pypi.org/legacy/ __token__
   keyring set https://test.pypi.org/legacy/ __token__
   ```

### One-Command Publishing

```bash
# Test publishing (recommended first)
python scripts/publish_python.py --test

# Production publishing
python scripts/publish_python.py
```

## 📋 Step-by-Step Publishing

### 1. Pre-Publishing Checks

```bash
# Verify Rust toolchain
rustc --version
cargo --version

# Verify Python environment
python --version
pip --version

# Install dependencies
pip install maturin twine build
```

### 2. Build Package

```bash
# Clean previous builds
rm -rf dist/ target/wheels/ python.egg-info/

# Build with maturin (release mode)
maturin build --release --features python

# Or build in debug mode for testing
maturin build --features python
```

### 3. Test Package Locally

```bash
# Install in development mode
pip install -e . --force-reinstall

# Test basic functionality
python -c "import fast_context; print(f'Version: {fast_context.__version__}')"

# Test CLI
fast-context --version
fast-context analyze --help
```

### 4. Validate Package

```bash
# Check package with twine
twine check dist/*

# Test installation from wheel
pip install dist/fast_context-*.whl --force-reinstall
```

### 5. Publish to Test PyPI

```bash
# Upload to Test PyPI first
twine upload --repository testpypi dist/*

# Test installation from Test PyPI
pip install --index-url https://test.pypi.org/simple/ fast-context
```

### 6. Publish to Production PyPI

```bash
# Upload to production PyPI
twine upload dist/*

# Verify installation
pip install fast-context
```

## 🔧 Build Configuration

### Maturin Configuration

The build is configured in `pyproject.toml`:

```toml
[tool.maturin]
features = ["python"]
python-source = "python"
module-name = "fast_context"
strip = true
include = [
    { path = "README.md", format = "sdist" },
    { path = "python/README.md", format = "wheel" },
    { path = "python/fast_context/*.pyi", format = "wheel" },
    { path = "python/fast_context/py.typed", format = "wheel" },
]
```

### Cargo Features

The Python bindings are enabled with the `python` feature:

```toml
[features]
python = ["pyo3", "pyo3-asyncio/tokio-runtime"]
```

## 📊 Package Contents

### Wheel Contents
```
fast_context/
├── __init__.py              # Main module exports
├── fast_context.so          # Rust binary extension
├── fast_context.pyi         # Type stubs
├── graph.pyi               # Graph type stubs
├── py.typed                # Type marker
├── cli.py                  # CLI implementation
├── config.py               # Configuration system
├── mcp_server.py           # MCP server
└── setup_mcp_servers.py    # MCP setup utilities
```

### Source Distribution Contents
```
fast-context-0.1.0.tar.gz
├── README.md
├── pyproject.toml
├── Cargo.toml
├── Cargo.lock
├── src/                    # Rust source code
├── python/                 # Python source code
└── examples/               # Usage examples
```

## 🧪 Testing Strategy

### Local Testing

```bash
# Unit tests
pytest tests/test_python_bindings.py

# Integration tests
pytest tests/test_cli.py
pytest tests/test_mcp_server.py

# Performance tests
python examples/python_example.py
```

### CI/CD Testing

```bash
# Test on multiple Python versions
tox

# Test wheel installation
pip install dist/*.whl
python -c "import fast_context; fast_context.get_version()"
```

## 🔍 Quality Checks

### Package Validation

```bash
# Check package metadata
twine check dist/*

# Verify wheel contents
unzip -l dist/fast_context-*.whl

# Check for missing dependencies
pip-missing-reqs python/
```

### Security Scanning

```bash
# Scan for vulnerabilities
safety check

# Check for secrets
truffleHog --regex --entropy=False .
```

## 📈 Version Management

### Semantic Versioning

- **Patch** (0.1.0 → 0.1.1): Bug fixes, minor improvements
- **Minor** (0.1.0 → 0.2.0): New features, backward compatible
- **Major** (0.1.0 → 1.0.0): Breaking changes

### Version Updates

Update version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"
```

The Python package version is automatically derived from Cargo.toml.

## 🚨 Troubleshooting

### Common Issues

**1. Build Failures**
```bash
# Update Rust toolchain
rustup update

# Clear cache
cargo clean
rm -rf target/
```

**2. Import Errors**
```bash
# Check Python path
python -c "import sys; print(sys.path)"

# Reinstall package
pip uninstall fast-context
pip install fast-context
```

**3. Missing Dependencies**
```bash
# Install build dependencies
pip install maturin[patchelf]

# On macOS, install additional tools
brew install patchelf
```

**4. Upload Errors**
```bash
# Check credentials
twine check dist/*

# Re-authenticate
keyring del https://upload.pypi.org/legacy/ __token__
keyring set https://upload.pypi.org/legacy/ __token__
```

## 📋 Publishing Checklist

- [ ] Rust code compiles without warnings
- [ ] Python tests pass
- [ ] Package builds successfully
- [ ] Twine check passes
- [ ] Local installation works
- [ ] CLI commands function
- [ ] MCP server starts
- [ ] Documentation is up to date
- [ ] Version number is correct
- [ ] Test PyPI upload succeeds
- [ ] Production PyPI upload ready

## 🔄 Automated Publishing

### GitHub Actions

```yaml
name: Publish Python Package
on:
  release:
    types: [published]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: pip install maturin twine
      - run: maturin build --release --features python
      - run: twine upload dist/*
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_TOKEN }}
```

## 🎯 Post-Publishing

1. **Verify Installation**
   ```bash
   pip install fast-context
   fast-context --version
   ```

2. **Update Documentation**
   - Update installation instructions
   - Create release notes
   - Update examples

3. **Monitor Usage**
   - Check PyPI download statistics
   - Monitor GitHub issues
   - Gather user feedback

4. **Plan Next Release**
   - Review feature requests
   - Plan breaking changes
   - Update roadmap

This comprehensive guide ensures successful PyPI publishing of the Fast-Context Python SDK with professional quality and reliability.
