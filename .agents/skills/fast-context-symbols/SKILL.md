# Fast-Context Symbols

Use this skill when you need symbol lookup or dependency lookup in a repository.

## When To Use

- list functions, classes, or other symbol kinds
- inspect symbols in a specific file
- resolve dependencies for a target symbol
- verify whether the analyzer can see a file or symbol before deeper investigation

## Commands

List symbols by kind:

```bash
fast-context --format json symbols by-kind function .
```

List symbols in a specific file:

```bash
fast-context --format json symbols in-file src/lib.rs .
```

Find dependencies for a symbol:

```bash
fast-context --format json deps FastContextAnalyzer .
```

Cargo fallback:

```bash
cargo run --bin fast-context --features cli -- --format json symbols by-kind function .
```

## Output Notes

### `symbols by-kind`

Returns:

- `project_path`
- `symbol_kind`
- `count`
- `symbols`

### `symbols in-file`

Returns:

- `project_path`
- `file_path`
- `count`
- `symbols`

### `deps`

Returns:

- `project_path`
- `symbol_name`
- `count`
- `dependencies`

## Interpretation

- empty `symbols` often means the kind name is wrong or the file path is not relative to the project root you passed
- empty `dependencies` is valid; it can mean the symbol is terminal or not resolved by the current extraction logic
- if results look too small, run `analyze` first and check `skipped_files`
