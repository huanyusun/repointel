# safe-edit-plan

Use this skill before implementing a non-trivial edit.

## Workflow

1. Gather architectural context with:
   - `repointel symbol <name> --json`
   - `repointel impact <name> --json`
   - `repointel explain <query> --json`
2. Summarize:
   - the target symbol or module
   - hidden dependencies
   - blast radius
   - likely verification points
3. Only then open the minimum set of files needed for the change.

## Rules

- Avoid broad repository reads unless the CLI indicates uncertainty.
- If impact spans multiple modules, list each affected area explicitly.
- Re-run impact analysis if the implementation plan changes.
