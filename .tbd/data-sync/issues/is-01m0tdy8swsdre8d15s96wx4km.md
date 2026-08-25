---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 12
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
updated_at: 2026-08-25T01:48:46.460Z
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

BOTH REVIEW P1s FIXED at 278457a+ (`make check` green, parity holds). The bead stays open
for its mapped remainder: per-batch work counters.

P1-A, THE BATCH IS THE JOURNAL SLICE NOW. `Session::next_batch` built its batch from the
deltas the watcher sink handed back. Those are not the same set as "everything the consumer
has not seen": `apply_next` reconciles through several separately locked flushes, so a
direct producer -- a caller refreshing, or ingesting its own hints, against the same handle
-- can commit between two of them. The batch omitted that commit and advanced its cursor
past it, so resuming from the cursor skipped it for good.

`Session` holds the consumer's `resume: Cursor` and each batch is `IndexHandle::since(resume)`:
one guard, the complete slice plus its terminal position, so the cursor cannot name a commit
the slice does not carry. `since.truncated` becomes reset *and* all_dirty.

My earlier fix detected the gap and reported `reset`. That was the weaker contract -- telling
a consumer to throw everything away is not the same as handing it what it missed -- and the
review was right that the test would have accepted a stream that simply never delivered the
commit. `stepped_over_a_commit` is gone: with the slice the gap is unrepresentable rather
than detected.

The re-tag rebind now happens before the slice is taken, so its commit is *inside* the
slice rather than appended after it. `rebind_tags_for` returns nothing: there is one place a
batch's contents come from.

`Recovery::of(escalated, truncated)` names the two incompletenesses because they differ in
one bit that matters. An escalation names the subtree it re-scanned, so the dirty list still
means something; a truncated journal cannot name what it dropped, so a list of survivors
reads as complete and is not. Only the second sets all_dirty.

Test: `a_commit_from_another_producer_is_delivered_rather_than_skipped` forces the
interleaving and asserts the other producer's path *arrives*, with no reset -- nothing was
lost. Mutation-checked: building from the sink again yields `["b.txt"]` and drops
`elsewhere.txt`.

P1-B, TEARDOWN CANNOT SUCCEED OVER A LIVE WORKER. Two independent causes, and fixing either
alone leaves the defect.

The caller's `interval` became the native wait the worker parked in, so `interval=60` meant
a stop went unnoticed for up to a minute. Those are different questions: the interval is how
often a caller wants to hear from an idle tree; the pull bound is how long the worker can be
unreachable. The adapter now pulls with its own `_PULL_INTERVAL` (0.25s) and costs nothing
observable, since an idle pull returns an empty batch and empty batches were already
filtered.

`Thread.join` reports nothing -- it returns whether the thread died or the timeout expired --
so the old code could not distinguish "stopped" from "gave up waiting", and returned normally
either way. `worker.is_alive()` is now checked and `WatchTeardownError` raised. A teardown
that returns normally says the registration is released, and a caller that believes it goes
on to open the next watch or exit the process.

Test: the cancellation test uses `interval=60.0`. At `interval=0.1` the two bounds are
indistinguishable, which is why the previous version could not see this. Mutation-checked:
passing the caller's options through to the watch raises `WatchTeardownError: the watch
worker was still running 5.0s after being told to stop` -- exactly the reported failure.

STILL OPEN: per-batch work counters. Provider *state* is deliberately not duplicated onto
the batch -- a batch is a delta, and `Batch.state` now carries the transitions while the
envelope is read under the same guard as the rows it describes. A second copy on the batch
could only agree with that one or be wrong about it, which is the same argument that keeps
`StateChange::RunFacts` payload-free.
