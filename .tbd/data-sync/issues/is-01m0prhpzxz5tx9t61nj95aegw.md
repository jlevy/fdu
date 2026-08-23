---
type: is
id: is-01m0prhpzxz5tx9t61nj95aegw
title: TreeNode remainder aggregates for bounded trees
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:19.068Z
updated_at: 2026-08-23T19:23:17.778Z
closed_at: 2026-08-23T19:23:17.777Z
close_reason: "TreeNode.truncated becomes a derived accessor over a new remainder: Option<Remainder>, the aggregate (rows, files, dirs, bytes, allocated) of the child rows a depth or limit bound withheld. Emitted in all four formats — the text truncation row now states what it dropped instead of a bare ellipsis. Asserted against the same query with the bound lifted, in Rust and in the Python smoke. Planes join the aggregate when partitioned tallies land (fdu-mvt3/fdu-7rwf)."
resolution: null
duplicate_of: null
---
When a tree view omits children, the node carries the aggregate of what was dropped (dirs, files, bytes, per plane) machine-readably, not just truncated: bool. 'Truncate freely, never silently' applied to the one place it is still a bare flag; a treemap's 'other' cell is this value.

## Notes

TreeNode.truncated is a bare bool today; it becomes the aggregate of what was dropped (dirs, files, bytes, per plane). Rendered by the tree section builder in query_report.rs and by all four formats in report_format.rs (render entry at :97).
