# Search And Agent

## Search

The thin slice starts with symbol-centric lexical search over the graph bundle. Results return:

- qualified symbol name
- file path
- line number
- concise reason string

The intent is compact retrieval, not broad context stuffing.

## Agent orchestration

`ci-agent` currently implements a small evidence-backed explanation path:

- exact symbol match: summarize definition, callers, callees, and importing files
- fallback: return top related symbol hits

This is deliberately constrained. The future product will expand this into full repository Q&A with graph-aware expansion and chunk packaging.

## Agent usage pattern

The desired flow for coding agents is:

1. ask the CLI for focused architectural context
2. summarize findings
3. identify blast radius
4. only then read the minimum set of files needed for an edit
