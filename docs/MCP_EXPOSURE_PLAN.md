# MCP Exposure Plan

## Goal

Keep the Node.js and Python library surfaces small and stable, and expose richer code-intelligence capabilities through the Rust MCP server.

This keeps the public language bindings truthful while still giving agent clients access to structured query results, dependency context, and architecture-oriented analysis.

## Principles

1. MCP is the primary rich agent surface.
2. Node.js and Python remain conservative library APIs.
3. Only expose capabilities that already have real implementations behind them.
4. Prefer structured results over string lists in MCP.
5. Do not expose placeholder, heuristic-only, or weakly tested capabilities.

## Current State

### Already exposed in MCP

- `analyze_codebase`
- `find_symbols_by_kind`
- `find_symbols_in_file`
- `find_dependencies`

These are intentionally narrow and map to the currently supported core analyzer behavior.

### Implemented internally but not yet exposed

The internal query layer already supports richer result shapes and additional queries:

- fuzzy symbol search by name/pattern
- dependents lookup
- rich symbol-by-kind results
- rich file-symbol results
- rich dependency results
- complex-symbol ranking
- architectural issue detection

These are backed by the query engine and structured result types rather than the thin string-array surfaces used by the Node.js analyzer today.

## Expose Now

These are the best next MCP additions because they are useful to agents and already map cleanly to real internal behavior.

### 1. `search_symbols`

Purpose:
- Find symbols by fuzzy name/pattern match across the codebase.

Input:
- `project_path`
- `pattern`

Output:
- structured symbols
- context summary
- suggestions

Why first:
- high agent value
- low ambiguity
- directly backed by the query engine

### 2. `find_dependents`

Purpose:
- Find which symbols depend on a target symbol.

Input:
- `project_path`
- `symbol_name`

Output:
- structured symbols
- relationships
- context
- suggestions

Why first:
- very useful for impact analysis and refactoring
- complements the already exposed `find_dependencies`

### 3. Rich `find_dependencies`

Purpose:
- Keep the existing tool name, but add an opt-in structured mode instead of returning only string lists.

Input:
- `project_path`
- `symbol_name`
- optional `detail_level = "basic" | "rich"`

Output:
- `basic`: current thin result
- `rich`: symbols + relationships + context + suggestions

Why:
- preserves compatibility while enabling a better agent-facing payload

### 4. Rich `find_symbols_in_file`

Purpose:
- Return symbol metadata and local relationships for one file.

Input:
- `project_path`
- `file_path`
- optional `detail_level = "basic" | "rich"`

Output:
- `basic`: current thin result
- `rich`: symbols + context + suggestions

### 5. `find_architectural_issues`

Purpose:
- Surface high-coupling symbols, circular dependencies, and other architecture signals.

Input:
- `project_path`

Output:
- affected symbols
- context
- suggestions

Why:
- valuable for agent review, refactoring, and codebase health scans

## Defer

These should wait until the earlier MCP expansion is stable.

### Complex-symbol ranking

Possible tool:
- `find_complex_symbols`

Reason to defer:
- lower priority than search/dependents/issues
- useful, but not as central to agent workflows

### Documentation metadata lookup

Possible tool:
- `get_symbol_documentation`

Reason to defer:
- current Python-side version was intentionally not exposed
- documentation quality depends heavily on source quality

### File or graph export tools

Possible tools:
- relationship graph export
- DOT/Mermaid/JSON graph exports

Reason to defer:
- useful, but not core for first-class MCP reasoning
- extra output-format surface increases maintenance

## Keep Internal Only For Now

- broad generic graph-algorithm APIs
- experimental architectural heuristics that are not yet well validated
- any query capability that fabricates metadata or relies on placeholder values

## Result Shape

For richer MCP tools, standardize on a structured response shape:

- `symbols`
- `relationships`
- `context`
- `suggestions`

Where possible:

- `symbols` should include name, kind, file path, line, column, language, and related metadata
- `relationships` should include source, target, type, and confidence when available
- `context` should summarize scale and notable patterns
- `suggestions` should remain short and actionable

## Implementation Order

1. Add a shared internal mapper from query-engine results to MCP response objects.
2. Add `search_symbols`.
3. Add `find_dependents`.
4. Add rich-mode support to `find_dependencies`.
5. Add rich-mode support to `find_symbols_in_file`.
6. Add `find_architectural_issues`.
7. Add tests for each new tool and one end-to-end MCP round-trip test per addition.

## Non-Goals

- Expanding the Node.js analyzer API at the same time
- Expanding the Python API at the same time
- Reintroducing TypeScript or Python MCP servers
- Turning MCP into a generic graph-algorithms service

## Success Criteria

This plan is complete when:

- agents can access richer structured code-intelligence results through MCP
- the Node.js and Python library APIs remain small and stable
- no exposed MCP tool returns placeholder or fabricated metadata
- each exposed tool has protocol-level test coverage
