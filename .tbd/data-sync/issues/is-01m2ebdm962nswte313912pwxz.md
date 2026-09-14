---
type: is
id: is-01m2ebdm962nswte313912pwxz
title: "PR #48 review LIFE-10: panic wakeups, first-failure masking, and a raw DirectoryComplete path"
kind: bug
status: open
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:40:21.925Z
updated_at: 2026-09-14T03:21:18.216Z
---
Low. opened/journal.rs:84-97; opened.rs:1510-1525; index.rs:1523-1529. After a worker panic or poisoned lock, blocked changes() waiters wake only on timeout or close; join_workers reports the first failure in spawn order, so discovery's poisoning error can mask the observation worker's panic; DirectoryComplete carries the producer's raw path, canonical only because discovery builds it. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).

## Notes

2026-09-13 (PR #48 review PY-1 suggestion; no disposition): "Consider catching worker panics in production spawn_worker, as the test build already does."

PY-1 itself was fixed (fdu-lddk): `IndexLockPoisoned` maps to `OpenedIndexError`, and PanicException and poisoning are documented. The suggestion was neither taken nor rejected.

It is the same failure path as this bead. A caught worker panic could publish a typed failure and wake blocked changes() waiters, instead of leaving them to their timeout. It would also stop discovery's poisoning error from masking the observation worker's panic in join_workers. Decide it together with this bead's fix.

2026-09-14 (fix wave, w-life): fix written, NOT yet compiled or tested -- host below the 6 GiB disk floor before any build. Local branch w-life in worktree agent-a365bdbfabb1cd605, off baf6c00. Decision on PY-1's suggestion: TAKEN. Production spawn_worker now catches the panic exactly long enough for the worker to record its own exit, then resume_unwind()s, so the thread still dies by panic and the default hook has already printed it. Reasoning: without the catch the process has no moment at which it knows a worker died, so neither a wakeup nor time-ordered reporting is possible; AssertUnwindSafe is sound because only poison-tolerant Arcs are touched afterwards. Mechanism: OpenedState.failures: Arc<WorkerFailures> (Mutex<Vec<CloseOutcome>> in exit order). Each worker exit records WorkerFailed{source} or WorkerPanicked and calls JournalWait::wake(). journal::poll returns Err(OpenedWorkerPanicked{worker}) when a panic is recorded, checked after the retained-commits check so history before the panic is still delivered; returned worker errors keep publishing their Failed transition as before, so changes() behavior for them is unchanged. join_workers(workers, &failures) joins for completion and reports WorkerFailures::first(): the earliest recorded failure, unless it is only a poison trace (WorkerFailed whose source is IndexLockPoisoned / OpenedLifecyclePoisoned / OpenedJournalPoisoned), which a panic outranks -- strict time order is impossible because the guard is poisoned during unwinding, before the panicking thread can record. Raw DirectoryComplete: commit_prepared_with canonicalizes discovery.directory_complete once and publishes the canonical path. Not covered: a client-thread panic that poisons the index lock wakes nobody (no worker exits); the next poll after any wake returns IndexLockPoisoned. Tests (opened.rs): a_worker_panic_wakes_a_blocked_change_poll_with_its_typed_failure (BeforeJournalWait gate, 60 s poll timeout), close_reports_the_failure_that_happened_first_not_the_worker_spawned_first, close_reports_a_panic_before_the_poisoning_it_left_behind, directory_completion_publishes_the_canonical_relative_path ('./known' -> 'known').
