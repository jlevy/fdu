---
type: is
id: is-01m3fnvwn015anap11wcfv6d0m
title: Unify ignored-population selection and derive required scan and content work
kind: feature
status: open
priority: 2
version: 6
spec_path: docs/project/research/research-2026-09-26-codebase-analysis.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3fxzxn647v2sg3xy28kth7a
  - type: blocks
    target: is-01m3fznvkp0zx3rhf2vg3wrsv5
created_at: 2026-09-26T20:17:51.263Z
updated_at: 2026-09-26T23:11:15.567Z
---
Replace the alpha ignored booleans with one include/exclude/only population choice. Exclude must prune safely ignored subtrees and skip ignored body analysis; only must discover ignored children through non-ignored ancestors. Include reports both populations separately. Default include consistently for metadata and content requests. Model population, measurements, presentation and retained coverage in the engine; prove cold/warm parity, control-read accounting, unknown coverage and rule-change invalidation. Complete traversal work in fdu-a0kr as part of the same public option.

## Notes

PR #130 now uses only --ignored for population, existing --analyze for measurement, and --view for presentation. Exclude prunes safely ignored subtrees as well as skipping bodies. There is no scan-ignored or analyze-scope flag. Mixed measurement populations use separate ordinary requests with independent denominators and compatible cache reuse.
