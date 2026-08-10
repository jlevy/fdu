---
name: fdu
description: >-
  Inspect directory trees with hierarchical file counts, apparent and allocated sizes,
  recency, and extension tallies. Use when investigating disk usage, finding large
  directories, summarizing file types, or collecting stable JSON filesystem roll-ups
  for scripts and coding agents.
---
# fdu Directory Roll-Ups

Use `fdu` to summarize a directory tree without modifying files in that tree.

## Run fdu

Use the local command when it is available:

```bash
fdu --json --depth 2 --number 20 PATH
```

If no local command exists and this release is published on PyPI, use the exact reviewed
version. Never use an unversioned `uvx` runner or `latest` in agent instructions:

```bash
uvx --from fdu==__FDU_VERSION__ fdu --json --depth 2 --number 20 PATH
```

## Choose the View Deliberately

- Use the default human tree for terminal investigation.
- Use `--by-type` for an extension summary.
- Use `--json` for scripts and agents; it never contains ANSI color.
- Use `--apparent-size` for logical bytes instead of allocated disk space.
- Use `--no-cache` when the run must not read or write the user cache.

`--depth` and `--number` limit only the returned view.
`--max-depth` limits what is scanned and retained, so do not use it merely to reduce
output.

## Validate Every Automated Result

Check the process exit status and these JSON fields:

- `schema` before parsing fields
- `complete` and `errors` before trusting totals
- `tree_truncated` before treating the rendered tree as exhaustive
- `scan_max_depth` before treating the scan scope as exhaustive
- `freshness` before presenting cached or partial data as current

Exit 0 is accepted success, exit 1 is a fatal failure, and exit 2 is incomplete data or
invalid usage. Do not discard useful stdout from exit 2; inspect the completeness fields
and use `--allow-partial` only when incomplete totals are acceptable.

Run `fdu --help` for the complete flag, stream, cache, color, scope, and exit contract.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
