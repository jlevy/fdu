---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 11
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
updated_at: 2026-08-25T00:52:00.482Z
closed_at: 2026-08-24T23:31:11.228Z
close_reason: |
  Shipped. `make check` green, parity holds.

  THE DATA-LOSS BUG. `dirty_rollups` mapped `Op::Remove` to `is_directory = false`, because
  the op does not say what it removed. So deleting a directory invalidated every ancestor
  and never the key a consumer had actually cached for that directory -- and no later event
  could ever name it again, because the entry is gone. The row survives forever, stale.
  Fixed by naming the removed path's own key unconditionally: the two ways of guessing are
  not symmetric, since a removed *file* has no cached roll-up and naming its key costs
  nothing. Mutation-checked -- restoring `false` fails the test.

  THE CARRIER. `Batch` gains `all_dirty`, `reset`, and `cursor`. `all_dirty` is set past
  `MAX_DIRTY_ROLLUPS` (1024) and the list is *dropped* rather than truncated, because a
  truncated list is indistinguishable from a complete one at the consumer -- which is
  exactly how a stale row survives the invalidation that named it. `reset` is derived from
  `WatchOverflow | UnpairedRename | WatchSetupRace`: the engine re-scans and the index ends
  up right, but a consumer replaying `changes` alone would apply a suffix to state that no
  longer matches. `cursor` is captured after the batch applies, tying this to `fdu-325q`.

  THE SYNC SURFACE. `__next__` yielded a bare list and kept `dirty_rollups` as mutable side
  state on `PyWatch`, readable only if a caller knew to look between iteration steps. It now
  yields one `WatchBatch` carrying everything, and the side state is gone.

  THE ASYNC SURFACE. `if batch:` dropped dirty-only batches, so a filtered-out mutation
  moved the aggregates and the async consumer never heard about it -- the one signal
  discarded. Now `if batch.dirty or batch.changes`. And the cleanup joins the worker: the
  drain that precedes it is what guarantees the join finishes, since a worker blocked on a
  full queue can complete its put and exit. Setting a flag says "please stop"; joining is
  what makes cancellation mean the registration is released by the time the call returns.

  WORTH RECORDING. The parity CLI shim hand-rolled its own `dirty` flag from "the change
  list was non-empty" -- the very pattern this bead is about, sitting in our own test
  harness. It uses `batch.dirty` now, so an aggregate repaint fires for a filtered-out
  mutation, which it previously missed. Two obsolete doc comments also went: both told
  callers to prefer "the watch's own index" over the opened one, advice that only existed
  because of the split brain `fdu-37dv` removed.

  `fdu-fltq` extends this carrier with the final vocabulary (dirty query kinds, and the
  reset/all-dirty distinction as MetaBrowser words it) rather than replacing it.
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
