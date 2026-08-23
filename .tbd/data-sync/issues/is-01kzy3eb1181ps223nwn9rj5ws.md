---
type: is
id: is-01kzy3eb1181ps223nwn9rj5ws
title: Reduce the argument counts introduced by the scan and reconcile fast paths
kind: chore
status: open
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T17:41:42.049Z
updated_at: 2026-08-23T02:11:33.033Z
---
PR #8 added two #[allow(clippy::too_many_arguments)] - record_walk_entry takes 13 parameters and reconcile_wave_worker takes 9 - plus a process_entry closure in reconcile_target_inner that re-threads target, queue, batch, sink, and report on every call purely to satisfy the borrow checker. A small per-walk context struct would remove all three allows and make the two backends' call sites read the same.

## Notes

Deferred from the PR #8 stability review. This is a real P3 maintainability cleanup, not a correctness or behavior defect; do it as a focused structural change after landing rather than expanding the reviewed concurrency diff.
