# Codex Integration

Phase 1 integration happens through repository-local instructions, not MCP.

## Files

- `AGENTS.md`
- `.agents/skills/repo-intel-query/SKILL.md`
- `.agents/skills/impact-analysis/SKILL.md`
- `.agents/skills/flow-trace/SKILL.md`
- `.agents/skills/safe-edit-plan/SKILL.md`

## Rules for agents

- use `repointel` first for architectural context
- request narrow symbol or impact data before reading broadly
- summarize findings before editing
- list blast radius for cross-module changes
- avoid making edits until hidden dependencies are checked through the CLI

The objective is to stop blind edits by making architectural awareness a first-class repo workflow.
