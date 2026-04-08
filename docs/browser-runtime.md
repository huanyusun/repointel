# Browser Runtime

The browser runtime is phase 1.5 rather than the first thin slice.

## Constraints

- analysis must run client-side
- uploaded repositories must stay local to the browser session
- cached artifacts should live in browser storage when helpful
- the UI must expose graph exploration, evidence panels, and repository Q&A

## Current status

- `ci-parser-web` exists as the browser-specific bridge point
- `ci-wasm` exists as the WASM packaging boundary
- `apps/web` is scaffolded for the future UI shell

## Planned implementation

1. compile the core IR, graph, search, and agent layers to WASM where practical
2. use a thin frontend shell only for input handling, graph rendering, and chat UI
3. keep query semantics aligned with the native CLI so human and agent workflows stay consistent
