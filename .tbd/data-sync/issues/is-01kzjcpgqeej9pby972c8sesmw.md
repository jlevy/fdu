---
type: is
id: is-01kzjcpgqeej9pby972c8sesmw
title: Retain unresolved invalidations for reconciliation retry
kind: bug
status: closed
priority: 1
version: 4
labels:
  - correctness
  - reconciliation
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:32:34.030Z
updated_at: 2026-08-09T05:10:37.425Z
closed_at: 2026-08-09T05:10:37.425Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
reconcile_pending drains every invalidation before work. A hard failure loses failed/unprocessed requests, a partial pass consumes unresolved work, and a child request below a missing/non-directory ancestor can retry ENOENT/ENOTDIR forever. Preserve collapsed roots until a complete pass succeeds and widen targeted reconciliation to the first missing/non-directory ancestor so retries converge.

## Notes

Implemented hard-error and partial retry retention, reason preservation, freshness assertions, missing/non-directory ancestor widening, and watch ENOTDIR escalation to the parent.
