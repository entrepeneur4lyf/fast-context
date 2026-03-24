# Release Remediation Checklist

This checklist captures the remaining production-facing issues identified in the pre-release code review.

## Priority 1

- [ ] Align the PyPI package with the current product surface.
  - [ ] Stop shipping the legacy Python CLI as the primary `fast-context` entrypoint.
  - [ ] Stop shipping the legacy Python MCP server as the primary `fast-context-mcp` entrypoint.
  - [ ] Remove unconditional Python runtime dependencies that only support the legacy Python CLI/MCP stack.
  - [ ] Keep the Python wheel focused on the Rust-backed Python bindings.

- [ ] Make `@fast-context/core` truthful or non-publishable.
  - [ ] Remove it from the coordinated public release path if it remains simulated.
  - [ ] Prevent accidental publish of the package while mock behavior remains.
  - [ ] Remove stale repository metadata if the package stays in the workspace.

## Priority 2

- [ ] Remove fabricated Python symbol metadata from the public API.
  - [ ] Replace placeholder metadata returns with explicit errors or narrower truthful APIs.
  - [ ] Avoid returning synthetic `Unknown` symbol metadata as if it were authoritative.

## Validation

- [ ] `cargo check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `npm run build:debug`
- [ ] `npm test`
- [ ] `python -m pytest tests/python`
- [ ] Verify release metadata still aligns across `Cargo.toml`, `package.json`, and `pyproject.toml`

## Release Gate

Do not cut the release until:

- the published Python package installs only truthful entrypoints
- the publishable npm workspace packages are truthful
- the Python metadata APIs no longer fabricate placeholder symbol details
