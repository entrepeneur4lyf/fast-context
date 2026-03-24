# Fast-Context Local Release

This project can be released locally instead of publishing from GitHub Actions.

That is the recommended path if you want direct control over publish order and credentials.

## Recommended Host

Use WSL or Linux for local release work when possible.

Reason:

- the Rust and Node native toolchains are easier to keep stable on Linux
- the Python wheel and native addon paths are closer to CI
- Windows-specific linker and environment issues are less likely to block the release

Windows PowerShell still works, but WSL/Linux should be treated as the preferred release host.

## Required Tokens

Set these in the shell session before publishing.

### PowerShell

```powershell
$env:CARGO_REGISTRY_TOKEN = "..."
$env:NPM_TOKEN = "..."
$env:PYPI_API_TOKEN = "..."
```

### Bash / WSL

```bash
export CARGO_REGISTRY_TOKEN="..."
export NPM_TOKEN="..."
export PYPI_API_TOKEN="..."
```

## Version Rule

Before release, keep the version aligned across:

- [Cargo.toml](../Cargo.toml)
- [package.json](../package.json)
- [pyproject.toml](../pyproject.toml)

The local release script will fail fast if these do not match.

## Release Script

Use either:

- [scripts/release.ps1](../scripts/release.ps1) for PowerShell
- [scripts/release.sh](../scripts/release.sh) for WSL/Linux

Default behavior:

- validates Rust, Node, and Python surfaces
- builds release artifacts
- publishes Cargo, npm, and PyPI in one session
- optionally tags the repo after publish

### Full Release

```powershell
.\scripts\release.ps1 -TagAfterPublish
```

```bash
./scripts/release.sh --tag-after-publish
```

### Validation Only

```powershell
.\scripts\release.ps1 -SkipPublish
```

```bash
./scripts/release.sh --skip-publish
```

### Build Only

```powershell
.\scripts\release.ps1 -SkipPublish -SkipValidation
```

```bash
./scripts/release.sh --skip-publish --skip-validation
```

### Use A Specific Python Interpreter

```powershell
.\scripts\release.ps1 -PythonExe "E:\models\bin\conda\envs\fast-context-py311\python.exe" -TagAfterPublish
```

```bash
./scripts/release.sh --python-exe /usr/bin/python3 --tag-after-publish
```

## What The Script Runs

Validation:

- `cargo check`
- `cargo test`
- `cargo test --test cli_tests --features cli`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `npm install`
- `npm run build:debug`
- `npm test`
- `python -m pytest tests/python`
- `cargo audit`
- `npm audit --audit-level moderate`

Build:

- `cargo build --release --bin fast-context --features cli`
- `cargo build --release --bin fast-context-mcp --features mcp`
- `npm run build`
- `npm pack`
- `python -m maturin build --release --features python`

Publish:

- `cargo publish --locked`
- `npm publish`
- `python -m maturin publish --features python`

Tagging:

- `git tag vX.Y.Z`
- `git push origin main --tags`

## Practical Release Order

The local scripts publish first and tag last.

That avoids creating a public tag for a release that only partially published.

## Notes

- [package.json](../package.json) may be rewritten by local native build steps; do not commit that churn unless intentional
- if you move daily development to WSL, keep Windows as an extra smoke-test environment rather than the primary release host
- GitHub Actions should still be kept green, but local release remains the source of truth for publication
