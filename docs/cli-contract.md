# CLI Contract

The CLI is the stable phase 1 interface.

## Commands

- `repointel index <path> [--watch] [--json]`
- `repointel status [--json]`
- `repointel search <query> [--json]`
- `repointel symbol <name> [--json]`
- `repointel impact <target> [--json]`
- `repointel callers <symbol> [--json]`
- `repointel callees <symbol> [--json]`
- `repointel trace <process-or-route> [--json]`
- `repointel graph export [--format json]`
- `repointel explain <query> [--json]`

## Persistence

Indexes are stored under:

```text
<repo>/.repointel/index.json
```

## JSON mode

JSON mode is intended for agent consumption:

- deterministic key names
- stable object shapes
- no terminal formatting noise

## Human mode

Human mode stays concise and summary-oriented so the CLI is still useful in a shell.

## Thin-slice gaps

- `--watch` is reserved but not implemented yet
- `trace` currently provides a lightweight call trace around symbol matches rather than full process tracing
