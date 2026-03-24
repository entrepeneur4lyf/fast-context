# Fast-Context Claude Plugin

This directory is a Claude Code plugin skeleton for Fast-Context.

It currently includes:

- plugin metadata in [plugin.json](/C:/Users/shawn/workspace/fast-context/plugin/.claude-plugin/plugin.json)
- MCP server registration in [.mcp.json](/C:/Users/shawn/workspace/fast-context/plugin/.mcp.json)
- bundled skills in [skills](/C:/Users/shawn/workspace/fast-context/plugin/skills)

## Requirement

The plugin expects `fast-context-mcp` to be available on `PATH`.

For development, you can adjust `.mcp.json` to use a local `cargo run --bin fast-context-mcp --features mcp` fallback if needed.

## Install Concept

Install this directory as a Claude Code plugin using Claude's native plugin install flow.

This repo does not use a custom plugin installer for Claude.
