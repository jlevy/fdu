---
type: is
id: is-01m15zr7prnyhnfgc5wa325x0e
title: Bound the tree level-advance search without stranding the cursor
kind: task
status: open
priority: 1
version: 6
labels: []
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
  - type: blocks
    target: is-01m1687g2cazrcaxzwkpdcazz5
created_at: 2026-08-29T05:26:49.303Z
updated_at: 2026-08-29T08:12:57.103Z
---
The breadth-first tree projection charges its level-advance search against `spent` but
never cuts it short, so one page can exceed `page.max_work` by a scan of one level.

## Why this blocks rather than waits

The plan spec's implementation table requires MetaBrowser's `assemble_tree_pages` to
enforce "stable provider version, positive row bound, unique advancing opaque
continuation, maximum pages, maximum rows, and request work budget". fdu can report
`rows_visited` above the requested `max_work`, so the deviation is unrepresentable in the
contract the other side is specified to enforce. It has to be closed in the engine or
renegotiated in both specs, and closing it is cleaner. See `fdu-3v0d` for the
currently-unimplemented half on the MetaBrowser side, which must land second.

The cost in isolation is modest: total work is unchanged, because walking a level to find
its directory children is work any breadth-first traversal must do, and the parent the
scan finds is what the cursor then records, so it is not repeated per page. The damage is
that the work is lumpy — one page spends steps proportional to a level's width instead of
its budget, on trees whose levels are mostly leaves.

## Approach: memoize instead of rescanning

The expensive scan is `directory_at_depth(deeper, None)` — discovering whether a level
below exists by asking every parent at the current depth for a directory child. But the
walk already visits every one of those parents while emitting.

So record it in passing: while emitting children of parent P at depth d, if P has a
non-ignored directory child and none is recorded yet, remember it. When the level is
exhausted that value is the first parent at d+1 in level order, because level order at d+1
is grouped by parent order at d. Nothing recorded means there is no level below.

O(1) instead of O(level width), and it removes the scan rather than making it resumable.

An earlier plan — teaching `ChildPosition` to express the searching state as well as the
emitting one — is the other way, and is worse: it keeps the scan and makes it resumable,
which needs `directory_at_depth` to report where it stopped, and resumption still re-pays
the inner search when the walk has to ascend more than one level.

## Rejected

Cutting the search short with the one-frame cursor as it stands. A search interrupted
mid-level has no emitted row to point at, so the next page restarts it and overruns the
same budget; a level wider than `max_work` is then never crossed and the tree becomes
unpageable rather than slow. Returning `ProjectionResult::Limit` has the same effect,
since re-asking cannot make progress.

## Scope

`crates/fdu-core/src/opened/read.rs`, the `tree_projection` level-advance block and
`directory_at_depth`; one more optional path on `ChildPosition`, still bounded by a single
path. Tests must cover a level of pure leaves (where the scan is worst), a sparse level,
resumption across the level boundary carrying the memo, and the ignored-subtree pruning
interaction — the memo records the first *non-ignored* directory child, so a fixture where
the first candidate is pruned distinguishes a correct memo from a plausible one.

Paging correctness is already pinned by
`opened::tests::a_tree_page_stopped_by_the_work_budget_is_resumable`; this must not
regress it.

## Notes

## Progress: descent fixed in `7509222`, same-depth advance still open

The descent no longer searches. The first directory at the next level is noticed while
the current level is emitted and carried in the cursor, so discovering "there is no level
below" costs nothing instead of a scan of the level just walked. Measured on sixty leaf
directories: 180 steps for 121 rows, against 241 before.

Verified by mutation, not just by passing:

- disabling the memo raises the same read to 241 steps, over the test's bound of 190
- recording before the ignore check expands a pruned forty-child subtree, 46 steps against
  a bound of 40 (no row leaks, because every child of a pruned directory is ignored and
  the row filter catches them again -- which is why that test asserts on work, not rows)
- dropping the memo from the cursor fails both resume tests

## What remains

The same-depth advance, `directory_at_depth(parent_depth, Some(&parent))`, can still scan
a run of childless directories at the level above before finding the next parent. So a
page can still exceed `max_work`, and the strict bound this bead asks for is unfinished.

What changed is reachability. Before, every leaf level cost a scan of its whole width --
universal, since every tree has a last level. Now it takes an adversarial shape: a long
contiguous run of childless directories inside one level, followed by one with children.

Also unreachable for the query that matters most. `'levels: while parent_depth < max_depth`
means a `depth: Limit(1)` read runs the body only at depth zero and its single advance
returns immediately, so a one-level directory listing -- MetaBrowser's Directory query --
performs no search at all. The residual is confined to `depth: All` and deep bounded reads.

## Open decision before finishing this

Closing the residual strictly means teaching the cursor the searching state after all:
`directory_at_depth` reporting where it stopped, as a path at some level above the target
plus that target depth. That is real work.

The alternative is `fdu-3v0d`: make the MetaBrowser-side work budget an observation that
is recorded and surfaced rather than an assertion that rejects the page. A provider
overrunning its budget is a performance fault, not a correctness one, and rejecting throws
away correct rows; the duplicate-path check already catches the correctness failure the
assertion would otherwise stand in for.

Decide that before spending the engine work, because if the budget is an observation the
residual needs measuring rather than eliminating.
