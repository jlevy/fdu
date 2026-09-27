---
type: is
id: is-01m3fnvwn015anap11wcfv6d0m
title: Unify ignored-population selection and derive required scan and content work
kind: feature
status: closed
priority: 2
version: 13
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: codex-integration
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
hold: null
hold_until: null
created_at: 2026-09-26T20:17:51.263Z
updated_at: 2026-09-27T03:12:08.841Z
started_at: 2026-09-27T01:19:38.212Z
closed_at: 2026-09-27T03:12:08.840Z
close_reason: Unified population and real traversal/read pruning across Rust, CLI, and Python in PR133. Full16,787-case path-independence matrix passes, including population and cache histories; unknown controls remain explicit. Correctness and work counters are linked from the plan.
resolution: null
duplicate_of: null
---
Replace the alpha ignored booleans with one include/exclude/only population choice. Exclude must prune safely ignored subtrees and skip ignored body analysis; only must discover ignored children through non-ignored ancestors. Include reports both populations separately. Default include consistently for metadata and content requests. Model population, measurements, presentation and retained coverage in the engine; prove cold/warm parity, control-read accounting, unknown coverage and rule-change invalidation. Complete traversal work in fdu-a0kr as part of the same public option.

## Notes

PR #130 now uses only --ignored for population, existing --analyze for measurement, and --view for presentation. Exclude prunes safely ignored subtrees as well as skipping bodies. There is no scan-ignored or analyze-scope flag. Mixed measurement populations use separate ordinary requests with independent denominators and compatible cache reuse.
