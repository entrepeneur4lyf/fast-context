# CLI Specification

## Purpose

`fast-context` will expose a Rust CLI focused on useful one-shot developer commands.

Design goals:

- fast
- scriptable
- deterministic
- quiet by default
- machine-readable when requested
- no REPL
- no placeholder commands

The CLI should feel closer to `cargo` than to an interactive assistant shell.

## Non-Goals

The CLI will not include:

- REPL mode
- watch mode in the first implementation
- placeholder or stub commands
- broad interactive wizards
- a giant kitchen-sink command surface

## Binary

Primary binary:

- `fast-context`

Optional short alias can be considered later, but is not part of the initial contract.

## Global Flags

These flags should be available wherever they make sense:

- `--format <text|json|yaml>`
- `--quiet`
- `--project-root <path>`
- `--language <lang>` (repeatable)
- `--ignore <pattern>` (repeatable)
- `--max-files <n>`

Default format:

- `text`

Behavior:

- `text` is optimized for humans
- `json` is stable and scriptable
- `yaml` mirrors the JSON structure

## Command Tree

Initial command surface:

```text
fast-context
  analyze <path>
  symbols by-kind <kind> <path>
  symbols in-file <file> <path>
  deps <symbol> <path>
  stats languages <path>
  stats loc <path>
  doctor
  mcp init
  mcp doctor
```

## Command Details

### `analyze <path>`

Runs project analysis using the real Rust analyzer.

Text output should include:

- file count
- symbol count
- relationship count
- languages
- duration
- skipped file count

Structured output should include:

- `project_path`
- `file_count`
- `symbol_count`
- `relationship_count`
- `languages`
- `duration_ms`
- `skipped_file_count`
- `skipped_files`

### `symbols by-kind <kind> <path>`

Returns symbol names for a given symbol kind.

Structured output should include:

- `project_path`
- `symbol_kind`
- `count`
- `symbols`

### `symbols in-file <file> <path>`

Returns symbol names for a specific file relative to the project root.

Structured output should include:

- `project_path`
- `file_path`
- `count`
- `symbols`

### `deps <symbol> <path>`

Returns dependency symbol names for the target symbol.

Structured output should include:

- `project_path`
- `symbol_name`
- `count`
- `dependencies`

### `stats languages <path>`

Returns language distribution for the project.

Primary metrics:

- file count by language
- line count by language
- percentage by language

Structured output should include:

- `project_path`
- `total_files`
- `total_lines`
- `languages`: array of
  - `language`
  - `file_count`
  - `line_count`
  - `percentage`

### `stats loc <path>`

Returns line counts for the project.

Primary metrics:

- total lines
- code lines
- comment lines
- blank lines
- optional per-language rollups

Structured output should include:

- `project_path`
- `total_lines`
- `code_lines`
- `comment_lines`
- `blank_lines`
- `languages` (optional array)

### `doctor`

Checks local runtime/build prerequisites for the current install.

Initial checks:

- project path validity
- native module availability
- Rust binary presence when applicable
- Node version when relevant
- Python version when relevant

Structured output should include:

- `status`
- `checks`
- `errors`
- `warnings`

### `mcp init`

Writes or prints MCP configuration for using the Rust MCP server.

Requirements:

- explicit target path or explicit default location
- no silent config mutation
- report exactly what file was written

Structured output should include:

- `config_path`
- `server_command`
- `arguments`
- `written`

### `mcp doctor`

Checks whether the MCP server can be launched correctly.

Initial checks:

- Rust MCP binary exists or can be built
- command path is valid
- stdio launch is possible

Structured output should include:

- `status`
- `binary_path`
- `checks`
- `errors`
- `warnings`

## Output Rules

Text mode:

- concise
- aligned columns where useful
- no decorative noise

JSON/YAML mode:

- predictable keys
- snake_case keys
- stable top-level structure per command

Exit codes:

- `0` success
- non-zero on validation, runtime, or configuration failures

## Implementation Phases

### Phase 1

- `analyze`
- `symbols by-kind`
- `symbols in-file`
- `deps`
- `doctor`
- output formatting (`text`, `json`, `yaml`)

### Phase 2

- `stats languages`
- `stats loc`

### Phase 3

- `mcp init`
- `mcp doctor`

## Implementation Notes

- Use `clap`
- Reuse `CoreAnalyzer` directly
- Reuse existing validation utilities
- Avoid introducing a second command architecture that duplicates analyzer logic
- Prefer explicit structs for structured output
- Keep command handlers small and testable
