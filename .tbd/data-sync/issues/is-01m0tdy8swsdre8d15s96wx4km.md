---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 9
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
updated_at: 2026-08-24T23:34:02.148Z
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

REOPENED at `558461a` by the owner; I then closed it again by mistake and have reopened
it. Two P1 defects in what I shipped, plus mapped remainder.

P1-A: THE TERMINAL CURSOR IS SAMPLED AFTER THE COMMIT. `Session::next_batch` lets
`Watcher::apply_next` finish and release its write guards, then calls
`self.index.cursor()` separately. A refresh committing between those makes the batch
carry no record of it while returning a cursor past it -- resuming skips that commit
permanently. This is the same defect `fdu-325q` fixed for `since`, reintroduced one path
over, which is worth noticing: fixing an instance is not fixing the class.

FIX. The batch's cursor is the clock of the last delta *this batch carried*, not wherever
the index has since reached. Those clocks were assigned under the write guard that applied
them, so the capture is already atomic and no new locking is needed. A commit landing
after is simply unseen and replays on the next resume, which is correct. `SessionId` is
immutable for the life of an index, so reading it separately is safe -- only the clock
needed atomic capture, and the delta already carries it. `cursor` becomes `Option<Cursor>`:
a batch carrying no deltas names no new position, and saying so beats inventing one.

P1-B: ASYNC CANCELLATION CAN BLOCK THE EVENT LOOP. `watch_batches`' cleanup calls a
blocking `worker.join(timeout=5)` on the loop thread, while the worker's exit path is
`run_coroutine_threadsafe(queue.put(None), loop).result()`. The loop waits for the worker;
the worker waits for the loop. It resolves by timeout, stalling every request for five
seconds, and then returns without checking `is_alive()`. I added that join in this same
bead to fix "cancellation does not join" -- and introduced a worse failure than the one it
fixed.

FIX. Await the join off-loop (`run_in_executor`) so the loop keeps servicing the worker's
handoff, and keep draining while waiting. Test must bound cancellation latency, prove an
unrelated loop heartbeat keeps running, and assert the worker is gone -- not merely that
the index still works.

REMAINDER, still open on this bead: the batch omits provider state and per-batch work,
which the observation envelope requires in the same atomic value.

MOVED OFF THIS BEAD: the `reset` mapping. Under the settled contract, provider observation
loss is stale state + typed issue + reconciliation + dirty/`all_dirty`; `reset` is reserved
for a consumer cursor/session that cannot resume. My implementation maps watcher
overflow/setup-race to `reset`, which is wrong under that split. `fdu-fltq` owns it.
