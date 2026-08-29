---
type: is
id: is-01m15zr7prnyhnfgc5wa325x0e
title: Bound the tree level-advance search without stranding the cursor
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-29T05:26:49.303Z
updated_at: 2026-08-29T05:26:49.303Z
---
The breadth-first tree projection charges its level-advance search against `spent` but never cuts it short, so a single page can exceed `page.max_work` by one scan of one level. That deviates from the plan spec's rule that exhausting the work budget returns a typed query-limit result.

Cutting the search short was tried and is worse. The one-frame cursor (ChildPosition) can only name a parent and a child within it; a search interrupted mid-level has no row to point at, so the next page restarts the same search and overruns the same budget. A level wider than max_work would then never be crossed: the tree becomes permanently unpageable rather than merely slow. Returning ProjectionResult::Limit has the same effect, since re-asking cannot make progress.

Fixing this properly means letting the cursor express the searching state as well as the emitting one, e.g. ChildPosition gaining a mode where {parent, depth, name} means 'children of parent are fully emitted; resume the search for the next parent at depth after name'. directory_at_depth would report where it stopped instead of only what it found.

Cost of the current behavior is bounded and paid once per level boundary, not per page, because the parent the search finds is what the cursor then records. Worth doing when tree paging meets a real wide tree; not worth blocking on.

Context: crates/fdu-core/src/opened/read.rs, tree_projection level-advance block and directory_at_depth. Correctness of paging itself is covered by opened::tests::a_tree_page_stopped_by_the_work_budget_is_resumable.
