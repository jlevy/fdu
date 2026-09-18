# fdu Documentation

Start with the [usage guide](usage.md) when you want to run fdu.
It covers the default report, every view and analyzer, `.gitignore` selection, cache
behavior, watch, and stable machine-output contracts.
The repository [README](../README.md) is the landing page: install, the command, the
live change feed, and the Rust and Python libraries.
`fdu --docs` is the offline guide and `fdu --help` is the complete flag reference.

## Use fdu

- [Command-line usage](usage.md)
- [Live updates](../README.md#live-updates)
- [Rust library examples](../README.md#as-a-rust-library)
- [Python package examples](../README.md#as-a-python-module)
- [0.1.0 release notes](project/release-notes/0.1.0.md)

## Understand the Design

- [Design principles](project/architecture/fdu-design-principles.md): the invariants
  behind defaults, ordering, output shapes, and resource bounds
- [Engine architecture](project/architecture/fdu-engine-architecture.md): retained
  state, commits, serving, paging, observation, and shutdown
- [Surface architecture](project/architecture/fdu-surface-architecture.md): how the
  engine, command line, and Python package stay in parity
- [Cache design](project/guides/cache-design.md): metadata snapshots, content sidecars,
  verification costs, and cache policies
- [Architecture index](project/architecture/README.md)

## Performance Evidence

- [Current performance status](project/reports/report-2026-08-14-performance-campaign-status.md)
- [Performance evidence report](project/reports/report-2026-08-20-fdu-performance-evidence.md)
- [Experiment ledger](project/reports/report-2026-08-10-fdu-performance-experiments.md)
- [Performance loop](project/guides/performance-loop.md)
- [Latest tool comparison](project/reports/report-2026-09-16-fdu-live-tool-comparison.md)

## Build, Test, and Release

- [Agent and contributor instructions](../AGENTS.md)
- [Supply-chain policy](../SUPPLY-CHAIN-SECURITY.md)
- [Integration runbook](project/guides/integration-runbook.md)
- [Release process](project/guides/release-process.md)
- [First-release verification](project/specs/active/plan-2026-09-18-fdu-first-release-verification.md)
- [Changelog](../CHANGELOG.md)

Plans, research notes, experiment records, and generated evidence live under
[`docs/project`](project/). They preserve engineering decisions and measurements; they
are not the shortest route to learning the command line.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
