---
type: is
id: is-01m2gbkzkyyamgrzdeay274xnh
title: changes() reports IndexLockPoisoned, not OpenedWorkerPanicked, when a worker panics holding the write lock
kind: bug
status: closed
priority: 3
version: 5
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T16:22:19.001Z
updated_at: 2026-09-15T00:16:58.948Z
closed_at: 2026-09-15T00:16:58.946Z
close_reason: "43a7331: journal::poll maps IndexLockPoisoned to OpenedWorkerPanicked when a worker panic is recorded; changes() doc and CHANGELOG (PR56B-DOC-1) now say a panic inside a commit leaves no retained commits to deliver. New cfg(test) hook IndexHandle::panic_holding_the_write_lock_for_test; the wake test covers a panic inside and outside the write lock, run 8x."
resolution: null
duplicate_of: null
---
PR #56 review PR56-LIFE-2 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5200187448). Locations at cfd1335: opened/journal.rs:73-86 and :98-103, opened.rs:288-291. When a worker panics while holding the index write lock, the blocked poll does wake, but read_with returns IndexLockPoisoned before the panicked() check. The caller gets the poison error, not the OpenedWorkerPanicked that the new changes() doc promises. close still names the worker correctly. Fix: check panicked() before the lock-poison mapping in changes(), or make the doc name both outcomes. Add a test.

## Notes

Also covers delta review 5203772881 finding PR56B-DOC-1: CHANGELOG.md:94-95 at 8d2eb7f promises OpenedWorkerPanicked from a poll after a worker panic. Disposition: fix the code so a recorded worker panic takes precedence over the poison mapping, which makes the CHANGELOG true.
