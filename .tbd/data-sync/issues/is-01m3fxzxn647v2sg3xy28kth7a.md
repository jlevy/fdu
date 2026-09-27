---
type: is
id: is-01m3fxzxn647v2sg3xy28kth7a
title: Prune excluded ignored subtrees under the unified population policy
kind: feature
status: in_progress
priority: 2
version: 8
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-cache-layout
labels: []
dependencies:
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
created_at: 2026-09-26T22:39:51.973Z
updated_at: 2026-09-27T01:27:23.795Z
started_at: 2026-09-27T01:27:23.782Z
---
Implement and verify traversal pruning for --ignored=exclude as part of the unified ignored-population design in fdu-gdg0. No separate scan-ignored flag. Follow effective ancestor and negation rules, do not prune unknown classifications, distinguish control reads from content reads, and mark pruned totals unavailable. Exercise rule changes and all-to-excluded/excluded-to-all cache transitions with enumeration counters. The unified exclusion option must not ship as a report-only filter.
