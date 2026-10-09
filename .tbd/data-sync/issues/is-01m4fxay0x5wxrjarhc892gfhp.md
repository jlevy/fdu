---
type: is
id: is-01m4fxay0x5wxrjarhc892gfhp
title: Docs, goldens, and parity for 0.5.0
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:06.044Z
updated_at: 2026-10-09T11:16:13.772Z
---
Usage, machine output (tree node table, roots, /11), output design (age column, total row, remainder), design principles scope axis, surface architecture schema table, README, CHANGELOG, --docs, skill. Goldens: AGE pattern on default trees; a two-fixture session (text tree, flat list, JSON, overlap refusal). Parity artifact via CI.

## Notes

Phase 1 part done on claude/tree-age-column (goldens/parity 7ba18265, docs ff609e61, schema fixtures 7d125b9c, YAML check d8c92b45; make check green at 66652033): goldens pattern the age column ([AGE], new [AGE_BLANK] for the remainder's blank cell) and the new machine fields ([MTIME_NS]/[AGE_NS]/[RFC3339]); parity classes accept [AGE]/[AGE_BLANK] as portable spellings and mask observed age_ns; deviations-python.diff re-recorded locally on macOS (CI re-records on Linux; download its artifact if it differs); docs: machine-output tree-node table + modified_at/age_reference_at, output design (column + ladder), usage, README Quick Start, --docs, skill, Python README, design principles watch + query tiers, engine architecture, correctness runbook (answer.py sets age_reference_at aside), CHANGELOG [Unreleased]. Phase 2 (several roots, two-fixture session, overlap refusal, roots docs, surface/scope-axis docs) still open.
