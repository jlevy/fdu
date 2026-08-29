---
type: is
id: is-01m15zr7prnyhnfgc5wa325x0e
title: Bound the tree level-advance search without stranding the cursor
kind: task
status: open
priority: 1
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
  - type: blocks
    target: is-01m1687g2cazrcaxzwkpdcazz5
created_at: 2026-08-29T05:26:49.303Z
updated_at: 2026-08-29T07:55:27.097Z
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

Reconsidered: there is a better approach than the resumable-search cursor this bead was
filed with, and a sharper reason to do it than latency.

## Why it matters more than milliseconds

The plan spec's implementation table requires MetaBrowser's `assemble_tree_pages` to
"Enforce stable provider version, positive row bound, unique advancing opaque
continuation, maximum pages, maximum rows, and request work budget."

fdu's tree projection can report `rows_visited` above the requested `max_work`. Verified
against MetaBrowser `codex/inventory-contract-alignment`: `assemble_tree_pages` does not
yet check work, so there is no live failure. But whoever implements that specified rule
will reject valid fdu pages on exactly the trees where the overrun happens. The deviation
is unrepresentable in the contract the other side is specified to enforce, so it has to be
closed in the engine or renegotiated in both specs.

## Better design: memoize instead of rescanning

The expensive scan is `directory_at_depth(deeper, None)` -- discovering whether a level
below exists by asking every parent at the current depth for a directory child. But the
breadth-first walk already visits every one of those parents while emitting.

So record it in passing: while emitting children of parent P at depth d, if P has a
non-ignored directory child and none is recorded yet, remember it. When the level is
exhausted, that value is the first parent at d+1 in level order, because level order at
d+1 is grouped by parent order at d. Nothing recorded means there is no level below.

O(1) instead of O(level width), and it removes the scan rather than making it resumable.

Cost: one more optional path in `ChildPosition`, still bounded by a single path. The
per-parent check is a `first_directory_child` call already made during today's descent
scan, moved to where the walk is already standing and remembered.

Residual: the same-depth advance can still spike on a long run of childless directories
mid-level, but its total cost is amortized O(1) per emitted parent across the level. The
pathological shape narrows from "the whole level is leaves" -- the common case, every leaf
level -- to "a long childless run inside one level".

Reasoned through, not yet implemented or tested.
