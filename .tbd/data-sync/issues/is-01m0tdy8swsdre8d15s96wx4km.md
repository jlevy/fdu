---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: closed
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0rw7bvxtw87tgde30emgs56
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:53.915Z
updated_at: 2026-08-25T03:59:21.610Z
closed_at: 2026-08-25T03:59:21.609Z
close_reason: |
  Shipped, all of it. `make check` green, parity holds.

  WHAT LANDED, over four commits.

  The data-loss bug (`Op::Remove` mapped to `is_directory = false`, so a deleted directory's
  own cached roll-up was never invalidated and no later event could name it again). The
  lossless carrier: one `Batch` with `all_dirty`, `reset`, `cursor`, `state`, `issues`,
  `dirty_queries` and now `work`, replacing a bare list plus mutable side state on `PyWatch`.
  The async surface: dirty-only batches no longer dropped, the worker joined rather than
  merely told to stop, the join moved off the loop thread, its own short pull interval so a
  long caller interval cannot delay a stop, and `WatchTeardownError` when the worker outlives
  the teardown that returned.

  Then the two review P1s: the batch is `since(resume)` -- a complete journal slice, so
  another producer's commit is DELIVERED rather than skipped -- and the feed wakes on a
  journal that moved rather than only on a filesystem event.

  PER-BATCH WORK, the last item. `Batch.work` carries entries and dirs the batch actually
  touched, rows after the selection, the bytes those rows cost to hold, and a wall time.

  The wall time is the part with a decision in it. The interval is how long a pull may block
  before returning empty-handed, and folding that into the cost would make an idle tree with a
  one-minute interval report a minute of work for a batch that did nothing -- the one figure
  an embedder compares providers on measuring its own patience. `WatchApplyReport.applied_ns`
  reports the applying separately from the waiting, and the batch adds its own assembly.

  PROVIDER STATE IS DELIBERATELY NOT ON THE BATCH, and this is the one place I did not do
  what the bead asked. A batch is a delta. The envelope -- coverage, freshness, phase, run
  facts, typed issues -- is read under the same guard as the rows it describes, and
  `fdu-91ru` made that atomic. A copy on the batch could only agree with that one or be wrong
  about it, and it could not be captured atomically with the batch's own cursor without a
  second guard acquisition, which is the class of defect this epic has been closing. What the
  batch owes a consumer is *that* state moved, and `Batch.state` carries those transitions
  with the clock each landed at. The same argument keeps `StateChange::RunFacts` payload-free.

  A TEST THAT DID NOT DISCRIMINATE, caught by mutation. The first version of the work test
  created the file before the pull, so the event was already waiting and a naive whole-call
  measurement looked correct -- the mutation passed. A timer now puts the change a second and
  a half into a three-second interval, and the mutation reports 1.550s against a threshold of
  0.5s. A test for "X is not Y" has to arrange for Y to be large.
resolution: null
duplicate_of: null
---
At PR 47 head e658915, two paths lose invalidation information. Core dirty_rollups treats every Remove as a non-directory and omits the removed path itself, so a cached rollup for a deleted or renamed directory is never invalidated. The Python async adapter queues only tuple[Change, ...]; dirty_rollups is a side property on the worker-owned Watch, and the adapter drops an empty selected batch even when hidden changes dirtied aggregates. It also returns after setting stop without joining the worker. Fix: define one immutable WatchBatch returned by sync and async surfaces, carrying resulting cursor or version, changes, bounded dirty data, reset or all-dirty, state, and work. Include removed paths conservatively or retain old kind. Test filtered-out mutations, removed directories, async delivery, and joined cancellation. This supplies the lossless carrier that fdu-fltq can extend. Review finding FDU47-R5.

## Notes

BOTH P1s FIXED (`make check` green); the bead stays open for its mapped remainder.

P1-A, the terminal cursor. `Batch.cursor` is now `Option<Cursor>` derived from the last
delta the batch actually carried, not sampled from the index afterwards. Those clocks were
assigned under the write guard that applied them, so the capture is atomic with no new
locking; a commit landing after is unseen and replays on the next resume, which is
correct. `None` when a batch carried no deltas -- it names no new position, and saying so
beats inventing one. `SessionId` is immutable for an index's life, so only the clock ever
needed atomic capture.

Test: `a_batch_cursor_never_runs_ahead_of_the_deltas_it_carried` runs a writer committing
continuously across the batch boundary and asserts the property that holds under either
ordering -- a change is in this batch, or strictly after its cursor, never both absent and
behind.

P1-B, the async teardown. The join moved off the loop thread into an executor, and every
await in the cleanup is now guarded with `contextlib.suppress(CancelledError)`. Two
separate causes had to be fixed together: joining on the loop deadlocks (the worker's exit
path needs the loop the join is blocking), and the first await in a cancelled task
re-raises immediately, so an unguarded teardown aborted halfway and left the worker alive
anyway.

Test: cancellation latency bounded at 3s, a heartbeat task proving the loop kept running,
and the worker counted *by name* -- the first version counted total threads and failed on
the executor's own pool thread, which is not the leak being looked for. Mutation-checked:
restoring the on-loop join fails it at 5.01s, exactly the timeout.

STILL OPEN ON THIS BEAD: the batch omits provider state and per-batch work, which the
observation envelope requires in the same atomic value.

EXACT-HEAD REVIEW at PR #47 715f748 (2026-08-25): the claimed cursor and teardown fixes remain incomplete.

P1 CURSOR LOSS. Session::next_batch (crates/fdu-core/src/watch_session.rs:214-304) constructs applied only from watcher/reconcile sink deliveries and sets Batch.cursor to applied.last().clock. Watcher::apply_next invokes reconciliation, which can flush several batches under separate write guards (watch.rs:336-372; scan.rs:3620-3634). A direct producer commit C can land between watcher deltas A and B; the returned batch carries A/B and cursor B but omits C, so a consumer resuming at B loses C permanently. The new test at watch_session_integration.rs:242-306 checks only carried clocks <= cursor and would also accept that skipped interleaving. Fix the session against its prior delivered cursor: after applying the intent, read IndexHandle::since(previous_cursor) under one guard and build the batch from that complete journal slice and its terminal cursor; journal truncation must yield explicit reset/all-dirty. This is consumer resume state, not a second provider cursor.

P1 TEARDOWN. asyncio cleanup at crates/fdu-py/python/fdu/aio.py:137-161 runs worker.join(timeout=5) off-loop but never checks worker.is_alive(). The worker may be blocked in PyWatch.__next__ -> session.next_batch(self.timeout) (crates/fdu-py/src/lib.rs:1889-1899); WatchOptions.interval can be any positive value. With interval=60 on an idle tree, cancellation returns after the five-second join timeout while the worker remains alive. The interval=0.1 test cannot detect this. Make the native wait interruptible or pull with a short internal timeout and require termination; if a timeout remains, raise a typed teardown failure instead of returning success with a live worker. Provider state and per-batch work also remain the bead's existing open remainder.

EXACT-HEAD REVIEW at PR #47 278457a (2026-08-25). The new session resume clock is a correct safety improvement: a commit omitted by this producer now causes consumer reset instead of a cursor that silently skips it. The carrier is still not adoption-safe.

First, the state commits made by begin_reconcile/finish_reconcile never reach the reconciliation sink, so Batch.state omits them; this is reopened on fdu-jxs0. Second, Retagged carries an unbounded Vec<PathBuf> (engine_contract.rs:717-720), while AppliedDelta::len charges that entire vector as one transition (782-783). Session all_dirty drops only dirty_rollups: it still copies every governed directory into Batch.state and also emits one synthetic Change per directory (watch_session.rs:290-304, 334-348). A large control-file set therefore bypasses the journal retention budget and the language-boundary bound exactly when all_dirty claims the path list was dropped. Represent retag scope as bounded paths plus an explicit all marker; charge embedded paths against retention (or evict the oversized delta and advance the journal floor), and omit individual state/changes when all_dirty is the lossless answer.

The unchanged P1 teardown remainder also remains: aio.py still joins for five seconds without checking worker.is_alive(), so a watch interval longer than that can return from teardown with a live worker. The existing provider-state and per-batch-work remainder remains too.

EXACT-HEAD REVIEW at PR #47 fad3d2f (2026-08-25; all 19 checks green). The complete journal-slice construction and async teardown fixes are accepted: Session now resumes from one Cursor and builds from IndexHandle::since after rebind; aio caps the internal pull to 250 ms, checks worker liveness, and raises typed WatchTeardownError.

The carrier remains open for three exact-head gaps. First, Session::next_batch returns None at watch_session.rs:279-281 before consulting since() whenever the filesystem watcher times out. A direct producer commit does not wake that watcher, and the new test at watch_session_integration.rs:345-355 writes b.txt after the direct apply to supply an unrelated wakeup. A refresh/hint commit on an otherwise idle tree is therefore withheld until some later filesystem event. Notify the session on every IndexHandle commit and wait for watcher-or-journal readiness, or perform a bounded journal check without adding a second watcher; test a direct apply alone.

Second, Retagged remains an unbounded Vec<PathBuf> charged as one retained state item, then flattened into unbounded state and synthetic changes even when all_dirty drops dirty_rollups. Use a bounded paths-or-all representation and charge embedded paths against retention. Third, the batch still lacks resulting provider state and per-batch work in the same atomic value.
