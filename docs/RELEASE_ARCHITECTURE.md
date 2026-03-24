# Fast-Context Release Architecture

## Goal

Fast-Context should ship as a coordinated multi-channel release with one version and one tag.

This project should not publish Rust, Python, and Node artifacts on unrelated timelines. A release is only complete when every supported distribution channel for that version is built, validated, and published.

## Release Contract

One release means all of the following share the same version:

- Rust crate
- Rust CLI binary
- Rust MCP binary
- Node package
- Python package
- GitHub Release artifacts

Example:

- git tag: `v0.2.0`
- Cargo crate version: `0.2.0`
- npm version: `0.2.0`
- PyPI version: `0.2.0`

If any channel fails, the release fails.

## Distribution Surfaces

### Rust

Rust is the implementation source of truth.

Rust outputs:

- library crate
- `fast-context` CLI binary
- `fast-context-mcp` MCP server binary

Primary Rust install path:

```bash
cargo install fast-context --features cli
```

Rust users should not need npm or Python to consume the project.

### Python

Python is a first-class binding and should ship as wheels and an sdist via PyPI.

Primary Python install path:

```bash
pip install fast-context
```

Python users should not compile Rust locally unless they explicitly choose to.

### Node

Node has two distinct use cases:

1. library/binding consumers
2. CLI consumers

These should not be conflated.

#### Node library package

The root npm package remains the Node-facing library/binding surface.

Primary Node library install path:

```bash
npm install fast-context
```

#### Node CLI installer package

If npm users need ergonomic CLI installation, use a thin npm package that installs prebuilt Rust binaries.

That package should:

- download the correct release artifact for the current platform
- install the prebuilt `fast-context` binary
- optionally install `fast-context-mcp`
- avoid local Rust compilation

It should not:

- run `cargo install` during `npm install`
- build Rust from source on the user machine
- require Python, a C toolchain, or a linker for normal install

### GitHub Releases

GitHub Releases are the binary distribution hub.

Each tagged release should publish platform artifacts for:

- `fast-context`
- `fast-context-mcp`

These artifacts are consumed by:

- direct users downloading binaries
- npm CLI installer packages
- internal smoke validation

## Versioning Rules

- one git tag per release
- one semver version across Cargo, npm, and PyPI
- no partial ecosystem release for a final version
- prereleases may use a shared prerelease suffix across all channels

Examples:

- `0.2.0-alpha.1`
- `0.2.0-beta.1`
- `0.2.0`

## Publish Rule

A release should publish only after all of these are green for the same version:

- Rust checks and tests
- Rust CLI tests
- Rust MCP tests
- Node build and tests
- Python build and tests
- npm pack or dry-run validation
- Python wheel and sdist validation
- GitHub Release artifact build

If any one of those fails, do not publish any channel for that version.

## Recommended Artifact Model

### GitHub Release artifacts

Use explicit binary archive names per target, for example:

- `fast-context-x86_64-unknown-linux-gnu.tar.gz`
- `fast-context-aarch64-apple-darwin.tar.gz`
- `fast-context-x86_64-pc-windows-msvc.zip`
- `fast-context-mcp-x86_64-unknown-linux-gnu.tar.gz`

Each archive should contain exactly one binary plus minimal metadata where useful.

### PyPI artifacts

Publish:

- wheels
- sdist

### npm artifacts

Publish:

- root Node library package
- optional CLI installer package if npm CLI install is supported

## Workflow Shape

The release pipeline should operate like this:

1. push tag `vX.Y.Z`
2. create draft or final GitHub Release
3. build and validate Rust binaries
4. build and validate Node package artifacts
5. build and validate Python wheels and sdist
6. publish npm package(s)
7. publish PyPI package
8. upload final release artifacts
9. mark release complete

The critical rule is ordering:

- validation before publish
- publish only after all channels are ready

## Non-Goals

This release model should not try to:

- compile Rust during end-user npm install
- publish different ecosystem versions independently
- hide platform support behind vague docs
- treat CI artifacts and published artifacts as different products

## Immediate Follow-Up

To align the repository with this architecture:

1. update deployment docs to describe the coordinated release model
2. update workflows so tag releases behave as one release train
3. ensure Python wheel builds include the Rust `python` feature where required
4. define binary artifact naming for CLI and MCP
5. decide whether to create a dedicated npm CLI installer package
