---
type: is
id: is-01m15zr7prnyhnfgc5wa325x0e
title: Bound the tree level-advance search without stranding the cursor
kind: task
status: open
priority: 2
version: 9
labels: []
dependencies:
  - type: blocks
    target: is-01m1687g2cazrcaxzwkpdcazz5
created_at: 2026-08-29T05:26:49.303Z
updated_at: 2026-08-29T20:11:32.766Z
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

## Reframed: no longer a merge blocker

`max_work` is a soft budget for the tree: it says where a page stops, not how much work to
refuse, and the page reports what it actually spent (`fdu-3v0d` records the consumer-side
decision to treat that as an observation). Slight overrun is the normal case.

What a bound has to guarantee is termination and forward progress, and both now hold:

- the level advance walks a finite index strictly forward, so it always ends
- every page emits at least one row or ends the traversal

The second was **not** true and is fixed in the same change as the descent memo. `spent`
starts at the cost of walking to the requested directory and only a strictly longer walk
was refused, so at a budget equal to that walk the first child pushed `spent` over before
any row was emitted: the page returned no rows and a cursor pointing at the child it was
about to read, and resuming reproduced it exactly. Reading `a/b` at `max_work: 3` paged
forever, seventeen pages and zero rows, until the guard was cut off. Pinned by
`opened::tests::a_page_moves_even_when_the_path_walk_spends_the_budget`, which sweeps
budgets from the path-walk cost upward rather than naming one.

## What is left, and why it is P2

The descent no longer searches (`7509222`): 180 steps for 121 rows on sixty leaf
directories, against 241 before. The same-depth advance can still cross a run of childless
directories before reaching the next parent -- bounded by one level's width, paid once at
that boundary because the parent it finds is what the cursor records, and never reached at
all by a `depth: Limit(1)` read.

Closing that strictly still means teaching the cursor the searching state. It is now a
performance refinement rather than a contract violation, worth doing when tree paging
meets a real wide tree.
