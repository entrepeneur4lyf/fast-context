# Fast-Context Deployment Guide

## Scope

This guide covers the deployment surfaces that actually exist in this repository:

- the Rust crate and binaries
- the Node.js package
- the Python package
- the GitHub Release artifacts
- the GitHub Actions release workflows

This project is a library, not a standalone web service. If you embed it in an API server or worker, that operational setup belongs to your application, not to Fast-Context itself.

The canonical release model is documented in [RELEASE_ARCHITECTURE.md](/C:/Users/shawn/workspace/fast-context/docs/RELEASE_ARCHITECTURE.md). This guide focuses on how to validate and publish that model in practice.

## Release Model

Fast-Context should release all supported surfaces together under one version:

- Cargo crate
- Rust CLI binary
- Rust MCP binary
- Node package
- Python package
- GitHub Release artifacts

Practical rule:

- if one channel fails, the release fails

Do not publish a final version to npm, PyPI, and Cargo on unrelated timelines.

## Rust

### Local Validation

```bash
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

### Binary Validation

```bash
cargo run --bin fast-context -- --help
cargo run --bin fast-context-mcp --features mcp
```

### Intended Distribution

- crate via crates.io
- prebuilt CLI and MCP binaries via GitHub Releases

## Node.js Package

### Local Build

```bash
npm install
npm run build:debug
npm test
```

### Release Build

```bash
npm run build
npm pack
```

Recommended checks before publishing:

- [README.md](/C:/Users/shawn/workspace/fast-context/README.md) quick-start examples still match the package
- [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts) matches the actual addon surface
- `npm pack` succeeds
- install the produced tarball into a clean temp project and run a smoke test

### Native Artifacts

The Node package depends on native binaries built through the release workflow. Treat GitHub Actions as the source of truth for supported targets and release artifacts.

The root npm package is the Node library surface.

If npm CLI installation is added, it should be a separate thin installer package that downloads prebuilt Rust binaries. It should not compile Rust during install.

Relevant files:

- [package.json](/C:/Users/shawn/workspace/fast-context/package.json)
- [release.yml](/C:/Users/shawn/workspace/fast-context/.github/workflows/release.yml)

## Python Package

### Local Validation

Use the Python version exercised during local qualification:

```bash
python -m pytest tests/python
cargo check --features python
```

On this repo, Python 3.11 has been the most reliable local validation target.

### Wheel Build

```bash
python -m maturin build --features python
```

Intended distribution:

- wheels and sdist via PyPI
- no local Rust compilation during normal end-user install unless explicitly chosen

Relevant files:

- [pyproject.toml](/C:/Users/shawn/workspace/fast-context/pyproject.toml)
- [build-wheels.yml](/C:/Users/shawn/workspace/fast-context/.github/workflows/build-wheels.yml)

## Release Workflows

The active release automation lives in:

- [test.yml](/C:/Users/shawn/workspace/fast-context/.github/workflows/test.yml)
- [release.yml](/C:/Users/shawn/workspace/fast-context/.github/workflows/release.yml)
- [build-wheels.yml](/C:/Users/shawn/workspace/fast-context/.github/workflows/build-wheels.yml)

Practical rule:

- trust hosted CI over a single local machine for cross-platform release status
- treat the tag-driven release pipeline as one release train, not separate ad hoc publishes

## Publish Checklist

Before tagging:

- versions across Cargo, npm, and PyPI metadata are aligned
- docs match the actual shipped CLI, MCP, Node, and Python surfaces

### Node

Run:

```bash
cargo check
cargo clippy --all-targets --all-features -- -D warnings
npm run build
npm test
npm pack
```

Then:

- test the tarball in a clean project
- verify the package contents are minimal and intentional

### Python

Run:

```bash
cargo check --features python
python -m pytest tests/python
python -m maturin build --features python
```

Then:

- inspect the built wheel
- install it in a clean environment and run a smoke test

### Rust Binaries

Run:

```bash
cargo build --release --bin fast-context
cargo build --release --bin fast-context-mcp --features mcp
```

Then:

- verify both binaries launch
- verify artifact names and target packaging are consistent with the release plan

## Operational Notes

- local native builds can rewrite [package.json](/C:/Users/shawn/workspace/fast-context/package.json) during package-prep steps; do not commit that churn unless it is intentional
- the generated Node typings in [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts) should be refreshed when the Node-facing Rust structs change
- release confidence should come from green CI, a clean tarball smoke test, a clean wheel smoke test, and verified Rust binary artifacts

## What This Guide Does Not Cover

This repository does not currently define or ship:

- a production HTTP server
- a Kubernetes deployment target
- Prometheus endpoints
- Redis cache backends
- API key middleware
- load balancer configs

If you need those, they belong in the integrating application.
