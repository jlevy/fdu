---
type: is
id: is-01m3fnvw9aarv10r96g3x7tn9m
title: Show a concise code-first overview with explicit ignored populations and coverage
kind: feature
status: open
priority: 2
version: 4
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies: []
created_at: 2026-09-26T20:17:50.887Z
updated_at: 2026-09-26T23:11:15.874Z
---
Implement the first presentation increment from the research: code-first language counts and percentages, a stated population, separate ignored/non-ignored/all tallies where measured, and unavailable coverage rather than zeros. Reuse engine metrics across Rust, Python and CLI; preserve every denominator and expose bounds. Default-population change is a proposal needing explicit design review.

## Notes

PR #130 defaults ignored population to include consistently. Show both contributions directly in the code/language overview, with all totals only where measured. Exclude and only collapse to the selected population. All metrics in a report use that report population; compose ordinary reports for different populations or analyzers. Existing analyze/view controls remain the only measurement/presentation controls. Preserve metric sorting, explicit denominators and unknown coverage.
