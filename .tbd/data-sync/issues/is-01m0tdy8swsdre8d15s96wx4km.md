---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 18
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
updated_at: 2026-08-25T07:18:51.773Z
closed_at: null
close_reason: null
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

Reopened: Exact-head review at PR #47 head `7aaaf84` against MetaBrowser #74 at `1e0f9b5`
found that this adoption gate was closed before the carrier matched the implemented
consumer contract.

1. **The resulting provider state is still absent.** MetaBrowser `ChangeBatch` requires
   `cursor`, `version`, and the complete `IndexState` from the same sequence, and the
   coordinator installs `batch.state` as its authoritative lifecycle, coverage,
   freshness, source, progress, and issue state. FDU `Batch.state` is only
   `Vec<CommittedState>` and `Batch.issues` is an interval event; `Index::since()` returns
   only deltas, cursor, and truncation. Reading afterward with
   `ReadRequest.expected=batch.cursor` races the next commit and can fail because only the
   current image is retained. Applying transitions in Python would create the forbidden
   mirror state. Return an owned terminal envelope from the same guard that captures the
   journal slice and cursor, then carry it through `Batch` and Python.
2. **The v1 invalidation feed still crosses entry rows.** The maintained integration
   spec says the v1 stream carries no entry rows, but `Session::next_batch()` constructs
   one `Change` per selected op and `PyWatch::__next__()` constructs a Python dictionary
   for every row before an adapter can discard it. Add a small closed interest mode so
   the MetaBrowser path derives bounded dirty paths/query kinds and state in Rust without
   materializing entry rows across the binding.
3. **Public-boundary batch work is incomplete.** `Batch.work` records native work and
   name bytes, but the watch conversion path does not count the logical binding payload,
   conversion time, or Python model construction; `_work()` silently defaults absent
   `binding_bytes` and `conversion_ns` to zero. Instrument the watch path as the bundled
   read path is instrumented, during conversion and without a second output traversal.
   CPU remains exact-or-absent.

Acceptance needs forced concurrent commit/read coverage proving the batch envelope and
cursor are one boundary, an invalidations-only test proving zero entry rows cross the
binding, and payload/cost tests for ordinary, state-only, all-dirty, reset, and issue
batches.

EXACT-HEAD FOLLOW-UP at PR #47 4eac1b2 (2026-08-25). The new `browser_provider` reference still constructs paths from `batch.changes` and drops provider state and batch work. That codifies the open adoption defect instead of exercising the required invalidations-only carrier. Keep this bead open until a closed-interest mode crosses zero entry rows and the same batch carries complete terminal state plus public-boundary work, with provider-harness tests.



FDU47-A1 SHIPPED: the terminal engine state now rides on every batch and delta range.

`Index::engine_state()` is the one place trust, coverage, lifecycle and run facts are
assembled, and `Index::since` captures it inside the guard that already produced the
journal slice and the cursor. `Since::state` and `Batch::state` carry it. The field the
transitions list used to occupy is now `Batch::transitions` / `WatchBatch.transitions` /
`ChangeSet.transitions`, because the two answer different questions and sharing a name
invited exactly the mistake this fixes.

Why a follow-up read is not an equivalent substitute, restated so it is not re-litigated:
the next commit can land between the two calls, and the index intentionally retains only
its current image, so there is nothing to ask for the state *as of* a position already
passed. Folding transitions into a consumer-side copy is the other way to get it wrong --
two authorities for one fact, diverging the first time one is dropped, reordered or
misapplied, with nothing able to detect it. This reverses the decision recorded in this
bead's earlier close reason, which held that transitions plus a follow-up read were
sufficient. They are not, for the reason above.

`EngineState` has a hand-written `Default` because `Freshness` deliberately has none:
there is no answer to "how much do you trust this?" that is safe to invent for an
arbitrary engine, and deriving one would put that invention in the contract.
`default_is_the_state_of_a_new_index` holds the hand-written value to
`Index::new(..).engine_state()` so the claim cannot drift from what it describes.

Acceptance, all three forcing the interleave rather than racing it -- a commit lands after
the capture and before the assertions, so an implementation that re-read at assertion time
would report the later state and fail:
- `index.rs:a_delta_range_carries_the_state_at_its_own_cursor`
- `tests/watch_session_integration.rs:a_batch_carries_the_terminal_state_at_its_own_cursor`
- `public_smoke.py:check_a_batch_carries_the_state_at_its_own_cursor`

Mutation-checked three ways, each confirmed to fail before being restored: `since()`
returning `EngineState::default()`, the batch carrying `EngineState::default()`, and the
Python binding dropping the field.

`browser_provider.Invalidation` now carries `state`, and its docstring no longer lists
terminal state as missing.

STILL OPEN on this bead (FDU47-A3): the invalidations-only interest mode, so the feed
derives bounded dirty paths, query kinds, issues and state in Rust without materializing
entry rows; and instrumenting the public watch conversion, where `Batch.work` stops before
binding payload, conversion and model construction.
