# flow-trace

Use this skill when you need to understand how control flows through the repository.

## Workflow

1. Start with `repointel explain <query> --json`.
2. For the most relevant symbol, follow with:
   - `repointel callers <symbol> --json`
   - `repointel callees <symbol> --json`
   - `repointel trace <symbol> --json`
3. Summarize the likely path through the system.

## Rules

- Prefer targeted flow commands before opening files.
- Call out when the trace is only partial.
