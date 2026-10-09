---
type: is
id: is-01m4fxankw8s2qrmnnkn3ejs75
title: "Age: tree nodes carry mtime_ns, complete, age_ns; --sort mtime uses them"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxar5gf3qrax2k0rwfczn0
  - type: blocks
    target: is-01m4fxasm0nae6c2tbjnqhd6rz
  - type: blocks
    target: is-01m4fxaw4tnmt77j099fhnk8j6
  - type: blocks
    target: is-01m4fxayv9jpnd3nnh65f2q1nw
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:43:57.433Z
updated_at: 2026-10-09T09:06:01.405Z
---
A tree row's age = newest mtime among the entries it counts (own entry when admitted, every counted entry beneath, any kind; root excluded; none -> no age). Unfiltered: the fast pass. Filtered: fold in walk() over admitted/covered dirs and symlinks as well as files. complete: true over a complete index with no scan depth, else from tree_measurements. TreeNode gains mtime_ns, complete (null for files), age_ns (null when incomplete/empty/unrepresentable); newest_mtime_ns keeps its files-only meaning. Tree --sort mtime orders by mtime_ns.
