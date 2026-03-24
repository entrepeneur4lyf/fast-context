#!/usr/bin/env bash

set -euo pipefail

PYTHON_EXE=""
SKIP_VALIDATION=0
SKIP_BUILD=0
SKIP_PUBLISH=0
SKIP_CARGO_PUBLISH=0
SKIP_NPM_PUBLISH=0
SKIP_PYTHON_PUBLISH=0
TAG_AFTER_PUBLISH=0

write_step() {
  local message="$1"
  printf '\n==> %s\n' "$message"
}

run_step() {
  local command="$1"
  printf 'bash> %s\n' "$command"
  eval "$command"
}

usage() {
  cat <<'EOF'
Usage: ./scripts/release.sh [options]

Options:
  --python-exe PATH         Python interpreter to use
  --skip-validation         Skip validation steps
  --skip-build              Skip build steps
  --skip-publish            Skip all publish steps
  --skip-cargo-publish      Skip cargo publish
  --skip-npm-publish        Skip npm publish
  --skip-python-publish     Skip PyPI publish
  --tag-after-publish       Tag and push after publish succeeds
  -h, --help                Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --python-exe)
      PYTHON_EXE="${2:-}"
      shift 2
      ;;
    --skip-validation)
      SKIP_VALIDATION=1
      shift
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
      ;;
    --skip-publish)
      SKIP_PUBLISH=1
      shift
      ;;
    --skip-cargo-publish)
      SKIP_CARGO_PUBLISH=1
      shift
      ;;
    --skip-npm-publish)
      SKIP_NPM_PUBLISH=1
      shift
      ;;
    --skip-python-publish)
      SKIP_PYTHON_PUBLISH=1
      shift
      ;;
    --tag-after-publish)
      TAG_AFTER_PUBLISH=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

get_python_command() {
  if [[ -n "$PYTHON_EXE" ]]; then
    printf '%s\n' "$PYTHON_EXE"
    return
  fi

  if command -v python3 >/dev/null 2>&1; then
    printf 'python3\n'
    return
  fi

  if command -v python >/dev/null 2>&1; then
    printf 'python\n'
    return
  fi

  printf 'No Python interpreter found on PATH\n' >&2
  exit 1
}

get_cargo_version() {
  local version
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
  if [[ -z "$version" ]]; then
    printf 'Unable to read version from Cargo.toml\n' >&2
    exit 1
  fi
  printf '%s\n' "$version"
}

get_npm_version() {
  node -p "require('./package.json').version"
}

get_python_version() {
  local version
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' pyproject.toml | head -n 1)"
  if [[ -n "$version" ]]; then
    printf '%s\n' "$version"
    return
  fi
  get_cargo_version
}

assert_version_alignment() {
  local cargo_version npm_version python_version
  cargo_version="$(get_cargo_version)"
  npm_version="$(get_npm_version)"
  python_version="$(get_python_version)"

  if [[ "$cargo_version" != "$npm_version" || "$cargo_version" != "$python_version" ]]; then
    printf 'Version mismatch detected. Cargo=%s npm=%s python=%s\n' \
      "$cargo_version" "$npm_version" "$python_version" >&2
    exit 1
  fi

  printf '%s\n' "$cargo_version"
}

assert_release_env() {
  local need_cargo="$1"
  local need_npm="$2"
  local need_python="$3"

  if [[ "$need_cargo" -eq 1 && -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    printf 'CARGO_REGISTRY_TOKEN is not set\n' >&2
    exit 1
  fi

  if [[ "$need_npm" -eq 1 && -z "${NPM_TOKEN:-}" ]]; then
    printf 'NPM_TOKEN is not set\n' >&2
    exit 1
  fi

  if [[ "$need_python" -eq 1 && -z "${PYPI_API_TOKEN:-}" ]]; then
    printf 'PYPI_API_TOKEN is not set\n' >&2
    exit 1
  fi
}

PYTHON_CMD="$(get_python_command)"
VERSION="$(assert_version_alignment)"

SHOULD_PUBLISH_CARGO=0
SHOULD_PUBLISH_NPM=0
SHOULD_PUBLISH_PYTHON=0

if [[ "$SKIP_PUBLISH" -eq 0 && "$SKIP_CARGO_PUBLISH" -eq 0 ]]; then
  SHOULD_PUBLISH_CARGO=1
fi

if [[ "$SKIP_PUBLISH" -eq 0 && "$SKIP_NPM_PUBLISH" -eq 0 ]]; then
  SHOULD_PUBLISH_NPM=1
fi

if [[ "$SKIP_PUBLISH" -eq 0 && "$SKIP_PYTHON_PUBLISH" -eq 0 ]]; then
  SHOULD_PUBLISH_PYTHON=1
fi

write_step "Preparing local coordinated release for version $VERSION"
printf 'Python interpreter: %s\n' "$PYTHON_CMD"

if [[ "$SKIP_VALIDATION" -eq 0 ]]; then
  write_step "Running validation"
  run_step "cargo check"
  run_step "cargo test"
  run_step "cargo test --test cli_tests --features cli"
  run_step "cargo clippy --all-targets --all-features -- -D warnings"
  run_step "npm install"
  run_step "npm run build:debug"
  run_step "npm test"
  run_step "\"$PYTHON_CMD\" -m pytest tests/python"
  run_step "cargo audit"
  run_step "npm audit --audit-level moderate"
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  write_step "Building release artifacts"
  run_step "cargo build --release --bin fast-context --features cli"
  run_step "cargo build --release --bin fast-context-mcp --features mcp"
  run_step "npm run build"
  run_step "npm pack"
  run_step "\"$PYTHON_CMD\" -m maturin build --release --features python"
fi

if [[ "$SKIP_PUBLISH" -eq 0 ]]; then
  write_step "Checking publish credentials"
  assert_release_env "$SHOULD_PUBLISH_CARGO" "$SHOULD_PUBLISH_NPM" "$SHOULD_PUBLISH_PYTHON"

  write_step "Publishing release channels"

  if [[ "$SHOULD_PUBLISH_CARGO" -eq 1 ]]; then
    run_step "cargo publish --locked"
  fi

  if [[ "$SHOULD_PUBLISH_NPM" -eq 1 ]]; then
    run_step "npm publish"
  fi

  if [[ "$SHOULD_PUBLISH_PYTHON" -eq 1 ]]; then
    run_step "\"$PYTHON_CMD\" -m maturin publish --features python"
  fi

  if [[ "$TAG_AFTER_PUBLISH" -eq 1 ]]; then
    write_step "Tagging release"
    run_step "git tag v$VERSION"
    run_step "git push origin main --tags"
  fi
else
  write_step "Publish skipped"
fi

write_step "Release script completed"
