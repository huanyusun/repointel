# Graph Model

## Node kinds in the thin slice

- `repo`
- `file`
- `symbol`

## Edge kinds in the thin slice

- `contains`
- `imports`
- `calls`

## Planned expansion

The crate boundaries already anticipate additional node and edge kinds requested for the full product:

- module
- route
- process
- external dependency
- test
- defines
- exports
- inherits
- implements
- uses_type
- reads
- writes
- tests
- belongs_to_process

## Why precompute intelligence

Raw graph export is useful for visualization but inefficient for agents. The graph bundle therefore also stores:

- symbol context
- impact reports
- caller and callee neighborhoods
- importing files

That lets an agent ask focused questions like `repointel impact login --json` instead of requesting a full graph dump.
