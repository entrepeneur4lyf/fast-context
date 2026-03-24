# Fast-Context Analysis

Use this skill when you need a quick, factual summary of a codebase with Fast-Context.

## When To Use

- understand repository size and language mix
- get a high-level symbol/relationship count
- confirm whether analysis skipped supported files
- establish a baseline before deeper symbol or dependency queries

## Commands

Use the Rust CLI when available:

```bash
fast-context --format json analyze .
```

If the binary is not on `PATH`, use Cargo:

```bash
cargo run --bin fast-context --features cli -- --format json analyze .
```

## What To Look For

Important fields in the result:

- `file_count`
- `symbol_count`
- `relationship_count`
- `languages`
- `duration_ms`
- `skipped_file_count`
- `skipped_files`

## Interpretation

- low `file_count` compared with repo size usually means ignore rules or unsupported languages
- non-zero `skipped_file_count` means the analysis completed, but some supported files were not processed
- `languages` gives the analyzer's detected language mix, not GitHub Linguist percentages

## Follow-Up

After `analyze`, the next useful commands are usually:

```bash
fast-context --format json stats languages .
fast-context --format json symbols by-kind function .
fast-context --format json deps <symbol> .
```
