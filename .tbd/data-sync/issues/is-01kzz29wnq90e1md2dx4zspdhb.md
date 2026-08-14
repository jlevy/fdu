---
type: is
id: is-01kzz29wnq90e1md2dx4zspdhb
title: Parallel scan leaks a directory claim on early exit, hanging the walk
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:02.135Z
updated_at: 2026-08-14T02:41:02.135Z
---
crates/fdu/src/scan.rs walk_worker calls queue.release(...) only at the bottom of the claim loop. The 'break 'walk' taken when the consumer disconnects skips it, and a worker that panics mid-chunk skips it too.

DirectoryQueue::claim blocks on the condvar while state.outstanding > 0 and only sets finished when it reaches zero, so a leaked claim means every remaining worker waits forever and the scoped join in scan_concurrent never returns. scan_concurrent already has code to report a panicked worker as a partial scan, but it can deadlock before reaching it.

PR #4 fixed this with a DirectoryClaim RAII guard whose Drop is the only release path, plus a WorkerAbortGuard that converts an unwinding worker into a queue-wide abort. Port both, adapted to main's release(entries, work_ns, timing) -> bool signature, which also carries the adaptive scale-up decision.
