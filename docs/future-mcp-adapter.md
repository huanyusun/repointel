# Future MCP Adapter

MCP is explicitly out of scope for phase 1, but the CLI is shaped so it can be wrapped later.

## Adapter direction

An MCP layer can map tool calls directly onto deterministic CLI operations:

- `symbol` -> symbol context tool
- `impact` -> blast radius tool
- `callers` / `callees` -> graph neighborhood tools
- `explain` -> structured repo reasoning tool

## Why delay MCP

- the core indexing and graph model need to stabilize first
- agent workflows can be improved immediately through repo-local skills
- deterministic CLI contracts are easier to test and evolve than transport-specific APIs

When MCP is added, it should be a transport wrapper, not a second implementation path.
