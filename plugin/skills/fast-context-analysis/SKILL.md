# Fast-Context Analysis

Use this skill when you need a quick, factual summary of a codebase with Fast-Context.

## When To Use

- understand repository size and language mix
- get a high-level symbol/relationship count
- confirm whether analysis skipped supported files
- establish a baseline before deeper symbol or dependency queries

## MCP Tool

Use the `analyze_codebase` MCP tool provided by `fast-context-mcp`.

Important request fields:

- `project_path`
- `languages`
- `ignore_patterns`
- `max_files`
- `parallel_processing`

## What To Look For

Important response fields:

- `fileCount`
- `symbolCount`
- `relationshipCount`
- `languages`
- `durationMs`
- `skippedFileCount`
- `skippedFiles`

## Interpretation

- low `fileCount` compared with repo size usually means ignore rules or unsupported languages
- non-zero `skippedFileCount` means the analysis completed, but some supported files were not processed
- `languages` gives the analyzer's detected language mix, not GitHub Linguist percentages
