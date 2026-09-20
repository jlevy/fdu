---
type: is
id: is-01m2ysreqwf3xczbbevbv4ve06
title: Terminal multi-path reconciliation clears unvisited issues
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:58:47.666Z
updated_at: 2026-09-20T06:58:47.666Z
---
A multi-path reconciliation begins every subtree before walking. If an earlier subtree returns a terminal error, later subtrees are closed without a walk; unconditional partial-pass cleanup drops their older scoped issues despite no disproving observation. Carry visited evidence into closure and retain issues for skipped subtrees.
