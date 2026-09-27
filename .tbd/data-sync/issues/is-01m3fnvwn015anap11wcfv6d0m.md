---
type: is
id: is-01m3fnvwn015anap11wcfv6d0m
title: Unify ignored-population selection and derive required scan and content work
kind: feature
status: open
priority: 2
version: 11
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3fxzxn647v2sg3xy28kth7a
  - type: blocks
    target: is-01m3fznvkp0zx3rhf2vg3wrsv5
  - type: blocks
    target: is-01m3fnvw9aarv10r96g3x7tn9m
  - type: blocks
    target: is-01m3g53gjm5c6ks1t77k1az0qh
  - type: blocks
    target: is-01m3g3awk2dm23cja0nvjhfnmf
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-26T20:17:51.263Z
updated_at: 2026-09-27T00:50:04.579Z
---
Replace the alpha ignored booleans with one include/exclude/only population choice. Exclude must prune safely ignored subtrees and skip ignored body analysis; only must discover ignored children through non-ignored ancestors. Include reports both populations separately. Default include consistently for metadata and content requests. Model population, measurements, presentation and retained coverage in the engine; prove cold/warm parity, control-read accounting, unknown coverage and rule-change invalidation. Complete traversal work in fdu-a0kr as part of the same public option.

## Notes

PR #130 now uses only --ignored for population, existing --analyze for measurement, and --view for presentation. Exclude prunes safely ignored subtrees as well as skipping bodies. There is no scan-ignored or analyze-scope flag. Mixed measurement populations use separate ordinary requests with independent denominators and compatible cache reuse.
