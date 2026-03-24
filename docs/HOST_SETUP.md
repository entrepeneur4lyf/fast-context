# Fast-Context Host Setup

This document covers the phase-1 host setup paths for:

- Codex
- Claude Code
- standalone MCP clients

## Quick Matrix

| Host | Skills | MCP | Hooks | Recommended path |
|------|--------|-----|-------|------------------|
| Codex | Native `.agents/skills` directories | Native `codex mcp` / `config.toml` | Optional beta feature | Use the repo skills and add `fast-context-mcp` as a stdio MCP server |
| Claude Code | Plugin-bundled `skills/` directories | Plugin-bundled `.mcp.json` | Optional plugin hooks | Install the `plugin/` directory as a Claude plugin |
| Other MCP clients | N/A | Standard stdio MCP config | Client-specific | Point the client at `fast-context-mcp` |

## Install the Runtime

Before configuring any host, make sure the MCP binary exists locally.

Release usage:

- install a released `fast-context-mcp` binary on `PATH`

Local development:

```bash
cargo build --release --bin fast-context --features cli
cargo build --release --bin fast-context-mcp --features mcp
```

If you do not want to install the binary globally yet, most host setups can use a development fallback such as:

```bash
cargo run --quiet --bin fast-context-mcp --features mcp
```

## Codex

### Skills

Codex skills are shipped in:

- [.agents/skills](../.agents/skills)

These are Codex-native skill directories and should be used through Codex's normal skills model.

Codex discovers skills from repository and user locations, including `.agents/skills` in the current repository tree and `$HOME/.agents/skills`. Source: [OpenAI Codex skills docs](https://developers.openai.com/codex/skills).

#### Install / enable Codex skills

Repo-scoped usage is already set up in this repository:

- keep the checked-in [`.agents/skills`](../.agents/skills) directory in the repo
- launch Codex from this repository or a subdirectory inside it

Optional user-scoped installation:

- copy or symlink the skill directories into `$HOME/.agents/skills`
- or use Codex's native skill installer flow for additional skills

The checked-in skills here are intended to work without a separate Fast-Context-specific installer.

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

Codex stores MCP configuration in `~/.codex/config.toml` by default, with optional project-scoped `.codex/config.toml`, and supports both CLI-based setup and direct config editing. Source: [OpenAI Codex MCP docs](https://developers.openai.com/codex/mcp).

#### Install / enable Codex MCP

Option 1: use Codex CLI

```bash
codex mcp add fast-context -- fast-context-mcp
```

Option 2: edit `config.toml` directly

```toml
[mcp_servers.fast-context]
command = "fast-context-mcp"
```

Development fallback:

```toml
[mcp_servers.fast-context]
command = "cargo"
args = ["run", "--quiet", "--bin", "fast-context-mcp", "--features", "mcp"]
```

### Hooks

Codex hooks are optional and should be treated as phase 2. They are not required for the baseline Fast-Context integration.

#### Install / enable Codex hooks

- keep hooks out of the baseline setup
- only add them if you have Codex hook support enabled in your environment
- prefer hooks for diagnostics or lightweight automation, not required functionality

At the moment this repository does not ship a dedicated Codex hooks package. Skills and MCP are the primary supported Codex integration surfaces.

## Claude Code

Claude integration is packaged as a native Claude plugin skeleton in:

- [plugin](../plugin)

The plugin includes:

- [plugin/.claude-plugin/plugin.json](../plugin/.claude-plugin/plugin.json)
- [plugin/.mcp.json](../plugin/.mcp.json)
- [plugin/skills](../plugin/skills)

Claude plugins automatically discover bundled skills in `skills/`, MCP server definitions in `.mcp.json`, and optional hooks in `hooks/hooks.json` or inline manifest config. Source: [Claude plugin reference](https://code.claude.com/docs/en/plugins-reference).

### Install / enable the Claude plugin

- use Claude Code's native plugin install flow
- point Claude at the repository [plugin](../plugin) directory, or a packaged copy of that directory
- once installed, Claude will discover the bundled skills automatically

This repository does not use a custom Claude plugin installer. The plugin directory itself is the install artifact.

### Skills

Claude plugin skills live in:

- [plugin/skills](../plugin/skills)

They are discovered automatically when the plugin is installed.

### MCP

The plugin `.mcp.json` assumes:

```bash
fast-context-mcp
```

is available on `PATH`.

#### Install / enable Claude MCP

- install the plugin
- ensure `fast-context-mcp` is available on `PATH`
- Claude will use the plugin's [`.mcp.json`](../plugin/.mcp.json) when the plugin is enabled

For development, you can change the plugin MCP config to a `cargo run --bin fast-context-mcp --features mcp` command until you have a released binary on `PATH`.

### Hooks

Claude hooks are optional and not part of the phase-1 plugin skeleton.

#### Install / enable Claude hooks

- add a `hooks/hooks.json` file under the plugin root, or declare hooks in `plugin.json`
- keep hooks additive and optional
- do not make hooks necessary for core Fast-Context behavior

The current plugin skeleton does not ship active hooks by default.

## Standalone MCP Install

If you are not using Codex or Claude plugin packaging, you can still run Fast-Context as a standard stdio MCP server.

### Generic stdio command

Preferred:

```bash
fast-context-mcp
```

Development fallback:

```bash
cargo run --quiet --bin fast-context-mcp --features mcp
```

### Generic MCP config shape

Most MCP clients can be pointed at the binary with a stdio config equivalent to:

```json
{
  "mcpServers": {
    "fast-context": {
      "command": "fast-context-mcp"
    }
  }
}
```

If your client supports command arguments or environment overrides, use them in the usual client-specific format.

### Diagnostics

Useful checks before wiring the server into a client:

```bash
fast-context --format json mcp doctor
fast-context --format json mcp init --stdout
```

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
