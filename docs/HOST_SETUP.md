# Fast-Context Host Setup

This document covers the phase-1 host setup paths for:

- Codex
- Claude Code

## Codex

### Skills

Codex skills are shipped in:

- [.agents/skills](/C:/Users/shawn/workspace/fast-context/.agents/skills)

These are Codex-native skill directories and should be used through Codex's normal skills model.

### MCP

Preferred setup when `fast-context-mcp` is installed on `PATH`:

```bash
codex mcp add fast-context -- fast-context-mcp
```

Development fallback:

```bash
codex mcp add fast-context -- cargo run --quiet --bin fast-context-mcp --features mcp
```

Useful local diagnostics:

```bash
fast-context --format json mcp doctor
fast-context --format json mcp init --stdout
```

### Hooks

Codex hooks are optional and should be treated as phase 2.

They are not required for the baseline Fast-Context integration.

## Claude Code

Claude integration is packaged as a native Claude plugin skeleton in:

- [plugin](/C:/Users/shawn/workspace/fast-context/plugin)

The plugin includes:

- [plugin/.claude-plugin/plugin.json](/C:/Users/shawn/workspace/fast-context/plugin/.claude-plugin/plugin.json)
- [plugin/.mcp.json](/C:/Users/shawn/workspace/fast-context/plugin/.mcp.json)
- [plugin/skills](/C:/Users/shawn/workspace/fast-context/plugin/skills)

### MCP

The plugin `.mcp.json` assumes:

```bash
fast-context-mcp
```

is available on `PATH`.

### Hooks

Claude hooks are optional and not part of the phase-1 plugin skeleton.

## Recommended Runtime Setup

For local development:

- install the Rust CLI binary with `--features cli`
- install the Rust MCP binary with `--features mcp`

Examples:

```bash
cargo build --release --bin fast-context --features cli
cargo build --release --bin fast-context-mcp --features mcp
```

For real release usage, prefer released binaries on `PATH` instead of `cargo run` fallbacks.
