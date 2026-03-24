# Fast-Context MCP

Use this skill when you need to set up or verify the Fast-Context MCP server for Codex or another MCP-capable host.

## Baseline

The MCP server binary is:

```bash
fast-context-mcp
```

If you do not have a released binary on `PATH`, a local Cargo fallback is:

```bash
cargo run --bin fast-context-mcp --features mcp
```

## Codex Setup

If `fast-context-mcp` is on `PATH`:

```bash
codex mcp add fast-context -- fast-context-mcp
```

Cargo fallback:

```bash
codex mcp add fast-context -- cargo run --quiet --bin fast-context-mcp --features mcp
```

## Validation

Use the Rust CLI to inspect the local MCP launch configuration:

```bash
fast-context --format json mcp doctor
fast-context --format json mcp init --stdout
```

Cargo fallback:

```bash
cargo run --bin fast-context --features cli -- --format json mcp doctor
```

## What To Check

- `status`
- `server_command`
- `arguments`
- `binary_path`
- any `warnings` or `errors`

## Interpretation

- `status: ok` means the current host can likely start the MCP server
- `status: warning` usually means the binary was not found and the CLI is falling back to `cargo run`
- `cargo run` fallback is acceptable for development, but a released binary on `PATH` is the cleaner production setup
