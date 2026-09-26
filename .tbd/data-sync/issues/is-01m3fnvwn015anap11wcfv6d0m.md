---
type: is
id: is-01m3fnvwn015anap11wcfv6d0m
title: Avoid reading ignored content when the analysis request excludes it
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3fxzxn647v2sg3xy28kth7a
created_at: 2026-09-26T20:17:51.263Z
updated_at: 2026-09-26T22:39:51.973Z
---
A cold fdu 0.1.0 probe with .gitignore, main.rs (1 LOC), ignored/vendor.rs (100 LOC) reports 101 versus 1 LOC with --exclude-ignored, but both invocations analyze 3 fresh files and read 1.7 KiB. Model content-selection scope, coverage and cache identity in the engine. Skip excluded body reads while preserving metadata totals. Prove cold/warm transitions and ignore-rule changes. Traversal pruning is separate because it loses exact ignored totals.

## Notes

Research PR #130 proposes --ignored=include|exclude|only to replace the boolean flags; selected content population automatically limits body reads. Preserve metadata observation by default, distinguish control-file reads, keep report bounds out of analysis scope, and use explicit content identity and warm/cold parity. Advanced --scan-ignored scope is a separate later increment.
