---
type: is
id: is-01m4fxammtqwc8mb1362jgtmhn
title: "Age: unfiltered fast pass beside measure(), pinned to it by test"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxankw8s2qrmnnkn3ejs75
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:43:56.437Z
updated_at: 2026-10-09T09:06:01.003Z
---
query_subtrees: the report root is a traversal boundary (its own time never counts) in measure() too. New unfiltered pass by entry id and depth, no paths: activity = max(own time unless root, non-dir children, subdir activity, roll-up newest file when files > 0, which covers folded files); completeness by measure()'s rule. Tests: equals measure() on every directory of a full index; on a folded index equals measure() on the full index of the same tree (path-independence harness); fixtures with empty dirs, symlinks, scan-depth boundary, failed subtree, a dir whose own time is newest.
