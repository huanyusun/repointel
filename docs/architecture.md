# Architecture

## Goals

`repointel` is designed around one shared analysis core that serves three surfaces:

1. native indexing and query workflows
2. browser-side exploration through WASM
3. AI coding agent integration through stable CLI contracts

The core rule for phase 1 is that MCP is not part of the implementation boundary. The native CLI is the contract.

## Layering

### `ci-loader`

Owns repository ingestion and ignore behavior. It turns a repository into a deterministic set of source files with normalized paths and language detection.

### `ci-parser-native`

Turns source files into `RepoIr`. The thin slice uses `tree-sitter` grammars for Rust, TypeScript/JavaScript, and Python so extraction is syntax-aware rather than regex-only.

### `ci-ir`

Defines the stable intermediate representation shared across the workspace:

- repository
- file
- symbol
- import
- callsite
- spans
- language metadata

### `ci-resolver`

Builds cross-file intelligence on top of the IR:

- import links
- call links
- symbol context
- impact summaries

### `ci-graph`

Converts IR plus resolved relationships into a graph bundle with stable node and edge identifiers and precomputed query outputs for agents.

### `ci-search`

Provides compact search results over the graph bundle. The thin slice starts with symbol-centric lexical search.

### `ci-agent`

Builds higher-level, evidence-backed explanations over search and graph context.

### `ci-cli`

Provides the stable user and agent entry point. It persists indexes locally and returns deterministic JSON.

### `ci-wasm` and `ci-parser-web`

Reserved for the browser runtime. The workspace shape is fixed now so web-specific work can build on the same contracts without later crate churn.

## Thin-slice tradeoffs

- Repository ingestion currently supports local paths first. ZIP and GitHub archive ingestion are designed into the crate layout but not implemented in this slice.
- Resolution intentionally prefers deterministic, simple heuristics over speculative whole-program analysis.
- The browser app is scaffolded but not yet wired to the analysis core.

Those tradeoffs are deliberate. The first priority is a trustworthy local intelligence loop for humans and coding agents.
