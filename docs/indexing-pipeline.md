# Indexing Pipeline

## Flow

1. `ci-loader` walks the repository and applies ignore rules.
2. Supported source files are normalized into `RepositorySnapshot`.
3. `ci-parser-native` parses each file with the language-specific `tree-sitter` grammar.
4. Syntax extraction produces `RepoIr`.
5. `ci-resolver` computes import and call links plus precomputed symbol and impact summaries.
6. `ci-graph` packages nodes, edges, and intelligence into `GraphBundle`.
7. `ci-cli` persists the bundle to `.repointel/index.json`.

## Ignore behavior

The default ignore list excludes common dependency, build, and cache directories:

- `.git`
- `node_modules`
- `target`
- `dist`
- `build`
- `.next`
- `__pycache__`
- `.venv`
- `vendor`
- `coverage`
- `.turbo`

This will become configurable at the CLI layer. The thin slice keeps the policy in `LoadOptions`.

## Incremental indexing direction

The persisted bundle includes file digests. Later work can use those digests to avoid reparsing unchanged files and to separate indexing from query serving more aggressively.
