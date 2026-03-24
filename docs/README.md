# Fast-Context Documentation

This directory contains the maintained project documentation.

## Start Here

- [Repository README](../README.md)
- [API reference](API_REFERENCE.md)
- [Deployment guide](DEPLOYMENT_GUIDE.md)
- [Host setup](HOST_SETUP.md)
- [Host integration spec](HOST_INTEGRATION_SPEC.md)
- [Host integration plan](HOST_INTEGRATION_PLAN.md)
- [Local release](LOCAL_RELEASE.md)

## What Is Here

### [API_REFERENCE.md](API_REFERENCE.md)

Current Node.js API surface, utility exports, graph classes, and return types.

### [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)

Operational guidance for packaging and deployment.

### [HOST_SETUP.md](HOST_SETUP.md)

Phase-1 setup instructions for Codex skills/MCP and the Claude plugin skeleton.

### [HOST_INTEGRATION_SPEC.md](HOST_INTEGRATION_SPEC.md)

Defines how Fast-Context integrates with Codex and Claude Code across skills, MCP, hooks, and plugin packaging.

### [HOST_INTEGRATION_PLAN.md](HOST_INTEGRATION_PLAN.md)

Phased implementation plan for Codex skills, Codex MCP setup, and Claude plugin packaging.

### [LOCAL_RELEASE.md](LOCAL_RELEASE.md)

Step-by-step local publish flow for Cargo, npm, and PyPI.

## Source of Truth

When documentation and generated files disagree, prefer:

1. the Rust source in [src](../src)
2. the generated Node typings in [index.d.ts](../index.d.ts)
3. the active GitHub Actions workflows in [.github/workflows](../.github/workflows)

## Maintenance Notes

Keep this directory focused on the current project rather than plans, experiments, or imported third-party material.
