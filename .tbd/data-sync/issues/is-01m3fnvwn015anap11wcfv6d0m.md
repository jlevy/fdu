---
type: is
id: is-01m3fnvwn015anap11wcfv6d0m
title: Avoid reading ignored content when the analysis request excludes it
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T20:17:51.263Z
updated_at: 2026-09-26T20:17:51.263Z
---
A cold fdu 0.1.0 probe with .gitignore, main.rs (1 LOC), ignored/vendor.rs (100 LOC) reports 101 versus 1 LOC with --exclude-ignored, but both invocations analyze 3 fresh files and read 1.7 KiB. Model content-selection scope, coverage and cache identity in the engine. Skip excluded body reads while preserving metadata totals. Prove cold/warm transitions and ignore-rule changes. Traversal pruning is separate because it loses exact ignored totals.
