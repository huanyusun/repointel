# impact-analysis

Use this skill before any change that might cross module boundaries.

## Workflow

1. Run `repointel impact <target> --json`.
2. If the target is ambiguous, run `repointel symbol <name> --json` first.
3. Extract:
   - defining file
   - blast radius files
   - blast radius symbols
   - direct callers and callees when available
4. State the blast radius before editing.

## Rules

- Do not start editing until the blast radius is explicit.
- If the graph looks incomplete, say what is missing.
