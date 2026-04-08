# repo-intel-query

Use this skill whenever you need repository context before editing.

## Workflow

1. Run `repointel status --json` from the repo root.
2. If no index exists, run `repointel index . --json`.
3. Use the narrowest query that answers the question:
   - `repointel symbol <name> --json`
   - `repointel search <query> --json`
   - `repointel explain <query> --json`
4. Summarize the returned evidence before reading files.

## Rules

- Prefer CLI evidence over broad file inspection.
- If there are multiple symbol matches, say so explicitly.
- Keep the summary compact and evidence-backed.
