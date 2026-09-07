---
type: is
id: is-01m1x444e4rnksjs8v8p37padv
title: Refresh the formal PR stack before final parity validation
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T05:05:45.411Z
updated_at: 2026-09-07T06:01:00.469Z
---
Inspect live remote heads and formal gh stack53, preserve other work, incorporate the latest PR48 parent fix into PR50 and descendants with safe stack-aware rebase/restack, use lease-guarded pushes for rewritten owned refs, and wait for CI on every pushed stack member. Final parity must use the integrated final branch identity.

## Notes

Formal gh stack #53 restacked with --no-trunk and pushed via gh stack push (lease-guarded). Current heads: PR48 c853f7c, PR50 5ecec17, PR51 19c0d73, PR52 95cd4b0. Only inherited code delta from pre-restack PR52 is PR48's paging progress fix and test plus its plan update. One absent checkout's stale 88 KiB Git registration was staged in Trash to release the branch; no actual worktree, branch, or agent log was deleted. CI pending on all updated descendants.
