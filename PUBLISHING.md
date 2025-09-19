# Fast-Context NPM Publishing Guide

This guide covers how to prepare and publish the Fast-Context TypeScript SDK packages to npm.

## 📦 Package Overview

The Fast-Context project consists of three npm packages:

1. **`@fast-context/core`** - Enhanced TypeScript SDK with streaming analysis
2. **`@fast-context/mcp-server`** - Model Context Protocol server for AI integration
3. **`@fast-context/cli`** - Command-line interface with interactive REPL

## 🚀 Quick Publishing

### Prerequisites

1. **npm Account**: Ensure you have an npm account and are logged in
   ```bash
   npm login
   npm whoami  # Verify you're logged in
   ```

2. **Build Dependencies**: Install all dependencies
   ```bash
   npm install
   ```

### One-Command Publishing

```bash
# Dry run to test everything
npm run publish:dry-run

# Actual publishing
npm run publish:packages
```

## 📋 Step-by-Step Publishing

### 1. Pre-Publishing Checks

```bash
# Clean previous builds
npm run clean:packages

# Build all packages
npm run build:packages

# Run tests
npm run test:packages

# Verify package contents
npm pack --workspace=packages/core
npm pack --workspace=packages/mcp-server
npm pack --workspace=packages/cli
```

### 2. Version Management

Update versions in all package.json files:

```bash
# For patch releases (0.1.0 -> 0.1.1)
npm version patch --workspace=packages/core
npm version patch --workspace=packages/mcp-server
npm version patch --workspace=packages/cli

# For minor releases (0.1.0 -> 0.2.0)
npm version minor --workspace=packages/core
npm version minor --workspace=packages/mcp-server
npm version minor --workspace=packages/cli

# For major releases (0.1.0 -> 1.0.0)
npm version major --workspace=packages/core
npm version major --workspace=packages/mcp-server
npm version major --workspace=packages/cli
```

### 3. Individual Package Publishing

```bash
# Publish core package
npm publish --workspace=packages/core

# Publish MCP server
npm publish --workspace=packages/mcp-server

# Publish CLI
npm publish --workspace=packages/cli
```

## 🔍 Package Verification

After publishing, verify the packages:

```bash
# Check package info
npm info @fast-context/core
npm info @fast-context/mcp-server
npm info @fast-context/cli

# Test installation
npm install -g @fast-context/cli
fast-context --version

# Test MCP server
npx @fast-context/mcp-server --help
```

## 📁 Package Contents

### @fast-context/core
```
dist/
├── index.js
├── index.d.ts
├── analyzer/
├── config/
├── query/
├── streaming/
├── types/
└── utils/
README.md
package.json
```

### @fast-context/mcp-server
```
dist/
├── index.js
├── index.d.ts
├── server/
├── tools/
├── transports/
├── types/
├── utils/
└── examples/
    ├── stdio-server.js
    └── http-server.js
README.md
package.json
```

### @fast-context/cli
```
dist/
├── index.js
├── index.d.ts
├── bin/
│   └── cli.js
├── commands/
├── config/
├── repl/
└── utils/
README.md
package.json
```

## 🏷️ Release Tags

Create Git tags for releases:

```bash
# Tag the release
git tag -a v0.1.0 -m "Release v0.1.0: Initial TypeScript SDK release"
git push origin v0.1.0

# Or tag all packages individually
git tag -a @fast-context/core@0.1.0 -m "Core SDK v0.1.0"
git tag -a @fast-context/mcp-server@0.1.0 -m "MCP Server v0.1.0"
git tag -a @fast-context/cli@0.1.0 -m "CLI Tools v0.1.0"
git push origin --tags
```

## 🔧 Troubleshooting

### Common Issues

**1. Authentication Error**
```bash
npm login
# Follow prompts to authenticate
```

**2. Package Already Exists**
```bash
# Check current version
npm info @fast-context/core version

# Increment version
npm version patch --workspace=packages/core
```

**3. Build Failures**
```bash
# Clean and rebuild
npm run clean:packages
npm install
npm run build:packages
```

**4. Permission Denied**
```bash
# Verify you have publish rights
npm owner ls @fast-context/core
npm owner add <username> @fast-context/core
```

### Rollback

If you need to unpublish (within 24 hours):

```bash
# Unpublish specific version
npm unpublish @fast-context/core@0.1.0

# Deprecate instead of unpublish (recommended)
npm deprecate @fast-context/core@0.1.0 "This version has issues, please upgrade"
```

## 📊 Publishing Checklist

- [ ] All packages build successfully
- [ ] Tests pass for all packages
- [ ] Version numbers are updated
- [ ] README files are up to date
- [ ] License files are included
- [ ] .npmignore files exclude source code
- [ ] Binary executables work correctly
- [ ] Dependencies are properly specified
- [ ] npm login is successful
- [ ] Dry run completes without errors

## 🎯 Post-Publishing

1. **Update Documentation**: Update main README with installation instructions
2. **Create Release Notes**: Document changes and new features
3. **Announce Release**: Share on social media, Discord, etc.
4. **Monitor Issues**: Watch for bug reports and user feedback
5. **Update Examples**: Ensure all examples work with published packages

## 🔄 Automated Publishing (Future)

Consider setting up GitHub Actions for automated publishing:

```yaml
# .github/workflows/publish.yml
name: Publish Packages
on:
  release:
    types: [published]
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - run: npm ci
      - run: npm run build:packages
      - run: npm run test:packages
      - run: npm run publish:packages
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

This ensures consistent, automated releases with proper testing.
