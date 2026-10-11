---
type: is
id: is-01m4kjcr3ywmgnc6reyzakeyzc
title: Make the list view fast over a million-entry tree
kind: task
status: in_progress
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T18:49:48.925Z
updated_at: 2026-10-11T05:29:24.512Z
started_at: 2026-10-11T05:29:24.511Z
---
fdu-fy0l's measurement (2026-10-10, Linux): fdu --view list --limit 100000 --format json over linux-balanced-1m takes 10.0 s on the command line and 11.4 s in fdu.report's native call, at 0.4.0 and 0.5.0 alike, against 1.2 s for the default tree over the same tree. Profile before changing anything: the candidates are the per-directory subtree measure for list rows, the sort of a million rows, and building rows the bound discards.
