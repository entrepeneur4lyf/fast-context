# Fast-Context Remediation Tasklist

## Objective

Get the project to a publishable state by fixing confirmed release blockers first, then repairing correctness bugs, then restoring trust in the Node and Python public APIs with real validation.

## Current State

Confirmed as of this review:

- `cargo check`: passes
- `npm run build:debug`: passes
- `cargo check --features python`: fails
- `npm test`: fails
- `npm run test:integration`: fails

That means the Rust core is in better shape than the published surfaces around it.

## Release Blockers

### A. Repair Python feature build

Status: `BLOCKED BY CODE`

Problems:

- [ ] Fix Python submodule declarations in [src/lib.rs](/C:/Users/shawn/workspace/fast-context/src/lib.rs)
  - `python_bindings_util`
  - `python_bindings_graph`
  - `python_bindings_export`
  - `python_bindings_query`
  - `python_bindings_config`
  - `python_bindings_cache`
- [ ] Point those declarations at the actual files under [src/python_bindings](/C:/Users/shawn/workspace/fast-context/src/python_bindings)
- [ ] Reconcile the Python result model in [src/python_bindings.rs](/C:/Users/shawn/workspace/fast-context/src/python_bindings.rs)
  - either make `AnalysisResult` actually contain `symbols` and `dependencies`
  - or stop methods from treating it as if it does
- [ ] Remove or rewrite helper methods that depend on unavailable fields
- [ ] Fix remaining Python compile errors after the structural cleanup

Acceptance criteria:

- [ ] `cargo check --features python` passes
- [ ] Python bindings import successfully in the Python 3.11 environment
- [ ] The documented Python API matches the implemented one

### B. Restore Node testability

Status: `BLOCKED BY HARNESS`

Problems:

- [ ] Fix the bad relative imports in:
  - [tests/nodejs/integration.test.mjs](/C:/Users/shawn/workspace/fast-context/tests/nodejs/integration.test.mjs)
  - [tests/nodejs/unit.test.mjs](/C:/Users/shawn/workspace/fast-context/tests/nodejs/unit.test.mjs)
- [ ] Standardize on one JS API contract
  - config naming: `projectRoot` vs `project_root`
  - result naming: `fileCount` vs `file_count`
  - method naming: `startWatching` vs `start_watching`
- [ ] Align tests with the generated Node contract in [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts)
- [ ] Verify the built addon can actually be loaded from [index.js](/C:/Users/shawn/workspace/fast-context/index.js)

Acceptance criteria:

- [ ] `npm test` passes
- [ ] `require('./index.js')` works in the repo root after build
- [ ] Tests exercise the real public API instead of a stale contract

### C. Restore Rust integration coverage

Status: `BLOCKED BY SCRIPTING`

Problems:

- [ ] Fix `test:integration` in [package.json](/C:/Users/shawn/workspace/fast-context/package.json)
- [ ] Decide how Rust integration tests should be discovered:
  - move files from `tests/rust/` to `tests/`
  - or add explicit `[[test]]` entries to [Cargo.toml](/C:/Users/shawn/workspace/fast-context/Cargo.toml)
- [ ] Confirm the intended files actually run:
  - [tests/rust/api_integration_tests.rs](/C:/Users/shawn/workspace/fast-context/tests/rust/api_integration_tests.rs)
  - [tests/rust/integration_tests.rs](/C:/Users/shawn/workspace/fast-context/tests/rust/integration_tests.rs)
  - [tests/rust/nodejs_api_tests.rs](/C:/Users/shawn/workspace/fast-context/tests/rust/nodejs_api_tests.rs)
  - [tests/rust/symbol_extraction_tests.rs](/C:/Users/shawn/workspace/fast-context/tests/rust/symbol_extraction_tests.rs)

Acceptance criteria:

- [ ] `npm run test:integration` passes
- [ ] `cargo test --tests -- --list` shows the expected integration binaries

## High-Priority Correctness Work

### D. Make Node analyzer config real

Status: `CONFIRMED BUG`

Problems in [src/analyzer/mod.rs](/C:/Users/shawn/workspace/fast-context/src/analyzer/mod.rs):

- [ ] Persist user config on `FastContextAnalyzer`
- [ ] Pass stored `languages` and `ignore_patterns` into `CoreAnalyzer`
- [ ] Decide how `max_files` and `parallel_processing` should be enforced
- [ ] Update all analyzer entrypoints, not just `analyze()`

Acceptance criteria:

- [ ] Language filtering behaves correctly
- [ ] Ignore patterns behave correctly
- [ ] Repeated analysis uses the same config consistently

### E. Fix large-file streaming reconstruction

Status: `CONFIRMED BUG`

Problems in [src/validation/mod.rs](/C:/Users/shawn/workspace/fast-context/src/validation/mod.rs):

- [ ] Rewrite `StreamingTextReader::read_next_line()` so it preserves all newlines within a chunk
- [ ] Ensure chunk remainders are buffered without collapsing multiple lines
- [ ] Add coverage for:
  - multiple newlines in one chunk
  - newline across chunk boundaries
  - last line without trailing newline
  - file above streaming threshold

Acceptance criteria:

- [ ] Streaming and non-streaming reads produce identical text for fixtures
- [ ] Large-file symbol counts are stable across repeated runs

### F. Stop silently excluding ordinary files

Status: `CONFIRMED BUG`

Problems in [src/validation/mod.rs](/C:/Users/shawn/workspace/fast-context/src/validation/mod.rs) and [src/core/mod.rs](/C:/Users/shawn/workspace/fast-context/src/core/mod.rs):

- [ ] Narrow the sensitive-path filter to actual system/secret locations
- [ ] Remove generic substring blocking for names like `key`, `token`, `secret`
- [ ] Stop swallowing file-read failures during analysis
- [ ] Surface skipped-file diagnostics in a structured way
- [ ] Add fixtures such as:
  - `api_key.ts`
  - `tokenizer.py`
  - `secret-santa.js`

Acceptance criteria:

- [ ] Ordinary project files with those names are analyzed
- [ ] Truly blocked files are reported explicitly

## API Contract Reconciliation

### G. Choose and enforce one Node contract

Status: `DRIFT DETECTED`

Tasks:

- [ ] Pick the public casing convention for Node
  - likely camelCase, since [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts) is generated that way
- [ ] Make runtime, tests, examples, and docs agree
- [ ] Remove stale snake_case expectations from Node-facing material
- [ ] Clean up duplicated type declarations in [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts)

Acceptance criteria:

- [ ] One contract exists across runtime, TS definitions, tests, and docs

### H. Choose and enforce one Python contract

Status: `DRIFT DETECTED`

Tasks:

- [ ] Decide whether the supported Python API is sync, async, or both
- [ ] Make [python/README.md](/C:/Users/shawn/workspace/fast-context/python/README.md) match the shipped API
- [ ] Make [tests/python/test_python_bindings.py](/C:/Users/shawn/workspace/fast-context/tests/python/test_python_bindings.py) and related Python tests match the same contract
- [ ] Remove examples that depend on nonexistent result fields

Acceptance criteria:

- [ ] README snippets run unchanged
- [ ] Python tests validate only supported behavior

## Verification Matrix

Run these after each major phase, and all of them before publish:

- [ ] `cargo check`
- [ ] `npm run build:debug`
- [ ] `npm test`
- [ ] `npm run test:integration`
- [ ] `cargo check --features python`
- [ ] Python binding smoke test in `fast-context-py311`

Recommended Python env:

- `E:\models\bin\conda\envs\fast-context-py311\python.exe`

## Recommended Execution Order

1. Restore broken validation paths
   - fix Python module wiring
   - fix Node test imports
   - fix Rust integration test discovery
2. Reconcile the public API contracts
   - Node casing and method names
   - Python result model and method surface
3. Fix confirmed correctness bugs
   - Node config propagation
   - streaming line reconstruction
   - sensitive-file filtering
4. Re-run the full verification matrix
5. Only then consider packaging dry runs and publication

## Publish Gate

Do not publish until all of the following are true:

- [ ] Python feature build is clean
- [ ] Node tests pass
- [ ] Rust integration tests are actually running
- [ ] Config-sensitive behaviors are covered by tests
- [ ] Large-file streaming behavior is covered by tests
- [ ] Filtered/skipped-file behavior is covered by tests
- [ ] Node and Python docs match the shipped APIs
