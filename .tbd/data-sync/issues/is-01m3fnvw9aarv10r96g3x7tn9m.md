---
type: is
id: is-01m3fnvw9aarv10r96g3x7tn9m
title: Show a concise code-first overview with explicit ignored populations and coverage
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T20:17:50.887Z
updated_at: 2026-09-26T22:39:51.539Z
---
Implement the first presentation increment from the research: code-first language counts and percentages, a stated population, separate ignored/non-ignored/all tallies where measured, and unavailable coverage rather than zeros. Reuse engine metrics across Rust, Python and CLI; preserve every denominator and expose bounds. Default-population change is a proposal needing explicit design review.

## Notes

Research PR #130 now proposes a new code overview view, complete language table, explicit coverage/populations, and registered numeric metric sorting (code_lines). Owner explicitly waives alpha backward compatibility; replace old interfaces without aliases. Metadata-only defaults include ignored files; content-analysis defaults exclude, resolved once in the engine and serialized. See Proposed Design for full contract.
