# Fast-Context Documentation

This directory contains the maintained project documentation.

## Start Here

- [Repository README](/C:/Users/shawn/workspace/fast-context/README.md)
- [API reference](/C:/Users/shawn/workspace/fast-context/docs/API_REFERENCE.md)
- [Deployment guide](/C:/Users/shawn/workspace/fast-context/docs/DEPLOYMENT_GUIDE.md)

## What Is Here

### [API_REFERENCE.md](/C:/Users/shawn/workspace/fast-context/docs/API_REFERENCE.md)

Current Node.js API surface, utility exports, graph classes, and return types.

### [DEPLOYMENT_GUIDE.md](/C:/Users/shawn/workspace/fast-context/docs/DEPLOYMENT_GUIDE.md)

Operational guidance for packaging and deployment.

## Source of Truth

When documentation and generated files disagree, prefer:

1. the Rust source in [src](/C:/Users/shawn/workspace/fast-context/src)
2. the generated Node typings in [index.d.ts](/C:/Users/shawn/workspace/fast-context/index.d.ts)
3. the active GitHub Actions workflows in [/.github/workflows](/C:/Users/shawn/workspace/fast-context/.github/workflows)

## Maintenance Notes

- Archived material has been removed from the repository.
- Reference copies of external SDKs have been removed.
- The removed Go SDK is not part of the supported surface.

Keep this directory focused on the current project rather than plans, experiments, or imported third-party material.
