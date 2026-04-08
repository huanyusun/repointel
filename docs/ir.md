# IR

`RepoIr` is the workspace contract between loading, parsing, resolution, graph construction, and later WASM integration.

## Entities

- `RepositorySnapshot`: raw loaded repository contents before syntax extraction
- `RepoIr`: parsed repository view
- `FileIr`: one source file with symbols, imports, and callsites
- `SymbolIr`: named architectural unit such as a function, class, struct, enum, or trait
- `ImportIr`: language-aware import or re-export statement
- `CallsiteIr`: unresolved call target observed in syntax
- `Span`: file coordinates for evidence and UI linking

## Identity

The thin slice uses deterministic string identifiers:

- file: `file:<relative-path>`
- symbol: `symbol:<relative-path>:<qualified-name>:<start-line>`
- import: `import:<relative-path>:<start-line>:<raw>`
- call: `call:<relative-path>:<start-line>:<target-name>`

These are intentionally easy to inspect, stable across repeated runs on unchanged content, and friendly to downstream wrappers.

## Extension points

The IR is designed to grow with:

- export relationships
- route and handler detection
- inheritance and implementation links
- reads and writes
- process traces
- test-to-code links

The current implementation already stores enough file and span metadata to add those without changing the overall architecture.
