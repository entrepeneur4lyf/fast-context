# Fast-Context Host Integration Spec

## Purpose

This document defines how Fast-Context integrates with host tools that can use:

- skills
- MCP servers
- hooks
- plugin packaging

The immediate hosts in scope are:

- Codex
- Claude Code

This spec exists to prevent the project from conflating different host models.

## Design Rules

1. Use the host's native integration model.
2. Do not invent a fake universal plugin format.
3. Keep the Rust implementation as the source of truth.
4. Treat skills as content, not runtime code extensions.
5. Treat hooks as optional advanced integrations, not required baseline behavior.

## Source Of Truth

Fast-Context provides these native artifacts:

- Rust core library
- Rust CLI binary: `fast-context`
- Rust MCP binary: `fast-context-mcp`
- Node package
- Python package

Host integrations should package or reference those artifacts. They should not duplicate analyzer logic in separate wrappers.

## Host Matrix

### Codex

Codex has native support for:

- skills
- MCP
- optional beta hooks

Codex should be supported through Codex-native configuration rather than a separate plugin abstraction.

#### Skills

Codex skills should be shipped as skill directories using the standard `SKILL.md` model.

Planned repo location:

- `.agents/skills/`

Skill content may include:

- `SKILL.md`
- optional `scripts/`
- optional `references/`
- optional supporting files

#### MCP

Codex MCP should use the Rust MCP binary:

- `fast-context-mcp`

Preferred integration model:

- user configures Codex with its native MCP configuration flow
- documentation provides the exact `codex mcp add ...` command or equivalent config snippet

Fast-Context may provide helper output via the Rust CLI, but Codex remains the owner of MCP registration.

#### Hooks

Codex hooks are optional.

They must be treated as:

- beta/advanced integration
- disabled by default
- host-specific

They must not be required for baseline Fast-Context usage.

### Claude Code

Claude Code uses a plugin directory model.

Claude integration should be packaged as a real Claude plugin directory rather than reusing the Codex layout directly.

Planned repo location:

- `plugin/`

#### Skills

Claude plugin skills should live under:

- `plugin/skills/`

They should mirror the actual maintained Fast-Context skills, not drift into a second independent set of instructions.

#### MCP

Claude plugin MCP should be defined in:

- `plugin/.mcp.json`

The plugin should reference:

- `fast-context-mcp` if available on `PATH`
- or another explicitly supported command path

#### Hooks

Claude hooks are optional.

They should only be added for concrete operational value.

They must not:

- silently mutate user code
- perform surprise network actions
- install dependencies without explicit user action

## What Skills Are

Skills are instruction bundles that help the host understand:

- when Fast-Context is useful
- how to invoke the relevant tools
- how to interpret the resulting output

Skills are not:

- dynamic Rust plugins
- runtime extensions to the analyzer
- arbitrary code execution packages

## What Hooks Are

Hooks are host lifecycle integrations.

Hooks may be useful for:

- validating environment/setup
- surfacing MCP availability issues
- lightweight host-specific diagnostics

Hooks are not part of the minimum viable integration.

## Minimum Supported Integration Surface

Phase 1 support should include:

### Codex

- Codex skills
- Codex MCP setup guidance
- Rust MCP binary

### Claude Code

- plugin directory
- plugin skills
- plugin `.mcp.json`

Hooks are phase 2 for both hosts.

## Files To Create

### Codex

- `.agents/skills/<skill-name>/SKILL.md`
- optional supporting files in the same skill directories
- docs for Codex MCP setup

### Claude Code

- `plugin/.claude-plugin/plugin.json`
- `plugin/.mcp.json`
- `plugin/skills/<skill-name>/SKILL.md`
- optional `plugin/hooks/hooks.json` later

## CLI Responsibilities

The Rust CLI should support host integrations, but not replace native host installation models.

Good CLI responsibilities:

- `mcp doctor`
- `mcp init`
- environment diagnostics
- printing config snippets
- scaffolding host integration files later

Non-goals for the CLI:

- a fake cross-host plugin installer
- dynamic plugin loading
- replacing native Codex or Claude install flows

## Distribution Implications

Host integrations should reference released binaries and maintained skill files.

That means:

- skills should be versioned with the repo
- Claude plugin packaging should be versioned with the repo
- MCP references should target the released `fast-context-mcp` binary or an explicitly supported fallback

## Open Decisions

The following still need explicit decisions:

1. which skills are shared between Codex and Claude
2. whether Claude plugin skills are copied or generated from a shared source
3. whether any hooks are worth shipping in phase 2
4. whether the Rust CLI should generate host-specific config snippets for Codex and Claude separately
