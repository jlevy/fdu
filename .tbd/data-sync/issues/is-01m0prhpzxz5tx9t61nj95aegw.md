---
type: is
id: is-01m0prhpzxz5tx9t61nj95aegw
title: TreeNode remainder aggregates for bounded trees
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:19.068Z
updated_at: 2026-08-23T07:32:19.068Z
---
When a tree view omits children, the node carries the aggregate of what was dropped (dirs, files, bytes, per plane) machine-readably, not just truncated: bool. 'Truncate freely, never silently' applied to the one place it is still a bare flag; a treemap's 'other' cell is this value.
