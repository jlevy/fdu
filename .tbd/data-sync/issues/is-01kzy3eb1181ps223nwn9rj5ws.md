---
type: is
id: is-01kzy3eb1181ps223nwn9rj5ws
title: Reduce the argument counts introduced by the scan and reconcile fast paths
kind: chore
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T17:41:42.049Z
updated_at: 2026-08-13T17:41:42.049Z
---
PR #8 added two #[allow(clippy::too_many_arguments)] - record_walk_entry takes 13 parameters and reconcile_wave_worker takes 9 - plus a process_entry closure in reconcile_target_inner that re-threads target, queue, batch, sink, and report on every call purely to satisfy the borrow checker. A small per-walk context struct would remove all three allows and make the two backends' call sites read the same.
