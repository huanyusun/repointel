# AGENTS

This repository is built for AI coding agents that need architectural awareness before editing.

## Required workflow

1. Start with `repointel status --json` if the repository is already indexed.
2. If the repo is not indexed or the index is stale, run `repointel index <path> --json`.
3. Before editing, query focused architecture with one or more of:
   - `repointel symbol <name> --json`
   - `repointel impact <target> --json`
   - `repointel callers <symbol> --json`
   - `repointel callees <symbol> --json`
   - `repointel explain <query> --json`
4. Summarize findings before making code changes.
5. Explicitly list blast radius for any cross-module change.
6. Avoid broad file reads unless the CLI output shows they are necessary.

## Editing policy

- Prefer narrow, local edits.
- Treat unresolved dependencies as a blocker and query the CLI again.
- If a change touches multiple modules, capture the impact surface first.

## Output expectations

When acting in this repository, agents should produce:

- a short architectural summary before edits
- the files likely affected
- the symbols or flows likely affected
- any uncertainty caused by partial resolution
