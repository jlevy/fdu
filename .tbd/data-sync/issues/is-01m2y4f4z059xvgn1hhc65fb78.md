---
type: is
id: is-01m2y4f4z059xvgn1hhc65fb78
title: "H139: Linux cache-hit stack same or different"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H139
dependencies:
  - type: blocks
    target: is-01m2y4f5w92kqgdwzzf75bm1yn
  - type: blocks
    target: is-01m2y4f6c3ea0q3ngn99nz7eek
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T00:46:42.654Z
updated_at: 2026-09-20T00:46:44.098Z
---
Pair #91 (H115+H120) control vs this branch on Linux content-cache-hit. Replication, not a new cut. Record same vs different. exp-138+. Do not revert landed engine on a miss.
