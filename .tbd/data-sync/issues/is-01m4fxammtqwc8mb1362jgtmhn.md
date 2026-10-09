---
type: is
id: is-01m4fxammtqwc8mb1362jgtmhn
title: "Age: unfiltered fast pass beside measure(), pinned to it by test"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxankw8s2qrmnnkn3ejs75
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:43:56.437Z
updated_at: 2026-10-09T11:16:01.457Z
started_at: 2026-10-09T09:25:42.681Z
closed_at: 2026-10-09T11:16:01.456Z
close_reason: "Landed in fac3d744: measure() treats the report root as a traversal boundary; query_subtrees::activity is the unfiltered pass (post-order by id and depth, dense per-slot table, roll-up newest file when files>0, measure's completeness rule). Pinned to measure() on full indexes (empty dirs, symlinks, other entries, stale-max repair, scan-depth boundary, unscoped partial, opened root mid-discovery) and on folded indexes against the full index of the same walk incl. a failed subtree; path-independence harness compares tree-node ages as residuals (7ba18265). make check green at 66652033."
resolution: null
duplicate_of: null
---
query_subtrees: the report root is a traversal boundary (its own time never counts) in measure() too. New unfiltered pass by entry id and depth, no paths: activity = max(own time unless root, non-dir children, subdir activity, roll-up newest file when files > 0, which covers folded files); completeness by measure()'s rule. Tests: equals measure() on every directory of a full index; on a folded index equals measure() on the full index of the same tree (path-independence harness); fixtures with empty dirs, symlinks, scan-depth boundary, failed subtree, a dir whose own time is newest.
