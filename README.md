# repointel

`repointel` is a Rust-first repository intelligence system for humans and AI coding agents.

Phase 1 focuses on a strong local CLI and shared analysis core rather than MCP. The core design principles are:

- Rust owns indexing, IR, graph construction, search, and agent-oriented intelligence.
- The CLI emits deterministic JSON so it can be wrapped later without changing the core logic.
- AI agents should query targeted architectural context instead of reading large file sets blindly.
- The browser runtime will stay privacy-first by running analysis client-side via Rust/WASM.

## Current thin slice

This repository already ships a working vertical slice:

- local repository ingestion
- language-aware parsing for Rust, TypeScript/JavaScript, and Python
- shared IR
- graph construction
- persisted local index under `.repointel/index.json`
- `repointel index`
- `repointel status`
- `repointel search`
- `repointel symbol`
- `repointel impact`
- `repointel callers`
- `repointel callees`
- `repointel graph export --format json`
- `repointel explain`
- repo-local Codex skills that instruct agents to use the CLI first

## Workspace layout

```text
crates/
  ci-ir/
  ci-loader/
  ci-parser-native/
  ci-parser-web/
  ci-resolver/
  ci-graph/
  ci-search/
  ci-agent/
  ci-cli/
  ci-wasm/
apps/
  web/
docs/
```

## Quick start

```bash
cargo build
cargo run -p ci-cli -- index ./tests/fixtures/sample_repo --json
cargo run -p ci-cli -- status --json
cargo run -p ci-cli -- symbol login --json
cargo run -p ci-cli -- impact login --json
```

## Documentation

- [Architecture](docs/architecture.md)
- [IR](docs/ir.md)
- [Indexing Pipeline](docs/indexing-pipeline.md)
- [Graph Model](docs/graph-model.md)
- [Search And Agent](docs/search-and-agent.md)
- [Browser Runtime](docs/browser-runtime.md)
- [CLI Contract](docs/cli-contract.md)
- [Codex Integration](docs/codex-integration.md)
- [Future MCP Adapter](docs/future-mcp-adapter.md)
