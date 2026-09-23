---
type: is
id: is-01m2ysrer2fjkw6qa1z57epf46
title: Concurrent reconciliation can erase newer omitted failures
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:58:47.680Z
updated_at: 2026-09-23T01:27:17.842Z
closed_at: 2026-09-20T07:05:15.908Z
close_reason: Fixed with bounded epoch-owned omission accounting and concurrent-order regression
resolution: null
duplicate_of: null
---
Two public full-root refreshes can overlap because IndexHandle locks only begin/finish boundaries. Baseline subtraction by an older closer can erase omission counts produced by a newer pass and permit false Complete coverage. Track omitted failures with bounded epoch ownership and preserve newer-pass omissions.

## Notes

Implemented epoch-owned omitted-count buckets bounded by active reconciliation boundaries. Root passes remove only buckets older than their own epoch; newer overlapping pass omissions survive an older closer. Regression recreates the public begin-newer/finish-newer/finish-older ordering with a full retained ObservationGap set and newer omitted permission errors, asserting coverage stays Partial(Inaccessible). Full core validation passes.
