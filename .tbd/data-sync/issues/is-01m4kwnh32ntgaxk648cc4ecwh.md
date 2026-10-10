---
type: is
id: is-01m4kwnh32ntgaxk648cc4ecwh
title: "child_rows: read a child's activity only once its row is admitted"
kind: task
status: open
priority: 4
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
created_at: 2026-10-10T21:49:22.401Z
updated_at: 2026-10-10T21:49:22.401Z
---
Review E9 on PR #192 (jlevy/fdu#192), item 3 of review C8 on #191. child_rows (query_report.rs) reads each directory child's activity before the share test. Compute complete first and read activity only for admitted rows: that removes O(children) arena reads per expanded row on retained reports, a small but free saving on wide directories. Measure with the retained second-report job (exp-212).
