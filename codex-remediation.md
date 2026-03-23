# Fast-Context Remediation Tasklist

## Objective

Get the project from "locally working" to "release-safe and enterprise-ready" by closing the remaining gaps in release automation, CI truthfulness, binding validation, and analysis trust signals.

## Current State

Confirmed locally:

- `cargo check`: passes
- `cargo check --features python`: passes with Python 3.11
- `npm run build:debug`: passes
- `npm run build`: passes
- `npm test`: passes
- `npm pack` + clean install smoke test: passes
- `cargo clippy --all-targets --all-features -- -D warnings`: passes

Known remaining issues:

- `cargo test --features nodejs --test nodejs_api_tests` still has older failing cases
- release automation is split across multiple workflows and one path is stale
- package metadata still references the old `rustworkx-nodejs` identity
- Python is buildable but not fully qualified by the main CI/test pipeline
- analysis still silently drops some unreadable/unparseable files
- local worktree still contains unrelated uncommitted changes outside the finished remediation commits

## Release Blockers

### A. Unify and fix release automation

Status: `BLOCKED BY STALE WORKFLOWS`

Problems:

- [ ] Choose one authoritative release pipeline between:
  - [/.github/workflows/ci.yml](C:/Users/shawn/workspace/fast-context/.github/workflows/ci.yml)
  - [/.github/workflows/release.yml](C:/Users/shawn/workspace/fast-context/.github/workflows/release.yml)
- [ ] Remove or rewrite the stale Yarn-based release flow in [/.github/workflows/release.yml](C:/Users/shawn/workspace/fast-context/.github/workflows/release.yml)
- [ ] Make release scripts match the current npm-based packaging/build flow
- [ ] Ensure release artifact names match the actual package/module names
- [ ] Verify the selected workflow publishes the right npm package and downloads the right native artifacts

Acceptance criteria:

- [ ] One release workflow exists for npm publication
- [ ] It uses the current package name and current build commands
- [ ] A dry-run release path can be executed without manual patching

### B. Fix stale package/project metadata

Status: `BLOCKED BY IDENTITY DRIFT`

Problems:

- [ ] Replace stale `rustworkx-nodejs` references in:
  - [/Cargo.toml](C:/Users/shawn/workspace/fast-context/Cargo.toml)
  - [/pyproject.toml](C:/Users/shawn/workspace/fast-context/pyproject.toml)
  - [/.github/workflows/release.yml](C:/Users/shawn/workspace/fast-context/.github/workflows/release.yml)
- [ ] Check README/docs/package metadata for stale install/package names
- [ ] Ensure homepage/repository/issues/changelog URLs all point at the current project identity

Acceptance criteria:

- [ ] No shipped metadata points at the old project name
- [ ] npm/PyPI/repo URLs are internally consistent

### C. Restore honest Node-feature validation

Status: `BLOCKED BY FAILING TEST TARGET`

Problems:

- [ ] Fix the remaining failures in [tests/rust/nodejs_api_tests.rs](C:/Users/shawn/workspace/fast-context/tests/rust/nodejs_api_tests.rs):
  - `test_nodejs_find_symbols_in_file`
  - `test_nodejs_get_file_dependencies`
- [ ] Decide whether these tests are asserting the wrong contract or exposing real runtime bugs
- [ ] Keep the new `maxFiles` / `parallelProcessing` coverage green
- [ ] Document the environment constraint around local Node-API loader noise on Windows

Acceptance criteria:

- [ ] `cargo test --features nodejs --test nodejs_api_tests` is green or intentionally narrowed with documented rationale
- [ ] Node-facing Rust tests match the shipped contract

### D. Add real Python qualification to CI

Status: `BLOCKED BY PIPELINE COVERAGE`

Problems:

- [ ] Add `cargo check --features python` to active CI
- [ ] Add Python install/import validation against the built extension
- [ ] Add `pytest` coverage for the supported Python API surface
- [ ] Decide which Python test files are authoritative and which are legacy/noise
- [ ] Make CI use a supported interpreter range and document it clearly

Acceptance criteria:

- [ ] Main CI proves Python bindings compile
- [ ] Main CI proves Python bindings import
- [ ] Main CI runs a meaningful Python test suite

## Correctness and Trust Gaps

### E. Stop silently dropping failed files

Status: `CONFIRMED TRUST GAP`

Problems in [/src/core/mod.rs](C:/Users/shawn/workspace/fast-context/src/core/mod.rs):

- [ ] Stop turning file read/stream/parse failures into silent `None`
- [ ] Surface skipped files in a structured result or diagnostics channel
- [ ] Distinguish:
  - unreadable files
  - blocked files
  - parse failures
  - unsupported files
- [ ] Add tests that verify these failures are visible to callers

Acceptance criteria:

- [ ] Analysis results expose skipped-file diagnostics explicitly
- [ ] Users can tell the difference between "0 symbols" and "file was skipped"

### F. Review remaining dirty core changes before release

Status: `BLOCKED BY LOCAL WORKTREE NOISE`

Problems:

- [ ] Review the remaining uncommitted changes under `/src` and `/tests`
- [ ] Separate intentional follow-up work from accidental/local churn
- [ ] Commit, shelve, or revert them before a release candidate

Acceptance criteria:

- [ ] Release branch/worktree is clean
- [ ] Every remaining code change has an owner and a reason

## Hardening and Polish

### G. Enterprise CI gate

Status: `NOT YET COMPLETE`

Tasks:

- [ ] Define one required release gate covering:
  - `cargo check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo check --features python`
  - `npm run build`
  - `npm test`
  - Node tarball smoke install
  - Python wheel/sdist smoke install
- [ ] Make the gate enforceable in CI, not just locally
- [ ] Ensure failures stop publication automatically

Acceptance criteria:

- [ ] Release cannot proceed unless the full gate is green

### H. Cross-platform packaging confidence

Status: `PARTIALLY VALIDATED`

Tasks:

- [ ] Validate npm native artifact flow for Linux/macOS/Windows
- [ ] Validate Python wheel output for supported Python versions/platforms
- [ ] Confirm package contents are minimal and correct on each publish target
- [ ] Do a dry-run for npm and PyPI publication

Acceptance criteria:

- [ ] Cross-platform artifacts build and install cleanly
- [ ] No platform depends on ad hoc local fixes

### I. Docs and support policy cleanup

Status: `NEEDS FINAL PASS`

Tasks:

- [ ] Define the supported Node versions
- [ ] Define the supported Python versions
- [ ] Define the supported OS/architecture matrix
- [ ] Align README/docs/examples with the actual shipped APIs and release flows

Acceptance criteria:

- [ ] A user can follow the docs without hitting stale names or dead paths

## Recommended Execution Order

1. Fix release workflow drift
2. Fix stale metadata/project identity
3. Make Node-feature Rust tests honest and green
4. Add real Python validation to active CI
5. Expose skipped-file diagnostics instead of silently dropping failures
6. Clean the remaining local worktree noise
7. Run full release dry-runs for npm and PyPI

## Publish Gate

Do not call this enterprise-ready until all of the following are true:

- [ ] Release workflows are current and consistent
- [ ] Metadata uses the current project identity everywhere
- [ ] Node-feature Rust tests are green or intentionally scoped with rationale
- [ ] Python compile/import/tests are enforced in CI
- [ ] Analysis reports skipped files explicitly
- [ ] Cross-platform package dry-runs succeed
- [ ] Release worktree is clean
