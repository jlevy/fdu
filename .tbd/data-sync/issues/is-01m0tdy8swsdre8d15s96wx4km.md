---
type: is
id: is-01m0tdy8swsdre8d15s96wx4km
title: Watch invalidation batches lose required dirty information
kind: bug
status: open
priority: 1
version: 24
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019372007
    at: 2026-08-25T13:21:14.300Z
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
updated_at: 2026-08-26T07:01:50.826Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR 47 head e658915, two paths lose invalidation information. Core dirty_rollups treats every Remove as a non-directory and omits the removed path itself, so a cached rollup for a deleted or renamed directory is never invalidated. The Python async adapter queues only tuple[Change, ...]; dirty_rollups is a side property on the worker-owned Watch, and the adapter drops an empty selected batch even when hidden changes dirtied aggregates. It also returns after setting stop without joining the worker. Fix: define one immutable WatchBatch returned by sync and async surfaces, carrying resulting cursor or version, changes, bounded dirty data, reset or all-dirty, state, and work. Include removed paths conservatively or retain old kind. Test filtered-out mutations, removed directories, async delivery, and joined cancellation. This supplies the lossless carrier that fdu-fltq can extend. Review finding FDU47-R5.

## Notes

FDU47-E5 addressed at 825fd92: the invalidations-only comparison is no longer
timing-dependent.

The macOS job for ce8d78b failed at "the row-carrying mode carries rows". The
test took the *first* dirty batch from each of two independently opened sessions,
and attaching a watch can commit a lifecycle transition of its own -- so "the
first dirty batch" is a different interval per session depending on scheduling,
and the row-carrying half came to be compared against a state-only batch. The
contract permits that batch; the test did not.

Each feed now gets its own identically seeded tree and is folded from open until
the write is applied, which is one interval by construction rather than by
timing. Per-batch shape is not the question -- either mode may split one write
across several batches -- so the fold compares whether a *signal* was lost:
changes, work rows and name bytes, dirty, dirty_rollups, dirty_queries,
all_dirty, reset, cursor and terminal freshness.

The same ordering mistake was in five other watch tests, all fixed in the same
commit: each created the thing under test, then an ordinary file, then waited for
*that file's* change. The comment even claimed the ordering was sound -- "one
backend, one queue, and this is behind it" -- which is true of inotify and not of
FSEvents, which reports directories and coalesces. wait_until now drains until
the index satisfies a predicate, which is the state the test is about; absence
cannot be waited for, so the negative assertions pair with a positive one and a
bounded settle.

Also at 353d48f, the reference adapter (FDU47-E4) now opens the watch with
Interest.INVALIDATIONS and prints state and work, so the example demonstrates
zero entry rows crossing the binding rather than repeating a caveat about a gap
that had closed.
