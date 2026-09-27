---
type: is
id: is-01m3fnvw9aarv10r96g3x7tn9m
title: Show a concise code-first overview with explicit ignored populations and coverage
kind: feature
status: open
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-26T20:17:50.887Z
updated_at: 2026-09-27T00:50:55.320Z
---
Implement the linked plan code view and analyzer-driven default: code totals, analyzed source/language counts, comment/blank totals, complete language table, ignored/non-ignored/combined contributions for include, collapsed selected population for exclude/only, and honest unknown/unavailable coverage. Core owns default include consistently; Rust, Python and CLI share report semantics, metric requirements and denominators.

## Notes

PR #130 defaults ignored population to include consistently. Show both contributions directly in the code/language overview, with all totals only where measured. Exclude and only collapse to the selected population. All metrics in a report use that report population; compose ordinary reports for different populations or analyzers. Existing analyze/view controls remain the only measurement/presentation controls. Preserve metric sorting, explicit denominators and unknown coverage.
