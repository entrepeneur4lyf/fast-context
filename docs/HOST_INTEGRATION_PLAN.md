# Fast-Context Host Integration Plan

## Goal

Ship truthful host integrations for:

- Codex
- Claude Code

without introducing wrapper drift or fake plugin abstractions.

## Principles

- use the host-native installation model
- keep the Rust MCP binary as the runtime integration point
- treat skills as maintained content
- keep hooks optional
- avoid building host-specific automation before the baseline integrations are real

## Phase 1

### 1. Codex Skills

Create Codex-native skills under:

- `.agents/skills/`

Initial skill set should be small and useful:

- codebase analysis
- symbol lookup
- dependency lookup
- MCP usage guidance where relevant

Deliverables:

- initial skill directory structure
- `SKILL.md` files
- any required helper references/scripts

Acceptance criteria:

- skills are discoverable by Codex in the documented location
- skill content reflects the actual Fast-Context CLI/MCP behavior

### 2. Codex MCP Setup Guidance

Document Codex-native MCP setup using the Rust MCP binary.

Deliverables:

- a short Codex MCP setup doc or README section
- exact example commands using Codex-native MCP configuration

Acceptance criteria:

- a user can register `fast-context-mcp` in Codex without guessing command syntax
- no custom plugin abstraction is required

### 3. Claude Plugin Skeleton

Create a Claude plugin directory with:

- `.claude-plugin/plugin.json`
- `.mcp.json`
- `skills/`

Deliverables:

- plugin root directory
- minimal plugin manifest
- MCP definition pointing at `fast-context-mcp`
- initial skills

Acceptance criteria:

- the plugin directory matches Claude's documented structure
- skills and MCP definitions are discoverable by Claude when installed

## Phase 2

### 4. Shared Skill Authoring Strategy

Decide whether Codex and Claude skills should:

- share a single source and be copied/generated
- or exist as separate maintained files

Recommendation:

- start with shared content where possible
- avoid premature generation machinery until drift becomes a real maintenance problem

Acceptance criteria:

- maintainers can update skills without confusion
- the repo does not accumulate two contradictory instruction surfaces

### 5. CLI Host Helpers

Extend the Rust CLI only where it adds real operator value.

Candidate additions:

- `mcp init codex`
- `mcp init claude`
- `doctor codex`
- `doctor claude`

Acceptance criteria:

- helpers emit accurate config snippets or diagnostics
- they do not replace native host install mechanisms

## Phase 3

### 6. Optional Hooks

Evaluate whether hooks are worth shipping.

Codex hooks:

- treat as beta and optional

Claude hooks:

- treat as plugin enhancements, not required baseline functionality

Only add hooks for concrete value, for example:

- validating MCP binary availability
- lightweight environment checks

Do not add hooks for:

- surprise mutation of user repos
- hidden installs
- opaque automation

Acceptance criteria:

- any shipped hook has a clear operational purpose
- baseline integration still works when hooks are disabled

## Out Of Scope

The following are explicitly out of scope for this plan:

- dynamic Rust plugin loading
- a universal cross-host plugin installer
- runtime code extensions through skills
- interactive setup wizards as a dependency for baseline usage

## Recommended Execution Order

1. Codex skills
2. Codex MCP setup docs
3. Claude plugin skeleton
4. shared skill strategy
5. optional CLI host helpers
6. optional hooks

## Immediate Next Task

Start with Codex skills and a minimal Claude plugin skeleton.

That gives the project:

- one native skill path for Codex
- one native plugin path for Claude
- both backed by the existing Rust MCP binary
