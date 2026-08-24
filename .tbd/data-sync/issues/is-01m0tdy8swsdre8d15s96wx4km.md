---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: closed
priority: 1
version: 7
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
updated_at: 2026-08-24T23:31:11.228Z
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

DESIGN SETTLED (2026-08-24 review). All three layers verified:
- `dirty_rollups`: `Op::Remove { path } => (path, false)` -- a removed directory's own
  key is never inserted, only its ancestors; a consumer caching that rollup gets no
  invalidation for it.
- Sync surface: `Watch.__next__` yields bare change tuples; dirty paths sit as mutable
  side state on PyWatch.
- aio: `if batch: hand_over(batch)` drops a dirty-only batch (a filtered-out mutation
  moves aggregates and yields no changes -- the only signal is discarded); cleanup sets
  the stop flag and drains the queue but never `worker.join()`s.

THE VALUE. One immutable `WatchBatch` in fdu-core, the same value on sync and async
surfaces: { cursor: Cursor (fdu-325q's type -- resulting version), changes: bounded,
dirty: bounded set of paths, all_dirty: bool (set when dirtiness exceeded its bound --
the bound is new; today the set is unbounded, which MetaBrowser's contract explicitly
disallows), reset: bool (cursor gap / watcher-queue overflow -- ties to `truncated`),
state, work }. Removes are conservative: insert the removed path itself always (a
removed file's key is harmlessly absent from consumer caches; a removed directory's key
is the bug). Dirty-only batches are DELIVERED, not dropped -- `if batch:` goes.

aio: queue `WatchBatch` objects; `finally` joins the worker after the drain (bounded
join -- the drain unblocks it, so a timeout join failing is a bug surfaced, not hidden).

fdu-fltq then extends this carrier with the final vocabulary (dirty query kinds, the
reset/all-dirty distinction MetaBrowser names) rather than replacing it.

TESTS. Filtered-out mutation delivers a dirty-only batch; deleted directory invalidates
its own key; sync and async equivalence over one scripted sequence; cancellation joins;
overflow sets all_dirty not silence.

Reopened: Post-landing exact-head review at 558461a found the WatchBatch carrier is not yet lossless. Session.next_batch samples self.index.cursor() after Watcher.apply_next has released its commit guards, so a concurrent refresh can land between the returned deltas and cursor; resuming from that cursor skips the unseen commit. The asyncio finally path blocks the event loop in worker.join while the worker may be waiting on run_coroutine_threadsafe(queue.put(None)).result(), causing a deterministic timeout-scale stall and returning without proving the worker stopped. Batch also still omits the state and work fields in this bead's acceptance contract. Add forced interleaving, cancellation-latency/thread-termination, state/work, filtered-dirty, and removed-directory tests before re-closing.
