# Fast-Context Symbols

Use this skill when you need symbol lookup or dependency lookup through the Fast-Context MCP server.

## When To Use

- list functions, classes, or other symbol kinds
- inspect symbols in a specific file
- resolve dependencies for a target symbol
- verify whether the analyzer can see a file or symbol before deeper investigation

## MCP Tools

List symbols by kind:

- `find_symbols_by_kind`

List symbols in a specific file:

- `find_symbols_in_file`

Find dependencies for a symbol:

- `find_dependencies`

## Output Notes

### `find_symbols_by_kind`

Returns:

- `projectPath`
- `symbolKind`
- `count`
- `symbols`

### `find_symbols_in_file`

Returns:

- `projectPath`
- `filePath`
- `count`
- `symbols`

### `find_dependencies`

Returns:

- `projectPath`
- `symbolName`
- `count`
- `dependencies`

## Interpretation

- empty `symbols` often means the kind name is wrong or the file path is wrong relative to `projectPath`
- empty `dependencies` is valid; it can mean the symbol is terminal or not resolved by the current extraction logic
- if results look too small, start with `analyze_codebase` and inspect `skippedFileCount`
