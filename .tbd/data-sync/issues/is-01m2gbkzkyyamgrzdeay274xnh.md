---
type: is
id: is-01m2gbkzkyyamgrzdeay274xnh
title: changes() reports IndexLockPoisoned, not OpenedWorkerPanicked, when a worker panics holding the write lock
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T16:22:19.001Z
updated_at: 2026-09-14T16:22:19.001Z
---
PR #56 review PR56-LIFE-2 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5200187448). Locations at cfd1335: opened/journal.rs:73-86 and :98-103, opened.rs:288-291. When a worker panics while holding the index write lock, the blocked poll does wake, but read_with returns IndexLockPoisoned before the panicked() check. The caller gets the poison error, not the OpenedWorkerPanicked that the new changes() doc promises. close still names the worker correctly. Fix: check panicked() before the lock-poison mapping in changes(), or make the doc name both outcomes. Add a test.
